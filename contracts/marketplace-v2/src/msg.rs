use crate::helpers::ExpiryRange;
use crate::state::{Ask, TokenId, Bid, Params};
// state::{Ask, Bid, CollectionBid, SaleType, SudoParams, TokenId},
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
    /// Update the price of an existing ask
    UpdateAskPrice {
        token_id: TokenId,
        price: Coin,
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
pub struct AskSellerExpiryOffset {
    pub expires_at: Timestamp,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidBidderExpiryOffset {
    pub expires_at: Timestamp,
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

// /// Offset for bid pagination
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct BidOffset {
//     pub price: Uint128,
//     pub token_id: TokenId,
//     pub bidder: Addr,
// }

// impl BidOffset {
//     pub fn new(price: Uint128, token_id: TokenId, bidder: Addr) -> Self {
//         BidOffset {
//             price,
//             token_id,
//             bidder,
//         }
//     }
// }
// /// Offset for collection pagination
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CollectionOffset {
//     pub collection: String,
//     pub token_id: TokenId,
// }

// impl CollectionOffset {
//     pub fn new(collection: String, token_id: TokenId) -> Self {
//         CollectionOffset {
//             collection,
//             token_id,
//         }
//     }
// }

// /// Offset for collection bid pagination
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CollectionBidOffset {
//     pub price: Uint128,
//     pub collection: Collection,
//     pub bidder: Bidder,
// }

// impl CollectionBidOffset {
//     pub fn new(price: Uint128, collection: String, bidder: Bidder) -> Self {
//         CollectionBidOffset {
//             price,
//             collection,
//             bidder,
//         }
//     }
// }

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
        query_options: QueryOptions<AskSellerExpiryOffset>
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
        query_options: QueryOptions<BidBidderExpiryOffset>
    },
    // /// Get all bids by a bidder, sorted by expiration
    // /// Return type: `BidsResponse`
    // BidsByBidderSortedByExpiration {
    //     bidder: Bidder,
    //     start_after: Option<CollectionOffset>,
    //     limit: Option<u32>,
    // },
    // /// Get all bids for a collection, sorted by price
    // /// Return type: `BidsResponse`
    // BidsSortedByPrice {
    //     collection: Collection,
    //     start_after: Option<BidOffset>,
    //     limit: Option<u32>,
    // },
    // /// Get all bids for a collection, sorted by price in reverse
    // /// Return type: `BidsResponse`
    // ReverseBidsSortedByPrice {
    //     collection: Collection,
    //     start_before: Option<BidOffset>,
    //     limit: Option<u32>,
    // },
    // /// Get data for a specific collection bid
    // /// Return type: `CollectionBidResponse`
    // CollectionBid {
    //     collection: Collection,
    //     bidder: Bidder,
    // },
    // /// Get all collection bids by a bidder
    // /// Return type: `CollectionBidsResponse`
    // CollectionBidsByBidder {
    //     bidder: Bidder,
    //     start_after: Option<CollectionOffset>,
    //     limit: Option<u32>,
    // },
    // /// Get all collection bids by a bidder, sorted by expiration
    // /// Return type: `CollectionBidsResponse`
    // CollectionBidsByBidderSortedByExpiration {
    //     bidder: Collection,
    //     start_after: Option<CollectionBidOffset>,
    //     limit: Option<u32>,
    // },
    // /// Get all collection bids for a collection sorted by price
    // /// Return type: `CollectionBidsResponse`
    // CollectionBidsSortedByPrice {
    //     collection: Collection,
    //     start_after: Option<CollectionBidOffset>,
    //     limit: Option<u32>,
    // },
    // /// Get all collection bids for a collection sorted by price in reverse
    // /// Return type: `CollectionBidsResponse`
    // ReverseCollectionBidsSortedByPrice {
    //     collection: Collection,
    //     start_before: Option<CollectionBidOffset>,
    //     limit: Option<u32>,
    // },
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

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CollectionBidResponse {
//     pub bid: Option<CollectionBid>,
// }

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct CollectionBidsResponse {
//     pub bids: Vec<CollectionBid>,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub struct SaleHookMsg {
//     pub collection: String,
//     pub token_id: u32,
//     pub price: Coin,
//     pub seller: String,
//     pub buyer: String,
// }

// impl SaleHookMsg {
//     pub fn new(
//         collection: String,
//         token_id: u32,
//         price: Coin,
//         seller: String,
//         buyer: String,
//     ) -> Self {
//         SaleHookMsg {
//             collection,
//             token_id,
//             price,
//             seller,
//             buyer,
//         }
//     }

//     /// serializes the message
//     pub fn into_binary(self) -> StdResult<Binary> {
//         let msg = SaleExecuteMsg::SaleHook(self);
//         to_binary(&msg)
//     }
// }

// // This is just a helper to properly serialize the above message
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum SaleExecuteMsg {
//     SaleHook(SaleHookMsg),
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum HookAction {
//     Create,
//     Update,
//     Delete,
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub struct AskHookMsg {
//     pub ask: Ask,
// }

// impl AskHookMsg {
//     pub fn new(ask: Ask) -> Self {
//         AskHookMsg { ask }
//     }

//     /// serializes the message
//     pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
//         let msg = match action {
//             HookAction::Create => AskHookExecuteMsg::AskCreatedHook(self),
//             HookAction::Update => AskHookExecuteMsg::AskUpdatedHook(self),
//             HookAction::Delete => AskHookExecuteMsg::AskDeletedHook(self),
//         };
//         to_binary(&msg)
//     }
// }

// // This is just a helper to properly serialize the above message
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum AskHookExecuteMsg {
//     AskCreatedHook(AskHookMsg),
//     AskUpdatedHook(AskHookMsg),
//     AskDeletedHook(AskHookMsg),
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub struct BidHookMsg {
//     pub bid: Bid,
// }

// impl BidHookMsg {
//     pub fn new(bid: Bid) -> Self {
//         BidHookMsg { bid }
//     }

//     /// serializes the message
//     pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
//         let msg = match action {
//             HookAction::Create => BidExecuteMsg::BidCreatedHook(self),
//             HookAction::Update => BidExecuteMsg::BidUpdatedHook(self),
//             HookAction::Delete => BidExecuteMsg::BidDeletedHook(self),
//         };
//         to_binary(&msg)
//     }
// }

// // This is just a helper to properly serialize the above message
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum BidExecuteMsg {
//     BidCreatedHook(BidHookMsg),
//     BidUpdatedHook(BidHookMsg),
//     BidDeletedHook(BidHookMsg),
// }

// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub struct CollectionBidHookMsg {
//     pub collection_bid: CollectionBid,
// }

// impl CollectionBidHookMsg {
//     pub fn new(collection_bid: CollectionBid) -> Self {
//         CollectionBidHookMsg { collection_bid }
//     }

//     /// serializes the message
//     pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
//         let msg = match action {
//             HookAction::Create => CollectionBidExecuteMsg::CollectionBidCreatedHook(self),
//             HookAction::Update => CollectionBidExecuteMsg::CollectionBidUpdatedHook(self),
//             HookAction::Delete => CollectionBidExecuteMsg::CollectionBidDeletedHook(self),
//         };
//         to_binary(&msg)
//     }
// }

// // This is just a helper to properly serialize the above message
// #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
// #[serde(rename_all = "snake_case")]
// pub enum CollectionBidExecuteMsg {
//     CollectionBidCreatedHook(CollectionBidHookMsg),
//     CollectionBidUpdatedHook(CollectionBidHookMsg),
//     CollectionBidDeletedHook(CollectionBidHookMsg),
// }
