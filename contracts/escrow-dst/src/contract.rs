use cosmwasm_std::Uint256;
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, ContractInfoResponse,
    CosmosMsg, BankMsg, Uint128,
};
use cw2::set_contract_version;

use shared::types::Immutables;
use shared::types::TimelockStage;
use shared::error::ContractError;
use interfaces::escrow_dst::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{FACTORY_CONTRACT_ADDRESS, ESCROW_DST_CODE_HASH, SAFETY_DEPOSIT_TOKEN};
use crate::RESCUE_DELAY;

use shared::utils::{validate_secret, compute_escrow_address};
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
        ExecuteMsg::Withdraw { secret, immutables } => withdraw(deps, env, info, secret, immutables),
        ExecuteMsg::PublicWithdraw { secret, immutables } => todo!(),
        ExecuteMsg::Cancel { immutables } => todo!(),
        ExecuteMsg::RescueFunds { token, amount,immutables } => todo!(),
    }
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    secret: Binary,
    immutables: Immutables,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != immutables.taker {
        return Err(ContractError::Unauthorized {});
    }

    if env.block.time.seconds() < immutables.timelocks.get_timelock(TimelockStage::DstWithdrawal) {
        return Err(ContractError::TimelockNotReached {});
    }
    if env.block.time.seconds() >= immutables.timelocks.get_timelock(TimelockStage::DstCancellation) {
        return Err(ContractError::TimelockNotReached {});
    }

    // Call internal withdraw function
    _withdraw(deps, &env, &info, secret, &immutables)
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    Err(cosmwasm_std::StdError::generic_err("Query not implemented"))
}

fn validate_immutables(
    deps: Deps,
    env: &Env,
    immutables: &Immutables,
) -> Result<(), ContractError> {    
    let escrow_code_hash = ESCROW_DST_CODE_HASH.load(deps.storage)?;
    let factory_contract_address = FACTORY_CONTRACT_ADDRESS.load(deps.storage)?;
    let expected_address = compute_escrow_address(&immutables, escrow_code_hash, &deps.api.addr_canonicalize(&factory_contract_address)?)?;
    
    // Verify that the current contract address matches the expected address
    if env.contract.address != expected_address {
        return Err(ContractError::InvalidImmutables{});
    }
    
    Ok(())
}

/// Internal withdraw function that handles fee distribution and token transfers
fn _withdraw(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    secret: Binary,
    immutables: &Immutables,
) -> Result<Response, ContractError> {
    validate_immutables(deps.as_ref(), env, immutables)?;
    
    if !validate_secret(&secret, &immutables.hashlock) {
        return Err(ContractError::InvalidSecret {});
    }
    
    let fee_info: FeeInfo = serde_json::from_slice(&immutables.parameters)
        .map_err(|_| ContractError::ErrorFeeParsing{})?;
    
    let mut messages = vec![];
    
    if fee_info.integrator_fee_amount > Uint128::zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: fee_info.integrator_fee_recipient.clone(),
            amount: vec![cosmwasm_std::Coin {
                denom: immutables.token.clone(),
                amount: fee_info.integrator_fee_amount,
            }],
        }));
    }
    
    if fee_info.protocol_fee_amount > Uint128::zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: fee_info.protocol_fee_recipient.clone(),
            amount: vec![cosmwasm_std::Coin {
                denom: immutables.token.clone(),
                amount: fee_info.protocol_fee_amount,
            }],
        }));
    }
    
    let total_fees = fee_info.integrator_fee_amount + fee_info.protocol_fee_amount;
    let remaining_amount = immutables.amount.checked_sub(total_fees.into())
        .map_err(|_| ContractError::InsufficientEscrowBalance {})?;
    
    // Transfer remaining amount to maker
    if remaining_amount > Uint256::zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: immutables.maker.clone(),
            amount: vec![cosmwasm_std::Coin {
                denom: immutables.token.clone(),
                amount: remaining_amount.try_into().map_err(|_| ContractError::UintConversionFailed{})?,
            }],
        }));
    }
    
    // Transfer safety deposit to caller (msg.sender)
    if immutables.safety_deposit > Uint256::zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![cosmwasm_std::Coin {
                denom: SAFETY_DEPOSIT_TOKEN.load(deps.storage).unwrap(),
                amount: immutables.safety_deposit.try_into().map_err(|_| ContractError::UintConversionFailed{})?,
            }],
        }));
    }
    
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "_withdraw")
        .add_attribute("secret", secret.to_string())
        .add_attribute("maker", immutables.maker.clone())
        .add_attribute("amount", remaining_amount.to_string())
        .add_attribute("integrator_fee", fee_info.integrator_fee_amount.to_string())
        .add_attribute("protocol_fee", fee_info.protocol_fee_amount.to_string())
        .add_attribute("safety_deposit", immutables.safety_deposit.to_string()))
}