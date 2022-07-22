use cosmwasm_std::{StdError, Uint128};
use cw_utils::PaymentError;
use thiserror::Error;

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

    // Expiry errors
    #[error("Invalid expiration range")]
    InvalidExpirationRange {},

    #[error("Expiry min > max")]
    InvalidExpiry {},

    // Auction errors
    #[error("InvalidReservePrice: reserve_price {0} < starting_price {1}")]
    InvalidReservePrice(Uint128, Uint128),

    #[error("AuctionAlreadyExists: token_id {0}")]
    AuctionAlreadyExists(String),

    #[error("AuctionNotFound: token_id {0}")]
    AuctionNotFound(String),

    #[error("AuctionNotExpired")]
    AuctionNotExpired {},

    #[error("ReservePriceRestriction: {0}")]
    ReservePriceRestriction(String),
}
