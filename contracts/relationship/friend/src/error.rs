use crate::hooks::HookError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Already sent friend request")]
    AlreadySentFriendRequest,

    #[error("Already friends")]
    AlreadyFriends,

    #[error("Not friends")]
    NotFriends,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("{0}")]
    Hook(#[from] HookError),
}
