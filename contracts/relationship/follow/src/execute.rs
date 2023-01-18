use crate::msg::{ExecuteMsg, HookAction, HookMsg, HookReply};
use crate::state::{follow_key, follows, Follow, FOLLOW_HOOKS, UNFOLLOW_HOOKS};
use crate::ContractError;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Addr, DepsMut, Env, Event, MessageInfo, Response, SubMsg, WasmMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::AddFollowHook { hook } => {
            execute_add_follow_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::RemoveFollowHook { hook } => {
            execute_remove_follow_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::AddUnfollowHook { hook } => {
            execute_add_unfollow_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::RemoveUnfollowHook { hook } => {
            execute_remove_unfollow_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::AddFollow { target } => {
            execute_add_follow(deps, env, info, api.addr_validate(&target)?)
        }
        ExecuteMsg::RemoveFollow { target } => {
            execute_remove_follow(deps, env, info, api.addr_validate(&target)?)
        }
    }
}

pub fn execute_add_follow_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    FOLLOW_HOOKS.add_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_follow_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_follow_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    FOLLOW_HOOKS.remove_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_follow_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_add_unfollow_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    UNFOLLOW_HOOKS.add_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_unfollow_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_unfollow_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    UNFOLLOW_HOOKS.remove_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_unfollow_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_add_follow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: Addr,
) -> Result<Response, ContractError> {
    follows().update(
        deps.storage,
        follow_key(&info.sender, &target),
        |follow| match follow {
            Some(_) => Err(ContractError::AlreadyFollowing),
            None => Ok(Follow {
                origin: info.sender.clone(),
                target: target.clone(),
                timestamp: env.block.time,
            }),
        },
    )?;

    let response = Response::new();

    let submsgs = FOLLOW_HOOKS.prepare_hooks(deps.storage, info.sender.clone(), |h| {
        let msg = HookMsg::new(info.sender.clone(), target.clone(), env.block.time, None);
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(HookAction::Follow)?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Follow as u64))
    })?;

    let event = Event::new("add_follow")
        .add_attribute("origin", info.sender.to_string())
        .add_attribute("target", target.to_string())
        .add_attribute("follow_timestamp", env.block.time.to_string());

    Ok(response.add_submessages(submsgs).add_event(event))
}

pub fn execute_remove_follow(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: Addr,
) -> Result<Response, ContractError> {
    let follow_key = follow_key(&info.sender, &target);
    let old_data = follows().may_load(deps.storage, follow_key.clone())?;
    match old_data {
        Some(_old_data) => follows().replace(deps.storage, follow_key, None, Some(&_old_data))?,
        None => return Err(ContractError::NotFollowing),
    }

    let response = Response::new();

    let submsgs = FOLLOW_HOOKS.prepare_hooks(deps.storage, info.sender.clone(), |h| {
        let msg = HookMsg::new(info.sender.clone(), target.clone(), env.block.time, None);
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(HookAction::Follow)?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Follow as u64))
    })?;

    let event = Event::new("add_follow")
        .add_attribute("origin", info.sender.to_string())
        .add_attribute("target", target.to_string())
        .add_attribute("follow_timestamp", env.block.time.to_string());

    Ok(response.add_submessages(submsgs).add_event(event))
}
