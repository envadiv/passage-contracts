use crate::msg::{
    QueryMsg, QueryOptions, TokenTimestampOffset, TokenPriceOffset,
    AuctionResponse, AuctionsResponse, ConfigResponse
};
use crate::state::{
    CONFIG, TokenId, auctions, AuctionStatus
};
use crate::helpers::option_bool_to_order;
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, StdResult, Uint128};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Config { } => to_binary(&query_config(deps)?),
        QueryMsg::Auction {
            token_id,
        } => to_binary(&query_auction(deps, env, token_id)?),
        QueryMsg::AuctionsByStartTime {
            query_options
        } => to_binary(&query_auctions_by_start_time(
            deps,
            &query_options,
        )?),
        QueryMsg::AuctionsByEndTime {
            query_options
        } => to_binary(&query_auctions_by_end_time(
            deps,
            &query_options,
        )?),
        QueryMsg::AuctionsByHighestBidPrice {
            query_options
        } => to_binary(&query_auctions_by_highest_bid_price(
            deps,
            &query_options,
        )?),
        QueryMsg::AuctionsBySellerEndTime {
            seller,
            query_options
        } => to_binary(&query_auctions_by_seller_end_time(
            deps,
            api.addr_validate(&seller)?,
            &query_options,
        )?),
        QueryMsg::AuctionsByBidderEndTime {
            bidder,
            query_options
        } => to_binary(&query_auctions_by_highest_bidder_end_time(
            deps,
            api.addr_validate(&bidder)?,
            &query_options,
        )?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse { config })
}

pub fn query_auction(deps: Deps, env: Env, token_id: TokenId) -> StdResult<AuctionResponse> {
    let auction = auctions().may_load(deps.storage, token_id)?;
    let config = CONFIG.load(deps.storage)?;

    let mut auction_status: Option<AuctionStatus> = None;
    let mut is_reserve_price_met: Option<bool> = None;
    let mut next_bid_min: Option<Uint128> = None;

    if let Some(_auction) = &auction {
        auction_status = Some(_auction.get_auction_status(&env.block.time, config.closed_duration));
        is_reserve_price_met = Some(_auction.is_reserve_price_met());
        next_bid_min = Some(_auction.get_next_bid_min(config.min_bid_increment));
    }

    Ok(AuctionResponse { auction, auction_status, is_reserve_price_met, next_bid_min })
}

pub fn query_auctions_by_start_time(
    deps: Deps,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .start_time
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, auction)) => match query_options.filter_expiry {
                Some(ts) => ts < auction.end_time,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auctions_by_end_time(
    deps: Deps,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .end_time
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, auction)) => match query_options.filter_expiry {
                Some(ts) => ts < auction.end_time,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auctions_by_highest_bid_price(
    deps: Deps,
    query_options: &QueryOptions<TokenPriceOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.price.u128(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .highest_bid_price
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, auction)) => match query_options.filter_expiry {
                Some(ts) => ts < auction.end_time,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auctions_by_seller_end_time(
    deps: Deps,
    seller: Addr,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .seller_end_time
        .sub_prefix(seller.to_string())
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, auction)) => match query_options.filter_expiry {
                Some(ts) => ts < auction.end_time,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auctions_by_highest_bidder_end_time(
    deps: Deps,
    bidder: Addr,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .highest_bidder_end_time
        .sub_prefix(bidder.to_string())
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, auction)) => match query_options.filter_expiry {
                Some(ts) => ts < auction.end_time,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}
