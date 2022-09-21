use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::hooks::Hooks;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// The NFT contract
    pub cw721_address: Addr,
    /// The addresses with admin permissions
    pub operators: Vec<Addr>,
    /// A human-readable string label for the vault
    pub label: String,
    /// The amount of time it takes to unstake a token
    pub unstake_period: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const VAULT_TOKENS: Map<String, Addr> = Map::new("vault-tokens");

pub const STAKE_HOOKS: Hooks = Hooks::new("stake-hooks");
pub const UNSTAKE_HOOKS: Hooks = Hooks::new("unstake-hooks");
pub const WITHDRAW_HOOKS: Hooks = Hooks::new("withdraw-hooks");