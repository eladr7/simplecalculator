use cosmwasm_std::{StdError, StdResult, Uint128};
pub type ArithmeticCalculation = fn(n1: Uint128, n2: Uint128) -> StdResult<Uint128>;

pub fn calculate_add(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    n1.u128()
        .checked_add(n2.u128())
        .ok_or_else(|| StdError::generic_err("Invalid input: The input numbers are too large"))
        .map(Uint128)
}

pub fn calculate_sub(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    n1.u128()
    .checked_sub(n2.u128())
    .ok_or_else(|| StdError::generic_err("Invalid input: The second argument is larger than the first, cannot calculate negative results"))
    .map(Uint128)
}

pub fn calculate_mul(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    n1.u128()
    .checked_mul(n2.u128())
    .ok_or_else(|| StdError::generic_err("Invalid input: The multiplication is too large. Cannot calculate results larger than"))
    .map(Uint128)
}

pub fn calculate_div(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    n1.u128()
        .checked_div(n2.u128())
        .ok_or_else(|| StdError::generic_err("Invalid input: Cannot devide by zero!"))
        .map(Uint128)
}

pub fn calculate_sqrt(n1: Uint128, _n2: Uint128) -> StdResult<Uint128> {
    let n1_u128 = n1.u128();
    let mut left: u128 = 0;
    let mut right: u128 = n1_u128;
    let mut middle: u128;
    let mut pow: u128;

    if n1_u128 == 0 || n1_u128 == 1 {
        return Ok(n1);
    }

    // Perform a form of binary search to find the sqrt of n1.
    // Notice that the result is rounded down. e.g. sqrt(70) = 8.
    //
    // * This calculation can be more efficient, but since it's still O(log(n1)), no need to go
    // further in this exercise.
    while left < right {
        middle = (left + right) / 2;

        if left == middle {
            if (left + 1) * (left + 1) > n1_u128 {
                return Ok(Uint128::from(left));
            }
            return Ok(Uint128::from(left + 1));
        }

        pow = middle * middle;
        if pow < n1_u128 {
            left = middle;
            continue;
        }

        if pow > n1_u128 {
            right = middle - 1;
            continue;
        }

        return Ok(Uint128::from(middle));
    }

    Ok(Uint128::from(left))
}

pub fn get_calculation_string(
    n1: Uint128,
    n2: Uint128,
    operation: &str,
    result: Uint128,
) -> String {
    if operation == "√" {
        return operation.to_string() + &n1.to_string() + " = " + &result.to_string();
    }

    n1.to_string() + " " + operation + " " + &n2.to_string() + " = " + &result.to_string()
}

#[test]
fn test_calculate_add() {
    let n1: u128 = 100;
    let n2: u128 = 200;
    let expected: u128 = 300;
    let actual = calculate_add(Uint128::from(n1), Uint128::from(n2));
    assert_eq!(Uint128::from(expected), actual.unwrap());
}

#[test]
fn test_calculate_sub() {
    let n1: u128 = 200;
    let n2: u128 = 150;
    let expected: u128 = 50;
    let actual = calculate_sub(Uint128::from(n1), Uint128::from(n2));
    assert_eq!(Uint128::from(expected), actual.unwrap());
}

#[test]
fn test_calculate_mul() {
    let n1: u128 = 100;
    let n2: u128 = 200;
    let expected: u128 = 20000;
    let actual = calculate_mul(Uint128::from(n1), Uint128::from(n2));
    assert_eq!(Uint128::from(expected), actual.unwrap());
}

#[test]
fn test_calculate_div() {
    let n1: u128 = 100;
    let n2: u128 = 3;
    let expected: u128 = 33;
    let actual = calculate_div(Uint128::from(n1), Uint128::from(n2));
    assert_eq!(Uint128::from(expected), actual.unwrap());
}

#[test]
fn test_calculate_div_fraction() {
    let n1: u128 = 100;
    let n2: u128 = 200;
    let expected: u128 = 0;
    let actual = calculate_div(Uint128::from(n1), Uint128::from(n2));
    assert_eq!(Uint128::from(expected), actual.unwrap());
}

#[test]
fn test_calculate_sqrt() {
    let n1: u128 = 5;
    let expected: u128 = 2;
    let actual = calculate_sqrt(Uint128::from(n1), Uint128::from(u128::MIN));
    assert_eq!(Uint128::from(expected), actual.unwrap());
}

#[test]
fn test_get_calculation_string() {
    let operation = String::from("+");
    let n1: u128 = 2;
    let n2: u128 = 5;
    let result: u128 = 7;
    let calculation_string = get_calculation_string(
        Uint128::from(n1),
        Uint128::from(n2),
        &operation,
        Uint128::from(result),
    );
    assert_eq!("2 + 5 = 7", calculation_string);
}

#[test]
fn test_get_calculation_string_sqrt() {
    let operation = String::from("√");
    let n1: u128 = 5;
    let n2: u128 = 5;
    let result: u128 = 2;
    let calculation_string = get_calculation_string(
        Uint128::from(n1),
        Uint128::from(n2),
        &operation,
        Uint128::from(result),
    );
    assert_eq!("√5 = 2", calculation_string);
}
