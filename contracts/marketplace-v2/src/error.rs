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

    #[error("BidExpired")]
    BidExpired {},

    #[error("{0}")]
    BidPaymentError(#[from] PaymentError),

    #[error("IncorrectBidPayment: expected {0}, actual {1}")]
    IncorrectBidPayment(Uint128, Uint128),

    #[error("{0}")]
    ExpiryRange(#[from] ExpiryRangeError),

    // Auction errors
    #[error("InvalidReservePrice: reserve_price {0} < starting_price {1}")]
    InvalidReservePrice(Uint128, Uint128),

    #[error("AuctionAlreadyExists: token_id {0}")]
    AuctionAlreadyExists(String),
}
