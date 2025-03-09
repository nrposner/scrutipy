use crate::rounding::*;
use crate::sd_binary::*;
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
pub fn decimal_places_scalar(x: Option<&str>, sep: &str) -> Option<i32> {
    let s = x?;

    let pattern = format!("{}(\\d+)", sep);
    let re = Regex::new(&pattern).ok()?;
    let caps = re.captures(s)?;
    match caps.get(1) {
        Some(c) => Some(c.as_str().len() as i32),
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
        "mean" => sd_binary_mean_n(x_num, n), // convenient aliases
        "0_n" => sd_binary_0_n(zeros, n),
        "0" => sd_binary_0_n(zeros, n), // convenient aliases
        "1_n" => sd_binary_1_n(ones, n),
        "1" => sd_binary_1_n(ones, n), // convenient aliases
        "groups" => sd_binary_groups(zeros, ones),
        "group" => sd_binary_groups(zeros, ones), // convenient aliases

        _ => return Err("incorrect formula"), //set a proper error here
    };

    Ok(sd_rec)
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
/// notably the R version of this function can take a vector as x or a scalar, we
/// should probably turn this into taking a vector with at least one element, and that means the
/// rounding functions also need to be updated to do that
/// but both this and the vectorized version return doubles, and the same number of them, just in a
/// different format
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
                round_down_from(vec![x], digits, threshold, symmetric)[0], // this is a hacky
                                                                           // solution to suppress the errors while we're migrating this from scalar to
                                                                           // vectors
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
            vec![round_down_from(vec![x], digits, threshold, symmetric)[0]]
        }
        "ceiling" => vec![round_ceiling(x, digits)],
        "floor" => vec![round_floor(x, digits)],
        "trunc" => vec![round_trunc(x, digits)],
        "anti_trunc" => vec![round_anti_trunc(x, digits)],
        _ => panic!("`rounding` must be one of the designated string keywords"),
    }
}

/// the reround function
/// redo so that it just takes a vec instead of a vec of vec, the R version can take arbitrarily
/// nested vectors, but it's the same as just flattening the input into a single vector
pub fn reround(
    x: Vec<f64>,
    digits: i32,
    rounding: Vec<&str>,
    threshold: f64,
    symmetric: bool,
) -> Vec<f64> {
    x.iter()
        .flat_map(|&x| {
            reconstruct_rounded_numbers_scalar(x, digits, rounding[0], threshold, symmetric)
        })
        .collect()

    //x.iter()
    //.map(|inner| {
    //inner
    //.iter()
    //.flat_map(|&x| {
    //reconstruct_rounded_numbers_scalar(x, digits, rounding[0], threshold, symmetric)
    //})
    //.collect()
    //})
    //.collect()

    //this compiles, but does it work???

    // not quite, it seems to not actually be returning quite enough elements, namely we're missing
    // the 5th and 6th elements
    // this might be downtream of reconstruct_rounded_numbers_scalar

    //reconstruct_rounded_numbers_scalar(x, digits, rounding[0], threshold, symmetric)
}

//pub fn reround_bugged(
//x: Vec<Vec<f64>>,
//_digits: i32,
//rounding: Vec<&str>,
//_threshold: f64,
//_symmetric: bool,
//) {
//if rounding.len() > 1 {
//check_rounding_singular(rounding.clone(), "up_or_down", "up", "down")
//.expect("Error in selecting rounding options");
//check_rounding_singular(
//rounding.clone(),
//"up_from_or_down_from",
//"up_from",
//"down_from",
//)
//.expect("Error in selecting rounding options");
//check_rounding_singular(rounding.clone(), "ceiling_or_floor", "ceiling", "floor")
//.expect("Error in selecting rounding options");
//
//if (x.len() > 1) && (x.len() != rounding.len()) {
//let x_len = x.len();
//let round_len = rounding.len();
//////////panic!("x and rounding must have the same length unless one of them have length 1. x has length {x_len} and rounding has length {round_len}")
//}
//}
//
//return reconstruct_rounded_numbers_scalar(x: f64, digits: i32, rounding[0]: &str, threshold: f64, symmetric: bool);
//}

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

    // testing reconstruct_sd_scalar
    #[test]
    fn reconstruct_sd_scalar_test_1() {
        let sd_rec_scalar = reconstruct_sd_scalar("mean_n", "0.3", 30, 12, 15);
        assert_eq!(0.4660916, rust_round(sd_rec_scalar.unwrap(), 7)); // rounding to 7th decimal
                                                                      // place to match R output
    }

    #[test]
    fn reconstruct_sd_scalar_test_2() {
        let sd_rec_scalar = reconstruct_sd_scalar("groups", "0.3", 30, 12, 15);
        assert_eq!(0.5063697, rust_round(sd_rec_scalar.unwrap(), 7));
    }

    #[test]
    fn reconstruct_sd_scalar_test_3() {
        let sd_rec_scalar = reconstruct_sd_scalar("0_n", "0.3", 30, 12, 15);
        assert_eq!(0.4982729, rust_round(sd_rec_scalar.unwrap(), 7));
    }

    #[test]
    fn reconstruct_sd_scalar_test_4() {
        let sd_rec_scalar = reconstruct_sd_scalar("1_n", "0.3", 30, 12, 15);
        assert_eq!(0.5085476, rust_round(sd_rec_scalar.unwrap(), 7));
    }

    #[test]
    fn reconstruct_rounded_numbers_scalar_test_1() {
        let res = reconstruct_rounded_numbers_scalar(2.9876, 3, "up_or_down", 5.0, false);
        assert_eq!(res, vec![2.988, 2.988])
    }
}
