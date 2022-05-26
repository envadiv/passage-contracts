use cosmwasm_std::{Addr, BlockInfo, Decimal, Timestamp, Uint128, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use cw_utils::Duration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// use crate::hooks::Hooks;
use crate::helpers::ExpiryRange;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Params {
    /// The NFT contract
    pub cw721_address: Addr,
    /// The token used to pay for NFTs
    pub denom: String,
    /// Marketplace fee
    pub trading_fee_percent: Decimal,
    /// Valid time range for Asks
    /// (min, max) in seconds
    pub ask_expiry: ExpiryRange,
    /// Valid time range for Bids
    /// (min, max) in seconds
    pub bid_expiry: ExpiryRange,
    /// The admin addresses that have access to certain functionality
    pub admins: Vec<Addr>,
    /// Min value for a bid
    pub min_price: Uint128,
}

pub const PARAMS: Item<Params> = Item::new("params");

pub type TokenId = String;

pub trait Order {
    fn expires_at(&self) -> Timestamp;

    fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires_at() <= block.time
    }
}

/// Represents an ask on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub token_id: TokenId,
    pub seller: Addr,
    pub price: Coin,
    pub funds_recipient: Option<Addr>,
    pub reserve_for: Option<Addr>,
    pub expires_at: Timestamp,
}

/// Primary key for asks
pub type AskKey = TokenId;

/// Defines indices for accessing Asks
pub struct AskIndicies<'a> {
    pub expiry: MultiIndex<'a, u64, Ask, AskKey>,
    pub price: MultiIndex<'a, u128, Ask, AskKey>,
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
}

impl<'a> IndexList<Ask> for AskIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.expiry, &self.price, &self.seller];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        expiry: MultiIndex::new(|d: &Ask|  d.expires_at.nanos(), "asks", "asks__expiry"),
        price: MultiIndex::new(|d: &Ask|  d.price.amount.u128(), "asks", "asks__price"),
        seller: MultiIndex::new(|d: &Ask| d.seller.clone(), "asks", "asks__seller"),
    };
    IndexedMap::new("asks", indexes)
}

// /// Represents a bid (offer) on the marketplace
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct Bid {
//     pub collection: Addr,
//     pub token_id: TokenId,
//     pub bidder: Addr,
//     pub price: Uint128,
//     pub finders_fee_bps: Option<u64>,
//     pub expires_at: Timestamp,
// }

// impl Bid {
//     pub fn new(
//         collection: Addr,
//         token_id: TokenId,
//         bidder: Addr,
//         price: Uint128,
//         finders_fee_bps: Option<u64>,
//         expires: Timestamp,
//     ) -> Self {
//         Bid {
//             collection,
//             token_id,
//             bidder,
//             price,
//             finders_fee_bps,
//             expires_at: expires,
//         }
//     }
// }

// impl Order for Bid {
//     fn expires_at(&self) -> Timestamp {
//         self.expires_at
//     }
// }

// /// Primary key for bids: (collection, token_id, bidder)
// pub type BidKey = (Addr, TokenId, Addr);
// /// Convenience bid key constructor
// pub fn bid_key(collection: &Addr, token_id: TokenId, bidder: &Addr) -> BidKey {
//     (collection.clone(), token_id, bidder.clone())
// }

// /// Defines incides for accessing bids
// pub struct BidIndicies<'a> {
//     pub collection: MultiIndex<'a, Addr, Bid, BidKey>,
//     pub collection_token_id: MultiIndex<'a, (Addr, TokenId), Bid, BidKey>,
//     pub collection_price: MultiIndex<'a, (Addr, u128), Bid, BidKey>,
//     pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
//     // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
//     pub bidder_expires_at: MultiIndex<'a, (Addr, u64), Bid, BidKey>,
// }

// impl<'a> IndexList<Bid> for BidIndicies<'a> {
//     fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Bid>> + '_> {
//         let v: Vec<&dyn Index<Bid>> = vec![
//             &self.collection,
//             &self.collection_token_id,
//             &self.collection_price,
//             &self.bidder,
//             &self.bidder_expires_at,
//         ];
//         Box::new(v.into_iter())
//     }
// }

// pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
//     let indexes = BidIndicies {
//         collection: MultiIndex::new(|d: &Bid| d.collection.clone(), "bids", "bids__collection"),
//         collection_token_id: MultiIndex::new(
//             |d: &Bid| (d.collection.clone(), d.token_id),
//             "bids",
//             "bids__collection_token_id",
//         ),
//         collection_price: MultiIndex::new(
//             |d: &Bid| (d.collection.clone(), d.price.u128()),
//             "bids",
//             "bids__collection_price",
//         ),
//         bidder: MultiIndex::new(|d: &Bid| d.bidder.clone(), "bids", "bids__bidder"),
//         bidder_expires_at: MultiIndex::new(
//             |d: &Bid| (d.bidder.clone(), d.expires_at.seconds()),
//             "bids",
//             "bids__bidder_expires_at",
//         ),
//     };
//     IndexedMap::new("bids", indexes)
// }

// /// Represents a bid (offer) across an entire collection in the marketplace
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CollectionBid {
//     pub collection: Addr,
//     pub bidder: Addr,
//     pub price: Uint128,
//     pub finders_fee_bps: Option<u64>,
//     pub expires_at: Timestamp,
// }

// impl Order for CollectionBid {
//     fn expires_at(&self) -> Timestamp {
//         self.expires_at
//     }
// }

// /// Primary key for bids: (collection, token_id, bidder)
// pub type CollectionBidKey = (Addr, Addr);
// /// Convenience collection bid key constructor
// pub fn collection_bid_key(collection: &Addr, bidder: &Addr) -> CollectionBidKey {
//     (collection.clone(), bidder.clone())
// }

// /// Defines incides for accessing collection bids
// pub struct CollectionBidIndicies<'a> {
//     pub collection: MultiIndex<'a, Addr, CollectionBid, CollectionBidKey>,
//     pub collection_price: MultiIndex<'a, (Addr, u128), CollectionBid, CollectionBidKey>,
//     pub bidder: MultiIndex<'a, Addr, CollectionBid, CollectionBidKey>,
//     // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
//     pub bidder_expires_at: MultiIndex<'a, (Addr, u64), CollectionBid, CollectionBidKey>,
// }

// impl<'a> IndexList<CollectionBid> for CollectionBidIndicies<'a> {
//     fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<CollectionBid>> + '_> {
//         let v: Vec<&dyn Index<CollectionBid>> = vec![
//             &self.collection,
//             &self.collection_price,
//             &self.bidder,
//             &self.bidder_expires_at,
//         ];
//         Box::new(v.into_iter())
//     }
// }

// pub fn collection_bids<'a>(
// ) -> IndexedMap<'a, CollectionBidKey, CollectionBid, CollectionBidIndicies<'a>> {
//     let indexes = CollectionBidIndicies {
//         collection: MultiIndex::new(
//             |d: &CollectionBid| d.collection.clone(),
//             "col_bids",
//             "col_bids__collection",
//         ),
//         collection_price: MultiIndex::new(
//             |d: &CollectionBid| (d.collection.clone(), d.price.u128()),
//             "col_bids",
//             "col_bids__collection_price",
//         ),
//         bidder: MultiIndex::new(
//             |d: &CollectionBid| d.bidder.clone(),
//             "col_bids",
//             "col_bids__bidder",
//         ),
//         bidder_expires_at: MultiIndex::new(
//             |d: &CollectionBid| (d.bidder.clone(), d.expires_at.seconds()),
//             "col_bids",
//             "col_bids__bidder_expires_at",
//         ),
//     };
//     IndexedMap::new("col_bids", indexes)
// }
