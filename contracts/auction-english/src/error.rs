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

    // Auction errors
    #[error("Invalid reserve price: reserve_price {0} < starting_price {1}")]
    AuctionInvalidReservePrice(Uint128, Uint128),

    #[error("Invalid start / end time: ${0}")]
    AuctionInvalidStartEndTime(String),

    #[error("Auction already exists: token_id {0}")]
    AuctionAlreadyExists(String),

    #[error("Auction not found: token_id {0}")]
    AuctionNotFound(String),

    #[error("Auction invalid status: {0}")]
    AuctionInvalidStatus(String),

    #[error("Auction bid too low")]
    AuctionBidTooLow {},

    #[error("Reserve price restriction: {0}")]
    AuctionReservePriceRestriction(String),
}
