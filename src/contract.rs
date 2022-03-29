use crate::calculations_utils::{
    calculate_add, calculate_div, calculate_mul, calculate_sqrt, calculate_sub,
    get_calculation_string, is_add_input_correct, is_div_input_correct, is_mul_input_correct,
    is_sqrt_input_correct, is_sub_input_correct,
};
use crate::msg::{GetHistory, HandleAnswer, HandleMsg, InitMsg, QueryAnswer, QueryMsg};
use crate::state::{
    load, may_load, read_viewing_key, save, write_viewing_key, CalculationsHistory, State,
    CONFIG_KEY,
};
use crate::utils::{
    bytes_vectors_vector_to_strings_vector, strings_vector_to_bytes_vectors_vector,
};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    QueryResult, StdError, StdResult, Storage, Uint128,
};

use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use secret_toolkit::crypto::sha_256;
use std::convert::TryInto;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = State {
        prng_seed: sha_256(base64::encode(msg.prng_seed).as_bytes()).to_vec(),
    };

    save(&mut deps.storage, CONFIG_KEY, &config);
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Add { n1, n2 } => try_add(deps, env, n1, n2),
        HandleMsg::Sub { n1, n2 } => try_sub(deps, env, n1, n2),
        HandleMsg::Mul { n1, n2 } => try_mul(deps, env, n1, n2),
        HandleMsg::Div { n1, n2 } => try_div(deps, env, n1, n2),
        HandleMsg::Sqrt { n } => try_sqrt(deps, env, n),
        HandleMsg::GenerateViewingKey { entropy, .. } => {
            try_generate_viewing_key(deps, env, entropy)
        }
    }
}

pub fn try_generate_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let config: State = load(&mut deps.storage, CONFIG_KEY)?;
    let prng_seed = config.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::GenerateViewingKey { key })?),
    })
}

fn insert_result<S: Storage, A: Api, Q: Querier>(
    calculation_string: String,
    deps: &mut Extern<S, A, Q>,
    env: Env,
    insertion_status: &mut String,
) -> Result<(), StdError> {
    let mut all_history: Vec<String> = Vec::new();

    let sender_address = env.message.sender;

    let GetHistory { status, history } = may_get_history(&deps, &sender_address, None)?;
    history.map(|stored_history| all_history = stored_history);

    all_history.push(calculation_string);
    let all_history_bytes = strings_vector_to_bytes_vectors_vector(all_history);
    let calculations_history = CalculationsHistory {
        history: all_history_bytes,
    };

    let sender_canonical_address = deps.api.canonical_address(&sender_address)?;
    save(
        &mut deps.storage,
        &sender_canonical_address.as_slice().to_vec(),
        &calculations_history,
    )?;
    insertion_status.push_str("Calculation performed and recorded!");
    Ok(())
}

fn try_calculate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
    operation: String,
    is_input_correct: fn(n1: u128, n2: u128, err_msg: &mut String) -> bool,
    calculate: fn(n1: Uint128, n2: Uint128) -> StdResult<Uint128>,
) -> StdResult<HandleResponse> {
    let mut result: Option<Uint128> = None;
    let mut status = String::new();
    let mut err_msg = String::new();

    if !is_input_correct(n1.u128(), n2.u128(), &mut err_msg) {
        status = String::from(err_msg);
    } else {
        result = Some(calculate(n1, n2)?);
        let calculation_string =
            get_calculation_string(n1, n2, &String::from(operation), result.unwrap());
        insert_result(calculation_string, deps, env, &mut status)?;
    }

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CalculationResult {
            n: result,
            status: status,
        })?),
    })
}

fn try_add<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    try_calculate(
        deps,
        env,
        n1,
        n2,
        String::from("+"),
        is_add_input_correct,
        calculate_add,
    )
}

fn try_sub<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    try_calculate(
        deps,
        env,
        n1,
        n2,
        String::from("-"),
        is_sub_input_correct,
        calculate_sub,
    )
}

fn try_mul<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    try_calculate(
        deps,
        env,
        n1,
        n2,
        String::from("*"),
        is_mul_input_correct,
        calculate_mul,
    )
}

fn try_div<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    try_calculate(
        deps,
        env,
        n1,
        n2,
        String::from("*"),
        is_div_input_correct,
        calculate_div,
    )
}

fn try_sqrt<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n: Uint128,
) -> StdResult<HandleResponse> {
    try_calculate(
        deps,
        env,
        n,
        Uint128::zero(),
        String::from("√"),
        is_sqrt_input_correct,
        calculate_sqrt,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetHistory { .. } => authenticated_queries(deps, msg),
    }
}

fn authenticated_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    let (addresses, key, steps_back) = msg.get_validation_params();

    for address in addresses {
        let canonical_addr = deps.api.canonical_address(address)?;

        let expected_key = read_viewing_key(&deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                QueryMsg::GetHistory {
                    address,
                    key,
                    steps_back,
                } => to_binary(&may_get_history(&deps, &address, steps_back)),
                _ => panic!("This query type does not require authentication"),
            };
        }
    }

    Err(StdError::unauthorized())
}

fn may_get_history<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    steps_back: Option<Uint128>,
) -> StdResult<GetHistory> {
    // ) -> StdResult<Binary> {
    let answer = query_read(&deps, &address)?;
    let QueryAnswer::GetHistory(get_history_obj) = answer;

    match steps_back {
        Some(steps_back) => {
            if let GetHistory { status, history } = get_history_obj {
                match history {
                    Some(history) => {
                        return Ok(GetHistory {
                            status,
                            history: Some(get_partial_history(&history, steps_back)),
                        });
                    }
                    None => {
                        return Ok(GetHistory {
                            status,
                            history: None,
                        })
                    }
                }
            }
        }
        None => {
            if let GetHistory { status, history } = get_history_obj {
                match history {
                    Some(history) => {
                        return Ok(GetHistory {
                            status,
                            history: Some(history),
                        })
                    }
                    None => {
                        return Ok(GetHistory {
                            status,
                            history: None,
                        })
                    }
                }
            }
        }
    }

    Err(StdError::unauthorized())
}

fn query_read<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
) -> StdResult<QueryAnswer> {
    let status: String;
    let mut history: Option<Vec<String>> = None;

    let sender_address = deps.api.canonical_address(&address)?;

    // read the reminder from storage
    let result: Option<CalculationsHistory> =
        may_load(&deps.storage, &sender_address.as_slice().to_vec())
            .ok()
            .unwrap();
    match result {
        // set all response field values
        Some(stored_history) => {
            status = String::from("Calculations history present");
            history = Some(bytes_vectors_vector_to_strings_vector(
                stored_history.history,
            ));
        }
        // unless there's an error
        None => {
            status = String::from("Calculations history not found.");
        }
    };

    Ok(QueryAnswer::GetHistory(GetHistory { status, history }))
}

fn get_partial_history(history: &Vec<String>, steps_back: Uint128) -> Vec<String> {
    let steps_back_size: usize = steps_back.u128().try_into().unwrap();
    if steps_back_size > history.len() {
        return history.to_vec();
    }

    let mut partial_history_vector = &history[history.len() - steps_back_size..history.len() - 1];
    return partial_history_vector.to_vec();
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            prng_seed: String::from("waehfjklasd"),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // // it worked, let's query the state
        // let res = query(&deps, QueryMsg::GetCount {}).unwrap();
        // let value: CountResponse = from_binary(&res).unwrap();
        // assert_eq!(17, value.count);
    }

    // #[test]
    // fn increment() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();

    //     // anyone can increment
    //     let env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Increment {};
    //     let _res = handle(&mut deps, env, msg).unwrap();

    //     // should increase counter by 1
    //     let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(18, value.count);
    // }

    // #[test]
    // fn reset() {
    //     let mut deps = mock_dependencies(20, &coins(2, "token"));

    //     let msg = InitMsg { count: 17 };
    //     let env = mock_env("creator", &coins(2, "token"));
    //     let _res = init(&mut deps, env, msg).unwrap();

    //     // not anyone can reset
    //     let unauth_env = mock_env("anyone", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let res = handle(&mut deps, unauth_env, msg);
    //     match res {
    //         Err(StdError::Unauthorized { .. }) => {}
    //         _ => panic!("Must return unauthorized error"),
    //     }

    //     // only the original creator can reset the counter
    //     let auth_env = mock_env("creator", &coins(2, "token"));
    //     let msg = HandleMsg::Reset { count: 5 };
    //     let _res = handle(&mut deps, auth_env, msg).unwrap();

    //     // should now be 5
    //     let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(5, value.count);
    // }
    #[test]
    fn test_is_input_correct_good_input() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
}
