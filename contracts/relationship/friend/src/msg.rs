use crate::state::Friend;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Addr, Binary, StdResult, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    AddFriendHook { hook: String },
    RemoveFriendHook { hook: String },
    AddUnfriendHook { hook: String },
    RemoveUnfriendHook { hook: String },
    AddFriend { target: String },
    AcceptFriendRequest { origin: String },
    RemoveFriend { friend_addr: String },
}

#[cw_serde]
pub struct QueryOptions<T> {
    pub descending: Option<bool>,
    pub start_after: Option<T>,
    pub limit: Option<u32>,
}

#[cw_serde]
pub enum QueryMsg {
    FriendHooks {
        target: String,
    },
    UnfriendHooks {
        target: String,
    },
    Friends {
        user: String,
        query_options: QueryOptions<Friend>,
    },
    SentFriendRequests {
        user: String,
        query_options: QueryOptions<Friend>,
    },
    ReceivedFriendRequests {
        user: String,
        query_options: QueryOptions<Friend>,
    },
    IsFriend {
        origin: String,
        target: String,
    },
}

#[cw_serde]
pub struct FriendsResponse {
    pub friends: Vec<Friend>,
}

#[cw_serde]
pub struct IsFriendResponse {
    pub is_friend: bool,
}

#[cw_serde]
pub enum HookAction {
    Friend,
    Unfriend,
}

#[cw_serde]
pub struct HookMsg {
    pub origin: Addr,
    pub target: Addr,
    pub friend_timestamp: Timestamp,
    pub unfriend_timestamp: Option<Timestamp>,
}

impl HookMsg {
    pub fn new(
        origin: Addr,
        target: Addr,
        friend_timestamp: Timestamp,
        unfriend_timestamp: Option<Timestamp>,
    ) -> Self {
        HookMsg {
            origin,
            target,
            friend_timestamp,
            unfriend_timestamp,
        }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Friend => HookExecuteMsg::FriendHook(self),
            HookAction::Unfriend => HookExecuteMsg::UnfriendHook(self),
        };
        to_binary(&msg)
    }
}

#[cw_serde]
pub enum HookExecuteMsg {
    FriendHook(HookMsg),
    UnfriendHook(HookMsg),
}

pub enum HookReply {
    Friend = 1,
    Unfriend,
}
