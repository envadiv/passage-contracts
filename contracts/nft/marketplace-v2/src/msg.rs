use crate::state::{Ask, TokenId, Bid, Config, CollectionBid};
use cosmwasm_std::{Addr, Coin, Uint128};
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
    UpdateConfig {
        collector_address: Option<String>,
        trading_fee_bps: Option<u64>,
        operators: Option<Vec<String>>,
        min_price: Option<Uint128>,
    },
    /// List an NFT on the marketplace by creating a new ask
    SetAsk {
        token_id: TokenId,
        price: Coin,
        funds_recipient: Option<String>,
    },
    /// Remove an existing ask from the marketplace
    RemoveAsk {
        token_id: TokenId,
    },
    /// Place a bid on an existing ask
    SetBid {
        token_id: TokenId,
        price: Coin,
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
    },
    /// Remove a bid (limit order) across an entire collection
    RemoveCollectionBid { },
    /// Accept a collection bid
    AcceptCollectionBid {
        token_id: TokenId,
        bidder: String,
    },
}

/// Options when querying for Asks and Bids
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenPriceOffset {
    pub token_id: TokenId,
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenAddrOffset {
    pub token_id: TokenId,
    pub address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidTokenPriceOffset {
    pub price: u128,
    pub bidder: Addr,
    pub token_id: TokenId,
}

/// Offset for collection bid pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidPriceOffset {
    pub bidder: Addr,
    pub price: u128,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Get the config for the contract
    /// Return type: `ConfigResponse`
    Config {},
    /// Get the current ask for specific NFT
    /// Return type: `AskResponse`
    Ask {
        token_id: TokenId,
    },
    /// Get all asks sorted by price
    /// Return type: `AsksResponse`
    AsksSortedByPrice {
        query_options: QueryOptions<TokenPriceOffset>
    },
    /// Get all asks by seller
    /// Return type: `AsksResponse`
    AsksBySeller {
        query_options: QueryOptions<TokenAddrOffset>
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
    /// Get all bids for a token sorted by price
    /// Return type: `BidsResponse`
    BidsByTokenPrice {
        token_id: TokenId,
        query_options: QueryOptions<BidTokenPriceOffset>
    },
    /// Get all bids by bidders sorted by expiry
    /// Return type: `BidsResponse`
    BidsByBidder {
        query_options: QueryOptions<TokenAddrOffset>
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
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidResponse {
    pub collection_bid: Option<CollectionBid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidsResponse {
    pub collection_bids: Vec<CollectionBid>,
}