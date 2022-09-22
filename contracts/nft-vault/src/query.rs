#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult, Addr};
use cw_storage_plus::Bound;
use crate::msg::{
    ConfigResponse, VaultTokenResponse, VaultTokensResponse, QueryMsg, QueryOptions,
    TokenTimestampOffset
};
use crate::state::{vault_tokens, CONFIG, STAKE_HOOKS, UNSTAKE_HOOKS, WITHDRAW_HOOKS};
use crate::helpers::{option_bool_to_order};

// Query limits
const DEFAULT_QUERY_LIMIT: u32 = 10;
const MAX_QUERY_LIMIT: u32 = 30;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let api = deps.api;
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps, env)?),
        QueryMsg::StakeHooks {} => to_binary(&STAKE_HOOKS.query_hooks(deps)?),
        QueryMsg::UnstakeHooks {} => to_binary(&UNSTAKE_HOOKS.query_hooks(deps)?),
        QueryMsg::WithdrawHooks {} => to_binary(&WITHDRAW_HOOKS.query_hooks(deps)?),
        QueryMsg::VaultToken { token_id } => to_binary(&query_vault_token(deps, env, token_id)?),
        QueryMsg::VaultTokensByOwner {
            owner,
            query_options
        } => to_binary(&query_vault_tokens_by_owner(deps, api.addr_validate(&owner)?, &query_options)?),
        QueryMsg::VaultTokensByStakeTimestamp {
            query_options
        } => to_binary(&query_vault_tokens_by_stake_timestamp(deps, &query_options)?),
        QueryMsg::VaultTokensByUnstakeTimestamp {
            query_options
        } => to_binary(&query_vault_tokens_by_unstake_timestamp(deps, &query_options)?),
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

fn query_vault_token(deps: Deps, _env: Env, token_id: String) -> StdResult<VaultTokenResponse> {
    let vault_token = vault_tokens().may_load(deps.storage, token_id)?;
    Ok(VaultTokenResponse { vault_token })
}

pub fn query_vault_tokens_by_owner(
    deps: Deps,
    owner: Addr,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<VaultTokensResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let vault_tokens = vault_tokens()
        .idx
        .owner_stake_timestamp
        .sub_prefix(owner)
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(VaultTokensResponse { vault_tokens })
}

pub fn query_vault_tokens_by_stake_timestamp(
    deps: Deps,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<VaultTokensResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let vault_tokens = vault_tokens()
        .idx
        .stake_timestamp
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(VaultTokensResponse { vault_tokens })
}

pub fn query_vault_tokens_by_unstake_timestamp(
    deps: Deps,
    query_options: &QueryOptions<TokenTimestampOffset>
) -> StdResult<VaultTokensResponse> {
    let limit = query_options.limit.unwrap_or(DEFAULT_QUERY_LIMIT).min(MAX_QUERY_LIMIT) as usize;
    let start = query_options.start_after.as_ref().map(|offset| {
        Bound::exclusive((offset.timestamp.seconds(), offset.token_id.clone()))
    });
    let order = option_bool_to_order(query_options.descending);

    let vault_tokens = vault_tokens()
        .idx
        .unstake_timestamp
        .range(deps.storage, start, None, order)
        .take(limit)
        .map(|res| res.map(|item| item.1))
        .collect::<StdResult<Vec<_>>>()?;

    Ok(VaultTokensResponse { vault_tokens })
}
