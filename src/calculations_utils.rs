use cosmwasm_std::{StdResult, Uint128};
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
    if n1 * n2 > max {
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

pub fn is_div_input_correct(n1: u128, n2: u128, err_msg: &mut String) -> bool {
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
    Ok(Uint128::from((n1.u128() as f64).sqrt() as u128 + 1))
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
