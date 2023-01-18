use crate::state::Follow;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Binary, StdResult, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    AddFollowHook { hook: String },
    RemoveFollowHook { hook: String },
    AddUnfollowHook { hook: String },
    RemoveUnfollowHook { hook: String },
    AddFollow { target: String },
    RemoveFollow { target: String },
}

#[cw_serde]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub enum QueryMsg {
    FollowHooks {
        target: String,
    },
    UnfollowHooks {
        target: String,
    },
    Follows {
        origin: String,
        query_options: QueryOptions<Follow>,
    },
    Followers {
        target: String,
        query_options: QueryOptions<Follow>,
    },
    IsFollow {
        origin: String,
        target: String,
    },
}

#[cw_serde]
pub struct FollowsResponse {
    pub follows: Vec<Follow>,
}

#[cw_serde]
pub struct IsFollowResponse {
    pub is_follow: bool,
}

#[cw_serde]
pub enum HookAction {
    Follow,
    Unfollow,
}

#[cw_serde]
pub struct HookMsg {
    pub origin: Addr,
    pub target: Addr,
    pub follow_timestamp: Timestamp,
    pub unfollow_timestamp: Option<Timestamp>,
}

impl HookMsg {
    pub fn new(
        origin: Addr,
        target: Addr,
        follow_timestamp: Timestamp,
        unfollow_timestamp: Option<Timestamp>,
    ) -> Self {
        HookMsg {
            origin,
            target,
            follow_timestamp,
            unfollow_timestamp,
        }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Follow => HookExecuteMsg::FollowHook(self),
            HookAction::Unfollow => HookExecuteMsg::UnfollowHook(self),
        };
        to_binary(&msg)
    }
}

#[cw_serde]
pub enum HookExecuteMsg {
    FollowHook(HookMsg),
    UnfollowHook(HookMsg),
}

pub enum HookReply {
    Follow = 1,
    Unfollow,
}
