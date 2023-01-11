#![cfg(test)]
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{Addr, Empty, Timestamp, coin, coins, Coin, Decimal, Uint128};
use cw_multi_test::{App, AppBuilder, BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};

const NATIVE_DENOM: &str = "ujunox";
const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 3_000_000;

const CREATOR: &str = "creator";

fn custom_mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(CREATOR),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(INITIAL_BALANCE),
                }],
            )
            .unwrap();
    })
}

pub fn contract_follow() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::execute::execute,
        crate::instantiate::instantiate,
        crate::query::query,
    );
    Box::new(contract)
}

fn setup_block_time(router: &mut App, seconds: u64) {
    let mut block = router.block_info();
    block.time = Timestamp::from_seconds(seconds);
    router.set_block(block);
}

// Intializes accounts with balances
fn setup_accounts(router: &mut App) -> Result<(Addr, Addr,), ContractError> {
    let creator: Addr = Addr::unchecked(CREATOR);
    let user: Addr = Addr::unchecked("user");

    let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: user.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let user_native_balances = router.wrap().query_all_balances(user.clone()).unwrap();
    assert_eq!(user_native_balances, funds);

    Ok((creator, user,))
}

// Instantiates all needed contracts for testing
fn setup_contracts(
    router: &mut App,
    creator: &Addr,
) -> Result<(Addr,), ContractError> {
    // Setup follow contract
    let follow_code_id = router.store_code(contract_follow());
    let msg = InstantiateMsg {};
    let follow = router
        .instantiate_contract(
            follow_code_id,
            creator.clone(),
            &msg,
            &[],
            "Follow",
            None,
        )
        .unwrap();

    Ok((follow,))
}

#[test]
fn test_instantiate_follow_contract() {
    let mut router = custom_mock_app();

    let (creator, _user,) = setup_accounts(&mut router).unwrap();

    let (follow_contract,) = setup_contracts(&mut router, &creator).unwrap();
    assert_eq!(follow_contract, Addr::unchecked("contract0"));
}
