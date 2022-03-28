use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    /// User supplied entropy string for pseudorandom number generator seed
    pub prng_seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Add {n1: Uint128, n2: Uint128},
    Sub {n1: Uint128, n2: Uint128},
    Mul {n1: Uint128, n2: Uint128},
    Div {n1: Uint128, n2: Uint128},
    Sqrt {n: Uint128},

    /// Generates a new viewing key with user supplied entropy
    GenerateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
}

/// Responses from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Add {
        n: Option<Uint128>,
        status: String,
    },
    Sub {
        n: Option<Uint128>,
        status: String,
    },
    Mul {
        n: Option<Uint128>,
        status: String,
    },
    Div {
        n: Option<Uint128>,
        status: String,
    },
    Sqrt {
        n: Option<Uint128>,
        status: String,
    },

    /// Return the generated key
    GenerateViewingKey {
        key: ViewingKey,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// GetHistory returns the user's calculations history.
    GetHistory {
        address: HumanAddr,
        key: String,

        /// steps_back is counted since the last operation performed; e.g. if steps_back is 3,
        /// then the history of the last 3 operations will be returned.
        steps_back: Option<Uint128>,
    },
}

impl QueryMsg {
    pub fn get_validation_params(&self) -> (Vec<&HumanAddr>, ViewingKey) {
        match self {
            Self::GetHistory { address, key, steps_back } => (vec![address], ViewingKey(key.clone()), steps_back),
            _ => panic!("This query type does not require authentication"),
        }
    }
}

/// Responses from query functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    /// Return a status message and the calculations history of the user, if it exists
    GetHistory {
        status: String,
        history: Option<Vec<String>>,
    },
}
