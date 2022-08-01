use cosmwasm_std::{StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Invalid price")]
    InvalidPrice {},

    #[error("Bid expired")]
    BidExpired {},

    #[error("{0}")]
    BidPaymentError(#[from] PaymentError),

    #[error("Incorrect bid payment: expected {0}, actual {1}")]
    IncorrectBidPayment(Uint128, Uint128),

    // Expiry errors
    #[error("Invalid expiration range")]
    InvalidExpirationRange {},

    #[error("Expiry min > max")]
    InvalidExpiry {},
}
