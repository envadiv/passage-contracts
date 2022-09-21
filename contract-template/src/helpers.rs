use crate::error::ContractError;
use crate::state::{Config};
use cosmwasm_std::{Addr, Api, StdResult, MessageInfo};

pub fn map_validate(api: &dyn Api, addresses: &[String]) -> StdResult<Vec<Addr>> {
    addresses
        .iter()
        .map(|addr| api.addr_validate(addr))
        .collect()
}

/// Checks to enforce only privileged operators
pub fn only_operator(info: &MessageInfo, config: &Config) -> Result<Addr, ContractError> {
    if !config
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::Unauthorized(String::from("only an operator can call this function")));
    }

    Ok(info.sender.clone())
}