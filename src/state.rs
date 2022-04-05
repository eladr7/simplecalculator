use cosmwasm_std::{CanonicalAddr, ReadonlyStorage, StdError, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use secret_toolkit::serialization::{Bincode2, Serde};
use secret_toolkit::storage::{AppendStore, AppendStoreMut};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::any::type_name;

pub static CONFIG_KEY: &[u8] = b"config";
const PREFIX_CALCULATIONS: &[u8] = b"calculations";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct State {
    pub prng_seed: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CalculationsHistory {
    /// history has the user's calculations history
    pub history: Vec<u8>,
}

impl CalculationsHistory {
    pub fn into_humanized(self) -> StdResult<String> {
        Ok(String::from_utf8(self.history).unwrap())
    }
}

pub fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}

pub fn save_calculation<S: Storage>(storage: &mut S, key: &[u8], value: &str) -> StdResult<()> {
    let mut storage = PrefixedStorage::multilevel(&[PREFIX_CALCULATIONS, key], storage);
    let mut storage = AppendStoreMut::attach_or_create(&mut storage)?;
    // storage.push(&Bincode2::serialize(value)?)
    storage.push(&value.to_string())

    // storage.set(key, &Bincode2::serialize(value)?);
    // Ok(())
}

pub fn load<T: DeserializeOwned, S: ReadonlyStorage>(storage: &S, key: &[u8]) -> StdResult<T> {
    Bincode2::deserialize(
        &storage
            .get(key)
            .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
    )
}

pub fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}

pub fn get_transfers<S: ReadonlyStorage>(
    storage: &S,
    for_address: &CanonicalAddr,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<String>, String)> {
    let store = ReadonlyPrefixedStorage::multilevel(
        &[PREFIX_CALCULATIONS, for_address.as_slice()],
        storage,
    );

    // Try to access the storage of transfers for the account.
    // If it doesn't exist yet, return an empty list of transfers.
    let store = AppendStore::<CalculationsHistory, _, _>::attach(&store);
    let store = if let Some(result) = store {
        result?
    } else {
        return Ok((vec![], "No calculations history".to_string()));
    };

    // Take `page_size` txs starting from the latest tx, potentially skipping `page * page_size`
    // txs from the start.
    let calculations_history_iter = store
        .iter()
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);

    // The `and_then` here flattens the `StdResult<StdResult<RichTx>>` to an `StdResult<RichTx>`
    let calculations_history: StdResult<Vec<String>> = calculations_history_iter
        .map(|history| {
            history
                .map(|history| history.into_humanized())
                .and_then(|x| x)
        })
        .collect();

    Ok((
        calculations_history.unwrap(),
        "Calculations history present".to_string(),
    ))
}
