use cosmwasm_schema::cw_serde;

use crate::hooks::Hooks;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Index, IndexList, IndexedMap, MultiIndex};

pub fn friend_key(origin: &Addr, target: &Addr) -> (Addr, Addr) {
    (origin.clone(), target.clone())
}

pub type FriendKey = (Addr, Addr);

#[cw_serde]
pub struct Friend {
    pub origin: Addr,
    pub target: Addr,
    pub created: Timestamp,
    pub accepted: Option<Timestamp>,
}

pub struct FriendIndexes<'a> {
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
    pub friends_originated: MultiIndex<'a, (u64, Addr), Friend, FriendKey>,
    pub friends_targeted: MultiIndex<'a, (u64, Addr), Friend, FriendKey>,
    pub sent_friend_requests: MultiIndex<'a, (u64, Addr), Friend, FriendKey>,
    pub received_friend_requests: MultiIndex<'a, (u64, Addr), Friend, FriendKey>,
}

impl<'a> IndexList<Friend> for FriendIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Friend>> + '_> {
        let v: Vec<&dyn Index<Friend>> = vec![
            &self.friends_originated,
            &self.friends_targeted,
            &self.sent_friend_requests,
            &self.received_friend_requests,
        ];
        Box::new(v.into_iter())
    }
}

pub fn friends<'a>() -> IndexedMap<'a, FriendKey, Friend, FriendIndexes<'a>> {
    let indexes = FriendIndexes {
        friends_originated: MultiIndex::new(
            |_pk, f: &Friend| (f.accepted.map_or(0, |a| a.seconds()), f.origin.clone()),
            "friends",
            "friends__friends_originated",
        ),
        friends_targeted: MultiIndex::new(
            |_pk, f: &Friend| (f.accepted.map_or(0, |a| a.seconds()), f.target.clone()),
            "friends",
            "friends__friends_targeted",
        ),
        sent_friend_requests: MultiIndex::new(
            |_pk, f: &Friend| (f.accepted.map_or(0, |a| a.seconds()), f.origin.clone()),
            "friends",
            "friends__sent_friend_requests",
        ),
        received_friend_requests: MultiIndex::new(
            |_pk, f: &Friend| (f.accepted.map_or(0, |a| a.seconds()), f.target.clone()),
            "friends",
            "friends__received_friend_requests",
        ),
    };
    IndexedMap::new("friends", indexes)
}

pub const FRIEND_HOOKS: Hooks = Hooks::new("friend-hooks");
pub const UNFRIEND_HOOKS: Hooks = Hooks::new("unfriend-hooks");
