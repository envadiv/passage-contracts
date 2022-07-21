use crate::helpers::ExpiryRange;
use crate::state::{Ask, TokenId, Bid, Params, CollectionBid};
use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// The NFT contract
    pub cw721_address: String,
    /// The token used to pay for NFTs
    pub denom: String,
    /// The address collecting marketplace fees
    pub collector_address: String,
    /// Fair Burn fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Valid time range for Asks
    /// (min, max) in seconds
    pub ask_expiry: ExpiryRange,
    /// Valid time range for Bids
    /// (min, max) in seconds
    pub bid_expiry: ExpiryRange,
    /// Valid time range for Auctions
    /// (min, max) in seconds
    pub auction_expiry: ExpiryRange,
    /// Operators are entites that are responsible for maintaining the active state of Asks.
    /// They listen to NFT transfer events, and update the active state of Asks.
    pub operators: Vec<String>,
    /// Min value for bids and asks
    pub min_price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update the contract parameters
    UpdateParams {
        trading_fee_bps: Option<u64>,
        ask_expiry: Option<ExpiryRange>,
        bid_expiry: Option<ExpiryRange>,
        auction_expiry: Option<ExpiryRange>,
        operators: Option<Vec<String>>,
        min_price: Option<Uint128>,
    },
    /// List an NFT on the marketplace by creating a new ask
    SetAsk {
        token_id: TokenId,
        price: Coin,
        funds_recipient: Option<String>,
        reserve_for: Option<String>,
        expires_at: Timestamp,
    },
    /// Remove an existing ask from the marketplace
    RemoveAsk {
        token_id: TokenId,
    },
    /// Place a bid on an existing ask
    SetBid {
        token_id: TokenId,
        price: Coin,
        expires_at: Timestamp,
    },
    /// Remove an existing bid from an ask
    RemoveBid {
        token_id: TokenId,
    },
    /// Accept a bid on an existing ask
    AcceptBid {
        token_id: TokenId,
        bidder: String,
    },
    /// Place a bid (limit order) across an entire collection
    SetCollectionBid {
        units: u32,
        price: Coin,
        expires_at: Timestamp,
    },
    /// Remove a bid (limit order) across an entire collection
    RemoveCollectionBid { },
    /// Accept a collection bid
    AcceptCollectionBid {
        token_id: TokenId,
        bidder: String,
    },
    /// Create an auction for a specified token
    SetAuction {
        token_id: TokenId,
        starting_price: Coin,
        reserve_price: Option<Coin>,
        funds_recipient: Option<String>,
        expires_at: Timestamp,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskExpiryOffset {
    pub expires_at: Timestamp,
    pub token_id: TokenId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskPriceOffset {
    pub price: Uint128,
    pub token_id: TokenId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidExpiryOffset {
    pub expires_at: Timestamp,
    pub bidder: Addr,
    pub token_id: TokenId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidTokenPriceOffset {
    pub price: u128,
    pub bidder: Addr,
    pub token_id: TokenId,
}

/// Options when querying for Asks and Bids
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub filter_expiry: Option<Timestamp>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

/// Offset for collection bid pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidPriceOffset {
    pub bidder: Addr,
    pub price: u128,
}

/// Offset for collection bid pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidExpiryOffset {
    pub bidder: Addr,
    pub expires_at: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get the config for the contract
    /// Return type: `ParamsResponse`
    Params {},
    /// Get the current ask for specific NFT
    /// Return type: `CurrentAskResponse`
    Ask {
        token_id: TokenId,
    },
    /// Get all asks sorted by expiry
    /// Return type: `AsksResponse`
    AsksSortedByExpiry {
        query_options: QueryOptions<AskExpiryOffset>
    },
    /// Get all asks sorted by price
    /// Return type: `AsksResponse`
    AsksSortedByPrice {
        query_options: QueryOptions<AskPriceOffset>
    },
    /// Get all asks by seller
    /// Return type: `AsksResponse`
    AsksBySellerExpiry {
        seller: String,
        query_options: QueryOptions<AskExpiryOffset>
    },
    /// Count of all asks
    /// Return type: `AskCountResponse`
    AskCount {},
    /// Get data for a specific bid
    /// Return type: `BidResponse`
    Bid {
        token_id: TokenId,
        bidder: String,
    },
    /// Get all bids sorted by expiry
    /// Return type: `BidsResponse`
    BidsSortedByExpiry {
        query_options: QueryOptions<BidExpiryOffset>
    },
    /// Get all bids for a token sorted by price
    /// Return type: `BidsResponse`
    BidsByTokenPrice {
        token_id: TokenId,
        query_options: QueryOptions<BidTokenPriceOffset>
    },
    /// Get all bids by bidders sorted by expiry
    /// Return type: `BidsResponse`
    BidsByBidderExpiry {
        bidder: String,
        query_options: QueryOptions<BidExpiryOffset>
    },
    /// Get a bidders collection_bid
    /// Return type: `CollectionBidResponse`
    CollectionBid {
        bidder: String,
    },
    /// Get all collection_bids sorted by price
    /// Return type: `CollectionBidsResponse`
    CollectionBidsByPrice {
        query_options: QueryOptions<CollectionBidPriceOffset>
    },
    /// Get all collection_bids sorted by expiry
    /// Return type: `CollectionBidsResponse`
    CollectionBidsByExpiry {
        query_options: QueryOptions<CollectionBidExpiryOffset>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskResponse {
    pub ask: Option<Ask>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AsksResponse {
    pub asks: Vec<Ask>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskCountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidResponse {
    pub bid: Option<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidsResponse {
    pub bids: Vec<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse {
    pub params: Params,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidResponse {
    pub collection_bid: Option<CollectionBid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidsResponse {
    pub collection_bids: Vec<CollectionBid>,
}