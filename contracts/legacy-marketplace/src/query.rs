#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use crate::msg::{ConfigResponse, QueryMsg, TokenResponse, TokensResponse};
use crate::state::{CONFIG, ON_SALE, token_map};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&ConfigResponse {
            config: CONFIG.load(deps.storage)?,
        }),
        QueryMsg::Token { id } => to_binary(&TokenResponse {
            token: token_map().load(deps.storage, id)?,
        }),
        QueryMsg::RangeTokens { start_after, limit } => {
            to_binary(&range_tokens(deps, start_after, limit)?)
        }
        QueryMsg::ListTokens { ids } => to_binary(&list_tokens(deps, ids)?),
        QueryMsg::ListTokensOnSale { start_after, limit } => to_binary(&range_tokens_on_sale(deps, start_after, limit)?)
    }
}

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

pub fn range_tokens(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let tokens = records?.into_iter().map(|r| r.1).collect();

    Ok(TokensResponse { tokens })
}

pub fn list_tokens(deps: Deps, ids: Vec<String>) -> StdResult<TokensResponse> {
    let tokens: StdResult<Vec<_>> = ids
        .into_iter()
        .map(|id| token_map().load(deps.storage, id))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}

pub fn list_tokens_on_sale(deps: Deps, ids: Vec<String>) -> StdResult<TokensResponse> {
    let tokens: StdResult<Vec<_>> = ids
        .into_iter()
        .map(|id| token_map().load(deps.storage, id))
        .collect();

    Ok(TokensResponse { tokens: tokens? })
}

pub fn range_tokens_on_sale(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<TokensResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let start = start_after.map(Bound::exclusive);

    let records: StdResult<Vec<_>> = token_map()
        .idx.on_sale
        .prefix(ON_SALE)
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let tokens = records?.into_iter().map(|r| r.1).collect();

    Ok(TokensResponse { tokens })
}
