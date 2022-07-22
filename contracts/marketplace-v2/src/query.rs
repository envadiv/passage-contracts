use crate::msg::{
    QueryMsg, AskResponse, AsksResponse, QueryOptions, AskExpiryOffset, AskPriceOffset,
    AskCountResponse, BidResponse, BidsResponse, BidExpiryOffset, BidTokenPriceOffset,
    ParamsResponse, CollectionBidResponse, CollectionBidsResponse, CollectionBidPriceOffset,
    CollectionBidExpiryOffset, AuctionResponse, AuctionsResponse, AuctionStartingPriceOffset,
    AuctionReservePriceOffset, AuctionExpiryOffset, AuctionBidResponse, AuctionBidsResponse
};
use crate::state::{
    PARAMS, asks, TokenId, bids, bid_key, collection_bids, auctions, auction_bids, auction_bid_key
};
use crate::helpers::option_bool_to_order;
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::{Bound};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::Params { } => to_binary(&query_params(deps)?),
        QueryMsg::Ask {
            token_id,
        } => to_binary(&query_ask(deps, token_id)?),
        QueryMsg::AsksSortedByExpiry {
            query_options
        } => to_binary(&query_asks_sorted_by_expiry(
            deps,
            &query_options
        )?),
        QueryMsg::AsksSortedByPrice {
            query_options
        } => to_binary(&query_asks_sorted_by_price(
            deps,
            &query_options,
        )?),
        QueryMsg::AsksBySellerExpiry {
            seller,
            query_options,
        } => to_binary(&query_asks_by_seller_expiry(
            deps,
            api.addr_validate(&seller)?,
            &query_options,
        )?),
        QueryMsg::AskCount { } => to_binary(&query_ask_count(deps)?),
        QueryMsg::Bid {
            token_id,
            bidder,
        } => to_binary(&query_bid(
            deps,
            token_id,
            api.addr_validate(&bidder)?,
        )?),
        QueryMsg::BidsSortedByExpiry {
            query_options,
        } => to_binary(&query_bids_sorted_by_expiry(
            deps,
            &query_options,
        )?),
        QueryMsg::BidsByTokenPrice {
            token_id,
            query_options,
        } => to_binary(&query_bids_token_price(
            deps,
            token_id,
            &query_options,
        )?),
        QueryMsg::BidsByBidderExpiry {
            bidder,
            query_options,
        } => to_binary(&query_bids_by_bidder_expiry(
            deps,
            api.addr_validate(&bidder)?,
            &query_options
        )?),
        QueryMsg::CollectionBid { 
            bidder,
        } => to_binary(&query_collection_bid(
            deps,
            api.addr_validate(&bidder)?,
        )?),
        QueryMsg::CollectionBidsByPrice {
            query_options,
        } => to_binary(&query_collection_bids_by_price(
            deps,
            &query_options,
        )?),
        QueryMsg::CollectionBidsByExpiry {
            query_options,
        } => to_binary(&query_collection_bids_by_expiry(
            deps,
            &query_options,
        )?),
        QueryMsg::Auction {
            token_id,
        } => to_binary(&query_auction(deps, token_id)?),
        QueryMsg::AuctionsByStartingPrice {
            query_options
        } => to_binary(&query_auctions_by_starting_price(
            deps,
            &query_options,
        )?),
        QueryMsg::AuctionsByReservePrice {
            query_options
        } => to_binary(&query_auctions_by_reserve_price(
            deps,
            &query_options,
        )?),
        QueryMsg::AuctionsByExpiry {
            query_options
        } => to_binary(&query_auctions_by_expiry(
            deps,
            &query_options,
        )?),
        QueryMsg::AuctionBid {
            token_id,
            bidder,
        } => to_binary(&query_auction_bid(
            deps,
            token_id,
            api.addr_validate(&bidder)?,
        )?),
    }
}

pub fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
    let params = PARAMS.load(deps.storage)?;

    Ok(ParamsResponse { params })
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<AskResponse> {
    let ask = asks().may_load(deps.storage, token_id)?;

    Ok(AskResponse { ask })
}

pub fn query_asks_sorted_by_expiry(
    deps: Deps,
    query_options: &QueryOptions<AskExpiryOffset>
) -> StdResult<AsksResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.expires_at.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let asks = asks()
        .idx
        .expiry
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_asks_sorted_by_price(
    deps: Deps,
    query_options: &QueryOptions<AskPriceOffset>
) -> StdResult<AsksResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.price.u128(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let asks = asks()
        .idx
        .price
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_asks_by_seller_expiry(
    deps: Deps,
    seller: Addr,
    query_options: &QueryOptions<AskExpiryOffset>
) -> StdResult<AsksResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.expires_at.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let asks = asks()
        .idx
        .seller_expiry
        .sub_prefix(seller)
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_ask_count(deps: Deps) -> StdResult<AskCountResponse> {
    let count = asks()
        .keys_raw(deps.storage, None, None, Order::Ascending)
        .count() as u32;

    Ok(AskCountResponse { count })
}

pub fn query_bid(
    deps: Deps,
    token_id: TokenId,
    bidder: Addr,
) -> StdResult<BidResponse> {
    let bid = bids().may_load(deps.storage, bid_key(token_id, &bidder))?;

    Ok(BidResponse { bid })
}

pub fn query_bids_sorted_by_expiry(
    deps: Deps,
    query_options: &QueryOptions<BidExpiryOffset>
) -> StdResult<BidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.expires_at.seconds(), bid_key(offset.token_id.clone(), &offset.bidder)))
    });
    let order = option_bool_to_order(query_options.descending);

    let bids = bids()
        .idx
        .expiry
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_bids_token_price(
    deps: Deps,
    token_id: String,
    query_options: &QueryOptions<BidTokenPriceOffset>
) -> StdResult<BidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.price, bid_key(offset.token_id.clone(), &offset.bidder)))
    });
    let order = option_bool_to_order(query_options.descending);

    let bids = bids()
        .idx
        .token_price
        .sub_prefix(token_id)
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_bids_by_bidder_expiry(
    deps: Deps,
    bidder: Addr,
    query_options: &QueryOptions<BidExpiryOffset>
) -> StdResult<BidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.expires_at.seconds(), bid_key(offset.token_id.clone(), &offset.bidder)))
    });
    let order = option_bool_to_order(query_options.descending);

    let bids = bids()
        .idx
        .bidder_expiry
        .sub_prefix(bidder)
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_collection_bid(
    deps: Deps,
    bidder: Addr,
) -> StdResult<CollectionBidResponse> {
    let collection_bid = collection_bids().may_load(deps.storage, bidder)?;

    Ok(CollectionBidResponse { collection_bid })
}

pub fn query_collection_bids_by_price(
    deps: Deps,
    query_options: &QueryOptions<CollectionBidPriceOffset>
) -> StdResult<CollectionBidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.price, offset.bidder.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let collection_bids = collection_bids()
        .idx
        .price
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(CollectionBidsResponse { collection_bids })
}

pub fn query_collection_bids_by_expiry(
    deps: Deps,
    query_options: &QueryOptions<CollectionBidExpiryOffset>
) -> StdResult<CollectionBidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.expires_at.seconds(), offset.bidder.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let collection_bids = collection_bids()
        .idx
        .expiry
        .range(deps.storage, start, None, order)
        .filter(|item| match item {
            Ok((_, ask)) => match query_options.filter_expiry {
                Some(ts) => ts < ask.expires_at,
                _ => true,
            },
            Err(_) => true,
        })
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(CollectionBidsResponse { collection_bids })
}

pub fn query_auction(deps: Deps, token_id: TokenId) -> StdResult<AuctionResponse> {
    let auction = auctions().may_load(deps.storage, token_id)?;

    Ok(AuctionResponse { auction })
}

pub fn query_auctions_by_starting_price(
    deps: Deps,
    query_options: &QueryOptions<AuctionStartingPriceOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.starting_price.u128(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .starting_price
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auctions_by_reserve_price(
    deps: Deps,
    query_options: &QueryOptions<AuctionReservePriceOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.reserve_price.u128(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .reserve_price
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auctions_by_expiry(
    deps: Deps,
    query_options: &QueryOptions<AuctionExpiryOffset>
) -> StdResult<AuctionsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.expires_at.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let auctions = auctions()
        .idx
        .expiry
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AuctionsResponse { auctions })
}

pub fn query_auction_bid(
    deps: Deps,
    token_id: TokenId,
    bidder: Addr,
) -> StdResult<AuctionBidResponse> {
    let auction_bid = auction_bids().may_load(deps.storage, auction_bid_key(token_id, &bidder))?;

    Ok(AuctionBidResponse { auction_bid })
}