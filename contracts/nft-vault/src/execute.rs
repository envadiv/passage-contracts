#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, Addr, Event, WasmMsg, SubMsg, Reply, StdError};
use cw_utils::{nonpayable};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, HookMsg, HookAction};
use crate::state::{CONFIG, VaultToken, VaultTokenStatus, VAULT_TOKENS, STAKE_HOOKS, UNSTAKE_HOOKS, WITHDRAW_HOOKS};
use crate::helpers::{map_validate, only_operator, transfer_nft};

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
        ExecuteMsg::Stake { token_id } => execute_stake(
            deps,
            env,
            info,
            token_id
        ),
        ExecuteMsg::Unstake { token_id } => execute_unstake(
            deps,
            env,
            info,
            token_id
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

pub fn execute_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    
    let mut response = Response::new();

    let config = CONFIG.load(deps.storage)?;
    let mut vault_token_option = VAULT_TOKENS.may_load(deps.storage, token_id.clone())?;

    if let Some(_vault_token) = &mut vault_token_option {
        // Allow users to re-stake tokens that are either unstaking or transferrable
        let status = _vault_token.get_status(&env.block.time, config.unstake_period);
        if status == VaultTokenStatus::Staked {
            return Err(ContractError::InvalidStatus(status.to_string()));
        }
        _vault_token.stake_timestamp = env.block.time;
        _vault_token.unstake_timestamp = None;
    } else {
        // Allow users to stake tokens
        transfer_nft(&token_id, &env.contract.address, &config.cw721_address, &mut response)?;

        vault_token_option = Some(VaultToken {
            token_id: token_id.clone(),
            owner: info.sender.clone(),
            stake_timestamp: env.block.time,
            unstake_timestamp: None
        });
    }

    let vault_token = vault_token_option.unwrap();
    VAULT_TOKENS.save(deps.storage, token_id.clone(), &vault_token)?;

    let submsgs = STAKE_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = HookMsg::new(
            &config.cw721_address,
            &vault_token,
            &env.block.time,
            config.unstake_period,
        );
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(HookAction::Stake)?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Stake as u64))
    })?;

    let event = Event::new("stake-token")
        .add_attribute("cw721_address", config.cw721_address.to_string())
        .add_attribute("token_id", &token_id.to_string());

    Ok(response.add_submessages(submsgs).add_event(event))
}

pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let vault_token = VAULT_TOKENS.load(deps.storage, token_id.clone())?;

    // Only original owner can unstake
    if vault_token.owner != info.sender {
        return Err(ContractError::Unauthorized("Only owner can unstake".to_string()));
    }

    let config = CONFIG.load(deps.storage)?;
    let vault_token = VAULT_TOKENS.update(
        deps.storage,
        token_id.clone(),
        |vault_token| -> Result<VaultToken, ContractError> {
            let mut updated_vault_token = vault_token.unwrap();
            let status = updated_vault_token.get_status(&env.block.time, config.unstake_period);
            if status != VaultTokenStatus::Staked {
                return Err(ContractError::InvalidStatus(status.to_string()));
            }
            updated_vault_token.unstake_timestamp = Some(env.block.time);
            Ok(updated_vault_token)
        }
    )?;

    let submsgs = UNSTAKE_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = HookMsg::new(
            &config.cw721_address,
            &vault_token,
            &env.block.time,
            config.unstake_period,
        );
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(HookAction::Unstake)?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Unstake as u64))
    })?;

    let event = Event::new("unstake-token")
        .add_attribute("cw721_address", config.cw721_address.to_string())
        .add_attribute("token_id", &token_id.to_string());

    let response = Response::new();
    Ok(response.add_submessages(submsgs).add_event(event))
}

enum HookReply {
    Stake = 1,
    Unstake,
    Withdraw,
}

impl From<u64> for HookReply {
    fn from(item: u64) -> Self {
        match item {
            1 => HookReply::Stake,
            2 => HookReply::Unstake,
            3 => HookReply::Withdraw,
            _ => panic!("invalid reply type"),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match HookReply::from(msg.id) {
        HookReply::Stake => {
            let res = Response::new()
                .add_attribute("action", "stake-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::Unstake => {
            let res = Response::new()
                .add_attribute("action", "unstake-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::Withdraw => {
            let res = Response::new()
                .add_attribute("action", "withdraw-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
    }
}