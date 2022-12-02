use cosmwasm_std::{Addr, Decimal, Uint128, Coin};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    /// The operator addresses that have access to certain functionality
    pub operators: Vec<Addr>,
    /// Min value for a bid
    pub min_price: Uint128,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub type TokenId = String;

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
    pub price: MultiIndex<'a, u128, Ask, AskKey>,
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
}

impl<'a> IndexList<Ask> for AskIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.price, &self.seller];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndices<'a>> {
    let indexes = AskIndices {
        price: MultiIndex::new(|d: &Ask|  d.price.amount.u128(), "asks", "asks__price"),
        seller: MultiIndex::new(|d: &Ask|  d.seller.clone(), "asks", "asks__seller"),
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub token_id: TokenId,
    pub bidder: Addr,
    pub price: Coin,
}

/// Primary key for bids: (token_id, bidder)
pub type BidKey = (Addr, TokenId);

/// Convenience bid key constructor
pub fn bid_key(bidder: &Addr, token_id: TokenId) -> BidKey {
    (bidder.clone(), token_id)
}

/// Defines incides for accessing bids
pub struct BidIndices<'a> {
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
    pub token_price: MultiIndex<'a, (String, u128), Bid, BidKey>,
}

impl<'a> IndexList<Bid> for BidIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Bid>> + '_> {
        let v: Vec<&dyn Index<Bid>> = vec![
            &self.token_price,
        ];
        Box::new(v.into_iter())
    }
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndices<'a>> {
    let indexes = BidIndices {
        token_price: MultiIndex::new(
            |d: &Bid| (d.token_id.clone(), d.price.amount.u128()),
            "bids",
            "bids__token_price",
        ),
    };
    IndexedMap::new("bids", indexes)
}

/// Represents a bid (offer) across an entire collection in the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBid {
    pub bidder: Addr,
    pub units: u32,
    pub price: Coin,
}

impl CollectionBid {
    pub fn total_cost(&self) -> u128 {
        &self.price.amount.u128() * u128::from(self.units)
    }
}

/// Primary key for collection bids
pub type CollectionBidKey = Addr;

/// Defines incides for accessing collection bids
pub struct CollectionBidIndices<'a> {
    pub price: MultiIndex<'a, u128, CollectionBid, CollectionBidKey>,
}

impl<'a> IndexList<CollectionBid> for CollectionBidIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<CollectionBid>> + '_> {
        let v: Vec<&dyn Index<CollectionBid>> = vec![
            &self.price,
        ];
        Box::new(v.into_iter())
    }
}

pub fn collection_bids<'a>(
) -> IndexedMap<'a, Addr, CollectionBid, CollectionBidIndices<'a>> {
    let indexes = CollectionBidIndices {
        price: MultiIndex::new(|d: &CollectionBid|  d.price.amount.u128(), "col_bids", "col_bids__price"),
    };
    IndexedMap::new("col_bids", indexes)
}
