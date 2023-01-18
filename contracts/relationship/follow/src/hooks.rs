use cosmwasm_schema::cw_serde;
use thiserror::Error;

use cosmwasm_std::{Addr, CustomQuery, Deps, StdError, StdResult, Storage, SubMsg};
use cw_storage_plus::Map;

#[cw_serde]
pub struct HooksResponse {
    pub hooks: Vec<String>,
}

#[derive(Error, Debug, PartialEq)]
pub enum HookError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Given address already registered as a hook")]
    HookAlreadyRegistered {},

    #[error("Given address not registered as a hook")]
    HookNotRegistered {},
}

pub struct Hooks<'a>(Map<'a, Addr, Vec<Addr>>);

impl<'a> Hooks<'a> {
    pub const fn new(storage_key: &'a str) -> Self {
        Hooks(Map::new(storage_key))
    }

    pub fn add_hook(
        &self,
        storage: &mut dyn Storage,
        target: Addr,
        hook: Addr,
    ) -> Result<(), HookError> {
        let mut hooks = self
            .0
            .may_load(storage, target.clone())?
            .unwrap_or_default();
        if !hooks.iter().any(|h| h == &hook) {
            hooks.push(hook);
        } else {
            return Err(HookError::HookAlreadyRegistered {});
        }
        Ok(self.0.save(storage, target, &hooks)?)
    }

    pub fn remove_hook(
        &self,
        storage: &mut dyn Storage,
        target: Addr,
        hook: Addr,
    ) -> Result<(), HookError> {
        let mut hooks = self.0.load(storage, target.clone())?;
        if let Some(p) = hooks.iter().position(|x| x == &hook) {
            hooks.remove(p);
        } else {
            return Err(HookError::HookNotRegistered {});
        }
        Ok(self.0.save(storage, target, &hooks)?)
    }

    pub fn prepare_hooks<F: Fn(Addr) -> StdResult<SubMsg>>(
        &self,
        storage: &dyn Storage,
        target: Addr,
        prep: F,
    ) -> StdResult<Vec<SubMsg>> {
        self.0
            .may_load(storage, target)?
            .unwrap_or_default()
            .into_iter()
            .map(prep)
            .collect()
    }

    pub fn query_hooks<Q: CustomQuery>(
        &self,
        deps: Deps<Q>,
        target: Addr,
    ) -> StdResult<HooksResponse> {
        let hooks = self.0.may_load(deps.storage, target)?.unwrap_or_default();
        let hooks = hooks.into_iter().map(String::from).collect();
        Ok(HooksResponse { hooks })
    }
}
