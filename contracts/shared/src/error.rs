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
}