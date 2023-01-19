use crate::helpers::option_bool_to_order;
use crate::msg::{FriendsResponse, IsFriendResponse, QueryMsg, QueryOptions};
use crate::state::{friend_key, friends, Friend, FRIEND_HOOKS, UNFRIEND_HOOKS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, Env, StdResult};
use cw_storage_plus::{Bound, PrefixBound};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;

    match msg {
        QueryMsg::FriendHooks { target } => {
            to_binary(&FRIEND_HOOKS.query_hooks(deps, api.addr_validate(&target)?)?)
        }
        QueryMsg::UnfriendHooks { target } => {
            to_binary(&UNFRIEND_HOOKS.query_hooks(deps, api.addr_validate(&target)?)?)
        }
        QueryMsg::Friends {
            user,
            query_options,
        } => to_binary(&query_friends(
            deps,
            api.addr_validate(&user)?,
            query_options,
        )?),
        QueryMsg::SentFriendRequests {
            user,
            query_options,
        } => to_binary(&query_sent_friend_requests(
            deps,
            api.addr_validate(&user)?,
            query_options,
        )?),
        QueryMsg::ReceivedFriendRequests {
            user,
            query_options,
        } => to_binary(&query_received_friend_requests(
            deps,
            api.addr_validate(&user)?,
            query_options,
        )?),
        QueryMsg::IsFriend { origin, target } => to_binary(&query_is_friend(
            deps,
            api.addr_validate(&origin)?,
            api.addr_validate(&target)?,
        )?),
    }
}

pub fn query_friends(
    deps: Deps,
    user: Addr,
    query_options: QueryOptions<Friend>,
) -> StdResult<FriendsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let order = option_bool_to_order(query_options.descending);

    let friends_originated = friends()
        .idx
        .friends_originated
        .prefix_range(
            deps.storage,
            Some(PrefixBound::exclusive((0, user.clone()))),
            Some(PrefixBound::exclusive((u64::MAX, user.clone()))),
            order,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    let friends_targeted = friends()
        .idx
        .friends_targeted
        .prefix_range(
            deps.storage,
            Some(PrefixBound::exclusive((0, user.clone()))),
            Some(PrefixBound::exclusive((u64::MAX, user))),
            order,
        )
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    let friends = friends_originated
        .into_iter()
        .chain(friends_targeted.into_iter())
        .collect();

    Ok(FriendsResponse { friends })
}

pub fn query_sent_friend_requests(
    deps: Deps,
    user: Addr,
    query_options: QueryOptions<Friend>,
) -> StdResult<FriendsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.origin.clone(), offset.target.clone())));
    let order = option_bool_to_order(query_options.descending);

    let friends = friends()
        .idx
        .sent_friend_requests
        .prefix((0, user))
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(FriendsResponse { friends })
}

pub fn query_received_friend_requests(
    deps: Deps,
    user: Addr,
    query_options: QueryOptions<Friend>,
) -> StdResult<FriendsResponse> {
    let limit = query_options
        .limit
        .unwrap_or(DEFAULT_QUERY_LIMIT)
        .min(MAX_QUERY_LIMIT) as usize;

    let start = query_options
        .start_after
        .as_ref()
        .map(|offset| Bound::exclusive((offset.origin.clone(), offset.target.clone())));
    let order = option_bool_to_order(query_options.descending);

    let friends = friends()
        .idx
        .received_friend_requests
        .prefix((0, user))
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(FriendsResponse { friends })
}

pub fn query_is_friend(deps: Deps, origin: Addr, target: Addr) -> StdResult<IsFriendResponse> {
    let friend = friends().may_load(deps.storage, friend_key(&origin, &target))?;

    Ok(IsFriendResponse {
        is_friend: friend.is_some(),
    })
}
