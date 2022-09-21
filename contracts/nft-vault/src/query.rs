use crate::msg::{ConfigResponse, QueryMsg};
use crate::state::{CONFIG};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
    }
}

fn query_config(deps: Deps, _env: Env) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        operators: config.operators.iter().map(|addr| addr.to_string()).collect::<Vec<String>>(),
        cw721_address: config.cw721_address.to_string(),
        label: config.label,
        unstake_period: config.unstake_period,
    })
}