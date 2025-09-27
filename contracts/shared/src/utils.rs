use cosmwasm_std::{Addr, Deps, HexBinary, StdError, StdResult, Uint256};
use sha2::{Digest as Sha2Digest, Sha256};
use sha3::Keccak256;

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

/// Check if current time is after given timelock
pub fn is_after_timelock(current_time: u64, timelock: u64) -> bool {
    current_time >= timelock
}

/// Check if current time is before given timelock
pub fn is_before_timelock(current_time: u64, timelock: u64) -> bool {
    current_time < timelock
}

/// Get the code hash of a contract using its code ID
/// This is essential for deterministic address computation in CosmWasm Instantiate2
pub fn get_code_hash(deps: Deps, code_id: u64) -> StdResult<HexBinary> {
    // Query the code info to get the code hash
    let code_info = deps.querier.query_wasm_code_info(code_id)?;
    
    // Convert the code hash to hex string
    Ok(code_info.checksum)
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
