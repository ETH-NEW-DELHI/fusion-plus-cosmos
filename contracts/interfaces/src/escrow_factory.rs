use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr,instantiate2_address};
use shared::types::Immutables;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    CreateEscrowDst {
        immutables: Immutables,
        src_cancellation_timestamp: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Addr)]
    AddressOfEscrowDst { immutables: Immutables },
}