use cosmwasm_std::{
    entry_point, DepsMut, Env, MessageInfo, Response, ContractInfoResponse,
    Uint128,
};
use cw2::set_contract_version;

use shared::error::ContractError;
use interfaces::escrow_dst::{ExecuteMsg, InstantiateMsg};
use crate::state::{FACTORY_CONTRACT_ADDRESS, ESCROW_DST_CODE_HASH, SAFETY_DEPOSIT_TOKEN};
use crate::RESCUE_DELAY;

use serde::{Deserialize, Serialize};

const CONTRACT_NAME: &str = "crates.io:escrow-dst";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Fee structure decoded from immutables parameters
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FeeInfo {
    pub protocol_fee_amount: Uint128,
    pub integrator_fee_amount: Uint128,
    pub protocol_fee_recipient: String,
    pub integrator_fee_recipient: String,
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // sender is the factory contract
    FACTORY_CONTRACT_ADDRESS.save(deps.storage, &info.sender.to_string())?;

    let contract_info: ContractInfoResponse = deps
        .querier
        .query_wasm_contract_info(info.sender)?;

    let contract_code_hash = deps.querier.query_wasm_code_info(contract_info.code_id)?.checksum;

    ESCROW_DST_CODE_HASH.save(deps.storage, &contract_code_hash)?;

    SAFETY_DEPOSIT_TOKEN.save(deps.storage, &msg.safety_deposit_denom)?;

    RESCUE_DELAY.save(deps.storage, &msg.rescue_delay)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Withdraw { secret, immutables } => todo!(),
        ExecuteMsg::PublicWithdraw { secret, immutables } => todo!(),
        ExecuteMsg::Cancel { immutables } => todo!(),
        ExecuteMsg::RescueFunds { token, amount,immutables } => todo!()
    }
}