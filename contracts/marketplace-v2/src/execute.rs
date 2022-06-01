use crate::error::ContractError;
use crate::helpers::map_validate;
// use crate::msg::{
//     AskHookMsg, BidHookMsg, CollectionBidHookMsg, ExecuteMsg, HookAction, InstantiateMsg,
//     SaleHookMsg,
// };
use crate::msg::{
    InstantiateMsg, ExecuteMsg
};
// use crate::state::{
//     ask_key, asks, bid_key, bids, collection_bid_key, collection_bids, Ask, Bid, CollectionBid,
//     Order, SaleType, Params, TokenId, ASK_HOOKS, BID_HOOKS, COLLECTION_BID_HOOKS, SALE_HOOKS,
//     PARAMS,
// };
use crate::state::{
    Params, PARAMS, Ask, asks, TokenId, bid_key, bids, Order, Bid
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Env, Event, MessageInfo, Reply,
    StdResult, Storage, Timestamp, Uint128, WasmMsg, Response, SubMsg, Attribute
};
use cw2::set_contract_version;
use cw721::{Cw721ExecuteMsg, OwnerOfResponse};
use cw721_base::helpers::Cw721Contract;
use cw_utils::{maybe_addr, must_pay, nonpayable, Expiration};
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
        admins: map_validate(deps.api, &msg.operators)?,
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
        ExecuteMsg::UpdateAskPrice {
            token_id,
            price,
        } => execute_update_ask_price(deps, info, token_id, price),
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
        // ExecuteMsg::AcceptBid {
        //     collection,
        //     token_id,
        //     bidder,
        //     finder,
        // } => execute_accept_bid(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     token_id,
        //     api.addr_validate(&bidder)?,
        //     maybe_addr(api, finder)?,
        // ),
        // ExecuteMsg::SetCollectionBid {
        //     collection,
        //     expires,
        //     finders_fee_bps,
        // } => execute_set_collection_bid(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     finders_fee_bps,
        //     expires,
        // ),
        // ExecuteMsg::RemoveCollectionBid { collection } => {
        //     execute_remove_collection_bid(deps, env, info, api.addr_validate(&collection)?)
        // }
        // ExecuteMsg::AcceptCollectionBid {
        //     collection,
        //     token_id,
        //     bidder,
        //     finder,
        // } => execute_accept_collection_bid(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     token_id,
        //     api.addr_validate(&bidder)?,
        //     maybe_addr(api, finder)?,
        // ),
        // ExecuteMsg::SyncAsk {
        //     collection,
        //     token_id,
        // } => execute_sync_ask(deps, info, api.addr_validate(&collection)?, token_id),
        // ExecuteMsg::RemoveStaleBid {
        //     collection,
        //     token_id,
        //     bidder,
        // } => execute_remove_stale_bid(
        //     deps,
        //     env,
        //     info,
        //     api.addr_validate(&collection)?,
        //     token_id,
        //     api.addr_validate(&bidder)?,
        // ),
        // ExecuteMsg::RemoveStaleCollectionBid { collection, bidder } => {
        //     execute_remove_stale_collection_bid(
        //         deps,
        //         env,
        //         info,
        //         api.addr_validate(&collection)?,
        //         api.addr_validate(&bidder)?,
        //     )
        // }
    }
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
    only_owner(deps.as_ref(), &info, &params.cw721_address, &ask.token_id)?;
    price_validate(&ask.price, &params)?;

    store_ask(deps.storage, &ask)?;

    let mut response = Response::new();

    transfer_nft(&ask.token_id, &env.contract.address, &params.cw721_address, &mut response)?;

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
    let mut res = Response::new();

    transfer_nft(&ask.token_id, &ask.seller, &params.cw721_address, &mut res)?;

    let event = Event::new("remove-ask")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", token_id.to_string());

    Ok(res.add_event(event))
}

/// Updates the ask price on a particular NFT
pub fn execute_update_ask_price(
    deps: DepsMut,
    info: MessageInfo,
    token_id: TokenId,
    price: Coin,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let params = PARAMS.load(deps.storage)?;
    price_validate(&price, &params)?;

    let mut ask = asks().load(deps.storage, token_id.clone())?;
    only_seller(&info, &ask)?;

    ask.price = price;
    asks().save(deps.storage, token_id.clone(), &ask)?;

    let event = Event::new("update-ask")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller)
        .add_attribute("price", ask.price.to_string())
        .add_attribute("expires_at", ask.expires_at.to_string());

    Ok(Response::new().add_event(event))
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
    price_validate(&bid.price, &params)?;
    params.bid_expiry.is_valid(&env.block, bid.expires_at)?;

    let mut response = Response::new();
    let bid_key = bid_key(bid.token_id.clone(), &bid.bidder);
    let ask_key = &bid.token_id;

    // If bid exists, refund the escrowed tokens
    if let Some(existing_bid) = bids().may_load(deps.storage, bid_key.clone())? {
        bids().remove(deps.storage, bid_key)?;
        transfer_token(existing_bid.price, bid.bidder.to_string(), "refund-bidder", &mut response)?;
    }

    let matching_ask = match_bid(deps.as_ref(), env, &bid, &mut response)?;

    // If existing ask found, finalize the sale
    match matching_ask {
        Some(ask) => {
            asks().remove(deps.storage, ask_key.clone())?;
            finalize_sale(
                deps.as_ref(),
                payment_amount,
                &ask,
                &bid,
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

// /// Seller can accept a bid which transfers funds as well as the token. The bid may or may not be associated with an ask.
// pub fn execute_accept_bid(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     token_id: TokenId,
//     bidder: Addr,
//     finder: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     only_owner(deps.as_ref(), &info, &collection, token_id)?;

//     let bid_key = bid_key(&collection, token_id, &bidder);
//     let ask_key = ask_key(&collection, token_id);

//     let bid = bids().load(deps.storage, bid_key.clone())?;
//     if bid.is_expired(&env.block) {
//         return Err(ContractError::BidExpired {});
//     }

//     let ask = if let Some(existing_ask) = asks().may_load(deps.storage, ask_key.clone())? {
//         if existing_ask.is_expired(&env.block) {
//             return Err(ContractError::AskExpired {});
//         }
//         if !existing_ask.is_active {
//             return Err(ContractError::AskNotActive {});
//         }
//         asks().remove(deps.storage, ask_key)?;
//         existing_ask
//     } else {
//         // Create a temporary Ask
//         Ask {
//             sale_type: SaleType::Auction,
//             collection: collection.clone(),
//             token_id,
//             price: bid.price,
//             expires_at: bid.expires_at,
//             is_active: true,
//             seller: info.sender,
//             funds_recipient: None,
//             reserve_for: None,
//             finders_fee_bps: bid.finders_fee_bps,
//         }
//     };

//     // Remove accepted bid
//     bids().remove(deps.storage, bid_key)?;

//     let mut res = Response::new();

//     // Transfer funds and NFT
//     finalize_sale(
//         deps.as_ref(),
//         ask,
//         bid.price,
//         bidder.clone(),
//         finder,
//         &mut res,
//     )?;

//     let event = Event::new("accept-bid")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("token_id", token_id.to_string())
//         .add_attribute("bidder", bidder)
//         .add_attribute("price", bid.price.to_string());

//     Ok(res.add_event(event))
// }

// /// Place a collection bid (limit order) across an entire collection
// pub fn execute_set_collection_bid(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     finders_fee_bps: Option<u64>,
//     expires: Timestamp,
// ) -> Result<Response, ContractError> {
//     let params = PARAMS.load(deps.storage)?;
//     let price = must_pay(&info, NATIVE_DENOM)?;
//     if price < params.min_price {
//         return Err(ContractError::PriceTooSmall(price));
//     }
//     params.bid_expiry.is_valid(&env.block, expires)?;

//     let bidder = info.sender;
//     let mut res = Response::new();

//     let key = collection_bid_key(&collection, &bidder);

//     let existing_bid = collection_bids().may_load(deps.storage, key.clone())?;
//     if let Some(bid) = existing_bid {
//         collection_bids().remove(deps.storage, key.clone())?;
//         let refund_bidder_msg = BankMsg::Send {
//             to_address: bid.bidder.to_string(),
//             amount: vec![coin(bid.price.u128(), NATIVE_DENOM)],
//         };
//         res = res.add_message(refund_bidder_msg);
//     }

//     let collection_bid = CollectionBid {
//         collection: collection.clone(),
//         bidder: bidder.clone(),
//         price,
//         finders_fee_bps,
//         expires_at: expires,
//     };
//     collection_bids().save(deps.storage, key, &collection_bid)?;

//     let hook = prepare_collection_bid_hook(deps.as_ref(), &collection_bid, HookAction::Create)?;

//     let event = Event::new("set-collection-bid")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("bidder", bidder)
//         .add_attribute("bid_price", price.to_string())
//         .add_attribute("expires", expires.to_string());

//     Ok(res.add_event(event).add_submessages(hook))
// }

// /// Remove an existing collection bid (limit order)
// pub fn execute_remove_collection_bid(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     collection: Addr,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     let bidder = info.sender;

//     let key = collection_bid_key(&collection, &bidder);

//     let collection_bid = collection_bids().load(deps.storage, key.clone())?;
//     collection_bids().remove(deps.storage, key)?;

//     let refund_bidder_msg = BankMsg::Send {
//         to_address: collection_bid.bidder.to_string(),
//         amount: vec![coin(collection_bid.price.u128(), NATIVE_DENOM)],
//     };

//     let hook = prepare_collection_bid_hook(deps.as_ref(), &collection_bid, HookAction::Delete)?;

//     let event = Event::new("remove-collection-bid")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("bidder", bidder);

//     let res = Response::new()
//         .add_message(refund_bidder_msg)
//         .add_event(event)
//         .add_submessages(hook);

//     Ok(res)
// }

// /// Owner/seller of an item in a collection can accept a collection bid which transfers funds as well as a token
// pub fn execute_accept_collection_bid(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     token_id: TokenId,
//     bidder: Addr,
//     finder: Option<Addr>,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     only_owner(deps.as_ref(), &info, &collection, token_id)?;

//     let bid_key = collection_bid_key(&collection, &bidder);
//     let ask_key = ask_key(&collection, token_id);

//     let bid = collection_bids().load(deps.storage, bid_key.clone())?;
//     if bid.is_expired(&env.block) {
//         return Err(ContractError::BidExpired {});
//     }
//     collection_bids().remove(deps.storage, bid_key)?;

//     let ask = if let Some(existing_ask) = asks().may_load(deps.storage, ask_key.clone())? {
//         if existing_ask.is_expired(&env.block) {
//             return Err(ContractError::AskExpired {});
//         }
//         if !existing_ask.is_active {
//             return Err(ContractError::AskNotActive {});
//         }
//         asks().remove(deps.storage, ask_key)?;
//         existing_ask
//     } else {
//         // Create a temporary Ask
//         Ask {
//             sale_type: SaleType::Auction,
//             collection: collection.clone(),
//             token_id,
//             price: bid.price,
//             expires_at: bid.expires_at,
//             is_active: true,
//             seller: info.sender.clone(),
//             funds_recipient: None,
//             reserve_for: None,
//             finders_fee_bps: bid.finders_fee_bps,
//         }
//     };

//     let mut res = Response::new();

//     // Transfer funds and NFT
//     finalize_sale(
//         deps.as_ref(),
//         ask,
//         bid.price,
//         bidder.clone(),
//         finder,
//         &mut res,
//     )?;

//     let event = Event::new("accept-collection-bid")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("token_id", token_id.to_string())
//         .add_attribute("bidder", bidder)
//         .add_attribute("seller", info.sender.to_string())
//         .add_attribute("price", bid.price.to_string());

//     Ok(res.add_event(event))
// }

// /// Synchronizes the active state of an ask based on token ownership.
// /// This is a privileged operation called by an operator to update an ask when a transfer happens.
// pub fn execute_sync_ask(
//     deps: DepsMut,
//     info: MessageInfo,
//     collection: Addr,
//     token_id: TokenId,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     only_operator(deps.storage, &info)?;

//     let key = ask_key(&collection, token_id);

//     let mut ask = asks().load(deps.storage, key.clone())?;
//     let res =
//         Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id.to_string(), false)?;
//     let new_is_active = res.owner == ask.seller;
//     if new_is_active == ask.is_active {
//         return Err(ContractError::AskUnchanged {});
//     }
//     ask.is_active = new_is_active;
//     asks().save(deps.storage, key, &ask)?;

//     let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Update)?;

//     let event = Event::new("update-ask-state")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("token_id", token_id.to_string())
//         .add_attribute("is_active", ask.is_active.to_string());

//     Ok(Response::new().add_event(event).add_submessages(hook))
// }

// /// Privileged operation to remove a stale bid. Operators can call this to remove and refund bids that are still in the
// /// state after they have expired. As a reward they get a governance-determined percentage of the bid price.
// pub fn execute_remove_stale_bid(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     token_id: TokenId,
//     bidder: Addr,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     let operator = only_operator(deps.storage, &info)?;

//     let bid_key = bid_key(&collection, token_id, &bidder);
//     let bid = bids().load(deps.storage, bid_key.clone())?;

//     let params = PARAMS.load(deps.storage)?;
//     let stale_time = (Expiration::AtTime(bid.expires_at) + params.stale_bid_duration)?;
//     if !stale_time.is_expired(&env.block) {
//         return Err(ContractError::BidNotStale {});
//     }

//     // bid is stale, refund bidder and reward operator
//     bids().remove(deps.storage, bid_key)?;

//     let reward = bid.price * params.bid_removal_reward_percent / Uint128::from(100u128);

//     let bidder_msg = BankMsg::Send {
//         to_address: bid.bidder.to_string(),
//         amount: vec![coin((bid.price - reward).u128(), NATIVE_DENOM)],
//     };
//     let operator_msg = BankMsg::Send {
//         to_address: operator.to_string(),
//         amount: vec![coin(reward.u128(), NATIVE_DENOM)],
//     };

//     let hook = prepare_bid_hook(deps.as_ref(), &bid, HookAction::Delete)?;

//     let event = Event::new("remove-stale-bid")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("token_id", token_id.to_string())
//         .add_attribute("bidder", bidder.to_string())
//         .add_attribute("operator", operator.to_string())
//         .add_attribute("reward", reward.to_string());

//     Ok(Response::new()
//         .add_event(event)
//         .add_message(bidder_msg)
//         .add_message(operator_msg)
//         .add_submessages(hook))
// }

// /// Privileged operation to remove a stale colllection bid. Operators can call this to remove and refund bids that are still in the
// /// state after they have expired. As a reward they get a governance-determined percentage of the bid price.
// pub fn execute_remove_stale_collection_bid(
//     deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     bidder: Addr,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     let operator = only_operator(deps.storage, &info)?;

//     let key = collection_bid_key(&collection, &bidder);
//     let collection_bid = collection_bids().load(deps.storage, key.clone())?;

//     let params = PARAMS.load(deps.storage)?;
//     let stale_time = (Expiration::AtTime(collection_bid.expires_at) + params.stale_bid_duration)?;
//     if !stale_time.is_expired(&env.block) {
//         return Err(ContractError::BidNotStale {});
//     }

//     // collection bid is stale, refund bidder and reward operator
//     collection_bids().remove(deps.storage, key)?;

//     let reward = collection_bid.price * params.bid_removal_reward_percent / Uint128::from(100u128);

//     let bidder_msg = BankMsg::Send {
//         to_address: collection_bid.bidder.to_string(),
//         amount: vec![coin((collection_bid.price - reward).u128(), NATIVE_DENOM)],
//     };
//     let operator_msg = BankMsg::Send {
//         to_address: operator.to_string(),
//         amount: vec![coin(reward.u128(), NATIVE_DENOM)],
//     };

//     let hook = prepare_collection_bid_hook(deps.as_ref(), &collection_bid, HookAction::Delete)?;

//     let event = Event::new("remove-stale-collection-bid")
//         .add_attribute("collection", collection.to_string())
//         .add_attribute("bidder", bidder.to_string())
//         .add_attribute("operator", operator.to_string())
//         .add_attribute("reward", reward.to_string());

//     Ok(Response::new()
//         .add_event(event)
//         .add_message(bidder_msg)
//         .add_message(operator_msg)
//         .add_submessages(hook))
// }

/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    payment_amount: Uint128,
    ask: &Ask,
    bid: &Bid,
    params: &Params,
    res: &mut Response,
) -> StdResult<()> {
    payout(deps, payment_amount, &ask, &params, res)?;

    transfer_nft(&ask.token_id, &bid.bidder, &params.cw721_address, res)?;

    let event = Event::new("finalize-sale")
        .add_attribute("collection", params.cw721_address.to_string())
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller.to_string())
        .add_attribute("buyer", bid.bidder.to_string())
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
        Some(royalty) => Some((ask.price.amount * royalty.share, &royalty.payment_address)),
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
    transfer_token(
        coin(seller_amount.u128(), &params.denom),
        ask.seller.to_string(),
        "payout-seller",
        response
    )?;

    Ok(())
}

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

fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask.token_id.clone(), ask)
}

/// Checks to enforce only NFT owner can call
fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &String,
) -> Result<OwnerOfResponse, ContractError> {
    let res =
        Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(res)
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

// /// Checks to enforce only privileged operators
// fn only_operator(store: &dyn Storage, info: &MessageInfo) -> Result<Addr, ContractError> {
//     let params = PARAMS.load(store)?;
//     if !params
//         .operators
//         .iter()
//         .any(|a| a.as_ref() == info.sender.as_ref())
//     {
//         return Err(ContractError::UnauthorizedOperator {});
//     }

//     Ok(info.sender.clone())
// }

// enum HookReply {
//     Ask = 1,
//     Sale,
//     Bid,
//     CollectionBid,
// }

// impl From<u64> for HookReply {
//     fn from(item: u64) -> Self {
//         match item {
//             1 => HookReply::Ask,
//             2 => HookReply::Sale,
//             3 => HookReply::Bid,
//             4 => HookReply::CollectionBid,
//             _ => panic!("invalid reply type"),
//         }
//     }
// }

// #[cfg_attr(not(feature = "library"), entry_point)]
// pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
//     match HookReply::from(msg.id) {
//         HookReply::Ask => {
//             let res = Response::new()
//                 .add_attribute("action", "ask-hook-failed")
//                 .add_attribute("error", msg.result.unwrap_err());
//             Ok(res)
//         }
//         HookReply::Sale => {
//             let res = Response::new()
//                 .add_attribute("action", "sale-hook-failed")
//                 .add_attribute("error", msg.result.unwrap_err());
//             Ok(res)
//         }
//         HookReply::Bid => {
//             let res = Response::new()
//                 .add_attribute("action", "bid-hook-failed")
//                 .add_attribute("error", msg.result.unwrap_err());
//             Ok(res)
//         }
//         HookReply::CollectionBid => {
//             let res = Response::new()
//                 .add_attribute("action", "collection-bid-hook-failed")
//                 .add_attribute("error", msg.result.unwrap_err());
//             Ok(res)
//         }
//     }
// }

// fn prepare_ask_hook(deps: Deps, ask: &Ask, action: HookAction) -> StdResult<Vec<SubMsg>> {
//     let submsgs = ASK_HOOKS.prepare_hooks(deps.storage, |h| {
//         let msg = AskHookMsg { ask: ask.clone() };
//         let execute = WasmMsg::Execute {
//             contract_addr: h.to_string(),
//             msg: msg.into_binary(action.clone())?,
//             funds: vec![],
//         };
//         Ok(SubMsg::reply_on_error(execute, HookReply::Ask as u64))
//     })?;

//     Ok(submsgs)
// }

// fn prepare_sale_hook(deps: Deps, ask: &Ask, buyer: Addr) -> StdResult<Vec<SubMsg>> {
//     let submsgs = SALE_HOOKS.prepare_hooks(deps.storage, |h| {
//         let msg = SaleHookMsg {
//             collection: ask.collection.to_string(),
//             token_id: ask.token_id,
//             price: coin(ask.price.clone().u128(), NATIVE_DENOM),
//             seller: ask.seller.to_string(),
//             buyer: buyer.to_string(),
//         };
//         let execute = WasmMsg::Execute {
//             contract_addr: h.to_string(),
//             msg: msg.into_binary()?,
//             funds: vec![],
//         };
//         Ok(SubMsg::reply_on_error(execute, HookReply::Sale as u64))
//     })?;

//     Ok(submsgs)
// }

// fn prepare_bid_hook(deps: Deps, bid: &Bid, action: HookAction) -> StdResult<Vec<SubMsg>> {
//     let submsgs = BID_HOOKS.prepare_hooks(deps.storage, |h| {
//         let msg = BidHookMsg { bid: bid.clone() };
//         let execute = WasmMsg::Execute {
//             contract_addr: h.to_string(),
//             msg: msg.into_binary(action.clone())?,
//             funds: vec![],
//         };
//         Ok(SubMsg::reply_on_error(execute, HookReply::Bid as u64))
//     })?;

//     Ok(submsgs)
// }

// fn prepare_collection_bid_hook(
//     deps: Deps,
//     collection_bid: &CollectionBid,
//     action: HookAction,
// ) -> StdResult<Vec<SubMsg>> {
//     let submsgs = COLLECTION_BID_HOOKS.prepare_hooks(deps.storage, |h| {
//         let msg = CollectionBidHookMsg {
//             collection_bid: collection_bid.clone(),
//         };
//         let execute = WasmMsg::Execute {
//             contract_addr: h.to_string(),
//             msg: msg.into_binary(action.clone())?,
//             funds: vec![],
//         };
//         Ok(SubMsg::reply_on_error(
//             execute,
//             HookReply::CollectionBid as u64,
//         ))
//     })?;

//     Ok(submsgs)
// }

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
