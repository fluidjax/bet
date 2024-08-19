use std::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Uint128;

use cosmwasm_std::Addr;
// 05 State
// - use cw_storage_plus::Item;
// + use cw_storage_plus::{Item, Map};
use cw_storage_plus::{Item, Map};
use cosmwasm_std::Timestamp;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
}
pub const CONFIG: Item<Config> = Item::new("config");



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Outcome {  // Define the Outcome enum
    Win,
    Lose,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BetItem {
    pub block: Timestamp,
    pub odds: u32, // 1 in odds
    pub guess: u32,
    pub result: u32,
    pub prize: u128,
    pub bet: Uint128,
    pub outcome: Outcome,
}



impl fmt::Display for BetItem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
               "Block: {}, Odds: {}, Guess: {}, Result: {}, Prize: {}, Bet: {}, Outcome {}",
               self.block, self.odds, self.guess, self.result, self.prize, self.bet.u128(), self.outcome
        )
    }
}

impl fmt::Display for Outcome {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Outcome::Win => write!(f, "Win"),
            Outcome::Lose => write!(f, "Lose"),
        }
    }
}


pub const BETLIST: Map<&str, BetItem> = Map::new("betlist");


pub const BETINDEX: Map<Addr, u32> = Map::new("betindex");
