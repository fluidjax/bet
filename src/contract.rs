#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, to_json_binary};
use cosmwasm_std::{Coin, BankMsg};
use cw2::set_contract_version;
use sha2::{Sha256, Digest};
// use cw2::set_contract_version;
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
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BetAt {address, index } => query_bet_at(deps, env, address, index),
    }
}


fn query_bet_at(deps: Deps, _env: Env, address: String, index: u32) -> StdResult<Binary> {
    let betlistkey = format!("{}.{}", address, index);
    let b = BETLIST.may_load(deps.storage,&betlistkey)?;
    to_json_binary(&BetAtResponse{bet_item:b.expect("REASON")})
}




pub mod execute {
    use cosmwasm_std::BankQuery;
    use cosmwasm_std::QueryRequest;
    use cosmwasm_std::BalanceResponse;
    use cosmwasm_std::CosmosMsg;
    use crate::state::Outcome::Lose;
    use crate::state::Outcome::Win;
    use super::*;

    pub fn bet(deps: DepsMut, info: MessageInfo, env: Env, guess: u32, odds: u32)  -> Result<Response, ContractError> {
        //get_random is calls different function depending on whether we are in test.debug mode
        //or running from a chain
        let array = get_random(env.clone());
        let res = int_in_range(array, 1, odds);

        let won = guess == res;

        let address = info.sender.clone();
        let (bet_amount,prize) = calculate_prize(&info, odds, won);
        let mut send_msg: Option<BankMsg> = None;

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


        let contract_address = env.contract.address;
        let bb = query_native_balance(deps.as_ref(), "urock".to_string(), contract_address);
        let value = bb.unwrap();
        let bank_balance: Uint128 = Uint128::from(value);

        let bi = BetItem {
            block: env.block.time.clone(),
            odds: odds,
            guess: guess,
            result: res,
            prize: prize.into(),
            bet: bet_amount,
            outcome: if won { Win } else { Lose },
            bank_balance: bank_balance,
        };

        let _ = BETLIST.save(deps.storage, &betlistkey, &bi);

        if won {
                let prize = Coin {
                    denom: "urock".to_string(),
                    amount: prize,
                };
                let send_msg = BankMsg::Send {
                    to_address: info.sender.to_string(),
                    amount: vec![prize],
                };

            Ok(Response::new()
                .add_message(CosmosMsg::Bank(send_msg))
               .add_attribute("action", if won { "win" } else { "lose" })
               .add_attribute("guess", guess.to_string())
               .add_attribute("key", betlistkey)
            )
        }else{
            Ok(Response::new()
                .add_attribute("action", if won { "win" } else { "lose" })
                .add_attribute("guess", guess.to_string())
                .add_attribute("key", betlistkey)
            )
        }
    }


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

    fn query_contract_balance(deps: Deps, env: Env) ->Uint128 {
        return Uint128::from(0u128)
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
                let msg = InstantiateMsg { admin: None };
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
                let msg = InstantiateMsg { admin: None };
                let _response = instantiate(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();

                for i in 1..=10 {
                    let msg = ExecuteMsg::Bet {
                        odds: 10,
                        guess: 8,
                    };
                    let _response = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();


                    // let mut key:String ="".to_string();
                    // let attributes = response.attributes;
                    // for attribute in attributes {
                    //     let k = attribute.key;
                    //     let v = attribute.value;
                    //     if k == "key" {
                    //         key = v.clone()
                    //     }
                    //     //println!("Key: {}, Value: {}", k, v.clone());
                    // }

                    let msg = QueryMsg::BetAt {address: ALICE.to_string(), index: i };
                    let bin = query(deps.as_ref(), env.clone(), msg).unwrap();
                    let res: BetAtResponse = from_json(&bin).unwrap();
                    println!("{} - {}",i,  res.bet_item);
                }

               }


        }
