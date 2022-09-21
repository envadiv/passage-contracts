#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Addr};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg};
use crate::state::{CONFIG, STAKE_HOOKS, UNSTAKE_HOOKS, WITHDRAW_HOOKS};
use crate::helpers::{map_validate, only_operator};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::UpdateConfig {
            operators,
            label,
            unstake_period,
        } => execute_update_config(
            deps,
            env,
            info,
            operators,
            label,
            unstake_period
        ),
        ExecuteMsg::AddStakeHook { hook } => execute_add_stake_hook(
            deps,
            api.addr_validate(&hook)?
        ),
        ExecuteMsg::RemoveStakeHook { hook } => execute_remove_stake_hook(
            deps,
            api.addr_validate(&hook)?
        ),
        ExecuteMsg::AddUnstakeHook { hook } => execute_add_unstake_hook(
            deps,
            api.addr_validate(&hook)?
        ),
        ExecuteMsg::RemoveUnstakeHook { hook } => execute_remove_unstake_hook(
            deps,
            api.addr_validate(&hook)?
        ),
        ExecuteMsg::AddWithdrawHook { hook } => execute_add_withdraw_hook(
            deps,
            api.addr_validate(&hook)?
        ),
        ExecuteMsg::RemoveWithdrawHook { hook } => execute_remove_withdraw_hook(
            deps,
            api.addr_validate(&hook)?
        ),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operators: Option<Vec<String>>,
    label: Option<String>,
    unstake_period: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    only_operator(&info, &config)?;

    if let Some(_operators) = operators {
        config.operators = map_validate(deps.api, &_operators)?;
    }
    if let Some(_label) = label {
        config.label = _label;
    }
    if let Some(_unstake_period) = unstake_period {
        config.unstake_period = _unstake_period;
    }
    
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("operators", config.operators.iter().map(|addr| addr.to_string()).collect::<Vec<String>>().join(","))
        .add_attribute("cw721_address", config.cw721_address)
        .add_attribute("label", config.label)
        .add_attribute("unstake_period", config.unstake_period.to_string())
    )
}

pub fn execute_add_stake_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    STAKE_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_stake_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_stake_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    STAKE_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_stake_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_add_unstake_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    UNSTAKE_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_unstake_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_unstake_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    UNSTAKE_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_unstake_hook")
        .add_attribute("hook", hook);
    Ok(res)
}


pub fn execute_add_withdraw_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    WITHDRAW_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_withdraw_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_withdraw_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    WITHDRAW_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_withdraw_hook")
        .add_attribute("hook", hook);
    Ok(res)
}
