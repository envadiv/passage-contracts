#![cfg(test)]
use cosmwasm_std::{Addr, Empty, coins, Coin, Decimal, Uint128, Timestamp};
use cw_multi_test::{App, AppBuilder, BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use cw721_base::msg::{MintMsg};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use pg721::msg::{InstantiateMsg as Pg721InstantiateMsg, ExecuteMsg as Pg721ExecuteMsg, RoyaltyInfoResponse};
use pg721::state::CollectionInfo;
use crate::error::ContractError;
use crate::msg::{
    ExecuteMsg, QueryMsg, QueryOptions, InstantiateMsg as NftVaultInstantiateMsg,
    VaultTokenResponse, VaultTokensResponse
};

const TOKEN_ID_A: &str = "1";
const TOKEN_ID_B: &str = "2";
const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 2000;
const NATIVE_DENOM: &str = "ujunox";
const USER: &str = "USER";
const UNSTAKE_PERIOD: u64 = 60;


fn custom_mock_app() -> App {
    AppBuilder::new().build(|router, _, storage| {
        router
            .bank
            .init_balance(
                storage,
                &Addr::unchecked(USER),
                vec![Coin {
                    denom: NATIVE_DENOM.to_string(),
                    amount: Uint128::new(3_000_000),
                }],
            )
            .unwrap();
    })
}

pub fn contract_pg721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        pg721::contract::execute,
        pg721::contract::instantiate,
        pg721::contract::query,
    );
    Box::new(contract)
}

pub fn contract_nft_vault() -> Box<dyn Contract<Empty>> {
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

// Instantiates all needed contracts for testing
fn setup_contracts(
    router: &mut App,
    creator: &Addr,
) -> Result<(Addr, Addr), ContractError> {
    // Setup media contract
    let pg721_id = router.store_code(contract_pg721());
    let msg = Pg721InstantiateMsg {
        name: String::from("Test Coin"),
        symbol: String::from("TEST"),
        minter: creator.to_string(),
        collection_info: CollectionInfo {
            creator: creator.to_string(),
            description: String::from("Passage Monkeys"),
            image: "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png".to_string(),
            external_link: Some("https://example.com/external.html".to_string()),
            royalty_info: Some(RoyaltyInfoResponse {
                payment_address: creator.to_string(),
                share: Decimal::percent(10),
            }),
        },
    };
    let collection = router
        .instantiate_contract(
            pg721_id,
            creator.clone(),
            &msg,
            &coins(CREATION_FEE, NATIVE_DENOM),
            "NFT",
            None,
        )
        .unwrap();
    
    let nft_vault_id = router.store_code(contract_nft_vault());
    let msg = NftVaultInstantiateMsg {
        cw721_address: collection.to_string(),
        label: String::from("Test Vault"),
        unstake_period: UNSTAKE_PERIOD,
    };
    let nft_vault = router
        .instantiate_contract(
            nft_vault_id,
            creator.clone(),
            &msg,
            &[],
            "NFT Vault",
            None,
        )
        .unwrap();

    Ok((collection, nft_vault))
}

// Intializes accounts with balances
fn setup_accounts(router: &mut App) -> Result<(Addr, Addr), ContractError> {
    let owner: Addr = Addr::unchecked("owner");
    let creator: Addr = Addr::unchecked("creator");
    let creator_funds: Vec<Coin> = coins(CREATION_FEE, NATIVE_DENOM);
    let funds: Vec<Coin> = coins(INITIAL_BALANCE, NATIVE_DENOM);
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: owner.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: creator.to_string(),
                amount: creator_funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();

    // Check native balances
    let owner_native_balances = router.wrap().query_all_balances(owner.clone()).unwrap();
    assert_eq!(owner_native_balances, funds);
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    Ok((owner, creator))
}

// Mints an NFT
fn mint(router: &mut App, creator: &Addr, owner: &Addr, collection: &Addr, token_id: String) {
    let mint_for_owner_msg = Pg721ExecuteMsg::Mint(MintMsg {
        token_id: token_id,
        owner: owner.clone().to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Empty {},
    });
    let res = router.execute_contract(
        creator.clone(),
        collection.clone(),
        &mint_for_owner_msg,
        &[],
    );
    assert!(res.is_ok());
}

fn approve(
    router: &mut App,
    owner: &Addr,
    collection: &Addr,
    spender: &Addr,
    token_id: String,
) {
    let approve_msg = Pg721ExecuteMsg::Approve {
        spender: spender.to_string(),
        token_id: token_id,
        expires: None,
    };
    let res = router.execute_contract(owner.clone(), collection.clone(), &approve_msg, &[]);
    assert!(res.is_ok());
}

#[test]
fn try_nft_staking_happy_path() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;

    // Setup intial accounts
    let (owner, creator) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (collection, nft_vault) = setup_contracts(&mut router, &creator).unwrap();

    // Mint NFT for creator
    mint(&mut router, &creator, &creator, &collection, TOKEN_ID_A.to_string());
    approve(&mut router, &creator, &collection, &nft_vault, TOKEN_ID_A.to_string());

    // Stake NFT of unowned NFT should fail
    let stake_msg = ExecuteMsg::Stake {
        token_id: TOKEN_ID_A.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &stake_msg, &[]);
    assert!(res.is_err());

    // Mint NFT for owner
    mint(&mut router, &creator, &owner, &collection, TOKEN_ID_B.to_string());

    // Stake NFT should fail if not approved
    let stake_msg = ExecuteMsg::Stake {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &stake_msg, &[]);
    assert!(res.is_err());

    // Stake NFT should succeed if approved
    approve(&mut router, &owner, &collection, &nft_vault, TOKEN_ID_B.to_string());
    let stake_msg = ExecuteMsg::Stake {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &stake_msg, &[]);
    assert!(res.is_ok());

    // Check NFT is transferred to marketplace contract
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID_B.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, nft_vault.to_string());

    // Query vault data for NFT
    let query_owner_msg = QueryMsg::VaultToken {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res: VaultTokenResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_owner_msg)
        .unwrap();
    let vault_token = res.vault_token.unwrap();
    assert_eq!(vault_token.owner, owner.to_string());
    assert_eq!(vault_token.token_id, TOKEN_ID_B.to_string());
    assert_eq!(vault_token.unstake_timestamp, None);

    // Creator cannot unstake owner's NFT
    let unstake_msg = ExecuteMsg::Unstake {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(creator.clone(), nft_vault.clone(), &unstake_msg, &[]);
    assert!(res.is_err());

    // Owner can unstake their NFT
    let unstake_msg = ExecuteMsg::Unstake {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &unstake_msg, &[]);
    assert!(res.is_ok());

    // Query vault data for NFT
    let query_owner_msg = QueryMsg::VaultToken {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res: VaultTokenResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_owner_msg)
        .unwrap();
    let vault_token = res.vault_token.unwrap();
    assert_eq!(vault_token.stake_timestamp, vault_token.unstake_timestamp.unwrap());

    // Owner can restake their NFT, which is in the unstaking phase
    let stake_msg = ExecuteMsg::Stake {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &stake_msg, &[]);
    assert!(res.is_ok());

    // Query vault data for NFT
    let query_owner_msg = QueryMsg::VaultToken {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res: VaultTokenResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_owner_msg)
        .unwrap();
    let vault_token = res.vault_token.unwrap();
    assert_eq!(vault_token.unstake_timestamp, None);

    // Owner cannot withdraw NFT before unstaking period ends
    let unstake_msg = ExecuteMsg::Unstake {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &unstake_msg, &[]);
    assert!(res.is_ok());
    let withdraw_msg = ExecuteMsg::Withdraw {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &withdraw_msg, &[]);
    assert!(res.is_err());

    // Creator cannot withdraw owner's NFT
    setup_block_time(&mut router, block_time.seconds() + UNSTAKE_PERIOD + 1u64);
    let withdraw_msg = ExecuteMsg::Withdraw {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(creator.clone(), nft_vault.clone(), &withdraw_msg, &[]);
    assert!(res.is_err());

    // Owner can withdraw their NFT after unstaking period ends
    let withdraw_msg = ExecuteMsg::Withdraw {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &withdraw_msg, &[]);
    assert!(res.is_ok());

    let query_owner_msg = QueryMsg::VaultToken {
        token_id: TOKEN_ID_B.to_string(),
    };
    let res: VaultTokenResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.vault_token, None);
}

#[test]
fn try_nft_staking_queries() {
    let mut router = custom_mock_app();

    // Setup intial accounts
    let (owner, creator) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (collection, nft_vault) = setup_contracts(&mut router, &creator).unwrap();

    for n in 1..6 {
        mint(&mut router, &creator, &owner, &collection, n.to_string());
        approve(&mut router, &owner, &collection, &nft_vault, n.to_string());

        let block_time = router.block_info().time;
        setup_block_time(&mut router, block_time.seconds() + n);
        let stake_msg = ExecuteMsg::Stake {
            token_id: n.to_string(),
        };
        let res = router.execute_contract(owner.clone(), nft_vault.clone(), &stake_msg, &[]);
        assert!(res.is_ok());
    }

    let query_msg = QueryMsg::VaultTokensByOwner {
        owner: owner.to_string(),
        query_options: QueryOptions {
            limit: Some(2),
            descending: Some(false),
            start_after: None,
        },
    };
    let res: VaultTokensResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_msg)
        .unwrap();
    for n in 1..2 {
        assert_eq!(res.vault_tokens[n - 1].owner, owner.to_string());
        assert_eq!(res.vault_tokens[n - 1].token_id, n.to_string());
        assert_eq!(res.vault_tokens[n - 1].unstake_timestamp, None);
    }

    let query_msg = QueryMsg::VaultTokensByStakeTimestamp {
        query_options: QueryOptions {
            limit: Some(3),
            descending: Some(true),
            start_after: None,
        },
    };
    let res: VaultTokensResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_msg)
        .unwrap();
    for n in 0..3 {
        assert_eq!(res.vault_tokens[n].owner, owner.to_string());
        assert_eq!(res.vault_tokens[n].token_id, (5 - n).to_string());
        assert_eq!(res.vault_tokens[n].unstake_timestamp, None);
    }

    let query_msg = QueryMsg::VaultTokensByUnstakeTimestamp {
        query_options: QueryOptions {
            limit: Some(3),
            descending: Some(true),
            start_after: None,
        },
    };
    let res: VaultTokensResponse = router
        .wrap()
        .query_wasm_smart(nft_vault.clone(), &query_msg)
        .unwrap();
    for n in 0..3 {
        assert_eq!(res.vault_tokens[n].owner, owner.to_string());
        assert_eq!(res.vault_tokens[n].token_id, (5 - n).to_string());
        assert_eq!(res.vault_tokens[n].unstake_timestamp, None);
    }
}

#[test]
fn try_unauthorized_hook_messages() {
    let mut router = custom_mock_app();

    // Setup intial accounts
    let (owner, creator) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (_, nft_vault) = setup_contracts(&mut router, &creator).unwrap();

    let dummy_addr = Addr::unchecked("dummy");
    let unauthorized_err = ContractError::Unauthorized("only an operator can call this function".to_string()).to_string();

    let add_stake_hook_msg = ExecuteMsg::AddStakeHook {
        hook: dummy_addr.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &add_stake_hook_msg, &[]);
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        unauthorized_err,
    );

    let remove_stake_hook_msg = ExecuteMsg::RemoveStakeHook {
        hook: dummy_addr.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &remove_stake_hook_msg, &[]);
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        unauthorized_err,
    );

    let add_unstake_hook_msg = ExecuteMsg::AddUnstakeHook {
        hook: dummy_addr.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &add_unstake_hook_msg, &[]);
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        unauthorized_err,
    );

    let remove_unstake_hook_msg = ExecuteMsg::RemoveUnstakeHook {
        hook: dummy_addr.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &remove_unstake_hook_msg, &[]);
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        unauthorized_err,
    );

    let add_withdraw_hook_msg = ExecuteMsg::AddWithdrawHook {
        hook: dummy_addr.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &add_withdraw_hook_msg, &[]);
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        unauthorized_err,
    );

    let remove_withdraw_hook_msg = ExecuteMsg::RemoveWithdrawHook {
        hook: dummy_addr.to_string(),
    };
    let res = router.execute_contract(owner.clone(), nft_vault.clone(), &remove_withdraw_hook_msg, &[]);
    assert_eq!(
        res.unwrap_err().source().unwrap().to_string(),
        unauthorized_err,
    );
}