use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Insufficient escrow balance")]
    InsufficientEscrowBalance {},

    #[error("Invalid creation time")]
    InvalidCreationTime {},

    #[error("Missing required token: {token}")]
    MissingRequiredToken { token: String },

    #[error("Insufficient token amount for {token}: expected {expected}, got {actual}")]
    InsufficientTokenAmount { 
        token: String, 
        expected: String, 
        actual: String 
    },

    #[error("Invalid safety deposit token: {token}")]
    InvalidSafetyDepositToken { token: String },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid secret")]
    InvalidSecret {},

    #[error("Invalid time")]
    InvalidTime {},

    #[error("Invalid immutables")]
    InvalidImmutables {},

    #[error("Timelock has not reached")]
    TimelockNotReached {},

    #[error("Failed to convert uint256 into uint128")]
    UintConversionFailed {},
    #[error("error fee parsing")]
    ErrorFeeParsing {},
}