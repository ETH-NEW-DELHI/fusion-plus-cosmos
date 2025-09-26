use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint256};
use shared::Immutables;

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub enum ExecuteMsg {
    Withdraw {
        secret: String,
        immutables: Immutables,
    },
    PublicWithdraw {
        secret: String,
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
