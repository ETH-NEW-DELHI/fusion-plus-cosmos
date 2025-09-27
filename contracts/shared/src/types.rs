use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdError, StdResult, Timestamp, Uint256};
use serde::{Deserialize, Serialize};

use crate::keccak256;

#[cw_serde]
pub struct Immutables {
    pub order_hash: String,
    pub hashlock: String,
    pub maker: String,
    pub taker: String,
    pub token: String,
    pub amount: Uint256,
    pub safety_deposit: Uint256,
    pub timelocks: Timelocks,
    pub parameters: Vec<u8>,
}

impl Immutables {
    /// Compute hash of immutables for deterministic address (EIP-712 style)
    pub fn compute_immutables_hash(&self) -> StdResult<String> {
        let parameters_hash = keccak256(&self.parameters);

        let mut modified_immutables = self.clone();

        // Convert the hex string hash back to bytes (32 bytes for keccak256)
        let parameters_hash_bytes = hex::decode(&parameters_hash)
            .map_err(|_| StdError::generic_err("Failed to decode parameters hash"))?;
        modified_immutables.parameters = parameters_hash_bytes;

        let serialized = bincode::serialize(&modified_immutables)
            .map_err(|_| StdError::serialize_err("Immutables", "Failed to serialize"))?;
        Ok(keccak256(&serialized))
    }
}

#[cw_serde]
pub struct Timelocks {
    pub deployed_at: u64,
    pub src_withdrawal: u32,
    pub src_public_withdrawal: u32,
    pub src_cancellation: u32,
    pub src_public_cancellation: u32,
    pub dst_withdrawal: u32,
    pub dst_public_withdrawal: u32,
    pub dst_cancellation: u32,
}

impl Timelocks {
    pub fn set_deployed_at(&mut self, time: Timestamp) {
        self.deployed_at = time.seconds()
    }

    /// Get timelock value for given stage
    pub fn get_timelock(&self, stage: TimelockStage) -> u64 {
        let base_time = self.deployed_at;
        match stage {
            TimelockStage::SrcWithdrawal => base_time + self.src_withdrawal as u64,
            TimelockStage::SrcPublicWithdrawal => {
                base_time + self.src_public_withdrawal as u64
            }
            TimelockStage::SrcCancellation => base_time + self.src_cancellation as u64,
            TimelockStage::SrcPublicCancellation => {
                base_time + self.src_public_cancellation as u64
            }
            TimelockStage::DstWithdrawal => base_time + self.dst_withdrawal as u64,
            TimelockStage::DstPublicWithdrawal => {
                base_time + self.dst_public_withdrawal as u64
            }
            TimelockStage::DstCancellation => base_time + self.dst_cancellation as u64,
            TimelockStage::RescueDelay(delay) => base_time + delay as u64
        }
    }
}

/// Timelock stages enum
#[cw_serde]
pub enum TimelockStage {
    SrcWithdrawal,
    SrcPublicWithdrawal,
    SrcCancellation,
    SrcPublicCancellation,
    DstWithdrawal,
    DstPublicWithdrawal,
    DstCancellation,
    RescueDelay(u32),
}

#[derive(Serialize, Deserialize)]
pub struct CodeChecksumResponse {
    pub checksum_hex: String,
}
