use std::collections::HashMap;

use cosmwasm_std::{Addr, Deps, HexBinary, StdError, StdResult, Uint256, MessageInfo};
use sha2::{Digest as Sha2Digest, Sha256};
use sha3::Keccak256;

use crate::{ContractError, Immutables};

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

/// Validates that all required tokens are provided with sufficient amounts
pub fn validate_token_amounts(
    immutables: &Immutables,
    info: &MessageInfo,
    safety_deposit_token: &str,
) -> Result<(), ContractError> {
    let funds_map: HashMap<String, Uint256> = info.funds
        .iter()
        .map(|coin| (coin.denom.clone(), Uint256::from(coin.amount)))
        .collect();
    
    let is_same_token = safety_deposit_token == immutables.token;
    
    if is_same_token {
        let total_expected = immutables.safety_deposit + immutables.amount;
        let actual_amount = funds_map.get(&immutables.token).copied().unwrap_or_else(Uint256::zero);
        
        if actual_amount < total_expected {
            return Err(ContractError::InsufficientTokenAmount {
                token: immutables.token.clone(),
                expected: total_expected.to_string(),
                actual: actual_amount.to_string(),
            });
        }
    } else {
        
        let actual_safety_deposit_amount = funds_map
            .get(safety_deposit_token)
            .copied()
            .unwrap_or_else(Uint256::zero);
        
        if actual_safety_deposit_amount < immutables.safety_deposit {
            return Err(ContractError::InsufficientTokenAmount {
                token: safety_deposit_token.to_string(),
                expected: immutables.safety_deposit.to_string(),
                actual: actual_safety_deposit_amount.to_string(),
            });
        }
        
        let actual_main_amount = funds_map
            .get(&immutables.token)
            .copied()
            .unwrap_or_else(Uint256::zero);
        
        if actual_main_amount < immutables.amount {
            return Err(ContractError::InsufficientTokenAmount {
                token: immutables.token.clone(),
                expected: immutables.amount.to_string(),
                actual: actual_main_amount.to_string(),
            });
        }
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
