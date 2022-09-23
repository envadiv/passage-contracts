use cosmwasm_std::{
    Addr, Api, StdResult, MessageInfo, SubMsg, Response, WasmMsg, Event, to_binary,
    Order, Deps
};
use cw721::{Cw721ExecuteMsg};
use cw721_base::helpers::Cw721Contract;
use crate::error::ContractError;
use crate::state::{Config};

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

pub type TokenId = String;

pub fn transfer_nft(token_id: &TokenId, recipient: &Addr, collection: &Addr, response: &mut Response,) -> StdResult<()> {
    let cw721_transfer_msg = Cw721ExecuteMsg::TransferNft {
        token_id: token_id.to_string(),
        recipient: recipient.to_string(),
    };

    let exec_cw721_transfer = SubMsg::new(WasmMsg::Execute {
        contract_addr: collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    });
    response.messages.push(exec_cw721_transfer);

    let event = Event::new("transfer-nft")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("recipient", recipient.to_string());
    response.events.push(event);
    
    Ok(())
}

pub fn option_bool_to_order(descending: Option<bool>) -> Order {
    match descending {
       Some(_descending) => if _descending { Order::Descending } else { Order::Ascending },
       _ => Order::Ascending
   }
}

/// Checks to enforce only NFT owner can call
pub fn only_owner(
    deps: Deps,
    info: &MessageInfo,
    collection: &Addr,
    token_id: &str,
) -> Result<(), ContractError> {
    let res = Cw721Contract(collection.clone()).owner_of(&deps.querier, token_id, false)?;
    if res.owner != info.sender {
        return Err(ContractError::Unauthorized(String::from("only the owner can call this function")));
    }
    Ok(())
}