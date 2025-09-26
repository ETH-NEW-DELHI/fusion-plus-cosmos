use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Uint256};

// TODO: Verify for exact same hash function
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
}
