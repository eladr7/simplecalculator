use crate::viewing_key::{ViewingKey, VIEWING_KEY_SIZE};
use cosmwasm_std::Api;
use cosmwasm_std::Extern;
use cosmwasm_std::HumanAddr;
use cosmwasm_std::Querier;
use cosmwasm_std::StdError;
use cosmwasm_std::StdResult;
use cosmwasm_std::Storage;
use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    /// User supplied entropy string for pseudorandom number generator seed
    pub prng_seed: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Add((Uint128, Uint128)),
    Sub((Uint128, Uint128)),
    Mul((Uint128, Uint128)),
    Div((Uint128, Uint128)),
    Sqrt(Uint128),

    /// Creates a new viewing key with user supplied entropy
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
}

/// Responses from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    CalculationResult {
        n: Option<Uint128>,
        status: String,
    },

    /// Return the created key
    CreateViewingKey {
        key: ViewingKey,
    },
    SetViewingKey {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// GetHistory returns the user's calculations history.
    GetHistory {
        address: HumanAddr,
        key: String,

        /// How many pages with 'page_size' items to skip
        page: Option<u32>,

        /// The number of items to take once the wanted page is reached
        page_size: u32,
    },
}

impl QueryMsg {
    pub fn authenticate<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &Extern<S, A, Q>,
    ) -> StdResult<HumanAddr> {
        let (address, key) = match self {
            QueryMsg::GetHistory { address, key, .. } => (address.clone(), ViewingKey(key.clone())),
        };

        let canonical_addr = deps.api.canonical_address(&address)?;

        let expected_key = ViewingKey::read_viewing_key(&deps.storage, &canonical_addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
            Err(StdError::generic_err("Wrong viewing key"))
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            Ok(address)
        } else {
            Err(StdError::generic_err("Wrong viewing key"))
        }
    }
}

/// Responses from query functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    GetHistory(GetHistory),
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct GetHistory {
    pub status: String,
    pub history: Vec<String>,
}
