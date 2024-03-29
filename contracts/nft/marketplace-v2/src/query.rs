use crate::msg::{
    QueryMsg, AskResponse, AsksResponse, QueryOptions, TokenPriceOffset,
    AskCountResponse, BidResponse, BidsResponse, BidTokenPriceOffset,
    ConfigResponse, CollectionBidResponse, CollectionBidsResponse, CollectionBidPriceOffset, TokenAddrOffset,
};
use crate::state::{
    CONFIG, asks, TokenId, bids, bid_key, collection_bids,
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
        QueryMsg::Config { } => to_binary(&query_config(deps)?),
        QueryMsg::Ask {
            token_id,
        } => to_binary(&query_ask(deps, token_id)?),
        QueryMsg::AsksSortedByPrice {
            query_options
        } => to_binary(&query_asks_sorted_by_price(
            deps,
            &query_options,
        )?),
        QueryMsg::AsksBySeller {
            query_options,
        } => to_binary(&query_asks_by_seller(
            deps,
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
        QueryMsg::BidsByTokenPrice {
            token_id,
            query_options,
        } => to_binary(&query_bids_token_price(
            deps,
            token_id,
            &query_options,
        )?),
        QueryMsg::BidsByBidder {
            query_options,
        } => to_binary(&query_bids_by_bidder(
            deps,
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
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;

    Ok(ConfigResponse { config })
}

pub fn query_ask(deps: Deps, token_id: TokenId) -> StdResult<AskResponse> {
    let ask = asks().may_load(deps.storage, token_id)?;

    Ok(AskResponse { ask })
}

pub fn query_asks_sorted_by_price(
    deps: Deps,
    query_options: &QueryOptions<TokenPriceOffset>
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
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AsksResponse { asks })
}

pub fn query_asks_by_seller(
    deps: Deps,
    query_options: &QueryOptions<TokenAddrOffset>
) -> StdResult<AsksResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.address.clone(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let asks = asks()
        .idx
        .seller
        .range(deps.storage, start, None, order)
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
    let bid = bids().may_load(deps.storage, bid_key(&bidder, token_id))?;

    Ok(BidResponse { bid })
}

pub fn query_bids_token_price(
    deps: Deps,
    token_id: String,
    query_options: &QueryOptions<BidTokenPriceOffset>
) -> StdResult<BidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.price, bid_key(&offset.bidder, offset.token_id.clone())))
    });
    let order = option_bool_to_order(query_options.descending);

    let bids = bids()
        .idx
        .token_price
        .sub_prefix(token_id)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|item| item.map(|(_, b)| b))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(BidsResponse { bids })
}

pub fn query_bids_by_bidder(
    deps: Deps,
    query_options: &QueryOptions<TokenAddrOffset>
) -> StdResult<BidsResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive(bid_key(&offset.address, offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let bids = bids()
        .range(deps.storage, start, None, order)
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
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(CollectionBidsResponse { collection_bids })
}
