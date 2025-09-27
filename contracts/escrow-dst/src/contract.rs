use cosmwasm_std::{to_json_binary, Uint256};
use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    CosmosMsg, BankMsg, Uint128,
};
use cw2::set_contract_version;

use shared::types::Immutables;
use shared::types::TimelockStage;
use shared::error::ContractError;
use interfaces::escrow_dst::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{IMMUTABLE_HASH, SAFETY_DEPOSIT_TOKEN};
use crate::RESCUE_DELAY;

use shared::utils::{validate_secret};
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
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    SAFETY_DEPOSIT_TOKEN.save(deps.storage, &msg.safety_deposit_denom)?;

    IMMUTABLE_HASH.save(deps.storage, &msg.immutable_hash)?;

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
        ExecuteMsg::PublicWithdraw { secret, immutables } => public_withdraw(deps, env, info, secret, immutables),
        ExecuteMsg::Cancel { immutables } => cancel(deps, env, info, immutables),
        ExecuteMsg::RescueFunds { token, amount,immutables } => rescue_funds(deps, env, info, token, amount,immutables),
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
        return Err(ContractError::TimelockHasCrossed {});
    }

    // Call internal withdraw function
    _withdraw(deps, &info, secret, &immutables)
}

pub fn public_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    secret: Binary,
    immutables: Immutables,
) -> Result<Response, ContractError> {
    if env.block.time.seconds() < immutables.timelocks.get_timelock(TimelockStage::DstPublicWithdrawal) {
        return Err(ContractError::TimelockNotReached {});
    }
    
    // Check if cancellation timelock has not been reached yet
    if env.block.time.seconds() >= immutables.timelocks.get_timelock(TimelockStage::DstCancellation) {
        return Err(ContractError::TimelockHasCrossed {});
    }

    // Call internal withdraw function
    _withdraw(deps, &info, secret, &immutables)
}

pub fn cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    immutables: Immutables,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != immutables.taker {
        return Err(ContractError::Unauthorized {});
    }
    
    validate_immutables(deps.as_ref(), &immutables)?;
    
    // Check if cancellation timelock has been reached
    let cancellation_time = immutables.timelocks.get_timelock(TimelockStage::DstCancellation);
    if env.block.time.seconds() < cancellation_time {
        return Err(ContractError::TimelockNotReached {});
    }

    let mut messages = vec![];

    messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: immutables.taker,
            amount: vec![cosmwasm_std::Coin {
                denom: immutables.token.clone(),
                amount: immutables.amount.try_into().map_err(|_| ContractError::UintConversionFailed{})?,
            }],
        }));

    messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![cosmwasm_std::Coin {
                denom: SAFETY_DEPOSIT_TOKEN.load(deps.storage)?,
                amount: immutables.safety_deposit.try_into().map_err(|_| ContractError::UintConversionFailed{})?,
            }],
        }));
    
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "cancel"))
}

pub fn rescue_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: String,
    amount: Uint256,
    immutables: Immutables,
) -> Result<Response, ContractError> {
    if info.sender.to_string() != immutables.taker {
        return Err(ContractError::Unauthorized {});
    }
    
    validate_immutables(deps.as_ref(), &immutables)?;
    
    if env.block.time.seconds() < immutables.timelocks.get_timelock(TimelockStage::RescueDelay(RESCUE_DELAY.load(deps.storage)?)) {
        return Err(ContractError::TimelockNotReached {});
    }

    let mut messages = vec![];

    messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: immutables.taker,
            amount: vec![cosmwasm_std::Coin {
                denom: token,
                amount: amount.try_into().map_err(|_| ContractError::UintConversionFailed{})?,
            }],
        }));
    
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "rescue_funds"))
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    return to_json_binary("no query")
}

fn validate_immutables(
    deps: Deps,
    immutables: &Immutables,
) -> Result<(), ContractError> {    
    let immutables_hash = immutables.compute_immutables_hash()?;
    if IMMUTABLE_HASH.load(deps.storage)? != immutables_hash {
        return Err(ContractError::InvalidImmutables{});
    }
    Ok(())
}

/// Internal withdraw function that handles fee distribution and token transfers
fn _withdraw(
    deps: DepsMut,
    info: &MessageInfo,
    secret: Binary,
    immutables: &Immutables,
) -> Result<Response, ContractError> {
    validate_immutables(deps.as_ref(), immutables)?;
    
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