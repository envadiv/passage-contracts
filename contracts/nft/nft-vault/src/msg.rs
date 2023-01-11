use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{to_binary, Addr, Binary, StdResult, Timestamp};
use crate::state::{VaultToken, VaultTokenStatus};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub cw721_address: String,
    pub label: String,
    pub unstake_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Update the contract configuration
    UpdateConfig {
        operators: Option<Vec<String>>,
        label: Option<String>,
        unstake_period: Option<u64>,
    },
    /// Add a new hook to be informed of all staking events
    AddStakeHook { hook: String },
    /// Remove a stake hook
    RemoveStakeHook { hook: String },
    /// Add a new hook to be informed of all unstaking events
    AddUnstakeHook { hook: String },
    /// Remove an unstake hook
    RemoveUnstakeHook { hook: String },
    /// Add a new hook to be informed of all withdraw events
    AddWithdrawHook { hook: String },
    /// Remove a withdraw hook
    RemoveWithdrawHook { hook: String },
    /// Stake an NFT
    Stake { token_id: String, },
    /// Unstake an NFT
    Unstake { token_id: String, },
    /// Withdraw an NFT
    Withdraw { token_id: String, },
}

/// Options when querying for VaultTokens
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenTimestampOffset {
    pub token_id: String,
    pub timestamp: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    StakeHooks {},
    UnstakeHooks {},
    WithdrawHooks {},
    VaultToken { token_id: String },
    VaultTokensByOwner { owner: String, query_options: QueryOptions<TokenTimestampOffset> },
    VaultTokensByStakeTimestamp { query_options: QueryOptions<TokenTimestampOffset> },
    VaultTokensByUnstakeTimestamp { query_options: QueryOptions<TokenTimestampOffset> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub cw721_address: String,
    pub operators: Vec<String>,
    pub label: String,
    pub unstake_period: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultTokenResponse {
    pub vault_token: Option<VaultToken>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultTokensResponse {
    pub vault_tokens: Vec<VaultToken>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HookAction {
    Stake,
    Unstake,
    Withdraw
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct HookMsg {
    pub cw721_address: Addr,
    pub token_id: String,
    pub owner: Addr,
    pub stake_timestamp: Timestamp,
    pub unstake_timestamp: Option<Timestamp>,
    pub status: VaultTokenStatus,
}

impl HookMsg {
    pub fn new(
        cw721_address: &Addr,
        vault_token: &VaultToken,
        now: &Timestamp,
        unstake_period: u64,
    ) -> Self {
        HookMsg {
            cw721_address: cw721_address.clone(),
            token_id: vault_token.token_id.clone(),
            owner: vault_token.owner.clone(),
            stake_timestamp: vault_token.stake_timestamp.clone(),
            unstake_timestamp: vault_token.unstake_timestamp.clone(),
            status: vault_token.get_status(now, unstake_period),
        }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Stake => HookExecuteMsg::StakeHook(self),
            HookAction::Unstake => HookExecuteMsg::UnstakeHook(self),
            HookAction::Withdraw => HookExecuteMsg::WithdrawHook(self),
        };
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HookExecuteMsg {
    StakeHook(HookMsg),
    UnstakeHook(HookMsg),
    WithdrawHook(HookMsg),
}