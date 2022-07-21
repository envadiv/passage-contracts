use crate::error::ContractError;
use crate::helpers::{map_validate, ExpiryRange};
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{
    Params, PARAMS, Ask, asks, TokenId, bid_key, bids, Order, Bid, CollectionBid, collection_bids,
    Auction, auctions
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env, Event, MessageInfo,
    StdError, StdResult, Storage, Uint128, WasmMsg, Response, SubMsg, Attribute
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{maybe_addr, must_pay, nonpayable};
use pg721::msg::{CollectionInfoResponse, QueryMsg as Pg721QueryMsg};

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:marketplace-v2";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.ask_expiry.validate()?;
    msg.bid_expiry.validate()?;

    let api = deps.api;
    let params = Params {
        cw721_address: api.addr_validate(&msg.cw721_address)?,
        denom: msg.denom,
        collector_address: api.addr_validate(&msg.collector_address)?,
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        ask_expiry: msg.ask_expiry,
        bid_expiry: msg.bid_expiry,
        auction_expiry: msg.auction_expiry,
        operators: map_validate(deps.api, &msg.operators)?,
        min_price: msg.min_price,
    };
    PARAMS.save(deps.storage, &params)?;

    Ok(Response::new())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;
    let message_info = info.clone();

    match msg {
        ExecuteMsg::UpdateParams {
            trading_fee_bps,
            ask_expiry,
            bid_expiry,
            auction_expiry,
            operators,
            min_price,
        } => execute_update_params(
            deps,
            env,
            info,
            trading_fee_bps,
            ask_expiry,
            bid_expiry,
            auction_expiry,
            operators,
            min_price,
        ),
        ExecuteMsg::SetAsk {
            token_id,
            price,
            funds_recipient,
            reserve_for,
            expires_at,
        } => execute_set_ask(
            deps,
            env,
            info,
            Ask {
                token_id,
                seller: message_info.sender,
                price,
                funds_recipient: maybe_addr(api, funds_recipient)?,
                reserve_for: maybe_addr(api, reserve_for)?,
                expires_at,
            },
        ),
        ExecuteMsg::RemoveAsk {
            token_id,
        } => execute_remove_ask(deps, info, token_id),
        ExecuteMsg::SetBid {
            token_id,
            price,
            expires_at,
        } => execute_set_bid(
            deps,
            env,
            info,
            Bid {
                token_id,
                bidder: message_info.sender,
                price,
                expires_at,
            },
        ),
        ExecuteMsg::RemoveBid {
            token_id,
        } => execute_remove_bid(deps, env, info, token_id),
        ExecuteMsg::AcceptBid {
            token_id,
            bidder,
        } => execute_accept_bid(
            deps,
            env,
            info,
            token_id,
            api.addr_validate(&bidder)?,
        ),
        ExecuteMsg::SetCollectionBid {
            units,
            price,
            expires_at,
        } => execute_set_collection_bid(
            deps,
            env,
            info,
            CollectionBid {
                units,
                price,
                bidder: message_info.sender,
                expires_at
            }
        ),
        ExecuteMsg::RemoveCollectionBid { } => {
            execute_remove_collection_bid(deps, env, info)
        }
        ExecuteMsg::AcceptCollectionBid {
            token_id,
            bidder,
        } => execute_accept_collection_bid(
            deps,
            env,
            info,
            token_id,
            api.addr_validate(&bidder)?,
        ),
        ExecuteMsg::SetAuction {
            token_id,
            starting_price,
            reserve_price,
            funds_recipient,
            expires_at,
        } => execute_set_auction(
            deps,
            env,
            info,
            Auction {
                token_id,
                seller: message_info.sender,
                starting_price,
                reserve_price,
                funds_recipient: maybe_addr(api, funds_recipient)?,
                expires_at,
            },
        ),
    }
}

/// An operator may update the marketplace params
pub fn execute_update_params(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    trading_fee_bps: Option<u64>,
    ask_expiry: Option<ExpiryRange>,
    bid_expiry: Option<ExpiryRange>,
    auction_expiry: Option<ExpiryRange>,
    operators: Option<Vec<String>>,
    min_price: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut params = PARAMS.load(deps.storage)?;
    only_operator(&info, &params)?;

    if let Some(_trading_fee_bps) = trading_fee_bps {
        params.trading_fee_percent = Decimal::percent(_trading_fee_bps);
    }
    if let Some(_ask_expiry) = ask_expiry {
        _ask_expiry.validate()?;
        params.ask_expiry = _ask_expiry;
    }
    if let Some(_bid_expiry) = bid_expiry {
        _bid_expiry.validate()?;
        params.bid_expiry = _bid_expiry;
    }
    if let Some(_auction_expiry) = auction_expiry {
        _auction_expiry.validate()?;
        params.auction_expiry = _auction_expiry;
    }
    if let Some(_operators) = operators {
        params.operators = map_validate(deps.api, &_operators)?;
    }
    if let Some(_min_price) = min_price {
        params.min_price = _min_price;
    }
    
    PARAMS.save(deps.storage, &params)?;
    Ok(Response::new())
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ask: Ask,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    
    let params = PARAMS.load(deps.storage)?;
    params.ask_expiry.is_valid(&env.block, ask.expires_at)?;
    price_validate(&ask.price, &params)?;

    let existing_ask = asks().load(deps.storage, ask.token_id.clone()).ok();
    only_owner_or_seller(
        deps.as_ref(),
        &info,
        &params.cw721_address.clone(),
        &ask.token_id,
        &existing_ask,
    )?;

    // Upsert ask
    asks().update(
        deps.storage,
        ask.token_id.clone(),
        |_| -> Result<Ask, StdError> { Ok(ask.clone()) },
    )?;

    let mut response = Response::new();

    match existing_ask {
        None => transfer_nft(&ask.token_id, &env.contract.address, &params.cw721_address, &mut response)?,
        _ => (),
    }

    let event = Event::new("set-ask")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller)
        .add_attribute("price", ask.price.to_string())
        .add_attribute("expires_at", ask.expires_at.to_string());

    Ok(response.add_event(event))
}

/// Removes the ask on a particular NFT
pub fn execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let ask = asks().load(deps.storage, token_id.clone())?;
    only_seller(&info, &ask)?;

    asks().remove(deps.storage, token_id.clone())?;

    let params = PARAMS.load(deps.storage)?;
    let mut response = Response::new();

    transfer_nft(&ask.token_id, &ask.seller, &params.cw721_address, &mut response)?;

    let event = Event::new("remove-ask")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", token_id.to_string());

    Ok(response.add_event(event))
}

/// Places a bid on a listed or unlisted NFT. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bid: Bid,
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;

    let payment_amount = must_pay(&info, &params.denom)?;
    if bid.price.amount != payment_amount  {
        return Err(ContractError::IncorrectBidPayment(bid.price.amount, payment_amount));
    }
    price_validate(&bid.price, &params)?;
    params.bid_expiry.is_valid(&env.block, bid.expires_at)?;

    let mut response = Response::new();
    let bid_key = bid_key(bid.token_id.clone(), &bid.bidder);
    let ask_key = &bid.token_id;

    // If bid exists, refund the escrowed tokens
    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        transfer_token(
            existing_bid.price,
            existing_bid.bidder.to_string(),
            "refund-bidder",
            &mut response,
        )?;
    }

    let matching_ask = match_bid(deps.as_ref(), env, &bid, &mut response)?;

    // If existing ask found, finalize the sale
    match matching_ask {
        Some(ask) => {
            asks().remove(deps.storage, ask_key.clone())?;
            finalize_sale(
                deps.as_ref(),
                bid.bidder.clone(),
                payment_amount,
                &ask,
                &params,
                &mut response,
            )?
        },
        None => store_bid(deps.storage, &bid)?,
    };

    let event = Event::new("set-bid")
        .add_attribute("token_id", bid.token_id.to_string())
        .add_attribute("bidder", bid.bidder)
        .add_attribute("price", bid.price.to_string())
        .add_attribute("expires_at", bid.expires_at.to_string());
    response.events.push(event);

    Ok(response)
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
pub fn execute_remove_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let bidder = info.sender;

    let key = bid_key(token_id.clone(), &bidder);
    let bid = bids().load(deps.storage, key.clone())?;
    bids().remove(deps.storage, key)?;

    let mut response = Response::new();
    transfer_token(bid.price, bid.bidder.to_string(), "refund-bidder", &mut response)?;

    let event = Event::new("remove-bid")
        .add_attribute("token_id", token_id.clone())
        .add_attribute("bidder", bidder);
    response.events.push(event);

    Ok(response)
}

/// Seller can accept a bid which transfers funds as well as the token. The bid may or may not be associated with an ask.
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
    bidder: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let bid_key = bid_key(token_id.clone(), &bidder);
    let bid = bids().load(deps.storage, bid_key.clone())?;
    if bid.is_expired(&env.block.time) {
        return Err(ContractError::BidExpired {});
    }

    let params = PARAMS.load(deps.storage)?;
    let existing_ask = asks().may_load(deps.storage, token_id.clone())?;

    only_owner_or_seller(
        deps.as_ref(),
        &info,
        &params.cw721_address.clone(),
        &token_id,
        &existing_ask,
    )?;

    // Validate sender, formalize Ask
    let ask = match existing_ask {
        Some(_ask) => {
            asks().remove(deps.storage, token_id.clone())?;
            _ask
        },
        None => {
            Ask {
                token_id: token_id.clone(),
                seller: info.sender.clone(),
                price: bid.price.clone(),
                funds_recipient: None,
                reserve_for: None,
                expires_at: bid.expires_at.clone(),
            }
        }
    };

    let mut response = Response::new();

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        bid.bidder,
        bid.price.amount,
        &ask,
        &params,
        &mut response,
    )?;

    // Remove accepted bid
    bids().remove(deps.storage, bid_key)?;

    let event = Event::new("accept-bid")
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("bidder", bidder)
        .add_attribute("price", bid.price.to_string())
        .add_attribute("expires_at", bid.expires_at.to_string());
    response.events.push(event);

    Ok(response)
}

/// Place a collection bid (limit order) across an entire collection
pub fn execute_set_collection_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection_bid: CollectionBid
) -> Result<Response, ContractError> {
    let params = PARAMS.load(deps.storage)?;
    
    // Escrows the amount (price * units)
    let payment_amount = must_pay(&info, &params.denom)?;
    price_validate(&coin(collection_bid.total_cost(), &params.denom), &params)?;
    if Uint128::from(collection_bid.total_cost()) != payment_amount  {
        return Err(ContractError::IncorrectBidPayment(
            Uint128::from(collection_bid.total_cost()),
            payment_amount,
        ));
    }
    params.bid_expiry.is_valid(&env.block, collection_bid.expires_at)?;

    let collection_bid_key = collection_bid.bidder.clone();
    let mut response = Response::new();

    // If collection bid exists, refund the escrowed tokens
    if let Some(existing_bid) = collection_bids().may_load(deps.storage, collection_bid_key.clone())? {
        collection_bids().remove(deps.storage, collection_bid_key.clone())?;
        transfer_token(
            existing_bid.price,
            existing_bid.bidder.to_string(),
            "refund-collection-bidder",
            &mut response,
        )?;
    }
    collection_bids().save(deps.storage, collection_bid_key, &collection_bid)?;

    let event = Event::new("set-collection-bid")
        .add_attribute("bidder", collection_bid.bidder)
        .add_attribute("price", collection_bid.price.to_string())
        .add_attribute("units", collection_bid.units.to_string())
        .add_attribute("expires_at", collection_bid.expires_at.to_string());
    response.events.push(event);

    Ok(response)
}

/// Remove an existing collection bid (limit order)
pub fn execute_remove_collection_bid(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    let mut response = Response::new();
    
    let collection_bid_key = info.sender.clone();

    let collection_bid = collection_bids().load(deps.storage, collection_bid_key.clone())?;

    collection_bids().remove(deps.storage, collection_bid_key)?;
    transfer_token(
        collection_bid.price,
        collection_bid.bidder.to_string(),
        "refund-collection-bidder",
        &mut response,
    )?;

    let event = Event::new("remove-collection-bid")
        .add_attribute("bidder", collection_bid.bidder);
    response.events.push(event);

    Ok(response)
}

/// Owner/seller of an item in a collection can accept a collection bid which transfers funds as well as a token
pub fn execute_accept_collection_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
    bidder: Addr,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let collection_bid_key = bidder.clone();
    let mut collection_bid = collection_bids().load(deps.storage, collection_bid_key.clone())?;
    if collection_bid.is_expired(&env.block.time) {
        return Err(ContractError::BidExpired {});
    }

    let params = PARAMS.load(deps.storage)?;
    let existing_ask = asks().may_load(deps.storage, token_id.clone())?;
    only_owner_or_seller(
        deps.as_ref(),
        &info,
        &params.cw721_address.clone(),
        &token_id.clone(),
        &existing_ask,
    )?;

    // Validate sender, formalize Ask
    let ask = match existing_ask {
        Some(_ask) => {
            asks().remove(deps.storage, token_id.clone())?;
            _ask
        },
        None => {
            Ask {
                token_id: token_id.clone(),
                seller: info.sender.clone(),
                price: collection_bid.price.clone(),
                funds_recipient: None,
                reserve_for: None,
                expires_at: collection_bid.expires_at.clone(),
            }
        }
    };

    match collection_bid.units {
        1 => {
            // Remove accepted bid
            collection_bids().remove(deps.storage, collection_bid_key)?;
        },
        _ => {
            // Remove accepted bid
            collection_bid.units -= 1;
            store_collection_bid(deps.storage, &collection_bid)?;
        }
    }

    let mut response = Response::new();

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        collection_bid.bidder.clone(),
        collection_bid.price.amount,
        &ask,
        &params,
        &mut response,
    )?;

    let event = Event::new("accept-collection-bid")
        .add_attribute("bidder", collection_bid.bidder)
        .add_attribute("price", collection_bid.price.to_string())
        .add_attribute("units", collection_bid.units.to_string())
        .add_attribute("expires_at", collection_bid.expires_at.to_string());
    response.events.push(event);

    Ok(response)
}

/// Owner of an NFT can create auction to begin accepting bids
pub fn execute_set_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    auction: Auction,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    
    let params = PARAMS.load(deps.storage)?;
    params.auction_expiry.is_valid(&env.block, auction.expires_at)?;
    
    price_validate(&auction.starting_price, &params)?;
    if let Some(_reserve_price) = &auction.reserve_price {
        price_validate(&_reserve_price, &params)?;
        if _reserve_price.amount < auction.starting_price.amount {
            return Err(ContractError::InvalidReservePrice(_reserve_price.amount, auction.starting_price.amount));
        }
    }

    let existing_auction = auctions().may_load(deps.storage, auction.token_id.clone())?;
    if let Some(_existing_auction) = existing_auction {
        return Err(ContractError::AuctionAlreadyExists(auction.token_id.clone()));
    }

    auctions().save(deps.storage, auction.token_id.clone(), &auction)?;

    let mut response = Response::new();

    transfer_nft(&auction.token_id, &env.contract.address, &params.cw721_address, &mut response)?;

    let event = Event::new("set-auction")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", auction.token_id.to_string())
        .add_attribute("seller", auction.seller)
        .add_attribute("price", auction.starting_price.to_string())
        .add_attribute("expires_at", auction.expires_at.to_string());

    Ok(response.add_event(event))
}

/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    bidder: Addr,
    payment_amount: Uint128,
    ask: &Ask,
    params: &Params,
    res: &mut Response,
) -> StdResult<()> {
    payout(deps, payment_amount, &ask, &params, res)?;

    transfer_nft(&ask.token_id, &bidder, &params.cw721_address, res)?;

    let event = Event::new("finalize-sale")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller.to_string())
        .add_attribute("buyer", bidder.to_string())
        .add_attribute("price", payment_amount.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
fn payout(
    deps: Deps,
    payment_amount: Uint128,
    ask: &Ask,
    params: &Params,
    response: &mut Response,
) -> StdResult<()> {
    let cw721_address = params.cw721_address.to_string();

    // Charge market fee
    let market_fee = payment_amount * params.trading_fee_percent / Uint128::from(100u128);
    transfer_token(
        coin(market_fee.u128(), &params.denom),
        params.collector_address.to_string(),
        "payout-market",
        response
    )?;

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
        transfer_token(
            coin(_royalties.0.u128(), &params.denom),
            _royalties.1.to_string(),
            "payout-royalty",
            response
        )?;
    };

    // Pay seller
    let mut seller_amount = payment_amount - market_fee;
    if let Some(_royalties) = &royalties {
        seller_amount -= _royalties.0;
    };

    let recipient = match &ask.funds_recipient {
        Some(_funds_recipient) => _funds_recipient,
        None => &ask.seller
    };
    transfer_token(
        coin(seller_amount.u128(), &params.denom),
        recipient.to_string(),
        "payout-seller",
        response
    )?;

    Ok(())
}

// Validate Bid or Ask price
fn price_validate(price: &Coin, params: &Params) -> Result<(), ContractError> {
    if
        price.amount.is_zero() ||
        price.denom != params.denom ||
        price.amount < params.min_price
    {
        return Err(ContractError::InvalidPrice {});
    }

    Ok(())
}

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(
        store,
        bid_key(bid.token_id.clone(), &bid.bidder),
        bid,
    )
}

fn store_collection_bid(store: &mut dyn Storage, collection_bid: &CollectionBid) -> StdResult<()> {
    collection_bids().save(
        store,
        collection_bid.bidder.clone(),
        collection_bid,
    )
}

/// Checks to enforce only NFT owner can call
fn only_owner_or_seller(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
    ask: &Option<Ask>,
) -> Result<(), ContractError> {
    match ask {
        Some(_ask) => only_seller(&info, &_ask),
        None => only_owner(deps, info, collection, &token_id),
    }
}

/// Checks to enforce only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<(), ContractError> {
    let res =
        Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }
    Ok(())
}

/// Checks to enforce only Ask seller can call
fn only_seller(
    info: &MessageInfo,
    ask: &Ask,
) -> Result<(), ContractError> {
    if info.sender != ask.seller {
        return Err(ContractError::UnauthorizedOwner {});
    }
    Ok(())
}

/// Checks to enforce only privileged operators
fn only_operator(info: &MessageInfo, params: &Params) -> Result<Addr, ContractError> {
    if !params
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::UnauthorizedOperator {});
    }

    Ok(info.sender.clone())
}

fn transfer_nft(token_id: &TokenId, recipient: &Addr, collection: &Addr, response: &mut Response,) -> StdResult<()> {
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

fn transfer_token(coin_send: Coin, recipient: String, event_label: &str, response: &mut Response) -> StdResult<()> {
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

fn match_bid(deps: Deps, env: Env, bid: &Bid, response: &mut Response) -> StdResult<Option<Ask>> {
    let matching_ask = asks().may_load(deps.storage, bid.token_id.clone())?;

    if let None = matching_ask {
        return Ok(None)
    }

    let existing_ask = matching_ask.unwrap();
    let mut event = Event::new("match-bid")
        .add_attribute("token-id", bid.token_id.clone())
        .add_attribute("outcome", "match");
    
    if existing_ask.is_expired(&env.block.time) {
        set_match_bid_outcome(&mut event, "ask-expired");
        response.events.push(event);
        return Ok(None)
    }
    if let Some(reserved_for) = &existing_ask.reserve_for {
        if reserved_for != &bid.bidder {
            set_match_bid_outcome(&mut event, "token-reserved");
            response.events.push(event);
            return Ok(None)
        }
    }
    if existing_ask.price != bid.price {
        set_match_bid_outcome(&mut event, "invalid-price");
        response.events.push(event);
        return Ok(None)
    }

    response.events.push(event);
    return Ok(Some(existing_ask))
}

fn set_match_bid_outcome(event: &mut Event, outcome: &str) -> () {
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
