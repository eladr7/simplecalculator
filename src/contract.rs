use crate::calculations_utils::{
    calculate_add, calculate_div, calculate_mul, calculate_sqrt, calculate_sub,
    get_calculation_string, ArithmeticCalculation,
};
use crate::msg::{GetHistory, HandleAnswer, HandleMsg, InitMsg, QueryMsg, ResponseStatus::Success};
use crate::state::{get_transfers, load, save, save_calculation, State, CONFIG_KEY};

use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryResult,
    StdError, StdResult, Storage, Uint128,
};

use crate::viewing_key::ViewingKey;
use secret_toolkit::crypto::sha_256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = State {
        prng_seed: sha_256(base64::encode(msg.prng_seed).as_bytes()).to_vec(),
    };

    save(&mut deps.storage, CONFIG_KEY, &config)?;
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Add((n1, n2)) => add(deps, env, n1, n2),
        HandleMsg::Sub((n1, n2)) => sub(deps, env, n1, n2),
        HandleMsg::Mul((n1, n2)) => mul(deps, env, n1, n2),
        HandleMsg::Div((n1, n2)) => div(deps, env, n1, n2),
        HandleMsg::Sqrt(n) => sqrt(deps, env, n),
        HandleMsg::CreateViewingKey { entropy, .. } => create_viewing_key(deps, env, entropy),
        HandleMsg::SetViewingKey { key, .. } => try_set_key(deps, env, key),
    }
}

pub fn create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let config: State = load(&deps.storage, CONFIG_KEY)?;
    let prng_seed = config.prng_seed;

    let key = ViewingKey::new(&env, &prng_seed, (&entropy).as_ref());

    let message_sender = deps.api.canonical_address(&env.message.sender)?;

    ViewingKey::write_viewing_key(&mut deps.storage, &message_sender, &key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key })?),
    })
}

pub fn try_set_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: String,
) -> StdResult<HandleResponse> {
    let vk = ViewingKey(key);

    let message_sender = deps.api.canonical_address(&env.message.sender)?;
    ViewingKey::write_viewing_key(&mut deps.storage, &message_sender, &vk);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}

fn insert_result<S: Storage, A: Api, Q: Querier>(
    calculation_string: String,
    deps: &mut Extern<S, A, Q>,
    env: Env,
    insertion_status: &mut String,
) -> Result<(), StdError> {
    let sender_address = env.message.sender;
    let sender_canonical_address = deps.api.canonical_address(&sender_address)?;

    save_calculation(
        &mut deps.storage,
        sender_canonical_address.as_slice(),
        &calculation_string,
    )?;
    insertion_status.push_str("Calculation performed and recorded!");
    Ok(())
}

fn calculate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
    operation: String,
    calculate: ArithmeticCalculation,
) -> StdResult<HandleResponse> {
    let mut result: Option<Uint128> = None;
    let mut status = String::new();

    match calculate(n1, n2) {
        Ok(res) => {
            result = Some(res);
            let calculation_string = get_calculation_string(n1, n2, &operation, res);
            insert_result(calculation_string, deps, env, &mut status)?;
        }
        Err(err) => {
            status = err.to_string();
        }
    };

    // Return a HandleResponse with the appropriate status message included in the data field
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CalculationResult {
            n: result,
            status,
        })?),
    })
}

fn add<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    calculate(deps, env, n1, n2, String::from("+"), calculate_add)
}

fn sub<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    calculate(deps, env, n1, n2, String::from("-"), calculate_sub)
}

fn mul<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    calculate(deps, env, n1, n2, String::from("*"), calculate_mul)
}

fn div<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n1: Uint128,
    n2: Uint128,
) -> StdResult<HandleResponse> {
    calculate(deps, env, n1, n2, String::from("/"), calculate_div)
}

fn sqrt<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    n: Uint128,
) -> StdResult<HandleResponse> {
    calculate(
        deps,
        env,
        n,
        Uint128::zero(),
        String::from("√"),
        calculate_sqrt,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetHistory { .. } => viewing_keys_queries(deps, msg),
    }
}

pub fn viewing_keys_queries<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    msg.authenticate(deps)?;

    match msg {
        QueryMsg::GetHistory {
            address,
            page,
            page_size,
            ..
        } => to_binary(&may_get_history(
            deps,
            &address,
            page.unwrap_or(0),
            page_size,
        )?),
    }
}

pub fn may_get_history<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &HumanAddr,
    page: u32,
    page_size: u32,
) -> StdResult<GetHistory> {
    let address = deps.api.canonical_address(account)?;
    let (history, status) = get_transfers(&deps.storage, &address, page, page_size)?;

    let result = GetHistory { status, history };
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, from_binary};

    fn init_helper() -> (
        StdResult<InitResponse>,
        Extern<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies(20, &[]);
        let env = mock_env("instantiator", &coins(1000, "token"));

        let init_msg = InitMsg {
            prng_seed: String::from("waehfjklasd"),
        };

        (init(&mut deps, env, init_msg), deps)
    }

    fn create_viewing_key(
        deps: &mut Extern<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>,
    ) -> ViewingKey {
        let msg = HandleMsg::CreateViewingKey {
            entropy: String::from("wefhjyr"),
            padding: None,
        };
        let handle_result = handle(deps, mock_env("bob", &[]), msg);
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        // Get the viewing key of the reply to HandleMsg::CreateViewingKey
        let answer: HandleAnswer = from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        let key = match answer {
            HandleAnswer::CreateViewingKey { key } => key,
            _ => panic!("NOPE"),
        };
        key
    }

    fn query_history_wrong_vk(deps: Extern<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>) {
        let wrong_vk_query_response = query(
            &deps,
            QueryMsg::GetHistory {
                address: HumanAddr("bob".to_string()),
                key: "wrong_vk".to_string(),
                page: None,
                page_size: 1,
            },
        );
        let error = match wrong_vk_query_response {
            Ok(_response) => "This line should not be reached!".to_string(),
            Err(_err) => "Wrong viewing key for this address or viewing key not set".to_string(),
        };
        assert_eq!(
            error,
            "Wrong viewing key for this address or viewing key not set".to_string()
        );
    }

    fn query_transactions_history(
        deps: &mut Extern<cosmwasm_std::MemoryStorage, MockApi, MockQuerier>,
    ) -> StdResult<Vec<String>> {
        let vk = create_viewing_key(deps);
        let query_response = query(
            &*deps,
            QueryMsg::GetHistory {
                address: HumanAddr("bob".to_string()),
                key: vk.0,
                page: None,
                page_size: 1,
            },
        )
        .unwrap();
        let hui: GetHistory = from_binary(&query_response)?;
        Ok(hui.history)
    }

    #[test]
    fn test_init_sanity() {
        let (init_result, _deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
    }

    #[test]
    fn test_create_viewing_key() {
        // Initialize the contract
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Compute the viewing key
        let key = create_viewing_key(&mut deps);

        // Get the viewing key from the storage
        let bob_canonical = deps
            .api
            .canonical_address(&HumanAddr("bob".to_string()))
            .unwrap();
        let saved_vk = ViewingKey::read_viewing_key(&deps.storage, &bob_canonical).unwrap();

        // Verify that the key in the storage is the same as the key from HandleAnswer::CreateViewingKey
        assert!(key.check_viewing_key(saved_vk.as_slice()));
    }

    #[test]
    fn test_add() -> StdResult<()> {
        // Initialize the contract
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Perform an Add operation
        let env = mock_env("bob", &coins(2, "token"));
        let n1: u128 = 3;
        let n2: u128 = 5;
        let msg = HandleMsg::Add((Uint128::from(n1), Uint128::from(n2)));
        let _res = handle(&mut deps, env, msg).unwrap();

        // Query the user's transactions history using their viewing key
        let history = query_transactions_history(&mut deps)?;

        // Verify the transactions history
        for i in history {
            assert_eq!("3 + 5 = 8".to_string(), i);
        }

        // Now try to hack into bob's account using the wrong key - and fail
        query_history_wrong_vk(deps);
        Ok(())
    }

    #[test]
    fn test_sub() -> StdResult<()> {
        // Initialize the contract
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Perform an Add operation
        let env = mock_env("bob", &coins(2, "token"));
        let n1: u128 = 20;
        let n2: u128 = 5;
        let msg = HandleMsg::Sub((Uint128::from(n1), Uint128::from(n2)));
        let _res = handle(&mut deps, env, msg).unwrap();

        // Query the user's transactions history using their viewing key
        let history = query_transactions_history(&mut deps)?;

        // Verify the transactions history
        for i in history {
            assert_eq!("20 - 5 = 15".to_string(), i);
        }

        // Now try to hack into bob's account using the wrong key - and fail
        query_history_wrong_vk(deps);
        Ok(())
    }

    #[test]
    fn test_mul() -> StdResult<()> {
        // Initialize the contract
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Perform an Add operation
        let env = mock_env("bob", &coins(2, "token"));
        let n1: u128 = 20;
        let n2: u128 = 5;
        let msg = HandleMsg::Mul((Uint128::from(n1), Uint128::from(n2)));
        let _res = handle(&mut deps, env, msg).unwrap();

        // Query the user's transactions history using their viewing key
        let history = query_transactions_history(&mut deps)?;

        // Verify the transactions history
        for i in history {
            assert_eq!("20 * 5 = 100".to_string(), i);
        }

        // Now try to hack into bob's account using the wrong key - and fail
        query_history_wrong_vk(deps);
        Ok(())
    }

    #[test]
    fn test_div() -> StdResult<()> {
        // Initialize the contract
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Perform an Add operation
        let env = mock_env("bob", &coins(2, "token"));
        let n1: u128 = 20;
        let n2: u128 = 5;
        let msg = HandleMsg::Div((Uint128::from(n1), Uint128::from(n2)));
        let _res = handle(&mut deps, env, msg).unwrap();

        // Query the user's transactions history using their viewing key
        let history = query_transactions_history(&mut deps)?;

        // Verify the transactions history
        for i in history {
            assert_eq!("20 / 5 = 4".to_string(), i);
        }

        // Now try to hack into bob's account using the wrong key - and fail
        query_history_wrong_vk(deps);
        Ok(())
    }

    #[test]
    fn test_sqrt() -> StdResult<()> {
        // Initialize the contract
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        // Perform an Add operation
        let env = mock_env("bob", &coins(2, "token"));
        let n: u128 = 121;
        let msg = HandleMsg::Sqrt(Uint128::from(n));
        let _res = handle(&mut deps, env, msg).unwrap();

        // Query the user's transactions history using their viewing key
        let history = query_transactions_history(&mut deps)?;

        // Verify the transactions history
        for i in history {
            assert_eq!("√121 = 11".to_string(), i);
        }

        // Now try to hack into bob's account using the wrong key - and fail
        query_history_wrong_vk(deps);
        Ok(())
    }
}
