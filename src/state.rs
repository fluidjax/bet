use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Uint128;

use cosmwasm_std::Addr;
// 05 State
// - use cw_storage_plus::Item;
// + use cw_storage_plus::{Item, Map};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BetItem {
    pub bet: i32,
    pub result: i32,
    pub amount: Uint128,
}

pub const BETLIST: Map<&str, BetItem> = Map::new("betlist");


pub const BETINDEX: Map<Addr, i32> = Map::new("betindex");
