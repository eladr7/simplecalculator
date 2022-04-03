use cosmwasm_std::{StdResult, Uint128};
use num::integer::Roots;

pub fn is_add_input_correct(n1: u128, n2: u128, err_msg: &mut String) -> bool {
    let max_half = u128::MAX / 2;
    if n1 >= max_half && n2 >= max_half {
        err_msg.push_str("Invalid input: The input numbers are too large");
        return false;
    }
    return true;
}

pub fn is_sub_input_correct(n1: u128, n2: u128, err_msg: &mut String) -> bool {
    if n2 > n1 {
        err_msg.push_str("Invalid input: The second argument is larger than the first, cannot calculate negative results");
        return false;
    }
    return true;
}

pub fn is_mul_input_correct(n1: u128, n2: u128, err_msg: &mut String) -> bool {
    let max = u128::MAX;
    if (max / n1) < n2 {
        let max_string: String = max.to_string();
        err_msg.push_str(
            &format!(
            "{} {}",
            "Invalid input: The multiplication is too large. Cannot calculate results larger than",
            max_string
        )
            .to_string(),
        );
        return false;
    }
    return true;
}

pub fn is_div_input_correct(_n1: u128, n2: u128, err_msg: &mut String) -> bool {
    if n2 == 0 {
        err_msg.push_str("Invalid input: Cannot devide by zero!");
        return false;
    }

    return true;
}

pub fn is_sqrt_input_correct(_n1: u128, _n2: u128, _err_msg: &mut String) -> bool {
    // Always true since the input only has to be positive, which is enforced by the Uint128
    // type of input.
    return true;
}

pub fn calculate_add(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    Ok(Uint128::from(n1.u128() + n2.u128()))
}

pub fn calculate_sub(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    Ok(Uint128::from(n1.u128() - n2.u128()))
}

pub fn calculate_mul(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    Ok(Uint128::from(n1.u128() * n2.u128()))
}

pub fn calculate_div(n1: Uint128, n2: Uint128) -> StdResult<Uint128> {
    Ok(Uint128::from(n1.u128() / n2.u128()))
}

pub fn calculate_sqrt(n1: Uint128, _n2: Uint128) -> StdResult<Uint128> {
    // Ok(Uint128::from((n1.u128() as f64).sqrt() as u128))
    Ok(Uint128::from((n1.u128()).sqrt() as u128))
}

pub fn get_calculation_string(
    n1: Uint128,
    n2: Uint128,
    operation: &String,
    result: Uint128,
) -> String {
    if operation == "√" {
        return operation.to_string() + &n1.to_string() + " = " + &result.to_string();
    }

    n1.to_string() + " " + operation + " " + &n2.to_string() + " = " + &result.to_string()
}

#[test]
fn test_add_input_correct() {
    let mut err_msg = String::new();
    let n1: u128 = u128::MAX / 2;
    let n2: u128 = (u128::MAX / 2) - 1;
    let input_correct = is_add_input_correct(n1, n2, &mut err_msg);
    assert_eq!(true, input_correct);
    assert_eq!("", err_msg);
}

#[test]
fn test_add_input_too_large() {
    let mut err_msg = String::new();
    let n1: u128 = (u128::MAX / 2) + 1;
    let n2: u128 = u128::MAX / 2;
    let input_correct = is_add_input_correct(n1, n2, &mut err_msg);
    assert_eq!(false, input_correct);
    assert_eq!("Invalid input: The input numbers are too large", err_msg);
}

#[test]
fn test_sub_input_correct() {
    let mut err_msg = String::new();
    let n1: u128 = 15;
    let n2: u128 = 15;
    let input_correct = is_sub_input_correct(n1, n2, &mut err_msg);
    assert_eq!(true, input_correct);
    assert_eq!("", err_msg);
}

#[test]
fn test_sub_input_negative_result() {
    let mut err_msg = String::new();
    let n1: u128 = 15;
    let n2: u128 = 16;
    let input_correct = is_sub_input_correct(n1, n2, &mut err_msg);
    assert_eq!(false, input_correct);
    assert_eq!("Invalid input: The second argument is larger than the first, cannot calculate negative results", err_msg);
}

#[test]
fn test_mul_input_correct() {
    let mut err_msg = String::new();
    let n1: u128 = u128::MAX / 2;
    let n2: u128 = 1;
    let input_correct = is_mul_input_correct(n1, n2, &mut err_msg);
    assert_eq!(true, input_correct);
    assert_eq!("", err_msg);
}

#[test]
fn test_mul_input_too_large_result() {
    let mut err_msg = String::new();
    let n1: u128 = u128::MAX / 2;
    let n2: u128 = 3;
    let input_correct = is_mul_input_correct(n1, n2, &mut err_msg);
    assert_eq!(false, input_correct);
    assert_eq!(
        "Invalid input: The multiplication is too large. Cannot calculate results larger than 340282366920938463463374607431768211455",
        err_msg
    );
}

#[test]
fn test_div_input_correct() {
    let mut err_msg = String::new();
    let n1: u128 = u128::MAX / 2;
    let n2: u128 = (u128::MAX / 2) - 1;
    let input_correct = is_div_input_correct(n1, n2, &mut err_msg);
    assert_eq!(true, input_correct);
    assert_eq!("", err_msg);
}

#[test]
fn test_div_input_division_by_zero() {
    let mut err_msg = String::new();
    let n1: u128 = u128::MAX / 2;
    let n2: u128 = 0;
    let input_correct = is_div_input_correct(n1, n2, &mut err_msg);
    assert_eq!(false, input_correct);
    assert_eq!("Invalid input: Cannot devide by zero!", err_msg);
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
