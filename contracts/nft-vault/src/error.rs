use cosmwasm_std::{StdError};
use thiserror::Error;
use crate::hooks::HookError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("{0}")]
    Hook(#[from] HookError),
}
