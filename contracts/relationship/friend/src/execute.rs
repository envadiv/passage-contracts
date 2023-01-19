use crate::helpers::fetch_friend;
use crate::msg::{ExecuteMsg, HookAction, HookMsg, HookReply};
use crate::state::{friend_key, friends, Friend, FRIEND_HOOKS, UNFRIEND_HOOKS};
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
        ExecuteMsg::AddFriendHook { hook } => {
            execute_add_friend_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::RemoveFriendHook { hook } => {
            execute_remove_friend_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::AddUnfriendHook { hook } => {
            execute_add_unfriend_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::RemoveUnfriendHook { hook } => {
            execute_remove_unfriend_hook(deps, info, api.addr_validate(&hook)?)
        }
        ExecuteMsg::AddFriend { target } => {
            execute_add_friend(deps, env, info, api.addr_validate(&target)?)
        }
        ExecuteMsg::AcceptFriendRequest { origin } => {
            execute_accept_friend_request(deps, env, info, api.addr_validate(&origin)?)
        }
        ExecuteMsg::RemoveFriend { friend_addr } => {
            execute_remove_friend(deps, env, info, api.addr_validate(&friend_addr)?)
        }
    }
}

pub fn execute_add_friend_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    FRIEND_HOOKS.add_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_friend_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_friend_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    FRIEND_HOOKS.remove_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_friend_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_add_unfriend_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    UNFRIEND_HOOKS.add_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_unfriend_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_remove_unfriend_hook(
    deps: DepsMut,
    info: MessageInfo,
    hook: Addr,
) -> Result<Response, ContractError> {
    UNFRIEND_HOOKS.remove_hook(deps.storage, info.sender.clone(), hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_unfriend_hook")
        .add_attribute("target", info.sender)
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn execute_add_friend(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    target: Addr,
) -> Result<Response, ContractError> {
    friends().update(
        deps.storage,
        friend_key(&info.sender, &target),
        |friend| match friend {
            Some(_friend) => match _friend.accepted {
                Some(_) => Err(ContractError::AlreadyFriends),
                None => Err(ContractError::AlreadySentFriendRequest),
            },
            None => Ok(Friend {
                origin: info.sender.clone(),
                target: target.clone(),
                created: env.block.time,
                accepted: None,
            }),
        },
    )?;

    let response = Response::new();

    let event = Event::new("add_friend")
        .add_attribute("origin", info.sender.to_string())
        .add_attribute("target", target.to_string())
        .add_attribute("created", env.block.time.to_string());

    Ok(response.add_event(event))
}

pub fn execute_accept_friend_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    origin: Addr,
) -> Result<Response, ContractError> {
    let key = friend_key(&origin, &info.sender);
    let friend_option = friends().may_load(deps.storage, key.clone())?;
    let new_friend = match friend_option {
        Some(_friend) => match _friend.accepted {
            Some(_) => Err(ContractError::AlreadyFriends),
            None => {
                let mut _friend = _friend;
                _friend.accepted = Some(env.block.time);
                friends().save(deps.storage, key.clone(), &_friend)?;
                Ok(_friend)
            }
        },
        None => Err(ContractError::NotFriends),
    }?;

    let response = Response::new();

    let submsgs = FRIEND_HOOKS.prepare_hooks(deps.storage, info.sender.clone(), |h| {
        let msg = HookMsg::new(
            info.sender.clone(),
            new_friend.target.clone(),
            env.block.time,
            None,
        );
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(HookAction::Friend)?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Friend as u64))
    })?;

    let event = Event::new("add_friend")
        .add_attribute("origin", info.sender.to_string())
        .add_attribute("target", new_friend.target.to_string())
        .add_attribute("friend_timestamp", env.block.time.to_string());

    Ok(response.add_submessages(submsgs).add_event(event))
}

pub fn execute_remove_friend(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    friend_addr: Addr,
) -> Result<Response, ContractError> {
    let friend_option = fetch_friend(deps.storage, info.sender.clone(), friend_addr)?;

    if friend_option.is_none() {
        return Err(ContractError::NotFriends);
    }

    let friend = friend_option.unwrap();
    if friend.accepted.is_none() && info.sender != friend.origin {
        return Err(ContractError::Unauthorized(
            "Only the originator can remove a pending friend request".to_string(),
        ));
    }

    friends().remove(deps.storage, friend_key(&friend.origin, &friend.target))?;

    let response = Response::new();

    let submsgs = UNFRIEND_HOOKS.prepare_hooks(deps.storage, info.sender.clone(), |h| {
        let msg = HookMsg::new(
            info.sender.clone(),
            friend.target.clone(),
            friend.created,
            Some(env.block.time),
        );
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(HookAction::Unfriend)?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Unfriend as u64))
    })?;

    let event = Event::new("add_friend")
        .add_attribute("origin", info.sender.to_string())
        .add_attribute("target", friend.target.to_string())
        .add_attribute("friend_timestamp", env.block.time.to_string());

    Ok(response.add_submessages(submsgs).add_event(event))
}
