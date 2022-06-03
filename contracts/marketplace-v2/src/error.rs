use cosmwasm_std::{StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

use crate::helpers::ExpiryRangeError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("UnauthorizedOwner")]
    UnauthorizedOwner {},

    #[error("UnauthorizedOperator")]
    UnauthorizedOperator {},

    #[error("InvalidPrice")]
    InvalidPrice {},

    #[error("AskExpired")]
    AskExpired {},

    #[error("AskUnchanged")]
    AskUnchanged {},

    #[error("BidExpired")]
    BidExpired {},

    #[error("BidNotStale")]
    BidNotStale {},

    #[error("PriceTooSmall: {0}")]
    PriceTooSmall(Uint128),

    #[error("Token reserved")]
    TokenReserved {},

    #[error("{0}")]
    BidPaymentError(#[from] PaymentError),

    #[error("{0}")]
    ExpiryRange(#[from] ExpiryRangeError),
}
