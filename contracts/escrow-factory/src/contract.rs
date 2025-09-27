use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    instantiate2_address,
};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::state::{ESCROW_DST_CODE_ID, ESCROW_DST_CODE_HASH, SAFETY_DEPOSIT_TOKEN};
use interfaces::escrow_factory::{ExecuteMsg, InstantiateMsg, QueryMsg};
use shared::types::Immutables;
use shared::utils::{get_code_hash};

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

    ESCROW_DST_CODE_ID.save(deps.storage, &msg.escrow_dst_code_id);
    SAFETY_DEPOSIT_TOKEN.save(deps.storage, &msg.safety_deposit_token);

    let escrow_dst_code_hash = get_code_hash(deps,msg.escrow_dst_code_id)?;
    ESCROW_DST_CODE_HASH.save(deps.storage, &escrow_dst_code_hash);

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
        } => todo!(),
    }
}


/// Compute the deterministic address for an escrow contract
/// This function extracts the core logic for computing deterministic addresses
/// using the same method as the Solidity Create2 equivalent
pub fn compute_escrow_address(deps: Deps, env: &Env, immutables: &Immutables) -> Result<String, ContractError> {
    let escrow_dst_code_id = ESCROW_DST_CODE_ID.load(deps.storage).unwrap();
    let escrow_dst_code_hash = ESCROW_DST_CODE_HASH.load(deps.storage).unwrap();

    let salt = immutables.compute_immutables_hash()?;
    
    // Convert salt string to bytes for instantiate2_address
    let salt_bytes = hex::decode(salt)
        .map_err(|_| ContractError::Std(cosmwasm_std::StdError::generic_err("Invalid salt format")))?;
    
    let escrow_address = instantiate2_address(
        &escrow_dst_code_hash, 
        &deps.api.addr_canonicalize(env.contract.address.as_str())?, 
        &salt_bytes
    ).map_err(|e| ContractError::Std(cosmwasm_std::StdError::generic_err(format!("Failed to compute address: {}", e))))?;
    
    Ok(escrow_address.to_string())
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AddressOfEscrowDst { immutables } => {
            let escrow_address = compute_escrow_address(deps, &env, &immutables)
                .map_err(|e| cosmwasm_std::StdError::generic_err(format!("Failed to compute address: {}", e)))?;
            to_json_binary(&escrow_address)
        }
    }
}
