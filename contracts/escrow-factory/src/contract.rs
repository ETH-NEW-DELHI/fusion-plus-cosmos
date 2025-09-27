use cosmwasm_std::{
    entry_point, to_json_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use shared::{validate_token_amounts, ContractError};

use crate::state::{ESCROW_DST_CODE_ID, SAFETY_DEPOSIT_TOKEN};
use crate::RESCUE_DELAY;
use interfaces::escrow_factory::{ExecuteMsg, InstantiateMsg, QueryMsg};
use shared::types::Immutables;

const CONTRACT_NAME: &str = "crates.io:escrow-factory";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    ESCROW_DST_CODE_ID.save(deps.storage, &msg.escrow_dst_code_id)?;
    SAFETY_DEPOSIT_TOKEN.save(deps.storage, &msg.safety_deposit_token)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("contract", "escrow-factory"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateEscrowDst {
            mut immutables,
            src_cancellation_timestamp,
        } => create_escrow_dst(deps, env, info, &mut immutables, src_cancellation_timestamp),
    }
}

/// Creates a new EscrowDst contract using deterministic deployment
pub fn create_escrow_dst(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    immutables: &mut Immutables,
    src_cancellation_timestamp: u64,
) -> Result<Response, ContractError> {
    // for verification in escrow dst
    let immutable_hash = immutables.compute_immutables_hash()?;

    immutables.timelocks.set_deployed_at(env.block.time);

    if immutables
        .timelocks
        .get_timelock(shared::TimelockStage::DstCancellation)
        > src_cancellation_timestamp
    {
        return Err(ContractError::InvalidCreationTime {});
    }


    // Get the safety deposit token from state
    let safety_deposit_token = SAFETY_DEPOSIT_TOKEN.load(deps.storage).unwrap();

    // Validate all required tokens with sufficient amounts
    validate_token_amounts(&immutables, &info, &safety_deposit_token)?;

    // Create instantiate message for escrow
    let instantiate_msg = to_json_binary(&interfaces::escrow_dst::InstantiateMsg {
        safety_deposit_denom: safety_deposit_token,
        rescue_delay: RESCUE_DELAY,
        immutable_hash: immutable_hash.clone()
    })?;

    let create_escrow_msg = CosmosMsg::Wasm(WasmMsg::Instantiate {
        admin: Some(info.sender.to_string()),
        code_id: ESCROW_DST_CODE_ID.load(deps.storage).unwrap(),
        msg: instantiate_msg,
        funds: info.funds,
        label: format!("escrow-dst-{}", &immutable_hash[..8]),
    });

    Ok(Response::new()
        .add_submessage(SubMsg::new(create_escrow_msg))
        .add_attribute("method", "create_escrow_dst")
        .add_attribute("hashlock", immutables.hashlock.clone())
        .add_attribute("taker", immutables.taker.clone()))
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    return to_json_binary("no queries")
}
