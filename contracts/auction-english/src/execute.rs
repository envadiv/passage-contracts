#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Decimal, DepsMut, Env, Event, MessageInfo, Uint128, Response};
use cw2::set_contract_version;
use cw_utils::{maybe_addr, must_pay, nonpayable};

use crate::error::ContractError;
use crate::helpers::{
    map_validate, finalize_sale, price_validate, only_seller,
    only_operator, transfer_nft, transfer_token, validate_auction_times
};
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::{
    Config, CONFIG, TokenId,
    Auction, AuctionStatus, auctions, AuctionBid,
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

    let api = deps.api;
    let config = Config {
        cw721_address: api.addr_validate(&msg.cw721_address)?,
        denom: msg.denom,
        collector_address: api.addr_validate(&msg.collector_address)?,
        trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        operators: map_validate(deps.api, &msg.operators)?,
        min_price: msg.min_price,
        min_bid_increment: msg.min_bid_increment,
        min_duration: msg.min_duration,
        max_duration: msg.max_duration,
        closed_duration: msg.closed_duration,
        buffer_duration: msg.buffer_duration,
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
            collector_address,
            trading_fee_bps,
            operators,
            min_price,
            min_bid_increment,
            min_duration,
            max_duration,
            closed_duration,
            buffer_duration,
        } => execute_update_config(
            deps,
            env,
            info,
            collector_address,
            trading_fee_bps,
            operators,
            min_price,
            min_bid_increment,
            min_duration,
            max_duration,
            closed_duration,
            buffer_duration,
        ),
        ExecuteMsg::SetAuction {
            token_id,
            start_time,
            end_time,
            starting_price,
            reserve_price,
            funds_recipient,
        } => execute_set_auction(
            deps,
            env,
            info,
            Auction {
                token_id,
                seller: message_info.sender,
                start_time,
                end_time,
                starting_price,
                reserve_price,
                funds_recipient: maybe_addr(api, funds_recipient)?,
                highest_bid: None
            },
        ),
        ExecuteMsg::SetAuctionBid {
            token_id,
            price,
        } => execute_set_auction_bid(
            deps,
            env,
            info,
            token_id,
            AuctionBid {
                bidder: message_info.sender,
                price,
            },
        ),
        ExecuteMsg::CloseAuction {
            token_id,
            accept_highest_bid,
        } => execute_close_auction(
            deps,
            env,
            info,
            token_id,
            accept_highest_bid,
        ),
        ExecuteMsg::FinalizeAuction {
            token_id,
        } => execute_finalize_auction(
            deps,
            env,
            info,
            token_id,
        ),
        ExecuteMsg::VoidAuction {
            token_id,
        } => execute_void_auction(
            deps,
            env,
            info,
            token_id,
        ),
    }
}

/// An operator may update the marketplace config
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    collector_address: Option<String>,
    trading_fee_bps: Option<u64>,
    operators: Option<Vec<String>>,
    min_price: Option<Uint128>,
    min_bid_increment: Option<Uint128>,
    min_duration: Option<u64>,
    max_duration: Option<u64>,
    closed_duration: Option<u64>,
    buffer_duration: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    
    only_operator(&info, &config)?;
    
    if let Some(_collector_address) = collector_address {
        config.collector_address = deps.api.addr_validate(&_collector_address)?;
    }
    if let Some(_trading_fee_bps) = trading_fee_bps {
        config.trading_fee_percent = Decimal::percent(_trading_fee_bps);
    }
    if let Some(_operators) = operators {
        config.operators = map_validate(deps.api, &_operators)?;
    }
    if let Some(_min_price) = min_price {
        config.min_price = _min_price;
    }
    if let Some(_min_bid_increment) = min_bid_increment {
        config.min_bid_increment = _min_bid_increment;
    }
    if let Some(_min_duration) = min_duration {
        config.min_duration = _min_duration;
    }
    if let Some(_max_duration) = max_duration {
        config.max_duration = _max_duration;
    }
    if let Some(_closed_duration) = closed_duration {
        config.closed_duration = _closed_duration;
    }
    if let Some(_buffer_duration) = buffer_duration {
        config.buffer_duration = _buffer_duration;
    }
    
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new())
}

/// Owner of an NFT can create auction to begin accepting bids
pub fn execute_set_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    auction: Auction,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    
    let config = CONFIG.load(deps.storage)?;
    validate_auction_times(&auction, &config, &env.block.time)?;
    
    price_validate(&auction.starting_price, &config)?;
    if let Some(_reserve_price) = &auction.reserve_price {
        price_validate(&_reserve_price, &config)?;
        if _reserve_price.amount < auction.starting_price.amount {
            return Err(ContractError::InvalidReservePrice(_reserve_price.amount, auction.starting_price.amount));
        }
    }

    let existing_auction = auctions().may_load(deps.storage, auction.token_id.clone())?;
    if let Some(_existing_auction) = existing_auction {
        return Err(ContractError::AlreadyExists(auction.token_id.clone()));
    }

    auctions().save(deps.storage, auction.token_id.clone(), &auction)?;

    let mut response = Response::new();

    transfer_nft(&auction.token_id, &env.contract.address, &config.cw721_address, &mut response)?;

    let event = Event::new("set-auction")
        .add_attribute("collection", config.cw721_address.to_string())
        .add_attribute("token_id", auction.token_id.to_string())
        .add_attribute("seller", auction.seller)
        .add_attribute("start_time", auction.start_time.to_string())
        .add_attribute("end_time", auction.end_time.to_string())
        .add_attribute("starting_price", auction.starting_price.to_string());

    Ok(response.add_event(event))
}

/// Places a bid for an NFT on an existing auction
pub fn execute_set_auction_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
    auction_bid: AuctionBid,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    let config = CONFIG.load(deps.storage)?; 

    // Validate auction exists, and is open
    let mut auction = auctions().load(deps.storage, token_id.clone())?;
    let auction_status = auction.get_auction_status(&env.block.time, config.closed_duration);
    match &auction_status {
        AuctionStatus::Open => {},
        _ => return Err(ContractError::InvalidStatus(auction_status.to_string())),
    }

    // Validate bid is higher than the minimum viable bid
    if auction_bid.price.amount < auction.get_next_bid_min(config.min_bid_increment) {
        return Err(ContractError::BidTooLow {});
    }
    
    // If previous bid exists, refund it
    if let Some(prev_highest_bid) = &auction.highest_bid {
        transfer_token(
            prev_highest_bid.price.clone(),
            prev_highest_bid.bidder.to_string(),
            "refund-auction-bidder",
            &mut response,
        )?;
    }

    price_validate(&auction_bid.price, &config)?;
    let payment_amount = must_pay(&info, &config.denom)?;
    if auction_bid.price.amount != payment_amount  {
        return Err(ContractError::IncorrectBidPayment(auction_bid.price.amount, payment_amount));
    }

    auction.highest_bid = Some(auction_bid.clone());
    
    // If auction end time is within buffer_duration, then update the end time
    let new_auction_end_time = env.block.time.plus_seconds(config.buffer_duration);
    if new_auction_end_time > auction.end_time {
        auction.end_time = new_auction_end_time;
    }
    
    auctions().save(deps.storage, auction.token_id.clone(), &auction)?;

    let event = Event::new("set-auction-bid")
        .add_attribute("token_id", &token_id.to_string())
        .add_attribute("bidder", &auction_bid.bidder)
        .add_attribute("price", &auction_bid.price.to_string());
    response.events.push(event);

    Ok(response)
}

/// Creator of an auction can close it prematurely if reserve price is not met
pub fn execute_close_auction(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: TokenId,
    accept_highest_bid: bool,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Validate auction exists, and if it exists, that it is being closed by the seller
    let auction = auctions().load(deps.storage, token_id.clone())?;
    only_seller(&info, &auction.seller)?;

    // If reserve price has been met, seller cannot close auction
    if auction.is_reserve_price_met() {
        return Err(ContractError::ReservePriceRestriction(
            "must finalize auction when reserve price is met".to_string(),
        ));
    }

    let mut response = Response::new();
    let config = CONFIG.load(deps.storage)?;

    let is_sale = auction.highest_bid.is_some() && accept_highest_bid;
    if is_sale {
        // if accept_highest_bid is true and highest bid exists, then perform sale
        let bid = auction.highest_bid.as_ref().unwrap();
        finalize_sale(
            deps.as_ref(),
            &bid.bidder,
            &auction.token_id,
            bid.price.amount,
            &auction.get_recipient(),
            &config,
            &mut response,
        )?;
    } else {
        // if sale does not occur return NFT to seller, then refund highest_bid if it exists
        transfer_nft(&auction.token_id, &auction.seller, &config.cw721_address, &mut response)?;
        if auction.highest_bid.is_some() {
            let bid = auction.highest_bid.unwrap();
            transfer_token(
                bid.price.clone(),
                bid.bidder.to_string(),
                "refund-auction-bidder",
                &mut response,
            )?;
        }   
    }

    auctions().remove(deps.storage, token_id)?;

    let event = Event::new("close-auction")
        .add_attribute("collection", &config.cw721_address.to_string())
        .add_attribute("token_id", &auction.token_id.to_string())
        .add_attribute("is_sale", &is_sale.to_string());
    
    Ok(response.add_event(event))
}

/// Anyone can finalize an expired auction where the reserve price has been met
pub fn execute_finalize_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    // Validate auction exists
    let auction = auctions().load(deps.storage, token_id.clone())?;

    // Validate reserve price is met
    if !auction.is_reserve_price_met() {
        return Err(ContractError::ReservePriceRestriction(
            "reserve price not met".to_string(),
        ));
    }

    // Validate Auction is either Closed or Expired
    let config = CONFIG.load(deps.storage)?;
    let auction_status = auction.get_auction_status(&env.block.time, config.closed_duration);
    match &auction_status {
        AuctionStatus::Closed | AuctionStatus::Expired => {},
        _ => return Err(ContractError::InvalidStatus(auction_status.to_string())),
    }

    // Perform sale
    let mut response = Response::new();
    let bid = auction.highest_bid.as_ref().unwrap();
    finalize_sale(
        deps.as_ref(),
        &bid.bidder,
        &auction.token_id,
        bid.price.amount,
        &auction.get_recipient(),
        &config,
        &mut response,
    )?;

    auctions().remove(deps.storage, token_id)?;

    let event = Event::new("finalize-auction")
        .add_attribute("collection", &config.cw721_address.to_string())
        .add_attribute("token_id", &auction.token_id.to_string());
    
    Ok(response.add_event(event))
}

/// If an auction is expired, and the seller has not made a determination, the bidder may remove their bid
pub fn execute_void_auction(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    // Only the bidder can remove their bid
    let auction = auctions().load(deps.storage, token_id.clone())?;
    let highest_bid = auction.highest_bid.as_ref().unwrap();
    if highest_bid.bidder != info.sender {
        return Err(ContractError::Unauthorized(String::from("only the bidder can remove their bid")));
    }
    
    // Validate the Auction is Expired
    let config = CONFIG.load(deps.storage)?; 
    let auction_status = auction.get_auction_status(&env.block.time, config.closed_duration);
    match &auction_status {
        AuctionStatus::Expired => {},
        _ => return Err(ContractError::InvalidStatus(auction_status.to_string())),
    }
    
    let mut response = Response::new();
    // Refund the bidder the bid amount
    transfer_token(
        highest_bid.price.clone(),
        highest_bid.bidder.to_string(),
        "refund-auction-bidder",
        &mut response,
    )?;
    // Return the NFT to the seller
    transfer_nft(&auction.token_id, &auction.seller, &config.cw721_address, &mut response)?;
    // Remove the auction
    auctions().remove(deps.storage, token_id)?;

    let event = Event::new("void-auction")
        .add_attribute("token_id", &auction.token_id.to_string())
        .add_attribute("seller", &auction.seller.to_string())
        .add_attribute("bidder", &highest_bid.bidder.to_string());
    response.events.push(event);

    Ok(response)
}