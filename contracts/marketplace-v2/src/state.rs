use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
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
    /// Marketplace fee collector address
    pub collector_address: Addr,
    /// Marketplace fee
    pub trading_fee_percent: Decimal,
    /// Valid time range for Asks
    /// (min, max) in seconds
    pub ask_expiry: ExpiryRange,
    /// Valid time range for Bids
    /// (min, max) in seconds
    pub bid_expiry: ExpiryRange,
    /// The operator addresses that have access to certain functionality
    pub operators: Vec<Addr>,
    /// Min value for a bid
    pub min_price: Uint128,
}

pub const PARAMS: Item<Params> = Item::new("params");

pub type TokenId = String;

pub trait Order {
    fn expires_at(&self) -> Timestamp;

    fn is_expired(&self, now: &Timestamp) -> bool {
        self.expires_at() <= *now
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

impl Order for Ask {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

/// Primary key for asks
pub type AskKey = TokenId;

/// Defines indices for accessing Asks
pub struct AskIndicies<'a> {
    pub expiry: MultiIndex<'a, u64, Ask, AskKey>,
    pub price: MultiIndex<'a, u128, Ask, AskKey>,
    pub seller_expiry: MultiIndex<'a, (Addr, u64), Ask, AskKey>,
}

impl<'a> IndexList<Ask> for AskIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.expiry, &self.price, &self.seller_expiry];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        expiry: MultiIndex::new(|d: &Ask|  d.expires_at.seconds(), "asks", "asks__expiry"),
        price: MultiIndex::new(|d: &Ask|  d.price.amount.u128(), "asks", "asks__price"),
        seller_expiry: MultiIndex::new(|d: &Ask| (d.seller.clone(), d.expires_at.seconds()), "asks", "asks__seller"),
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub token_id: TokenId,
    pub bidder: Addr,
    pub price: Coin,
    pub expires_at: Timestamp,
}

impl Order for Bid {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

/// Primary key for bids: (token_id, bidder)
pub type BidKey = (TokenId, Addr);

/// Convenience bid key constructor
pub fn bid_key(token_id: TokenId, bidder: &Addr) -> BidKey {
    (token_id, bidder.clone())
}

/// Defines incides for accessing bids
pub struct BidIndices<'a> {
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
    pub expiry: MultiIndex<'a, u64, Bid, BidKey>,
    pub token_price: MultiIndex<'a, (String, u128), Bid, BidKey>,
    pub bidder_expiry: MultiIndex<'a, (Addr, u64), Bid, BidKey>,
}

impl<'a> IndexList<Bid> for BidIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Bid>> + '_> {
        let v: Vec<&dyn Index<Bid>> = vec![
            &self.expiry,
            &self.token_price,
            &self.bidder_expiry,
        ];
        Box::new(v.into_iter())
    }
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndices<'a>> {
    let indexes = BidIndices {
        expiry: MultiIndex::new(|d: &Bid|  d.expires_at.seconds(), "bids", "bids__expiry"),
        token_price: MultiIndex::new(
            |d: &Bid| (d.token_id.clone(), d.price.amount.u128()),
            "bids",
            "bids__token_price",
        ),
        bidder_expiry: MultiIndex::new(|d: &Bid| (d.bidder.clone(), d.expires_at.seconds()), "bids", "bids__bidder_expiry"),
    };
    IndexedMap::new("bids", indexes)
}

/// Represents a bid (offer) across an entire collection in the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBid {
    pub bidder: Addr,
    pub units: u32,
    pub price: Coin,
    pub expires_at: Timestamp,
}

impl CollectionBid {
    pub fn total_cost(&self) -> u128 {
        &self.price.amount.u128() * u128::from(self.units)
    }
}

impl Order for CollectionBid {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

/// Primary key for collection bids
pub type CollectionBidKey = Addr;

/// Defines incides for accessing collection bids
pub struct CollectionBidIndices<'a> {
    pub expiry: MultiIndex<'a, u64, CollectionBid, CollectionBidKey>,
    pub price: MultiIndex<'a, u128, CollectionBid, CollectionBidKey>,
}

impl<'a> IndexList<CollectionBid> for CollectionBidIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<CollectionBid>> + '_> {
        let v: Vec<&dyn Index<CollectionBid>> = vec![
            &self.expiry,
            &self.price,
        ];
        Box::new(v.into_iter())
    }
}

pub fn collection_bids<'a>(
) -> IndexedMap<'a, Addr, CollectionBid, CollectionBidIndices<'a>> {
    let indexes = CollectionBidIndices {
        expiry: MultiIndex::new(|d: &CollectionBid|  d.expires_at.seconds(), "col_bids", "col_bids__expiry"),
        price: MultiIndex::new(|d: &CollectionBid|  d.price.amount.u128(), "col_bids", "col_bids__price"),
    };
    IndexedMap::new("col_bids", indexes)
}
