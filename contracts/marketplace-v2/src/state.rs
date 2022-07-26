use std::fmt::{Display, Formatter, Result};
use cosmwasm_std::{Addr, Decimal, Timestamp, Uint128, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::helpers::ExpiryRange;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
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
    /// The minimum duration of an auction 
    pub auction_min_duration: u64,
    /// The maximum duration of an auction 
    pub auction_max_duration: u64,
    /// The amount of time a seller has to finalize an auction
    pub auction_expiry_offset: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub type TokenId = String;

pub trait Expiration {
    fn expires_at(&self) -> Timestamp;

    fn is_expired(&self, now: &Timestamp) -> bool {
        self.expires_at() <= *now
    }
}

pub trait Recipient {
    fn get_recipient(&self) -> Addr;
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

impl Expiration for Ask {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

impl Recipient for Ask {
    fn get_recipient(&self) -> Addr {
        let self_cpy = self.clone();
        self_cpy.funds_recipient.map_or(self_cpy.seller, |a| a)
    }
}

/// Primary key for asks
pub type AskKey = TokenId;

/// Defines indices for accessing Asks
pub struct AskIndices<'a> {
    pub expiry: MultiIndex<'a, u64, Ask, AskKey>,
    pub price: MultiIndex<'a, u128, Ask, AskKey>,
    pub seller_expiry: MultiIndex<'a, (Addr, u64), Ask, AskKey>,
}

impl<'a> IndexList<Ask> for AskIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.expiry, &self.price, &self.seller_expiry];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndices<'a>> {
    let indexes = AskIndices {
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

impl Expiration for Bid {
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

impl Expiration for CollectionBid {
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

/// Represents an auction on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Auction {
    pub token_id: TokenId,
    pub seller: Addr,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
    pub starting_price: Coin,
    pub reserve_price: Option<Coin>,
    pub funds_recipient: Option<Addr>,
}

impl Recipient for Auction {
    fn get_recipient(&self) -> Addr {
        let self_cpy = self.clone();
        self_cpy.funds_recipient.map_or(self_cpy.seller, |a| a)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum AuctionStatus {
    Pending,
    Open,
    Closed,
    Expired,
}

impl Display for AuctionStatus {
    fn fmt(&self, f: &mut Formatter) -> Result {
       write!(f, "{:?}", self)
    }
}

impl Auction {
    pub fn get_auction_status(&self, now: &Timestamp, void_offset: u64) -> AuctionStatus {
        if now < &self.start_time {
            AuctionStatus::Pending
        } else if now < &self.end_time {
            AuctionStatus::Open
        } else if now < &self.end_time.plus_seconds(void_offset) {
            AuctionStatus::Closed
        } else {
            AuctionStatus::Expired
        }
    }
}

/// Primary key for asks
pub type AuctionKey = TokenId;

/// Defines indices for accessing Auctions
pub struct AuctionIndices<'a> {
    pub starting_price: MultiIndex<'a, u128, Auction, AuctionKey>,
    pub reserve_price: MultiIndex<'a, u128, Auction, AuctionKey>,
    pub seller_end_time: MultiIndex<'a, (Addr, u64), Auction, AuctionKey>,
}

impl<'a> IndexList<Auction> for AuctionIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Auction>> + '_> {
        let v: Vec<&dyn Index<Auction>> = vec![&self.starting_price, &self.reserve_price, &self.seller_end_time];
        Box::new(v.into_iter())
    }
}

pub fn auctions<'a>() -> IndexedMap<'a, AuctionKey, Auction, AuctionIndices<'a>> {
    let indexes = AuctionIndices {
        starting_price: MultiIndex::new(
            |a: &Auction|  a.starting_price.amount.u128(),
            "auctions",
            "auctions__starting_price",
        ),
        reserve_price: MultiIndex::new(
            |a: &Auction|  a.reserve_price.as_ref().map_or(Uint128::MAX.u128(), |p| p.amount.u128()),
            "auctions",
            "auctions__reserve_price"
        ),
        seller_end_time: MultiIndex::new(
            |d: &Auction|  (d.seller.clone(), d.end_time.seconds()),
            "auctions",
            "auctions__seller_end_time",
        ),
    };
    IndexedMap::new("auctions", indexes)
}

/// Represents a bid (offer) on an auction in the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AuctionBid {
    pub token_id: TokenId,
    pub bidder: Addr,
    pub price: Coin,
}

/// Primary key for auction_bids: (token_id, bidder)
pub type AuctionBidKey = (TokenId, Addr);

/// Convenience auction_bid key constructor
pub fn auction_bid_key(token_id: TokenId, bidder: &Addr) -> BidKey {
    (token_id, bidder.clone())
}

/// Defines indices for accessing bids
pub struct AuctionBidIndices<'a> {
    pub token_price: MultiIndex<'a, (String, u128), AuctionBid, AuctionBidKey>,
}

impl<'a> IndexList<AuctionBid> for AuctionBidIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<AuctionBid>> + '_> {
        let v: Vec<&dyn Index<AuctionBid>> = vec![&self.token_price];
        Box::new(v.into_iter())
    }
}

pub fn auction_bids<'a>() -> IndexedMap<'a, AuctionBidKey, AuctionBid, AuctionBidIndices<'a>> {
    let indexes = AuctionBidIndices {
        token_price: MultiIndex::new(
            |d: &AuctionBid| (d.token_id.clone(), d.price.amount.u128()),
            "auction_bids",
            "auction_bids__token_price",
        ),
    };
    IndexedMap::new("auction_bids", indexes)
}