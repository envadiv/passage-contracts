use cosmwasm_schema::cw_serde;

use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Index, IndexedMap, IndexList, MultiIndex};

pub type FollowKey = (Addr, Addr);

#[cw_serde]
pub struct Follow {
    pub origin: Addr,
    pub target: Addr,
    pub timestamp: Timestamp,
}

pub struct FollowIndexes<'a> {
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
    pub followers: MultiIndex<'a, (Addr, u64), Follow, FollowKey>,
    pub following: MultiIndex<'a, (Addr, u64), Follow, FollowKey>,
}

impl<'a> IndexList<Follow> for FollowIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Follow>> + '_> {
        let v: Vec<&dyn Index<Follow>> = vec![
            &self.followers,
            &self.following,
        ];
        Box::new(v.into_iter())
    }
}

pub fn follower_map<'a>() -> IndexedMap<'a, String, Follow, FollowIndexes<'a>> {
    let indexes = FollowIndexes {
        followers: MultiIndex::new(
            |_pk, f: &Follow| (f.origin.clone(), f.timestamp.seconds()),
            "follows",
            "follows__followers",
        ),
        following: MultiIndex::new(
            |_pk, f: &Follow| (f.target.clone(), f.timestamp.seconds()),
            "follows",
            "follows__following",
        ),
    };
    IndexedMap::new("follows", indexes)
}
