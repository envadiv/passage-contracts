use crate::state::{friend_key, friends, Friend};
use cosmwasm_std::{Addr, Order, StdResult, Storage};

pub fn option_bool_to_order(descending: Option<bool>) -> Order {
    match descending {
        Some(_descending) => {
            if _descending {
                Order::Descending
            } else {
                Order::Ascending
            }
        }
        _ => Order::Ascending,
    }
}

pub fn fetch_friend(
    storage: &dyn Storage,
    origin: Addr,
    friend_addr: Addr,
) -> StdResult<Option<Friend>> {
    let key = friend_key(&origin, &friend_addr);
    let friend = friends().may_load(storage, key)?;

    if let Some(_friend) = friend {
        return Ok(Some(_friend));
    }

    let key = friend_key(&friend_addr, &origin);
    let friend = friends().may_load(storage, key)?;

    Ok(friend)
}
