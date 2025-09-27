use cosmwasm_schema::{cw_serde, QueryResponses};
use shared::types::Immutables;

#[cw_serde]
pub struct InstantiateMsg {
    pub escrow_dst_code_id: u64,
    pub safety_deposit_token: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateEscrowDst {
        immutables: Immutables,
        src_cancellation_timestamp: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}