use std::collections::HashMap;

use cosmwasm_std::{instantiate2_address, Addr, Binary, CanonicalAddr, HexBinary, MessageInfo, QuerierWrapper, StdError, StdResult, Uint256};
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
pub fn validate_secret(secret: &Binary, hashlock: &str) -> bool {
    let secret_hash = keccak256(secret);
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
pub fn get_code_hash(querier: QuerierWrapper, code_id: u64) -> StdResult<HexBinary> {
    // Query the code info to get the code hash
    let code_info = querier.query_wasm_code_info(code_id)?;
    
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

/// Compute the deterministic address for an escrow contract
/// This function extracts the core logic for computing deterministic addresses
pub fn compute_escrow_address(
    immutables: &Immutables, 
    escrow_dst_code_hash: HexBinary,
    factory_contract_address: &CanonicalAddr,
) -> Result<String, ContractError> {

    let salt = immutables.compute_immutables_hash()?;
    
    // Convert salt string to bytes for instantiate2_address
    let salt_bytes = hex::decode(salt)
        .map_err(|_| ContractError::Std(cosmwasm_std::StdError::generic_err("Invalid salt format")))?;
    
    let escrow_address = instantiate2_address(
        &escrow_dst_code_hash, 
        factory_contract_address, 
        &salt_bytes
    ).map_err(|e| ContractError::Std(cosmwasm_std::StdError::generic_err(format!("Failed to compute address: {}", e))))?;
    
    Ok(escrow_address.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_secret_validation() {
        let secret = Binary::from(b"my_secret" as &[u8]);
        let wrong_secret = Binary::from(b"wrong_secret" as &[u8]);
        let hashlock = keccak256(&secret);

        assert!(validate_secret(&secret, &hashlock));
        assert!(!validate_secret(&wrong_secret, &hashlock));
    }
}
