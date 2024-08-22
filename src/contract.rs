
use cosmwasm_std::entry_point;
//use serde::ser::StdError;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, to_json_binary, Binary};
use cosmwasm_std::{Coin, BankMsg,StdResult,StdError};
use cw2::set_contract_version;
use sha2::{Sha256, Digest};
use cosmwasm_std::Addr;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg,BetAtResponse};
use crate::state::{Config, CONFIG};
use crate::state::{BetItem, BETLIST, BETINDEX};
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
    //Instantiate the contract, setup an admin address (currently unused)
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    let admin = msg.admin.unwrap_or_else(|| info.sender.to_string());
    let rake_basis_points = msg.rake_basis_points;

    let admin_addr =Addr::unchecked(&admin);
    let config = Config {
        admin: admin_addr.clone(),
        rake_basis_points: rake_basis_points,
    };
    CONFIG.save(deps.storage, &config)?;


    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("admin", admin_addr.to_string())
        .add_attribute("rake_basis_points", rake_basis_points.to_string()))
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BetAt {address, index } => query_bet_at(deps, env, address, index),
    }
}


fn query_bet_at(deps: Deps, _env: Env, address: String, index: u32) -> StdResult<Binary> {
    let betlistkey = format!("{}.{}", address, index);
    let bet_option = BETLIST.may_load(deps.storage, &betlistkey)?;

    match bet_option {
        Some(bet_item) => {
            to_json_binary(&BetAtResponse { bet_item })
        },
        None => {
            Err(StdError::generic_err("Failed to load bet item"))
            // Or return some default Binary value using: Ok(Binary::from(vec![]))
        }
    }
}




pub mod execute {
    use cosmwasm_std::BankQuery;
use cosmwasm_std::QueryRequest;
use cosmwasm_std::BalanceResponse;
use crate::state::Outcome::VoidOutcome;
    use cosmwasm_std::CosmosMsg;
    use crate::state::Outcome::Lose;
    use crate::state::Outcome::Win;
    use super::*;

    pub fn bet(deps: DepsMut, info: MessageInfo, env: Env, guess: u32, odds: u32)  -> Result<Response, ContractError> {
        //get_random is calls different function depending on whether we are in test.debug mode
        //or running from a chain
        let config = CONFIG.may_load(deps.storage)?;

        let rake_basis_points: u128;
        match config {
            Some(config) => {
                rake_basis_points = config.rake_basis_points
            }
            None => {
                rake_basis_points = 0
            }
        }



        let random_array = get_random(env.clone());
        let random_result = int_in_range(random_array, 1, odds);
        let mut message = "";
        let won = guess == random_result;
        let mut outcome = if won { Win } else { Lose };
        let address = info.sender.clone();
        let (bet_amount,prize) = calculate_prize(&info, odds, won);


        //Index is used to store each result
        let bet_index = BETINDEX.may_load(deps.storage, address)?;
        let next_index :u32;

        match bet_index {
            Some(bet_index) => {
                next_index = bet_index + 1;
            }
            None => {
                next_index = 1;
            }
        }
        let _ = BETINDEX.save(deps.storage,info.sender.clone(), &next_index);
        let sender = info.sender.clone().into_string().clone();
        let betlistkey = format!("{}.{}", &sender.to_string(), next_index);


        //Check Bank has sufficient balance to pay
        let bank_balance = bank_balance(&deps, &env);
        if bank_balance < bet_amount {
            outcome = VoidOutcome;
            message = "Insufficient bank funds";
        }

        //The amount held back by the bank as a service fee
        let rake =Uint128::from( (prize.u128() * rake_basis_points) / 10000);

         let bi = BetItem {
            block: env.block.time.clone(),
            odds: odds,
            guess: guess,
            result: random_result,
            prize: prize.into(),
            bet: bet_amount,
            outcome: outcome.clone(),
            rake: rake.u128(),
            bank_balance_before: Uint128::from(bank_balance),
            bank_balance_after: Uint128::from(bank_balance) - prize + rake,
            message: message.to_string(),
        };

        let _ = BETLIST.save(deps.storage, &betlistkey, &bi);


        match outcome {
            Win => {
                //Bet is won, send prize-rake back to user
                let prize_coin = Coin {
                    denom: "urock".to_string(),
                    amount: prize-rake,
                };
                let send_msg = BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![prize_coin],
                };

                return Ok(Response::new()
                    .add_message(CosmosMsg::Bank(send_msg))
                    .add_attribute("action", if won { "win" } else { "lose" })
                    .add_attribute("guess", guess.to_string())
                    .add_attribute("key", betlistkey)
                )
            }
            Lose => {
                //Bet is lost, return result
                return Ok(Response::new()
                    .add_attribute("action", if won { "win" } else { "lose" })
                    .add_attribute("guess", guess.to_string())
                    .add_attribute("key", betlistkey)
                );
                // Add your code for the Lose outcome here
            }
            VoidOutcome => {
                //Something went wrong, return bet to sender
                let prize_coin = Coin {
                    denom: "urock".to_string(),
                    amount: bet_amount,
                };
                let send_msg = BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![prize_coin],
                };

                return Ok(Response::new()
                    .add_message(CosmosMsg::Bank(send_msg))
                    .add_attribute("action", if won { "win" } else { "lose" })
                    .add_attribute("guess", guess.to_string())
                    .add_attribute("key", betlistkey)
                );
                // Add your code for the VoidOutcome here
            }
        }


    }


    #[cfg(not(test))]
    fn bank_balance(deps: &DepsMut, env: &Env) -> Uint128 {
        let contract_address = env.contract.address.clone();
        let bb = query_native_balance(deps.as_ref(), "urock".to_string(), contract_address);
        let value = bb.unwrap();
        value
    }



   // #[cfg(not(test))]
    fn query_native_balance(
        deps: Deps,
        denom: String,
        contract_address: Addr,
    ) -> StdResult<Uint128> {
        let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: contract_address.to_string(),
            denom: denom.clone(),
        }))?;

        Ok(balance.amount.amount)
    }
    #[cfg(not(test))]
    fn get_random(env: Env)  ->[u8; 32] {
        let nsecs = env.block.time.subsec_nanos();
        let mut hasher = Sha256::new();
        hasher.update(nsecs.to_string());
        let result = hasher.finalize();
        let hex_string = hex::encode(result);
        let vec_u8 = hex::decode(hex_string).expect("Decoding failed");
        let array: [u8; 32] = vec_u8.try_into().expect("Expected length 32");
        array
    }


    #[cfg(test)]
    fn bank_balance(_deps: &DepsMut, _env: &Env) -> Uint128 {
        Uint128::from(10000u128)
    }

    #[cfg(test)]
     fn get_random(_env: Env) ->[u8; 32] {
        use rand::{Rng};
        let mut rng = rand::thread_rng();
        let num: u32 = rng.gen(); // Generate a random number
        let rnd_string = num.to_string();
        let mut hasher;
        hasher = Sha256::new();
        hasher.update(rnd_string.to_string());
        let result = hasher.finalize();
        let hex_string = hex::encode(result);
        let vec_u8 = hex::decode(hex_string).expect("Decoding failed");
        let array: [u8; 32] = vec_u8.try_into().expect("Expected length 32");
        array
    }



    fn calculate_prize(info: &MessageInfo, odds: u32, won: bool) -> (Uint128,Uint128)  {
        let sent_tokens_denom = "urock";  // replace with your token's denom
        let mut sent_tokens = Uint128::from(0u128);
        for coin in &info.funds {
            if coin.denom == sent_tokens_denom {
                sent_tokens = coin.amount;
                break;
            }
        }
        let mut prize= Uint128::from(0u128);
        if won == true {
             prize = Uint128::from(odds) * sent_tokens;
        }
        (sent_tokens, prize)
    }



}


#[cfg(test)]
mod tests {
            use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
            use cosmwasm_std::{Addr, Coin, from_json, attr};
            use crate::contract::{execute, instantiate, query};

            use crate::msg::{BetAtResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
            pub const ALICE: &str ="zen13y3tm68gmu9kntcxwvmue82p6akacnpt2v7nty";
            //pub const BOB: &str ="zen126hek6zagmp3jqf97x7pq7c0j9jqs0ndxeaqhq";




            #[test]
            fn test_instantiate() {
                // Mock the dependencies, must be mutable so we can pass it as a mutable, empty vector means our contract has no balance
                let mut deps = mock_dependencies();
                let env = mock_env();
                let info = message_info(&Addr::unchecked(ALICE), &[]);
                let msg = InstantiateMsg { admin: None, rake_basis_points: 150 };
                let res = instantiate(deps.as_mut(), env, info, msg).unwrap();

                assert_eq!(
                    res.attributes,
                    vec![attr("action", "instantiate"), attr("admin", ALICE)]
                )
            }

            #[test]
            fn test_bet(){
                let mut deps = mock_dependencies();
                let env = mock_env();

                let coins = vec![Coin {
                    denom: "urock".into(),
                    amount: 100u128.into(),
                }];
                let info = message_info(&Addr::unchecked(ALICE), &coins);
                let msg = InstantiateMsg { admin: None, rake_basis_points: 150 };
                let _response = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

                for _i in 1..=5 {
                    let msg = ExecuteMsg::Bet {
                        odds: 10,
                        guess: 8,
                    };
                    let response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
                    println!("{:?}",response);
                }

                for i in 1..=10 {
                    let msg = QueryMsg::BetAt {address: ALICE.to_string(), index: i };
                    let bin_result = query(deps.as_ref(), env.clone(), msg);
                    match bin_result {
                        Ok(bin) if bin.len() > 0 => {
                            let res: BetAtResponse = from_json(&bin).unwrap();
                            println!("{} - {}", i, res.bet_item);
                        },
                        Ok(_) => {
                            // handle the situation when bin is empty
                        },
                        Err(_) => {
                            // handle the case when an error occurred during `query`
                        }
                    }
                }

               }


        }
