#![cfg(test)]
use crate::error::ContractError;
use crate::helpers::ExpiryRange;
use crate::msg::{
    ExecuteMsg, QueryMsg, AskResponse, AsksResponse, QueryOptions, TokenPriceOffset, AskCountResponse,
    BidResponse, BidsResponse, BidExpiryOffset, ConfigResponse, CollectionBidResponse, CollectionBidsResponse,
};
use crate::state::{Ask, Bid, Config, CollectionBid};
use cosmwasm_std::{Addr, Empty, Timestamp, Attribute, coin, coins, Coin, Decimal, Uint128};
use cw721::{Cw721QueryMsg, OwnerOfResponse};
use cw721_base::msg::{ExecuteMsg as Cw721ExecuteMsg, MintMsg};
use cw_multi_test::{App, AppBuilder, BankSudo, Contract, ContractWrapper, Executor, SudoMsg as CwSudoMsg};
use pg721::msg::{InstantiateMsg as Pg721InstantiateMsg, RoyaltyInfoResponse};
use pg721::state::CollectionInfo;

const TOKEN_ID: &str = "123";
const CREATION_FEE: u128 = 1_000_000_000;
const INITIAL_BALANCE: u128 = 2000;
const NATIVE_DENOM: &str = "ujunox";
const USER: &str = "USER";

// Governance parameters
const TRADING_FEE_BPS: u64 = 200; // 2%
const ONE_DAY: u64 = 24 * 60 * 60; // 24 hours (in seconds)
const SIX_MOS: u64 = 180 * 24 * 60 * 60; // 6 months (in seconds)

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

pub fn contract_marketplace() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::execute::execute,
        crate::execute::instantiate,
        crate::query::query,
    );
    // .with_sudo(crate::sudo::sudo)
    // .with_reply(crate::execute::reply);
    Box::new(contract)
}

pub fn contract_pg721() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        pg721::contract::execute,
        pg721::contract::instantiate,
        pg721::contract::query,
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
            image:
                "ipfs://bafybeigi3bwpvyvsmnbj46ra4hyffcxdeaj6ntfk5jpic5mx27x6ih2qvq/images/1.png"
                    .to_string(),
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

    // Instantiate marketplace contract
    let marketplace_id = router.store_code(contract_marketplace());
    let msg = crate::msg::InstantiateMsg {
        cw721_address: collection.to_string(),
        denom: String::from(NATIVE_DENOM),
        collector_address: creator.to_string(),
        trading_fee_bps: TRADING_FEE_BPS,
        ask_expiry: ExpiryRange::new(ONE_DAY, SIX_MOS),
        bid_expiry: ExpiryRange::new(ONE_DAY, SIX_MOS),
        operators: vec!["operator".to_string()],
        min_price: Uint128::from(5u128),
    };
    let marketplace = router
        .instantiate_contract(
            marketplace_id,
            creator.clone(),
            &msg,
            &[],
            "Marketplace",
            None,
        )
        .unwrap();

    Ok((marketplace, collection))
}

// Intializes accounts with balances
fn setup_accounts(router: &mut App) -> Result<(Addr, Addr, Addr, Addr), ContractError> {
    let owner: Addr = Addr::unchecked("owner");
    let bidder: Addr = Addr::unchecked("bidder");
    let bidder2: Addr = Addr::unchecked("bidder2");
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
                to_address: bidder.to_string(),
                amount: funds.clone(),
            }
        }))
        .map_err(|err| println!("{:?}", err))
        .ok();
    router
        .sudo(CwSudoMsg::Bank({
            BankSudo::Mint {
                to_address: bidder2.to_string(),
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
    let bidder_native_balances = router.wrap().query_all_balances(bidder.clone()).unwrap();
    assert_eq!(bidder_native_balances, funds);
    let bidder2_native_balances = router.wrap().query_all_balances(bidder2.clone()).unwrap();
    assert_eq!(bidder2_native_balances, funds);
    let creator_native_balances = router.wrap().query_all_balances(creator.clone()).unwrap();
    assert_eq!(creator_native_balances, creator_funds);

    Ok((owner, bidder, creator, bidder2))
}

// Mints an NFT for a creator
fn mint(router: &mut App, creator: &Addr, collection: &Addr, token_id: String) {
    let mint_for_creator_msg = Cw721ExecuteMsg::Mint(MintMsg {
        token_id: token_id,
        owner: creator.clone().to_string(),
        token_uri: Some("https://starships.example.com/Starship/Enterprise.json".into()),
        extension: Empty {},
    });
    let res = router.execute_contract(
        creator.clone(),
        collection.clone(),
        &mint_for_creator_msg,
        &[],
    );
    assert!(res.is_ok());
}

fn approve(
    router: &mut App,
    creator: &Addr,
    collection: &Addr,
    marketplace: &Addr,
    token_id: String,
) {
    let approve_msg = Cw721ExecuteMsg::<Empty>::Approve {
        spender: marketplace.to_string(),
        token_id: token_id,
        expires: None,
    };
    let res = router.execute_contract(creator.clone(), collection.clone(), &approve_msg, &[]);
    assert!(res.is_ok());
}

fn ask(
    router: &mut App,
    creator: &Addr,
    marketplace: &Addr,
    token_id: String,
    price: u128,
    expires_at: Timestamp,
    reserve_for: Option<String>
) {
    let set_ask = ExecuteMsg::SetAsk {
        token_id: token_id,
        price: coin(price, NATIVE_DENOM),
        funds_recipient: None,
        reserve_for: reserve_for,
        expires_at: expires_at,
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    assert!(res.is_ok());
}

fn bid(
    router: &mut App,
    creator: &Addr,
    marketplace: &Addr,
    token_id: String,
    price: u128,
    expires_at: Timestamp,
) {
    let coin_send = coin(price, NATIVE_DENOM);
    let set_bid = ExecuteMsg::SetBid {
        token_id: token_id,
        price: coin_send.clone(),
        expires_at: expires_at,
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_bid, &[coin_send]);
    assert!(res.is_ok());
}

#[test]
fn try_add_update_remove_ask() {
    let mut router = custom_mock_app();

    // Setup intial accounts
    let (_owner, _bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

    // Mint NFT for creator
    mint(&mut router, &creator, &collection, TOKEN_ID.to_string());
    approve(&mut router, &creator, &collection, &marketplace, TOKEN_ID.to_string());

    // Should error with expiry lower than min
    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        price: coin(110, NATIVE_DENOM),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY - 1),
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    assert!(res.is_err());

    // Should error with invalid denom
    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        price: coin(110, "ujuno"),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY + 1),
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    assert!(res.is_err());

    // Should error with price below min
    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        price: coin(1, "ujuno"),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY + 1),
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    assert!(res.is_err());

    // An asking price is made by the creator
    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        price: coin(110, NATIVE_DENOM),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY + 1),
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    assert!(res.is_ok());

    // Validate Ask data is correct
    let query_ask = QueryMsg::Ask {
        token_id: TOKEN_ID.to_string(),
    };
    let res: AskResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_ask)
        .unwrap();

    let ask = match res.ask {
        Some(ask) => Ok(ask),
        None => Err("Ask not found")
    }.unwrap();
    assert_eq!(Ask {
        token_id: TOKEN_ID.to_string(),
        price: coin(110, NATIVE_DENOM),
        seller: creator.clone(),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY + 1),
    }, ask);

    // Check NFT is transferred to marketplace contract
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection.clone(), &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, marketplace.to_string());

    // Update asking price
    let set_ask = ExecuteMsg::SetAsk {
        token_id: TOKEN_ID.to_string(),
        price: coin(200, NATIVE_DENOM),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY + 1),
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &set_ask, &[]);
    assert!(res.is_ok());

    // Validate Ask data is correct
    let query_ask = QueryMsg::Ask {
        token_id: TOKEN_ID.to_string(),
    };
    let res: AskResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_ask)
        .unwrap();

    let ask = match res.ask {
        Some(ask) => Ok(ask),
        None => Err("Ask not found")
    }.unwrap();
    assert_eq!(Ask {
        token_id: TOKEN_ID.to_string(),
        price: coin(200, NATIVE_DENOM),
        seller: creator.clone(),
        funds_recipient: None,
        reserve_for: None,
        expires_at: router.block_info().time.plus_seconds(ONE_DAY + 1),
    }, ask);

    // Remove an ask
    let remove_ask = ExecuteMsg::RemoveAsk {
        token_id: TOKEN_ID.to_string(),
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &remove_ask, &[]);
    assert!(res.is_ok());

    // Validate Ask is removed
    let query_ask = QueryMsg::Ask {
        token_id: TOKEN_ID.to_string(),
    };
    let res: AskResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_ask)
        .unwrap();

    let _ask = match res.ask {
        Some(_) => Err("Ask found"),
        None => Ok(())
    }.unwrap();

    // Check NFT is transferred back to the seller
    let query_owner_msg = Cw721QueryMsg::OwnerOf {
        token_id: TOKEN_ID.to_string(),
        include_expired: None,
    };
    let res: OwnerOfResponse = router
        .wrap()
        .query_wasm_smart(collection, &query_owner_msg)
        .unwrap();
    assert_eq!(res.owner, creator.to_string());
}

#[test]
fn try_ask_queries() {
    let mut router = custom_mock_app();

    // Setup intial accounts
    let (_owner, _bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

    let query_asks = QueryMsg::Config {};
    let res: ConfigResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_asks)
        .unwrap();
    assert_eq!(Config {
        cw721_address: Addr::unchecked("contract0"),
        denom: String::from("ujunox"),
        collector_address: Addr::unchecked("creator"),
        trading_fee_percent: Decimal::percent(TRADING_FEE_BPS),
        ask_expiry: ExpiryRange::new(ONE_DAY, SIX_MOS),
        bid_expiry: ExpiryRange::new(ONE_DAY, SIX_MOS),
        operators: vec![Addr::unchecked("operator")],
        min_price: Uint128::from(5u128),
    }, res.config);

    let block_time = router.block_info().time;

    // Mint NFT for creator
    for n in 1..6 {
        mint(&mut router, &creator, &collection, n.to_string());
        approve(&mut router, &creator, &collection, &marketplace, n.to_string());

        let ts = block_time.plus_seconds(ONE_DAY + n as u64);
        ask(&mut router, &creator, &marketplace, n.to_string(), 100 + n, ts, None);
    }

    let query_asks = QueryMsg::AsksSortedByExpiry {
        query_options: QueryOptions {
            descending: Some(true),
            filter_expiry: None,
            start_after: None,
            limit: None,
        }
    };
    let res: AsksResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_asks)
        .unwrap();
    
    for n in 1..6 {
        let idx = 6 - n;
        assert_eq!(Ask {
            token_id: idx.to_string(),
            price: coin(100 + idx, NATIVE_DENOM),
            seller: creator.clone(),
            funds_recipient: None,
            reserve_for: None,
            expires_at: block_time.plus_seconds(ONE_DAY + idx as u64),
        }, res.asks[(n as usize) - 1]);
    }

    let query_asks = QueryMsg::AsksSortedByPrice {
        query_options: QueryOptions {
            descending: Some(false),
            filter_expiry: None,
            start_after: Some(TokenPriceOffset {
                price: Uint128::from(102u128),
                token_id: String::from("2")
            }),
            limit: Some(2),
        }
    };
    let res: AsksResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_asks)
        .unwrap();
    
    for n in 3..5 {
        assert_eq!(Ask {
            token_id: n.to_string(),
            price: coin(100 + n, NATIVE_DENOM),
            seller: creator.clone(),
            funds_recipient: None,
            reserve_for: None,
            expires_at: block_time.plus_seconds(ONE_DAY + n as u64),
        }, res.asks[(n as usize) - 3]);
    }

    let query_asks = QueryMsg::AsksBySellerExpiry {
        seller: creator.to_string(),
        query_options: QueryOptions {
            descending: None,
            filter_expiry: Some(block_time.plus_seconds(ONE_DAY + 2u64)),
            start_after: None,
            limit: None,
        }
    };
    let res: AsksResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_asks)
        .unwrap();
    
    for n in 3..6 {
        assert_eq!(Ask {
            token_id: n.to_string(),
            price: coin(100 + n, NATIVE_DENOM),
            seller: creator.clone(),
            funds_recipient: None,
            reserve_for: None,
            expires_at: block_time.plus_seconds(ONE_DAY + n as u64),
        }, res.asks[(n as usize) - 3]);
    }

    let query_asks = QueryMsg::AskCount { };
    let res: AskCountResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_asks)
        .unwrap();
    assert_eq!(res.count, 5u32);
}

#[test]
fn try_set_bid() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

    let n = 1;
    let ts = block_time.plus_seconds(ONE_DAY + n as u64);
    mint(&mut router, &creator, &collection, n.to_string());
    approve(&mut router, &creator, &collection, &marketplace, n.to_string());
    ask(&mut router, &creator, &marketplace, n.to_string(), 100, ts, None);

    // Create bid
    let coin_send = coin(100, NATIVE_DENOM);
    let set_bid = ExecuteMsg::SetBid {
        token_id: n.to_string(),
        price: coin_send.clone(),
        expires_at: block_time.plus_seconds(ONE_DAY + 100 as u64),
    };
    let res = router.execute_contract(bidder.clone(), marketplace.clone(), &set_bid, &[coin_send.clone()]).unwrap();

    assert_eq!(res.events[1].ty, "wasm-match-bid");
    assert_eq!(res.events[1].attributes[2], Attribute {
        key: String::from("outcome"),
        value: String::from("match")
    });

    assert_eq!(res.events[2].ty, "wasm-payout-market");
    assert_eq!(res.events[2].attributes[1], Attribute {
        key: String::from("coin"),
        value: String::from("2ujunox")
    });

    assert_eq!(res.events[3].ty, "wasm-payout-royalty");
    assert_eq!(res.events[3].attributes[1], Attribute {
        key: String::from("coin"),
        value: String::from("10ujunox")
    });

    assert_eq!(res.events[4].ty, "wasm-payout-seller");
    assert_eq!(res.events[4].attributes[1], Attribute {
        key: String::from("coin"),
        value: String::from("88ujunox")
    });

    assert_eq!(res.events[6].ty, "wasm-finalize-sale");
    assert_eq!(res.events[6].attributes[5], Attribute {
        key: String::from("payment_recipient"),
        value: String::from("creator")
    });

    let n = 2;
    let ts = block_time.plus_seconds(ONE_DAY + n as u64);
    bid(&mut router, &bidder, &marketplace, n.to_string(), 100 + n, ts);

    let query_bid_msg = QueryMsg::Bid {
        token_id: n.to_string(),
        bidder: bidder.to_string(),
    };
    let res: BidResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_bid_msg)
        .unwrap();
    assert_eq!(Some(Bid {
        token_id: n.to_string(),
        bidder: bidder.clone(),
        price: coin(100 + n, NATIVE_DENOM),
        expires_at: ts,
    }), res.bid);

    // Remove bid
    let remove_bid = ExecuteMsg::RemoveBid {
        token_id: n.to_string(),
    };
    let _res = router.execute_contract(bidder.clone(), marketplace.clone(), &remove_bid, &[]).unwrap();

    let query_bid_msg = QueryMsg::Bid {
        token_id: n.to_string(),
        bidder: bidder.to_string(),
    };
    let res: BidResponse = router
        .wrap()
        .query_wasm_smart(marketplace, &query_bid_msg)
        .unwrap();
    assert_eq!(res.bid, None);
}

#[test]
fn try_bid_queries() {
    let mut router = custom_mock_app();

    // Setup intial accounts
    let (_owner, bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, _collection) = setup_contracts(&mut router, &creator).unwrap();

    let block_time = router.block_info().time;

    // Mint NFT for creator
    for n in 1..6 {
        let ts = block_time.plus_seconds(ONE_DAY + n as u64);
        bid(&mut router, &bidder, &marketplace, n.to_string(), 100 + n, ts);
    }

    let query_bids = QueryMsg::BidsSortedByExpiry {
        query_options: QueryOptions {
            descending: Some(true),
            filter_expiry: None,
            start_after: Some(BidExpiryOffset {
                expires_at: block_time.plus_seconds(ONE_DAY + 3 as u64),
                bidder: bidder.clone(),
                token_id: String::from("3"),
            }),
            limit: None,
        }
    };
    let res: BidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_bids)
        .unwrap();
    
    for n in 1..3 {
        let idx = 6 - n;
        let ts = block_time.plus_seconds(ONE_DAY + idx as u64);
        assert_eq!(Bid {
            token_id: idx.to_string(),
            price: coin(100 + idx, NATIVE_DENOM),
            bidder: bidder.clone(),
            expires_at: ts,
        }, res.bids[(n as usize) - 1]);
    }

    let query_bids = QueryMsg::BidsByTokenPrice {
        token_id: String::from("3"),
        query_options: QueryOptions {
            descending: Some(false),
            filter_expiry: None,
            start_after: None,
            limit: None,
        }
    };
    let res: BidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_bids)
        .unwrap();

    assert_eq!(1, res.bids.len());
    assert_eq!(Bid {
        token_id: String::from("3"),
        price: coin(103, NATIVE_DENOM),
        bidder: bidder.clone(),
        expires_at: block_time.plus_seconds(ONE_DAY + 3u64),
    }, res.bids[0]);

    let query_bids = QueryMsg::BidsByBidderExpiry {
        bidder: bidder.to_string(),
        query_options: QueryOptions {
            descending: None,
            filter_expiry: Some(block_time.plus_seconds(ONE_DAY + 3u64)),
            start_after: None,
            limit: None,
        }
    };
    let res: BidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_bids)
        .unwrap();
    
    for n in 4..6 {
        let ts = block_time.plus_seconds(ONE_DAY + n as u64);
        assert_eq!(Bid {
            token_id: n.to_string(),
            price: coin(100 + n, NATIVE_DENOM),
            bidder: bidder.clone(),
            expires_at: ts,
        }, res.bids[(n as usize) - 4]);
    }
}

#[test]
fn try_collection_bid_flow() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, collection) = setup_contracts(&mut router, &creator).unwrap();

    // Cannot Bid 0 units
    let ts = block_time.plus_seconds(ONE_DAY + 10 as u64);
    let collection_bid_price = coin(100u128, NATIVE_DENOM);
    let set_collection_bid = ExecuteMsg::SetCollectionBid {
        units: 0,
        price: collection_bid_price.clone(),
        expires_at: ts,
    };
    let res = router.execute_contract(bidder.clone(), marketplace.clone(), &set_collection_bid, &[collection_bid_price.clone()]);
    assert!(res.is_err());

    // Can create and remove collection_bid
    let set_collection_bid = ExecuteMsg::SetCollectionBid {
        units: 1,
        price: collection_bid_price.clone(),
        expires_at: ts,
    };
    let res = router.execute_contract(bidder.clone(), marketplace.clone(), &set_collection_bid, &[collection_bid_price.clone()]);
    assert!(res.is_ok());

    let query_collection_bid_msg = QueryMsg::CollectionBid {
        bidder: bidder.to_string(),
    };
    let res: CollectionBidResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_collection_bid_msg)
        .unwrap();
    assert_eq!(Some(CollectionBid {
        units: 1,
        bidder: bidder.clone(),
        price: collection_bid_price.clone(),
        expires_at: ts,
    }), res.collection_bid);

    let remove_collection_bid = ExecuteMsg::RemoveCollectionBid { };
    let res = router.execute_contract(bidder.clone(), marketplace.clone(), &remove_collection_bid, &[]);
    assert!(res.is_ok());

    let query_collection_bid_msg = QueryMsg::CollectionBid {
        bidder: bidder.to_string(),
    };
    let res: CollectionBidResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_collection_bid_msg)
        .unwrap();
    assert_eq!(None, res.collection_bid);

    // Can sell to collection bid with and without Ask
    let set_collection_bid = ExecuteMsg::SetCollectionBid {
        units: 2,
        price: collection_bid_price.clone(),
        expires_at: ts,
    };
    let res = router.execute_contract(bidder.clone(), marketplace.clone(), &set_collection_bid, &[
        coin(collection_bid_price.amount.u128() * 2u128, NATIVE_DENOM)
    ]);
    assert!(res.is_ok());

    // Sell to collection bid without Ask
    let token_id = String::from("1");
    mint(&mut router, &creator, &collection, token_id.clone());
    approve(&mut router, &creator, &collection, &marketplace, token_id.clone());

    let accept_collection_bid = ExecuteMsg::AcceptCollectionBid {
        token_id: token_id.clone(),
        bidder: bidder.to_string()
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &accept_collection_bid, &[]);
    assert!(res.is_ok());

    let query_collection_bids_by_price_msg = QueryMsg::CollectionBidsByPrice {
        query_options: QueryOptions {
            descending: Some(true),
            filter_expiry: None,
            start_after: None,
            limit: Some(1),
        }
    };
    let res: CollectionBidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_collection_bids_by_price_msg)
        .unwrap();
    assert_eq!(res.collection_bids.len(), 1);
    assert_eq!(res.collection_bids[0], CollectionBid {
        units: 1,
        bidder: bidder.clone(),
        price: collection_bid_price.clone(),
        expires_at: ts,
    });

    // Sell to collection bid with Ask
    let token_id = String::from("2");
    mint(&mut router, &creator, &collection, token_id.clone());
    approve(&mut router, &creator, &collection, &marketplace, token_id.clone());
    ask(&mut router, &creator, &marketplace, token_id.clone(), collection_bid_price.amount.u128() + 10u128, ts, None);

    let accept_collection_bid = ExecuteMsg::AcceptCollectionBid {
        token_id: token_id.clone(),
        bidder: bidder.to_string()
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &accept_collection_bid, &[]);
    assert!(res.is_ok());

    let query_collection_bids_by_price_msg = QueryMsg::CollectionBidsByPrice {
        query_options: QueryOptions {
            descending: Some(true),
            filter_expiry: None,
            start_after: None,
            limit: Some(1),
        }
    };
    let res: CollectionBidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_collection_bids_by_price_msg)
        .unwrap();
    assert_eq!(res.collection_bids.len(), 0);

    // Cannot sell to expired CollectionBid
    let set_collection_bid = ExecuteMsg::SetCollectionBid {
        units: 2,
        price: collection_bid_price.clone(),
        expires_at: ts,
    };
    let res = router.execute_contract(bidder.clone(), marketplace.clone(), &set_collection_bid, &[
        coin(collection_bid_price.amount.u128() * 2u128, NATIVE_DENOM)
    ]);
    assert!(res.is_ok());

    let query_collection_bids_by_expiry_msg = QueryMsg::CollectionBidsByExpiry {
        query_options: QueryOptions {
            descending: None,
            filter_expiry: Some(block_time),
            start_after: None,
            limit: None,
        }
    };
    let res: CollectionBidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_collection_bids_by_expiry_msg)
        .unwrap();
    assert_eq!(res.collection_bids.len(), 1);
    assert_eq!(res.collection_bids[0], CollectionBid {
        units: 2,
        bidder: bidder.clone(),
        price: collection_bid_price.clone(),
        expires_at: ts,
    });

    setup_block_time(&mut router, ts.seconds() + 1);

    let token_id = String::from("3");
    mint(&mut router, &creator, &collection, token_id.clone());
    approve(&mut router, &creator, &collection, &marketplace, token_id.clone());

    let accept_collection_bid = ExecuteMsg::AcceptCollectionBid {
        token_id: token_id.clone(),
        bidder: bidder.to_string()
    };
    let res = router.execute_contract(creator.clone(), marketplace.clone(), &accept_collection_bid, &[]);
    assert!(res.is_err());

    let block_time = router.block_info().time;
    let query_collection_bids_by_expiry_msg = QueryMsg::CollectionBidsByExpiry {
        query_options: QueryOptions {
            descending: None,
            filter_expiry: Some(block_time),
            start_after: None,
            limit: None,
        }
    };
    let res: CollectionBidsResponse = router
        .wrap()
        .query_wasm_smart(marketplace.clone(), &query_collection_bids_by_expiry_msg)
        .unwrap();
    assert_eq!(res.collection_bids.len(), 0);
}

#[test]
fn test_refund_collection_bid() {
    let mut router = custom_mock_app();
    let block_time = router.block_info().time;
    // Setup intial accounts
    let (_owner, bidder, creator, _bidder2) = setup_accounts(&mut router).unwrap();

    // Instantiate and configure contracts
    let (marketplace, _collection) = setup_contracts(&mut router, &creator).unwrap();

    // Create a bid of 10 units for 100 tokens each
    let ten_units = 10u32;
    let collection_bid_price = coin(100u128, NATIVE_DENOM);
    let ts = block_time.plus_seconds(ONE_DAY + 10 as u64);
    let set_collection_bid = ExecuteMsg::SetCollectionBid {
        units: ten_units.clone(),
        price: collection_bid_price.clone().clone(),
        expires_at: ts.clone(),
    };
    let sent_coin = coin(
        collection_bid_price.clone().amount.u128() * ten_units as u128,
        collection_bid_price.clone().denom,
    );
    let res = router.execute_contract(
        bidder.clone(),
        marketplace.clone(),
        &set_collection_bid,
        &[sent_coin],
    );
    assert!(res.is_ok());

    // Now we create a new bid of 1 unit for 100 tokens each
    // We expect the first bid to be refunded
    let one_unit = 1u32;
    let set_collection_bid = ExecuteMsg::SetCollectionBid {
        units: one_unit.clone(),
        price: collection_bid_price.clone(),
        expires_at: ts.clone(),
    };
    let sent_coin = coin(
        collection_bid_price.clone().amount.u128() * one_unit as u128,
        collection_bid_price.clone().denom,
    );
    let res = router.execute_contract(
        bidder.clone(),
        marketplace.clone(),
        &set_collection_bid,
        &[sent_coin],
    );
    assert!(res.is_ok());

    // Find the res.unwrap().events that has field "ty" named "transfer":
    let refund_event = res
        .unwrap()
        .events
        .into_iter()
        .find(|e| e.ty == "transfer")
        .unwrap();
    // Make sure the value is of 10 units * 100 tokens = 1000ujunox
    assert_eq!(
        refund_event.attributes[2].value,
        format!(
            "{}ujunox",
            collection_bid_price.amount.u128() * ten_units as u128
        )
    );
}