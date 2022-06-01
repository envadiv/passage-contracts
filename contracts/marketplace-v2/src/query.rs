use crate::msg::{
    QueryMsg, AskResponse, AsksResponse, QueryOptions, AskExpiryOffset, AskPriceOffset, AskSellerExpiryOffset,
    AskCountResponse, BidResponse, BidsResponse, BidExpiryOffset, BidTokenPriceOffset, BidBidderExpiryOffset
};
use crate::state::{
    PARAMS, asks, TokenId, bids, bid_key
};
use crate::helpers::option_bool_to_order;
use cosmwasm_std::{entry_point, to_binary, Addr, Binary, Deps, Env, Order, StdResult, Timestamp};
use cw_storage_plus::{Bound, PrefixBound};
use cw_utils::maybe_addr;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
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
        // QueryMsg::CollectionBid { collection, bidder } => to_binary(&query_collection_bid(
        //     deps,
        //     api.addr_validate(&collection)?,
        //     api.addr_validate(&bidder)?,
        // )?),
        // QueryMsg::CollectionBidsSortedByPrice {
        //     collection,
        //     start_after,
        //     limit,
        // } => to_binary(&query_collection_bids_sorted_by_price(
        //     deps,
        //     api.addr_validate(&collection)?,
        //     start_after,
        //     limit,
        // )?),
        // QueryMsg::ReverseCollectionBidsSortedByPrice {
        //     collection,
        //     start_before,
        //     limit,
        // } => to_binary(&reverse_query_collection_bids_sorted_by_price(
        //     deps,
        //     api.addr_validate(&collection)?,
        //     start_before,
        //     limit,
        // )?),
        // QueryMsg::CollectionBidsByBidder {
        //     bidder,
        //     start_after,
        //     limit,
        // } => to_binary(&query_collection_bids_by_bidder(
        //     deps,
        //     api.addr_validate(&bidder)?,
        //     start_after,
        //     limit,
        // )?),
        // QueryMsg::CollectionBidsByBidderSortedByExpiration {
        //     bidder,
        //     start_after,
        //     limit,
        // } => to_binary(&query_collection_bids_by_bidder_sorted_by_expiry(
        //     deps,
        //     api.addr_validate(&bidder)?,
        //     start_after,
        //     limit,
        // )?),
        // QueryMsg::AskHooks {} => to_binary(&ASK_HOOKS.query_hooks(deps)?),
        // QueryMsg::BidHooks {} => to_binary(&BID_HOOKS.query_hooks(deps)?),
        // QueryMsg::SaleHooks {} => to_binary(&SALE_HOOKS.query_hooks(deps)?),
        // QueryMsg::Params {} => to_binary(&query_params(deps)?),
    }
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
    query_options: &QueryOptions<AskSellerExpiryOffset>
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
    query_options: &QueryOptions<BidBidderExpiryOffset>
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

// pub fn query_collection_bid(
//     deps: Deps,
//     collection: Addr,
//     bidder: Addr,
// ) -> StdResult<CollectionBidResponse> {
//     let bid = collection_bids().may_load(deps.storage, collection_bid_key(&collection, &bidder))?;

//     Ok(CollectionBidResponse { bid })
// }

// pub fn query_collection_bids_sorted_by_price(
//     deps: Deps,
//     collection: Addr,
//     start_after: Option<CollectionBidOffset>,
//     limit: Option<u32>,
// ) -> StdResult<CollectionBidsResponse> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

//     let start: Option<Bound<(u128, CollectionBidKey)>> = match start_after {
//         Some(offset) => {
//             let bidder = deps.api.addr_validate(&offset.bidder)?;
//             Some(Bound::exclusive((
//                 offset.price.u128(),
//                 collection_bid_key(&collection, &bidder),
//             )))
//         }
//         None => None,
//     };

//     let bids = collection_bids()
//         .idx
//         .collection_price
//         .sub_prefix(collection)
//         .range(deps.storage, start, None, Order::Ascending)
//         .take(limit)
//         .map(|item| item.map(|(_, b)| b))
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(CollectionBidsResponse { bids })
// }

// pub fn reverse_query_collection_bids_sorted_by_price(
//     deps: Deps,
//     collection: Addr,
//     start_before: Option<CollectionBidOffset>,
//     limit: Option<u32>,
// ) -> StdResult<CollectionBidsResponse> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
//     let end: Option<Bound<(u128, CollectionBidKey)>> = match start_before {
//         Some(offset) => {
//             let bidder = deps.api.addr_validate(&offset.bidder)?;
//             Some(Bound::exclusive((
//                 offset.price.u128(),
//                 collection_bid_key(&collection, &bidder),
//             )))
//         }
//         None => None,
//     };

//     let bids = collection_bids()
//         .idx
//         .collection_price
//         .sub_prefix(collection)
//         .range(deps.storage, None, end, Order::Descending)
//         .take(limit)
//         .map(|item| item.map(|(_, b)| b))
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(CollectionBidsResponse { bids })
// }

// pub fn query_collection_bids_by_bidder(
//     deps: Deps,
//     bidder: Addr,
//     start_after: Option<CollectionOffset>,
//     limit: Option<u32>,
// ) -> StdResult<CollectionBidsResponse> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
//     let start: Option<Bound<(Addr, Addr)>> = match start_after {
//         Some(offset) => {
//             let collection = deps.api.addr_validate(&offset.collection)?;
//             Some(Bound::exclusive((collection, bidder.clone())))
//         }
//         None => None,
//     };
//     let bids = collection_bids()
//         .idx
//         .bidder
//         .prefix(bidder)
//         .range(deps.storage, start, None, Order::Ascending)
//         .take(limit)
//         .map(|item| item.map(|(_, b)| b))
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(CollectionBidsResponse { bids })
// }

// pub fn query_collection_bids_by_bidder_sorted_by_expiry(
//     deps: Deps,
//     bidder: Addr,
//     start_after: Option<CollectionBidOffset>,
//     limit: Option<u32>,
// ) -> StdResult<CollectionBidsResponse> {
//     let limit = limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;

//     let start = match start_after {
//         Some(offset) => {
//             let bidder = deps.api.addr_validate(&offset.bidder)?;
//             let collection = deps.api.addr_validate(&offset.collection)?;
//             let collection_bid =
//                 query_collection_bid(deps, collection.clone(), bidder.clone())?.bid;
//             let bound = match collection_bid {
//                 Some(collection_bid) => Some(Bound::exclusive((
//                     collection_bid.expires_at.seconds(),
//                     (collection, bidder),
//                 ))),
//                 None => None,
//             };
//             bound
//         }
//         None => None,
//     };

//     let bids = collection_bids()
//         .idx
//         .bidder_expires_at
//         .sub_prefix(bidder)
//         .range(deps.storage, start, None, Order::Ascending)
//         .take(limit)
//         .map(|item| item.map(|(_, b)| b))
//         .collect::<StdResult<Vec<_>>>()?;

//     Ok(CollectionBidsResponse { bids })
// }

// pub fn query_params(deps: Deps) -> StdResult<ParamsResponse> {
//     let config = SUDO_PARAMS.load(deps.storage)?;

//     Ok(ParamsResponse { params: config })
// }
