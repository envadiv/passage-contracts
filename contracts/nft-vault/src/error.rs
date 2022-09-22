use cosmwasm_std::{StdError};
use thiserror::Error;
use crate::hooks::HookError;
use cw_utils::PaymentError;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("{0}")]
    Hook(#[from] HookError),

    #[error("{0}")]
    PaymentError(#[from] PaymentError),

    #[error("InvalidStatus: {0}")]
    InvalidStatus(String),
}
