use crate::msg::{ExecuteMsg};
use crate::error::ContractError;
use crate::state::{
    Config, TokenId, Bid, bids, bid_key, Ask, asks, CollectionBid, collection_bids,
};
use cosmwasm_std::{
    to_binary, Addr, Api, StdResult, WasmMsg,CosmosMsg, Order,
    Deps, Event, Coin, coin, Uint128, Response, MessageInfo, Storage, Attribute,
    BankMsg, SubMsg, Env
};
use pg721::msg::{CollectionInfoResponse, QueryMsg as Pg721QueryMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw721::{Cw721ExecuteMsg};
use cw721_base::helpers::Cw721Contract;

// MarketplaceContract is a wrapper around Addr that provides a lot of helpers
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MarketplaceContract(pub Addr);

impl MarketplaceContract {
    pub fn addr(&self) -> Addr {
        self.0.clone()
    }

    pub fn call<T: Into<ExecuteMsg>>(&self, msg: T) -> StdResult<CosmosMsg> {
        let msg = to_binary(&msg.into())?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }
}

pub fn map_validate(api: &dyn Api, addresses: &[String]) -> StdResult<Vec<Addr>> {
    addresses
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}

pub fn option_bool_to_order(descending: Option<bool>) -> Order {
     match descending {
        Some(_descending) => if _descending { Order::Descending } else { Order::Ascending },
        _ => Order::Ascending
    }
}

/// Transfers funds and NFT, updates bid
pub fn finalize_sale(
    deps: Deps,
    bidder: &Addr,
    token_id: &TokenId,
    payment_amount: Uint128,
    payment_recipient: &Addr,
    surplus_amount: Uint128,
    surplus_recipient: &Addr,
    config: &Config,
    res: &mut Response,
) -> StdResult<()> {
    payout(
        deps,
        payment_amount,
        payment_recipient,
        surplus_amount,
        surplus_recipient,
        &config,
        res,
    )?;

    transfer_nft(&token_id, bidder, &config.cw721_address, res)?;

    let event = Event::new("finalize-sale")
        .add_attribute("collection", config.cw721_address.to_string())
        .add_attribute("buyer", bidder.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("payment_amount", payment_amount.to_string())
        .add_attribute("payment_recipient", payment_recipient.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
pub fn payout(
    deps: Deps,
    payment_amount: Uint128,
    payment_recipient: &Addr,
    surplus_amount: Uint128,
    surplus_recipient: &Addr,
    config: &Config,
    response: &mut Response,
) -> StdResult<()> {
    if surplus_amount > Uint128::zero() {
        transfer_token(
            coin(surplus_amount.u128(), &config.denom),
            surplus_recipient.to_string(),
            "payout-surplus",
            response
        )?;
    }

    let cw721_address = config.cw721_address.to_string();

    // Charge market fee
    let market_fee = payment_amount * config.trading_fee_percent / Uint128::from(100u128);
    if market_fee > Uint128::zero() {
        transfer_token(
            coin(market_fee.u128(), &config.denom),
            config.collector_address.to_string(),
            "payout-market",
            response
        )?;
    }

    // Query royalties
    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(&cw721_address, &Pg721QueryMsg::CollectionInfo {})?;

    // Charge royalties if they exist
    let royalties = match &collection_info.royalty_info {
        Some(royalty) => Some((payment_amount * royalty.share, &royalty.payment_address)),
        None => None
    };
    if let Some(_royalties) = &royalties {
        if _royalties.0 > Uint128::zero() {
            transfer_token(
                coin(_royalties.0.u128(), &config.denom),
                _royalties.1.to_string(),
                "payout-royalty",
                response
            )?;
        }
    };

    // Pay seller
    let mut seller_amount = payment_amount - market_fee;
    if let Some(_royalties) = &royalties {
        seller_amount -= _royalties.0;
    };

    transfer_token(
        coin(seller_amount.u128(), &config.denom),
        payment_recipient.to_string(),
        "payout-seller",
        response
    )?;

    Ok(())
}

// Validate Bid or Ask price
pub fn price_validate(price: &Coin, config: &Config) -> Result<(), ContractError> {
    if
        price.amount.is_zero() ||
        price.denom != config.denom ||
        price.amount < config.min_price
    {
        return Err(ContractError::InvalidPrice {});
    }

    Ok(())
}

pub fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(
        store,
        bid_key(&bid.bidder, bid.token_id.clone()),
        bid,
    )
}

pub fn store_collection_bid(store: &mut dyn Storage, collection_bid: &CollectionBid) -> StdResult<()> {
    collection_bids().save(
        store,
        collection_bid.bidder.clone(),
        collection_bid,
    )
}

/// Checks to enforce only NFT owner can call
pub fn only_owner_or_seller(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
    existing_seller: &Option<Addr>,
) -> Result<(), ContractError> {
    match existing_seller {
        Some(_seller) => only_seller(&info, &_seller),
        None => only_owner(deps, info, collection, &token_id),
    }
}

/// Checks to enforce only NFT owner can call
pub fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<(), ContractError> {
    let res = Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from("only the owner can call this function")));
    }
    Ok(())
}

/// Checks to enforce only Ask seller can call
pub fn only_seller(
    info: &MessageInfo,
    seller: &Addr,
) -> Result<(), ContractError> {
    if &info.sender != seller {
        return Err(ContractError::Unauthorized(String::from("only the seller can call this function")));
    }
    Ok(())
}

/// Checks to enforce only privileged operators
pub fn only_operator(info: &MessageInfo, config: &Config) -> Result<Addr, ContractError> {
    if !config
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::Unauthorized(String::from("only an operator can call this function")));
    }

    Ok(info.sender.clone())
}

pub fn transfer_nft(token_id: &TokenId, recipient: &Addr, collection: &Addr, response: &mut Response,) -> StdResult<()> {
    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: token_id.to_string(),
        recipient: recipient.to_string(),
    };

    let exec_cw721_transfer = SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    });
    response.messages.push(exec_cw721_transfer);

    let event = Event::new("transfer-nft")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);
    
    Ok(())
}

pub fn transfer_token(coin_send: Coin, recipient: String, event_label: &str, response: &mut Response) -> StdResult<()> {
    let token_transfer_msg = BankMsg::Send {
        to_address: recipient.clone(),
        amount: vec![coin_send.clone()]
    };
    response.messages.push(SubMsg::new(token_transfer_msg));

    let event = Event::new(event_label)
        .add_attribute("coin", coin_send.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);

    Ok(())
}

pub fn match_ask(deps: Deps, ask: &Ask, response: &mut Response) -> StdResult<Option<Bid>> {
    let highest_bid_results = bids()
        .idx
        .token_price
        .sub_prefix(ask.token_id.clone())
        .range(deps.storage, None, None, Order::Descending)
        .take(1usize)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    let highest_bid_option = highest_bid_results.get(0);
    if let None = highest_bid_option {
        return Ok(None)
    }

    let highest_bid = highest_bid_option.unwrap().clone();
    let mut event = Event::new("match-ask")
        .add_attribute("token-id", ask.token_id.clone())
        .add_attribute("outcome", "match");
    
    if highest_bid.price.amount < ask.price.amount {
        set_match_outcome(&mut event, "ask-too-high");
        response.events.push(event);
        return Ok(None)
    }

    response.events.push(event);
    return Ok(Some(highest_bid))
}

pub fn match_bid(deps: Deps, _env: &Env, bid: &Bid, response: &mut Response) -> StdResult<Option<Ask>> {
    let matching_ask = asks().may_load(deps.storage, bid.token_id.clone())?;

    if let None = matching_ask {
        return Ok(None)
    }

    let existing_ask = matching_ask.unwrap();
    let mut event = Event::new("match-bid")
        .add_attribute("token-id", bid.token_id.clone())
        .add_attribute("outcome", "match");
    
    if existing_ask.price.amount > bid.price.amount {
        set_match_outcome(&mut event, "bid-too-low");
        response.events.push(event);
        return Ok(None)
    }

    response.events.push(event);
    return Ok(Some(existing_ask))
}

fn set_match_outcome(event: &mut Event, outcome: &str) -> () {
    event.attributes = event.attributes.iter_mut().map(|attr| {
        if attr.key == "outcome" {
            return Attribute {
                key: String::from("outcome"),
                value: String::from(outcome),
            }
        }
        attr.clone()
    }).collect();
}
