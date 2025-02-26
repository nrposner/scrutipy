use regex::Regex;

const FUZZ_VALUE: f64 = 1e-12;

/// Fuzzes the value of a float by 1e-12
///
/// Parameters:
///     x: floating-point number
///
/// Returns:
///     a vector of 2 floating-point numbers
///
/// Raises:
///     ValueError: If x is not a floating-point number
pub fn dustify(x: f64) -> Vec<f64> {
    vec![x - FUZZ_VALUE, x + FUZZ_VALUE]
}

/// Returns the number of values after the decimal point, or else None if there are no such values,
/// no decimal, or if the string cannot be converted to a numeric type
///
/// Note that this function will only record the number of values after the first decimal point
/// ```
/// let num_decimals = crate::decimal_places_scalar(Some("1.52.1"), ".");
/// assert_eq!(num_decimals, Some(2));
/// ```
pub fn decimal_places_scalar(x: Option<&str>, sep: &str) -> Option<usize> {
    let s = x?;

    let pattern = format!("{}(\\d+)", sep);
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(s)?;
    match caps.get(1) {
        Some(c) => Some(c.as_str().len()),
        None => Some(0),
    }
}

pub fn reconstruct_sd_scalar(
    formula: &str,
    x: &str,
    n: u32,
    zeros: u32,
    ones: u32,
) -> Result<f64, &'static str> {
    let x_num: f64 = match x.parse() {
        Ok(v) => v,
        Err(_) => return Err("could not parse {x} into a number"),
    };
    let sd_rec: f64 = match formula {
        "mean_n" => sd_binary_mean_n(x_num, n),
        "0_n" => sd_binary_0_n(zeros, n),
        "1_n" => sd_binary_1_n(ones, n),
        "groups" => sd_binary_groups(zeros, ones),
        _ => return Err("incorrect formula"), //set a proper error here
    };

    Ok(sd_rec)
}

/// round down function
pub fn round_down(number: f64, decimals: i32) -> f64 {
    let to_round =
        (number * 10.0f64.powi(decimals + 1)) - (number * 10f64.powi(decimals)).floor() * 10.0;

    match to_round {
        5.0 => (number * 10f64.powi(decimals)).floor() / 10f64.powi(decimals),
        _ => rust_round(number, decimals),
    }
}

pub fn round_up(number: f64, decimals: i32) -> f64 {
    let to_round =
        (number * 10.0f64.powi(decimals + 1)) - (number * 10f64.powi(decimals)).floor() * 10.0;

    match to_round {
        5.0 => (number * 10f64.powi(decimals)).ceil() / 10f64.powi(decimals),
        _ => rust_round(number, decimals),
    }
}

/// rust does not have a native funciton that rounds binary floating point numbers to a set number
/// of decimals. This is a hacky workaround that nevertheless seems to be the best option.
pub fn rust_round(x: f64, y: i32) -> f64 {
    (x * 10.0f64.powi(y)).round() / 10.0f64.powi(y)
}

//to_round <- number * 10^(decimals + 1) - floor(number * 10^(decimals)) * 10
//    number_rounded <- ifelse(to_round == 5,
//                             floor(number * 10^decimals) / 10^decimals,
//                             round(number, digits = decimals))
//    return(number_rounded)

pub fn check_threshold_specified(threshold: f64) {
    if threshold == 5.0 {
        panic!("Threshold must be set to some number other than its default, 5");
    }
}

/// reconstruct_rounded_numbers fn for reround
pub fn reconstruct_rounded_numbers_scalar(
    x: f64,
    digits: i32,
    rounding: &str,
    threshold: f64,
    symmetric: bool,
) -> Vec<f64> {
    // requires the round_up and round_down functions
    match rounding {
        "up_or_down" => vec![round_up(x, digits), round_down(x, digits)], // this is supposed to
        // contain a `symmetric` argument in the R code, but that's not present in the definition
        // for round up and round down ??
        "up_or_down_from" => {
            check_threshold_specified(threshold);
            vec![
                round_up_from(x, digits, threshold, symmetric),
                round_down_from(x, digits, threshold, symmetric),
            ]
        }
        "cieling_or_floor" => vec![round_ceiling(x, digits), round_floor(x, digits)],
        "even" => vec![rust_round(x, digits)],
        "up" => vec![round_up(x, digits)], // supposed to have a symmetric keyword, but round up
        // definition doesn't have it, ???
        "down" => vec![round_down(x, digits)], // supposed to have a symmetric keyword, but round down definition doesn't have it ???
        "up_from" => {
            check_threshold_specified(threshold);
            vec![round_up_from(x, digits, threshold, symmetric)]
        }
        "down_from" => {
            check_threshold_specified(threshold);
            vec![round_down_from(x, digits, threshold, symmetric)]
        }
        "ceiling" => vec![round_ceiling(x, digits)],
        "floor" => vec![round_floor(x, digits)],
        "trunc" => vec![round_trunc(x, digits)],
        "anti_trunc" => vec![round_anti_trunc(x, digits)],
        _ => panic!("`rounding` must be one of the designated string keywords"),
    }
}

pub fn round_trunc(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);

    //For symmetry between positive and negative numbers, use the absolute value:
    let core = (x.abs() * p10).trunc() / p10; // the rust f64::trunc() function may have
                                              // different properties than the R trunc() function, check to make sure
    match x < 0.0 {
        true => -core,
        false => core,
    }

    //If `x` is negative, its truncated version should be negative or zero.
    //Therefore, in this case, the function returns the negative of `core`, the
    //absolute value; otherwise it simply returns `core` itself:
}

pub fn anti_trunc(x: f64) -> f64 {
    let core = x.abs().trunc() + 1.0;

    match x < 0.0 {
        true => -core,
        false => core,
    }
}

pub fn round_anti_trunc(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);

    anti_trunc(x * p10) / p10
}

pub fn round_ceiling(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);
    (x * p10).ceil() / p10
    //ceiling(x * p10) / p10;
}

pub fn round_floor(x: f64, digits: i32) -> f64 {
    let p10 = 10.0f64.powi(digits);
    (x * p10).floor() / p10
    //ceiling(x * p10) / p10;
}
// not sure if x and rounding are meant to be scalars or vectors, because it seems elsewhere like
// we can pass multiple arguments to rounding, and that x can also be a vector????
// But the example values for the above function look exclusively scalars

pub fn round_up_from(x: f64, digits: i32, threshold: f64, symmetric: bool) -> f64 {
    let p10 = 10.0f64.powi(digits);
    let threshold = threshold - f64::MIN_POSITIVE.powf(0.5);

    if symmetric {
        match x < 0.0 {
            true => -(x.abs() * p10 + (1.0 - (threshold / 10.0))).floor(), // - (floor(abs(x) * p10 + (1 - (threshold / 10))) / p10)
            false => (x * p10 + (1.0 - (threshold / 10.0))).floor(),
        }
    } else {
        (x * p10 + (1.0 - (threshold / 10.0))).floor()
    }
}

pub fn round_down_from(x: f64, digits: i32, threshold: f64, symmetric: bool) -> f64 {
    let p10 = 10.0f64.powi(digits);
    let threshold = threshold - f64::MIN_POSITIVE.powf(0.5);

    if symmetric {
        match x < 0.0 {
            true => -(x.abs() * p10 + (1.0 - (threshold / 10.0))).ceil(), // - (floor(abs(x) * p10 + (1 - (threshold / 10))) / p10)
            false => (x * p10 + (1.0 - (threshold / 10.0))).ceil(),
        }
    } else {
        (x * p10 + (1.0 - (threshold / 10.0))).ceil()
    }
}

/// the reround function
pub fn reround(x: Vec<Vec<f64>>, rounding: Vec<&str>) {
    if rounding.len() > 1 {
        check_rounding_singular(rounding.clone(), "up_or_down", "up", "down")
            .expect("Error in selecting rounding options");
        check_rounding_singular(
            rounding.clone(),
            "up_from_or_down_from",
            "up_from",
            "down_from",
        )
        .expect("Error in selecting rounding options");
        check_rounding_singular(rounding.clone(), "ceiling_or_floor", "ceiling", "floor")
            .expect("Error in selecting rounding options");

        if (x.len() > 1) && (x.len() != rounding.len()) {
            let x_len = x.len();
            let round_len = rounding.len();
            panic!("x and rounding must have the same length unless one of them have length 1. x has length {x_len} and rounding has length {round_len}")
        }
    }
}

/// check rounding singular, necessary for the reround function
pub fn check_rounding_singular(
    rounding: Vec<&str>,
    bad: &str,
    good1: &str,
    good2: &str,
) -> Result<(), String> {
    if rounding.contains(&bad) {
        Err(format!("If rounding has length > 1, only single rounding procedures are supported, such as {good1} and {good2}. Instead, rounding was given as {bad} plus others. You can still concatenate multiple of them; just leave out those with or."))
    } else {
        Ok(())
    }
}

// now do the sd_binary functions, originally from the sd-binary.R file not utils, but for now we
// can keep them here, they're short enough

/// Returns the standard deviation of binary value counts
///
/// Parameters:
///     zeros: count of observations in the 0-binary condition
///     ones: count of observations in the 1-binary condition
///
/// Returns:
///     the floating-point standard deviation of the binary groups
///
/// Raises:
///     ValueError is zeros or ones are not usigned integers
///
/// Panics:
///     If the total number of observations is not greater than one
pub fn sd_binary_groups(zeros: u32, ones: u32) -> f64 {
    // though we take in the counts as unsigned integers, we transform them into
    // floating point values in order to perform the
    let n: f64 = zeros as f64 + ones as f64;

    // we assert that between the two groups there are at least two observations. There's not
    // much point otherwise
    assert!(n > 1.0, "Expecting at least two observations");

    (n - (n - 1.0)).sqrt() * ((zeros * ones) as f64 / n.powi(2))

    //sqrt((n / (n - 1)) * ((group_0 * group_1) / (n ^ 2)))
}

/// Returns the standard deviation of binary variables from the count of zero values and the total
///
/// Parameters:
///     zeros: count of observations in the 0-binary condition
///     n: count of total observations
///
/// Returns:
///     the floating-point standard deviation of the binary groups
///
/// Raises:
///     ValueError: if zeros or n are not unsigned integers
///
/// Panics:
///     If there are more observations in the zero condition than in the total
///     If the total number of observations is not greater than one
pub fn sd_binary_0_n(zeros: u32, n: u32) -> f64 {
    let ones: f64 = n as f64 - zeros as f64;

    assert!(
        n >= zeros,
        "There cannot be more observations in one condition than in the whole system"
    );
    assert!(n > 1, "Expecting at least two observations");

    ((n - (n - 1)) as f64).sqrt() * ((zeros as f64 * ones) / (n as f64).powi(2))
}
/// Returns the standard deviation of binary variables from the count of one values and the total
///
/// Parameters:
///     ones: count of observations in the 1-binary condition
///     n: count of total observations
///
/// Returns:
///     the floating-point standard deviation of the binary groups
///
/// Raises:
///     ValueError: if ones or n are not unsigned integers
///
/// Panics:
///     If there are more observations in the one condition than in the total
///     If the total number of observations is not greater than one
pub fn sd_binary_1_n(ones: u32, n: u32) -> f64 {
    let zeros: f64 = n as f64 - ones as f64;

    assert!(
        n >= ones,
        "There cannot be more observations in one condition than in the whole system"
    );
    assert!(n > 1, "Expecting at least two observations");

    ((n - (n - 1)) as f64).sqrt() * ((zeros * ones as f64) / (n as f64).powi(2))
}
/// Returns the standard deviation of binary variables from the mean and the total
///
/// Parameters:
///     mean: mean of the binary observations, namely the proportion of values in the 1-binary
///     condition
///     n: count of total observations
///
/// Returns:
///     the floating-point standard deviation of the binary system
///
/// Raises:
///     ValueError: if mean is not a floating-point number
///     ValueError: if n is not an unsigned integer
///
/// Panics:
///     if the mean is greater than one or less than zero
pub fn sd_binary_mean_n(mean: f64, n: u32) -> f64 {
    assert!(
        mean >= 0.0,
        "The mean of binary observations cannot be less than 0"
    );
    assert!(
        mean <= 1.0,
        "The mean of binary observations cannot be greater than 1"
    );
    ((n - (n - 1)) as f64) * (mean * (1.0 - mean))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_places_test_1() {
        assert_eq!(decimal_places_scalar(Some("9.846"), "."), Some(3));
    }

    #[test]
    fn decimal_places_test_2() {
        assert_eq!(decimal_places_scalar(Some(".9678"), "."), Some(4));
    }

    #[test]
    fn decimal_places_test_3() {
        assert_eq!(decimal_places_scalar(Some("1."), "."), None);
    }

    #[test]
    fn decimal_places_test_4() {
        assert_eq!(decimal_places_scalar(Some("0"), "."), None);
    }

    #[test]
    fn decimal_places_test_5() {
        assert_eq!(decimal_places_scalar(Some("1.52.0"), "."), Some(2));
    }

    #[test]
    fn decimal_places_test_6() {
        assert_eq!(decimal_places_scalar(Some("Not a Number"), "."), None);
    }
}
