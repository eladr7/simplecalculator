use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, QueryResult, HumanAddr, Uint128,
};

use std::convert::TryFrom;
use crate::msg::{HandleAnswer, HandleMsg, InitMsg, QueryMsg, QueryAnswer};
use crate::state::{load, may_load, save, State, CalculationsHistory, CONFIG_KEY, write_viewing_key, read_viewing_key,};
use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use secret_toolkit::crypto::sha_256;

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
        HandleMsg::GenerateViewingKey { entropy, .. } => try_generate_viewing_key(deps, env, entropy),
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
        data: Some(to_binary(&HandleAnswer::GenerateViewingKey { 
            key,
        })?),
    })
}

// Elad - Should convert first to u128?
// limit the max message size to values in 1..65535
fn is_add_input_correct(n1: Uint128, n2: Uint128, err_msg: &mut String) -> bool {
    let max_half = u128::MAX / 2;
    if n1 >= max_half && n2 >= max_half {
        err_msg = "The input numbers are too large";
        return false
    }
    return true
}

// Elad - Should convert first to u128?
// limit the max message size to values in 1..65535
fn is_sub_input_correct(n1: Uint128, n2: Uint128, err_msg: &mut String) -> bool {
    if n2 > n1 {
        err_msg = "The second argument is larger than the first, cannot calculate negative results";
        return false
    }
    return true
}


// Elad - Should convert first to u128?
fn is_mul_input_correct(n1: Uint128, n2: Uint128, err_msg: &mut String) -> bool {
    let max = u128::MAX;
    if n1 * n2 > max {
        err_msg = "The multiplication is too large. Cannot calculate results larger than " + max.to_string();
        return false
    }
    return true
}



// Elad - Should convert first to u128?
// limit the max message size to values in 1..65535
fn is_div_input_correct(n1: Uint128, n2: Uint128, err_msg: &mut String) -> bool {
    if n2.u128() == 0 {
        err_msg = "Cannot devide by zero!";
        return false
    }

    return true
}

// limit the max message size to values in 1..65535
fn get_calculation_string(n1: Uint128, n2: Uint128, operation: String, result: Uint128) -> String {
    n1.to_string() + " " + operation + " " + n2.to_string() + " = " + result.to_string()
}

// limit the max message size to values in 1..65535
fn get_sqrt_calculation_string(n: Uint128, operation: String, result: Uint128) -> String {
    operation + n.to_string() + " = " + result.to_string()
}

fn try_add<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    let result: Uint128 = None; // Elad
    let status: String;
    let err_msg: String;

    if !is_add_input_correct(n1, n2, &mut err_msg) {
        status = String::from("Invalid input: " + err_msg);
    } else {
        result = n1 + n2;
        let calculation_string = get_calculation_string(n1, n2, "+", result);
        insert_result(calculation_string, deps, env, &mut status)?;
    }

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Add {
            n: result,
            status: status,
        })?),
    })
}

fn insert_result(calculation_string: String, deps: &mut Extern<S, A, Q>, env: Env, status: &mut String) -> Result<(), StdError> {
    let mut history: Vec<String> = None;

    let sender_address = env.message.sender;
    let history_query_obj: QueryAnswer::GetHistory = may_get_history(&mut deps.storage, &sender_address, None);
    history_query_obj.map(|stored_history| {history = stored_history});
    // match may_get_history(&mut deps.storage, &sender_address, None) {
    //     Some(stored_history) => {
    //         history = stored_history
    //     } 
    // }

    history.push(calculation_string);
    let calculations_history = CalculationsHistory {
        history: history.as_slice().to_vec()
    };
    
    let sender_canonical_address = deps.api.canonical_address(&sender_address)?;
    save(&mut deps.storage, &sender_canonical_address.as_slice().to_vec(), &calculations_history)?;
    *status = String::from("Calculation performed and recorded!");
}

fn try_sub<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    let result: Uint128 = None;// Elad
    let status: String;
    let err_msg: String;

    if !is_sub_input_correct(n1, n2, &err_msg) {
        status = String::from("Invalid input: " + err_msg);
    } else {
        result = n1 - n2;
        let calculation_string = get_calculation_string(n1, n2, "-", result);
        insert_result(calculation_string, deps, env, &mut status)?;
    }

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Sub {
            n: result,
            status: status,
        })?),
    })
}


fn try_mul<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    let result: Uint128 = None;// Elad
    let status: String;
    let err_msg: String;

    if !is_mul_input_correct(n1, n2, &err_msg) {
        status = String::from("Invalid input: " + err_msg);
    } else {
        result = n1 * n2;
        let calculation_string = get_calculation_string(n1, n2, "*", result);
        insert_result(calculation_string, deps, env, &mut status)?;
    }

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Mul {
            n: result,
            status: status,
        })?),
    })
}

fn try_div<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    let result: Uint128 = None;// Elad
    let status: String;
    let err_msg: String;

    if !is_div_input_correct(n1, n2, &err_msg) {
        status = String::from("Invalid input: " + err_msg);
    } else {
        result = n1 / n2;
        let calculation_string = get_calculation_string(n1, n2, "/", result);
        insert_result(calculation_string, deps, env, &mut status)?;
    }

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Div {
            n: result,
            status: status,
        })?),
    })
}

fn try_sqrt<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n: Uint128,
) -> StdResult<HandleResponse> {
    let result: Uint128 = None; // Elad
    let status: String;
    let err_msg: String;

    result = (n.u128() as f64).sqrt() as u128 + 1;
    let calculation_string = get_sqrt_calculation_string(n, "√", result);
    insert_result(calculation_string, deps, env, &mut status)?;

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Sqrt {
            n: result,
            status: status,
        })?),
    })
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
                QueryMsg::GetHistory { address, key, steps_back } =>
                    to_binary(&may_get_history(&deps, &address, steps_back)),
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
) -> QueryAnswer::GetHistory {
// ) -> StdResult<Binary> {
    let answer = query_read(&deps, &address)?;
    
    if let QueryAnswer::GetHistory { status, history } = answer {
        if steps_back && let Some(history) = history {
            return QueryAnswer::GetHistory {
                status,
                history: get_partial_history(&history, steps_back),
            }
            // return to_binary(&get_partial_history(&history, steps_back))
        }
        return answer
        // return to_binary(&answer)
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
    let result: Option<CalculationsHistory> = may_load(&deps.storage, &sender_address.as_slice().to_vec()).ok().unwrap();
    match result {
        // set all response field values
        Some(stored_history) => {
            status = String::from("Calculations history present");
            history = Some(stored_history.history);
        }
        // unless there's an error
        None => { status = String::from("Calculations history not found."); }
    };

    Ok(QueryAnswer::GetHistory{ status, history })
}

fn get_partial_history(history: &Vec<String>, steps_back: Uint128) -> Vec<String> {
    if steps_back > history.len() {
        return history
    }
    history[history.len() - steps_back..history.len() - 1];
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_is_input_correct_good_input() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
    #[test]
    fn test_is_input_correct_too_large_input() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
    #[test]
    fn test_is_input_correct_wrong_input_type() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }



    #[test]
    fn test_try_div_good_input() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
    #[test]
    fn test_try_div_division_by_zero() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
    #[test]
    fn test_try_div_wrong_input_type() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }



    #[test]
    fn test_try_sqrt_good_input() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
    #[test]
    fn test_try_sqrt_too_large_input() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }
    #[test]
    fn test_try_sqrt_wrong_input_type() {
        /// input: deps, env, n1, n2
        unimplemented!();
    }

    // use super::*;
    // use cosmwasm_std::testing::{mock_dependencies, mock_env};
    // use cosmwasm_std::{coins, from_binary, StdError};

    // #[test]
    // fn proper_initialization() {
    //     let mut deps = mock_dependencies(20, &[]);

    //     let msg = InitMsg { count: 17 };
    //     let env = mock_env("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = init(&mut deps, env, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(&deps, QueryMsg::GetCount {}).unwrap();
    //     let value: CountResponse = from_binary(&res).unwrap();
    //     assert_eq!(17, value.count);
    // }

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
}
