use cosmwasm_std::{Coin, Timestamp};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use pg721_metadata_onchain::msg::{InstantiateMsg as Pg721InstantiateMsg, Metadata};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base_token_uri: String,
    pub max_num_tokens: u32,
    pub cw721_code_id: u64,
    pub cw721_instantiate_msg: Pg721InstantiateMsg,
    pub start_time: Timestamp,
    pub per_address_limit: u32,
    pub unit_price: Coin,
    pub whitelist: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpsertTokenMetadatas { token_metadatas: Vec<TokenMetadata> },
    Mint {},
    SetWhitelist { whitelist: String },
    UpdateStartTime(Timestamp),
    UpdatePerAddressLimit { per_address_limit: u32 },
    MintTo { recipient: String },
    MintFor { token_id: u32, recipient: String },
    Withdraw {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    MintableNumTokens {},
    StartTime {},
    MintPrice {},
    MintCount { address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub admin: String,
    pub base_token_uri: String,
    pub max_num_tokens: u32,
    pub per_address_limit: u32,
    pub cw721_address: String,
    pub cw721_code_id: u64,
    pub start_time: Timestamp,
    pub unit_price: Coin,
    pub whitelist: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintableNumTokensResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StartTimeResponse {
    pub start_time: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintPriceResponse {
    pub public_price: Coin,
    pub whitelist_price: Option<Coin>,
    pub current_price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintCountResponse {
    pub address: String,
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenMetadata {
    pub token_id: u32,
    pub metadata: Metadata
}
