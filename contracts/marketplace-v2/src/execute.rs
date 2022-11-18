#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, Addr, Decimal, DepsMut, Env, Event, MessageInfo, StdError,
    Uint128, Response,
};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay, nonpayable};

use crate::error::ContractError;
use crate::helpers::{
    map_validate, ExpiryRange, finalize_sale, price_validate, store_bid,
    store_collection_bid, only_owner_or_seller, only_seller,
    only_operator, transfer_nft, transfer_token, match_bid,
};
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{
    Config, CONFIG, Ask, asks, TokenId, bid_key, bids, Expiration, Recipient,
    Bid, CollectionBid, collection_bids
};

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
    let config = Config {
        cw721_address: api.addr_validate(&msg.cw721_address)?,
        denom: msg.denom,
        collector_address: api.addr_validate(&msg.collector_address)?,
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        ask_expiry: msg.ask_expiry,
        bid_expiry: msg.bid_expiry,
        operators: map_validate(deps.api, &msg.operators)?,
        min_price: msg.min_price,
    };
    CONFIG.save(deps.storage, &config)?;

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
        ExecuteMsg::UpdateConfig {
            trading_fee_bps,
            ask_expiry,
            bid_expiry,
            operators,
            min_price,
        } => execute_update_config(
            deps,
            env,
            info,
            trading_fee_bps,
            ask_expiry,
            bid_expiry,
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
    }
}

/// An operator may update the marketplace config
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    trading_fee_bps: Option<u64>,
    ask_expiry: Option<ExpiryRange>,
    bid_expiry: Option<ExpiryRange>,
    operators: Option<Vec<String>>,
    min_price: Option<Uint128>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    only_operator(&info, &config)?;

    if let Some(_trading_fee_bps) = trading_fee_bps {
        config.trading_fee_percent = Decimal::percent(_trading_fee_bps);
    }
    if let Some(_ask_expiry) = ask_expiry {
        _ask_expiry.validate()?;
        config.ask_expiry = _ask_expiry;
    }
    if let Some(_bid_expiry) = bid_expiry {
        _bid_expiry.validate()?;
        config.bid_expiry = _bid_expiry;
    }
    if let Some(_operators) = operators {
        config.operators = map_validate(deps.api, &_operators)?;
    }
    if let Some(_min_price) = min_price {
        config.min_price = _min_price;
    }
    
    CONFIG.save(deps.storage, &config)?;
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
    
    let config = CONFIG.load(deps.storage)?;
    config.ask_expiry.is_valid(&env.block, ask.expires_at)?;
    price_validate(&ask.price, &config)?;

    let existing_ask = asks().load(deps.storage, ask.token_id.clone()).ok();
    only_owner_or_seller(
        deps.as_ref(),
        &info,
        &config.cw721_address,
        &ask.token_id,
        &existing_ask.clone().map_or(None, |a| Some(a.seller)),
    )?;

    // Upsert ask
    asks().update(
        deps.storage,
        ask.token_id.clone(),
        |_| -> Result<Ask, StdError> { Ok(ask.clone()) },
    )?;

    let mut response = Response::new();

    match existing_ask {
        None => transfer_nft(&ask.token_id, &env.contract.address, &config.cw721_address, &mut response)?,
        _ => (),
    }

    let event = Event::new("set-ask")
        .add_attribute("collection", config.cw721_address.to_string())
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
    only_seller(&info, &ask.seller)?;

    asks().remove(deps.storage, token_id.clone())?;

    let config = CONFIG.load(deps.storage)?;
    let mut response = Response::new();

    transfer_nft(&ask.token_id, &ask.seller, &config.cw721_address, &mut response)?;

    let event = Event::new("remove-ask")
        .add_attribute("collection", config.cw721_address.to_string())
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
    let config = CONFIG.load(deps.storage)?;

    let payment_amount = must_pay(&info, &config.denom)?;
    if bid.price.amount != payment_amount  {
        return Err(ContractError::IncorrectBidPayment(bid.price.amount, payment_amount));
    }
    price_validate(&bid.price, &config)?;
    config.bid_expiry.is_valid(&env.block, bid.expires_at)?;

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
                &bid.bidder,
                &ask.token_id,
                payment_amount,
                &ask.get_recipient(),
                &config,
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

    let config = CONFIG.load(deps.storage)?;
    let existing_ask = asks().may_load(deps.storage, token_id.clone())?;

    only_owner_or_seller(
        deps.as_ref(),
        &info,
        &config.cw721_address,
        &token_id,
        &existing_ask.clone().map_or(None, |a| Some(a.seller)),
    )?;

    // Remove ask if it exists, define recipient
    let payment_recipient = match existing_ask {
        Some(ask) => {
            asks().remove(deps.storage, ask.token_id.clone())?;
            ask.get_recipient()
        },
        None => info.sender,
    };

    let mut response = Response::new();

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        &bid.bidder,
        &token_id,
        bid.price.amount,
        &payment_recipient,
        &config,
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
    let config = CONFIG.load(deps.storage)?;
    
    // Escrows the amount (price * units)
    let payment_amount = must_pay(&info, &config.denom)?;
    price_validate(&collection_bid.price, &config)?;
    if Uint128::from(collection_bid.total_cost()) != payment_amount  {
        return Err(ContractError::IncorrectBidPayment(
            Uint128::from(collection_bid.total_cost()),
            payment_amount,
        ));
    }
    config.bid_expiry.is_valid(&env.block, collection_bid.expires_at)?;

    let collection_bid_key = collection_bid.bidder.clone();
    let mut response = Response::new();

    // If collection bid exists, refund the escrowed tokens
    if let Some(existing_bid) = collection_bids().may_load(deps.storage, collection_bid_key.clone())? {
        collection_bids().remove(deps.storage, collection_bid_key.clone())?;
        transfer_token(
            coin(existing_bid.total_cost(), existing_bid.price.denom),
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
        coin(collection_bid.total_cost(), collection_bid.price.denom),
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

    let config = CONFIG.load(deps.storage)?;
    let existing_ask = asks().may_load(deps.storage, token_id.clone())?;
    only_owner_or_seller(
        deps.as_ref(),
        &info,
        &config.cw721_address,
        &token_id,
        &existing_ask.clone().map_or(None, |a| Some(a.seller)),
    )?;

    // Remove ask if it exists, define recipient
    let payment_recipient = match existing_ask {
        Some(ask) => {
            asks().remove(deps.storage, ask.token_id.clone())?;
            ask.get_recipient()
        },
        None => info.sender,
    };

    match collection_bid.units {
        1 => {
            // Remove accepted collection bid when no units remain
            collection_bids().remove(deps.storage, collection_bid_key)?;
        },
        _ => {
            // Decrement the number of units on the collection bid by 1
            collection_bid.units -= 1;
            store_collection_bid(deps.storage, &collection_bid)?;
        }
    }

    let mut response = Response::new();

    // Transfer funds and NFT
    finalize_sale(
        deps.as_ref(),
        &collection_bid.bidder,
        &token_id,
        collection_bid.price.amount,
        &payment_recipient,
        &config,
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
