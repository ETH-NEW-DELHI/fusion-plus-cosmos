use cosmwasm_std::{Addr, Uint256, StdResult, StdError};
use sha2::{Sha256, Digest as Sha2Digest};
use sha3::{Keccak256};
use crate::types::{Timelocks, TimelockStage};

/// Compute Keccak256 hash (compatible with Ethereum)
pub fn keccak256(data: &[u8]) -> String {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute SHA256 hash
pub fn sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Validate secret against hashlock
pub fn validate_secret(secret: &str, hashlock: &str) -> bool {
    let secret_hash = keccak256(secret.as_bytes());
    secret_hash == hashlock
}

/// Get timelock value for given stage
pub fn get_timelock(timelocks: &Timelocks, stage: TimelockStage) -> u64 {
    let base_time = timelocks.deployed_at;
    match stage {
        TimelockStage::SrcWithdrawal => base_time + timelocks.src_withdrawal as u64,
        TimelockStage::SrcPublicWithdrawal => base_time + timelocks.src_public_withdrawal as u64,
        TimelockStage::SrcCancellation => base_time + timelocks.src_cancellation as u64,
        TimelockStage::SrcPublicCancellation => base_time + timelocks.src_public_cancellation as u64,
        TimelockStage::DstWithdrawal => base_time + timelocks.dst_withdrawal as u64,
        TimelockStage::DstPublicWithdrawal => base_time + timelocks.dst_public_withdrawal as u64,
        TimelockStage::DstCancellation => base_time + timelocks.dst_cancellation as u64,
    }
}

/// Check if current time is after given timelock
pub fn is_after_timelock(current_time: u64, timelock: u64) -> bool {
    current_time >= timelock
}

/// Check if current time is before given timelock
pub fn is_before_timelock(current_time: u64, timelock: u64) -> bool {
    current_time < timelock
}

/// Validate caller is authorized
pub fn validate_caller(caller: &Addr, authorized: &Addr) -> StdResult<()> {
    if caller != authorized {
        return Err(StdError::generic_err("Unauthorized caller"));
    }
    Ok(())
}

/// Validate token amount is sufficient
pub fn validate_sufficient_balance(available: Uint256, required: Uint256) -> StdResult<()> {
    if available < required {
        return Err(StdError::generic_err("Insufficient balance"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_secret_validation() {
        let secret = "my_secret";
        let hashlock = keccak256(secret.as_bytes());
        
        assert!(validate_secret(secret, &hashlock));
        assert!(!validate_secret("wrong_secret", &hashlock));
    }
}