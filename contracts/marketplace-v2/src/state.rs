use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_storage_plus::{Index, IndexedMap, IndexList, Item, MultiIndex};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub nft_contract_addr: Addr,
    pub allowed_native: String,
    pub fee_percentage: Decimal,
    pub collector_addr: Addr,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub id: String,
    pub price: Uint128,
    pub on_sale: bool,
}

pub struct TokenIndexes<'a> {
    pub on_sale: MultiIndex<'a, &'a [u8], Token, String>,
}

impl<'a> IndexList<Token> for TokenIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Token>> + '_> {
        let v: Vec<&dyn Index<Token>> = vec![&self.on_sale];
        Box::new(v.into_iter())
    }
}

pub const ON_SALE: &[u8] = &[0u8];
pub const NON_SALE: &[u8] = &[1u8];

pub fn token_map<'a>() -> IndexedMap<'a, String, Token, TokenIndexes<'a>> {
    let indexes = TokenIndexes {
        on_sale: MultiIndex::new(
            |d: &Token| if d.on_sale { ON_SALE } else { NON_SALE },
            "tokens",
            "tokens__on_sale",
        ),
    };
    IndexedMap::new("tokens", indexes)
}
