use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg};
use crate::state::{Config, Token, CONFIG, token_map};
use crate::ContractError;
use std::ops::Mul;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, Decimal, DepsMut, Env, MessageInfo, Response, SubMsg,
    Uint128, WasmMsg, StdError,Event
};
use cw2::{set_contract_version, get_contract_version};

const CONTRACT_NAME: &str = "crates.io:cw721-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        admin: deps.api.addr_validate(msg.admin.as_str())?,
        nft_contract_addr: deps.api.addr_validate(msg.nft_addr.as_str())?,
        allowed_native: msg.allowed_native,
        fee_percentage: msg.fee_percentage,
        collector_addr: deps.api.addr_validate(msg.collector_addr.as_str())?,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
     return Err(ContractError::Std(StdError::GenericErr { msg: "contract decommissioned, cannont execute any messages".to_string() }));

}

pub fn execute_list_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tokens: Vec<Token>,
) -> Result<Response, ContractError> {
    if tokens.is_empty() {
        return Err(ContractError::WrongInput {});
    }
    let cfg = CONFIG.load(deps.storage)?;
    let nft_contract = cw721_base::helpers::Cw721Contract(cfg.nft_contract_addr.clone());

    let mut res = Response::new();
    for t in tokens {
        let token = &mut t.clone();
        // check if sender has approval
        nft_contract
            .approval(
                &deps.querier,
                token.id.clone(),
                info.sender.clone().into_string(),
                None,
            )
            .map_err(|_e| ContractError::Unauthorized {})?;
        // will not return approval if not found
        nft_contract
            .approval(
                &deps.querier,
                token.id.clone(),
                env.contract.address.clone().into_string(),
                None,
            )
            .map_err(|_e| ContractError::NotApproved {})?;

        token.on_sale = true;
        token_map().save(deps.storage, token.id.clone(), &token)?;
        res = res.add_attribute("token", format!("token{:?}", t.id));
    }

    Ok(res.add_attribute("action", "list_token"))
}

pub fn execute_delist_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    tokens: Vec<String>,
) -> Result<Response, ContractError> {
    let mut res = Response::new();
    let cfg = CONFIG.load(deps.storage)?;

    let nft_contract = cw721_base::helpers::Cw721Contract(cfg.nft_contract_addr);
    for t in tokens {
        let mut token = token_map().load(deps.storage, t.clone())?;
        // check if sender has approval
        nft_contract
            .approval(
                &deps.querier,
                token.id.clone(),
                info.sender.clone().into_string(),
                None,
            )
            .map_err(|_e| ContractError::Unauthorized {})?;

        token.on_sale = false;
        token_map().save(deps.storage, t.clone(), &token)?;
        res = res.add_attribute("token", format!("token{:?}", t));
    }

    Ok(res.add_attribute("action", "delist_tokens"))
}

pub fn execute_buy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient_opt: Option<String>,
    token_id: String,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.funds.len() != 1 {
        return Err(ContractError::SendSingleNativeToken {});
    }
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.denom != cfg.allowed_native {
        return Err(ContractError::NativeDenomNotAllowed {
            denom: sent_fund.clone().denom,
        });
    }

    let recipient = match recipient_opt {
        None => Ok(info.sender),
        Some(r) => deps.api.addr_validate(&r),
    }?;

    let mut nft_token = token_map().load(deps.storage, token_id.clone())?;

    // check if nft is on sale
    if !nft_token.on_sale {
        return Err(ContractError::NftNotOnSale {});
    }

    // check covers the fee
    let fee = nft_token.price.mul(cfg.fee_percentage);
    if nft_token.price + fee < sent_fund.amount {
        return Err(ContractError::InsufficientBalance {
            need: nft_token.price + fee,
            sent: sent_fund.amount,
        });
    }

    // now we can buy
    let transfer_msg = cw721::Cw721ExecuteMsg::TransferNft {
        recipient: recipient.clone().into_string(),
        token_id: token_id.clone(),
    };

    let execute_transfer_msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: cfg.nft_contract_addr.clone().into_string(),
        msg: to_binary(&transfer_msg)?,
        funds: vec![],
    }
    .into();

    let nft_contract = cw721_base::helpers::Cw721Contract(cfg.nft_contract_addr.clone());
    let owner = nft_contract.owner_of(&deps.querier, token_id.clone(), false)?;
    // payout
    let owner_payout = BankMsg::Send {
        to_address: owner.owner,
        amount: vec![Coin {
            denom: cfg.allowed_native.clone(),
            amount: nft_token.price,
        }],
    };
    let fee_payout = BankMsg::Send {
        to_address: cfg.collector_addr.into_string(),
        amount: vec![Coin {
            denom: cfg.allowed_native,
            amount: fee,
        }],
    };

    // update token owner and sale status
    nft_token.on_sale = false;

    token_map().save(deps.storage, token_id.clone(), &nft_token)?;

    let res = Response::new()
        .add_submessage(SubMsg::new(execute_transfer_msg))
        .add_messages(vec![owner_payout, fee_payout])
        .add_attribute("action", "buy_native")
        .add_attribute("token_id", token_id)
        .add_attribute("recipient", recipient.to_string())
        .add_attribute("price", nft_token.price)
        .add_attribute("fee", fee);

    Ok(res)
}

pub fn execute_update_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    price: Uint128,
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    let nft_contract = cw721_base::helpers::Cw721Contract(cfg.nft_contract_addr);

    let mut token = token_map()
        .may_load(deps.storage, token_id.clone())?
        .ok_or(ContractError::NotFound {})?;

    // check if sender has approval
    nft_contract
        .approval(
            &deps.querier,
            token.id.clone(),
            info.sender.into_string(),
            None,
        )
        .map_err(|_e| ContractError::Unauthorized {})?;

    token.price = price;
    token_map().save(deps.storage, token_id.clone(), &token)?;

    Ok(Response::new()
        .add_attribute("action", "update_price")
        .add_attribute("token_id", token_id)
        .add_attribute("price", price))
}

#[allow(clippy::too_many_arguments)]
pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    nft_addr: Option<String>,
    allowed_native: Option<String>,
    fee_percentage: Option<Decimal>,
    collector_addr: Option<String>,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    if cfg.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        cfg.admin = deps.api.addr_validate(&admin)?
    }
    if let Some(nft_addr) = nft_addr {
        cfg.nft_contract_addr = deps.api.addr_validate(&nft_addr)?
    }

    if let Some(allowed_native) = allowed_native {
        cfg.allowed_native = allowed_native
    }

    if let Some(fee_percentage) = fee_percentage {
        cfg.fee_percentage = fee_percentage
    }

    if let Some(collector_addr) = collector_addr {
        cfg.collector_addr = deps.api.addr_validate(&collector_addr)?
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

#[entry_point]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let storage_version: &str = &get_contract_version(deps.storage)?.version.to_string();

    let mut response = Response::new();
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let event = Event::new("contract-migrated")
        .add_attribute("prev-version", storage_version)
        .add_attribute("next-version", CONTRACT_VERSION);
    response.events.push(event);
    Ok(response)
}
