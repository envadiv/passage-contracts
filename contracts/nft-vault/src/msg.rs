use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    StakeHooks {},
    UnstakeHooks {},
    WithdrawHooks {}, 
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub cw721_address: String,
    pub operators: Vec<String>,
    pub label: String,
    pub unstake_period: u64,
}