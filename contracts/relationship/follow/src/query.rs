use crate::helpers::option_bool_to_order;
use crate::msg::{FollowsResponse, IsFollowResponse, QueryMsg, QueryOptions};
use crate::state::{follow_key, follows, Follow, FOLLOW_HOOKS, UNFOLLOW_HOOKS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use cw_storage_plus::Bound;

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::FollowHooks { target } => {
            to_binary(&FOLLOW_HOOKS.query_hooks(deps, api.addr_validate(&target)?)?)
        }
        QueryMsg::UnfollowHooks { target } => {
            to_binary(&UNFOLLOW_HOOKS.query_hooks(deps, api.addr_validate(&target)?)?)
        }
        QueryMsg::Follows {
            origin,
            query_options,
        } => to_binary(&query_follows(
            deps,
            api.addr_validate(&origin)?,
            query_options,
        )?),
        QueryMsg::Followers {
            target,
            query_options,
        } => to_binary(&query_followers(
            deps,
            api.addr_validate(&target)?,
            query_options,
        )?),
        QueryMsg::IsFollow { origin, target } => to_binary(&query_is_follow(
            deps,
            api.addr_validate(&origin)?,
            api.addr_validate(&target)?,
        )?),
    }
}

pub fn query_follows(
    deps: Deps,
    origin: Addr,
    query_options: QueryOptions<Follow>,
) -> StdResult<FollowsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((
            offset.timestamp.seconds(),
            (offset.origin.clone(), offset.target.clone()),
        ))
    });
    let order = option_bool_to_order(query_options.descending);

    let follows = follows()
        .idx
        .following
        .sub_prefix(origin)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(FollowsResponse { follows })
}

pub fn query_followers(
    deps: Deps,
    target: Addr,
    query_options: QueryOptions<Follow>,
) -> StdResult<FollowsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((
            offset.timestamp.seconds(),
            (offset.origin.clone(), offset.target.clone()),
        ))
    });
    let order = option_bool_to_order(query_options.descending);

    let follows = follows()
        .idx
        .followers
        .sub_prefix(target)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(FollowsResponse { follows })
}

pub fn query_is_follow(deps: Deps, origin: Addr, target: Addr) -> StdResult<IsFollowResponse> {
    let follow = follows().may_load(deps.storage, follow_key(&origin, &target))?;

    Ok(IsFollowResponse {
        is_follow: follow.is_some(),
    })
}
