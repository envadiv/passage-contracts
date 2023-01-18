use crate::hooks::HookError;
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Already following")]
    AlreadyFollowing,

    #[error("Not following")]
    NotFollowing,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("{0}")]
    Hook(#[from] HookError),
}
