#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use crate::error::ContractError;
use crate::msg::{ExecuteMsg};
use crate::state::{CONFIG};
use crate::helpers::{map_validate, only_operator};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            operators,
            label,
            unstake_period,
        } => execute_update_config(
            deps,
            env,
            info,
            operators,
            label,
            unstake_period
        ),
    }
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    operators: Option<Vec<String>>,
    label: Option<String>,
    unstake_period: Option<u64>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    only_operator(&info, &config)?;

    if let Some(_operators) = operators {
        config.operators = map_validate(deps.api, &_operators)?;
    }
    if let Some(_label) = label {
        config.label = _label;
    }
    if let Some(_unstake_period) = unstake_period {
        config.unstake_period = _unstake_period;
    }
    
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("action", "update_config")
        .add_attribute("operators", config.operators.iter().map(|addr| addr.to_string()).collect::<Vec<String>>().join(","))
        .add_attribute("cw721_address", config.cw721_address)
        .add_attribute("label", config.label)
        .add_attribute("unstake_period", config.unstake_period.to_string())
    )
}