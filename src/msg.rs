use cosmwasm_schema::{cw_serde, QueryResponses};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::state::BetItem;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bet {
        guess: i32
    }
}



#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BetAtResponse)]
    BetAt {index: i32},
}



#[cw_serde]
pub struct BetAtResponse {
    pub bet_item: BetItem,
}