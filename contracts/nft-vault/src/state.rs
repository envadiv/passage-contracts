use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result};
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultToken {
    /// The token_id of the staked NFT
    pub token_id: String,
    /// The owner of the NFT
    pub owner: Addr,
    /// The time at which the NFT was originally staked
    pub stake_timestamp: Timestamp,
    /// The time at which the NFT unstaking began
    pub unstake_timestamp: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub enum VaultTokenStatus {
    Staked,
    Unstaking,
    Transferrable,
}

impl Display for VaultTokenStatus {
    fn fmt(&self, f: &mut Formatter) -> Result {
       write!(f, "{:?}", self)
    }
}

impl VaultToken {
    pub fn get_status(&self, now: &Timestamp, unstake_period: u64) -> VaultTokenStatus {
        if self.unstake_timestamp.is_none() {
            return VaultTokenStatus::Staked;
        }
        let unstake_timestamp = self.unstake_timestamp.unwrap();
        let unstake_time = unstake_timestamp.plus_seconds(unstake_period);
        if now < &unstake_time {
            return VaultTokenStatus::Unstaking;
        }
        VaultTokenStatus::Transferrable
    }
}

pub const VAULT_TOKENS: Map<String, VaultToken> = Map::new("vault-tokens");

pub const STAKE_HOOKS: Hooks = Hooks::new("stake-hooks");
pub const UNSTAKE_HOOKS: Hooks = Hooks::new("unstake-hooks");
pub const WITHDRAW_HOOKS: Hooks = Hooks::new("withdraw-hooks");