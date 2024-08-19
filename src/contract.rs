#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_binary};
use cosmwasm_std::{coin, Coin, BankMsg};
use cw2::set_contract_version;
use sha2::{Sha256, Digest};
// use cw2::set_contract_version;
use cosmwasm_std::Addr;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg,BetAtResponse};
use crate::state::{Config, CONFIG};
use crate::state::{BetItem, BETLIST, BETINDEX};
use std::str::FromStr;
use nois::{int_in_range};
use cosmwasm_std::Uint128;


// version info for migration info
const CONTRACT_NAME: &str = "crates.io:bet";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());

    let admin_addr =Addr::unchecked(&admin);
    let config = Config {
        admin: admin_addr.clone(),
    };
    CONFIG.save(deps.storage, &config)?;


    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin_addr.to_string()))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Bet{ guess , odds} => execute::bet(deps, info, env, guess, odds),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps,  env: Env, info: MessageInfo, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BetAt {index: i32} => query_bet_at(deps, env, info),
    }
}


fn query_bet_at(deps: Deps, env: Env, info: MessageInfo) -> StdResult<Binary> {
    let sender = info.sender.clone().into_string().clone();
    let betlistkey = format!("{}.{}", &sender.to_string(), 1);

    let b = BETLIST.may_load(deps.storage,&betlistkey)?;
    to_binary(&BetAtResponse{bet_item:b.expect("REASON")})
}


pub mod execute {
    use crate::state::Outcome::Lose;
    use crate::state::Outcome::Win;
    use super::*;

    pub fn bet(deps: DepsMut, info: MessageInfo, env: Env, guess: u32, odds: u32)  -> Result<Response, ContractError> {
        let nsecs = env.block.time.subsec_nanos();
        let mut hasher = Sha256::new();
        hasher.update(nsecs.to_string());
        let result = hasher.finalize();
        let hex_string = hex::encode(result);
        let vec_u8 = hex::decode(hex_string).expect("Decoding failed");
        let array: [u8; 32] = vec_u8.try_into().expect("Expected length 32");
        let res = int_in_range(array, 1, odds);


        let won = guess == res;
        let betAmount = Uint128::from(100u128);
        let sender = info.sender.clone().into_string().clone();
        let address = info.sender.clone();


        let prize = calculate_prize(&info, odds, won);

        if won {
            let prize = Coin {
                         denom: "urock".to_string(),
                         amount: prize,
                     };
                     let send = BankMsg::Send {
                         to_address: sender.clone(),
                         amount: vec![prize],
                     };
        }


        let betIndex = BETINDEX.may_load(deps.storage, address)?;
        let mut nextIndex :u32;
        match betIndex {
            Some(mut betIndex) => {
                nextIndex = betIndex + 1;
            }
            None => {
                nextIndex = 1;
            }
        }
        BETINDEX.save(deps.storage,info.sender.clone(), &nextIndex);


        let betlistkey = format!("{}.{}", &sender.to_string(), nextIndex);

        let bi = BetItem {
            block: env.block.time,
            odds: odds,
            guess: guess,
            result: res,
            prize: prize.into(),
            bet: betAmount,
            outcome: if won { Win } else { Lose },
        };

        BETLIST.save(deps.storage, &betlistkey, &bi);


        Ok(Response::new()
           .add_attribute("action", if won { "win" } else { "lose" })
           .add_attribute("guess", guess.to_string())
        )
    }

    fn calculate_prize(info: &MessageInfo, odds: u32, won: bool) -> Uint128 {
        if won == false {
            return Uint128::from(0u128);
        }
        let sent_tokens_denom = "urock";  // replace with your token's denom
        let mut sent_tokens = Uint128::from(0u128);
        for coin in &info.funds {
            if coin.denom == sent_tokens_denom {
                sent_tokens = coin.amount;
                break;
            }
        }
        let odds_as_u128: u128 = odds as u128;


        let prize = Uint128::from(odds) * sent_tokens;
        prize
    }
}


        #[cfg(test)]
mod tests {
            use cosmwasm_std::testing::{mock_dependencies,mock_env,mock_info};
            use cosmwasm_std::{Coin, from_json};
            use cosmwasm_std::{attr };
            use crate::contract::{execute, instantiate, query};
            use crate::msg::{BetAtResponse, ExecuteMsg, InstantiateMsg, QueryMsg};

            pub const ALICE: &str ="zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty";
            pub const BOB: &str ="zen126hek6zagmp3jqf97x7pq7c0j9jqs0ndxeaqhq";


            #[test]
            fn test_instantiate() {
                // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
                let mut deps = mock_dependencies();
                let env = mock_env();
                let info = mock_info(ALICE, &[]);
                let msg = InstantiateMsg { admin: None };
                let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

                assert_eq!(
                    res.attributes,
                    vec![attr("action", "instantiate"), attr("admin", ALICE)]
                )
            }

            #[test]
            fn test_bet(){
                let won = 0;
                let lost = 0;


                let mut deps = mock_dependencies();
                let env = mock_env();

                let coins = vec![Coin {
                    denom: "urock".into(),
                    amount: 100u128.into(),
                }];

                let info =mock_info(ALICE, &coins);
                let msg = InstantiateMsg { admin: None };
                let res = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

                for i in 1..=10 {
                    let msg = ExecuteMsg::Bet {
                        odds: 10,
                        guess: 8,
                    };
                    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();


                    let msg = QueryMsg::BetAt { index: 1 };
                    let bin = query(deps.as_ref(), env.clone(), info.clone(), msg).unwrap();
                    let res: BetAtResponse = from_json(&bin).unwrap();

                    println!("{} - {}",i,  res.bet_item);
                }

               }


        }
