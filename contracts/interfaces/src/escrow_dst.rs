use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Binary, Uint256};
use shared::Immutables;

#[cw_serde]
pub struct InstantiateMsg {
    pub safety_deposit_denom: String,
    pub rescue_delay: u32,
    pub immutable_hash: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Withdraw {
        secret: Binary,
        immutables: Immutables,
    },
    PublicWithdraw {
        secret: Binary,
        immutables: Immutables,
    },
    Cancel {
        immutables: Immutables,
    },
    RescueFunds {
        token: String,
        amount: Uint256,
        immutables: Immutables,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
     #[returns(Addr)]
    QueryFactoryAddress {},
     #[returns(Addr)]
    QueryEscrowDstCodeHash {},
}