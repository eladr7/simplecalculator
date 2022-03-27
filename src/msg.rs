use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    /// User supplied entropy string for pseudorandom number generator seed
    pub prng_seed: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Add {n1: i64, n2: i64},
    Sub {n1: i64, n2: i64},
    Mul {n1: i64, n2: i64},
    Div {n1: i64, n2: i64},
    Sqrt {n: u128},
}

/// Responses from handle functions
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Add {n: i64},
    Sub {n: i64},
    Mul {n: i64},
    Div {n: i64},
    Sqrt {n: u128},
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
        steps_back: Option<u16>,
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
