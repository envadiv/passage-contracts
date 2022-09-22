use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Index, IndexList, IndexedMap, MultiIndex};
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

/// Defines indices for accessing VaultTokens
pub struct VaultTokenIndices<'a> {
    pub owner: MultiIndex<'a, Addr, VaultToken, String>,
    pub stake_timestamp: MultiIndex<'a, u64, VaultToken, String>,
    pub unstake_timestamp: MultiIndex<'a, u64, VaultToken, String>,
}

impl<'a> IndexList<VaultToken> for VaultTokenIndices<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<VaultToken>> + '_> {
        let v: Vec<&dyn Index<VaultToken>> = vec![&self.owner, &self.stake_timestamp, &self.unstake_timestamp];
        Box::new(v.into_iter())
    }
}

pub fn vault_tokens<'a>() -> IndexedMap<'a, String, VaultToken, VaultTokenIndices<'a>> {
    let indexes = VaultTokenIndices {
        owner: MultiIndex::new(|d: &VaultToken|  d.owner.clone(), "vault_tokens", "vault_tokens__owner"),
        stake_timestamp: MultiIndex::new(|d: &VaultToken|  d.stake_timestamp.seconds(), "vault_tokens", "vault_tokens__stake_timestamp"),
        unstake_timestamp: MultiIndex::new(
            |d: &VaultToken| d.unstake_timestamp.map_or(u64::MAX, |ts| ts.seconds()),
            "vault_tokens",
            "vault_tokens__unstake_timestamp",
        ),
    };
    IndexedMap::new("vault_tokens", indexes)
}

pub const STAKE_HOOKS: Hooks = Hooks::new("stake-hooks");
pub const UNSTAKE_HOOKS: Hooks = Hooks::new("unstake-hooks");
pub const WITHDRAW_HOOKS: Hooks = Hooks::new("withdraw-hooks");