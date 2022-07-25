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
    InvalidReservePrice(Uint128, Uint128),

    #[error("Auction already exists: token_id {0}")]
    AuctionAlreadyExists(String),

    #[error("Auction not found: token_id {0}")]
    AuctionNotFound(String),

    #[error("Auction expired")]
    AuctionExpired {},

    #[error("Auction not expired")]
    AuctionNotExpired {},

    #[error("Auction bid too low")]
    AuctionBidTooLow {},

    #[error("Reserve price restriction: {0}")]
    ReservePriceRestriction(String),

    #[error("Cannot remove highest bid")]
    CannotRemoveHighestBid {},
}
