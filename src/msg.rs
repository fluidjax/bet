use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::state::BetItem;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Bet {
        guess: u32,
        odds: u32,
    },
    Junk {},
}



#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(BetAtResponse)]
    BetAt {address: String, index: u32},
}



#[cw_serde]
pub struct BetAtResponse {
    pub bet_item: BetItem,
}