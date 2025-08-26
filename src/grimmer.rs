use core::f64;

use crate::decimal_places_scalar;
use crate::grim::{grim_scalar_rust, is_near, GrimReturn};
use crate::rounding::rust_round;
use crate::utils::{dustify, reround};
use pyo3::pyfunction;
//  146-148, 150, 169-171, 173, 180, 186, 198-201, 204-211, 217-219
const EPS: f64 = f64::EPSILON;

// bool params in this case takes show_reason, default false, and symmetric, default false
// no getting around it, the original contains lots of arguments, even if we condense the bools
// into a vec, we still have 8

/// Determines whether a standard deviation is possible from the listed mean and sample size
///
/// Implements L. Jung's adaptation of A. Allard's A-GRIMMER algorithm for testing the possibility
/// of standard deviations. (<https://aurelienallard.netlify.app/post/anaytic-grimmer-possibility-standard-deviations/>).
///
/// # Arguments
///     x: the sample mean
///     sd: sample standard deviation
///     n: sample size
///     items: number of items
///     bool_params: booleans for options in GRIMMER and the underlying GRIM function, in the form
///     [percent, show_reason, symmetric]
///     rounding: method of rounding
///     threshold: rounding threshold, ordinarily 5.0
///     tolerance: rounding tolerance usually the square root of machine epsilon
///
/// # Panics
///     - If x or sd are given as numbers instead of strings. This is necessary in order to
///     preserve trailing 0s
///     - If items is not 1. Items > 1 may be implemented in a later update
///
/// # Returns
#[allow(clippy::too_many_arguments)]
pub fn grimmer_scalar(
    x: &str,
    sd: &str,
    n: u32,
    items: u32,
    bool_params: Vec<bool>,
    rounding: &str,
    threshold: f64,
    tolerance: f64,
) -> bool {
    // in the original, items does not work as intended, message Jung about this once I have more
    // time
    if items != 1 {
        todo!("GRIMMER for items > 1 is not yet implemented")
    };

    let digits_sd = decimal_places_scalar(Some(sd), ".").unwrap();

    let _percent: bool = bool_params[0];
    let show_rec: bool = bool_params[1];
    let symmetric: bool = bool_params[2];

    let grim_return = grim_scalar_rust(
        x,
        n,
        bool_params.clone(),
        items,
        rounding,
        threshold,
        tolerance,
    );

    let pass_grim = match grim_return {
        Ok(grim_return) => match grim_return {
            GrimReturn::Bool(b) => b,
            GrimReturn::List(a, _, _, _, _, _, _, _) => a,
        },
        Err(_) => panic!(),
    };

    let n_items = n * items;

    let x: f64 = x.parse().unwrap();

    let sum = x * f64::from(n_items) ;
    let sum_real = rust_round(sum, 0);
    let x_real = sum_real / f64::from(n_items) ;

    if !pass_grim {
        if show_rec {
            println!("{x} is GRIM inconsistent")
        };
        return false;
    };

    let p10 = 10.0f64.powi(digits_sd + 1i32);
    let p10_frac = 5.0 / p10;

    let sd: f64 = sd.parse().unwrap(); // why can't this be a ?

    let sd_lower = (sd - p10_frac).max(0.0); // returning 0 if p10_frac is greater than sd and
                                             // would thus return a negative number

    let sd_upper = sd + p10_frac;

    let sum_squares_lower =
        (f64::from(n - 1) * sd_lower.powi(2) + f64::from(n) * x_real.powi(2)) * f64::from(items.pow(2)) ;
    let sum_squares_upper =
        (f64::from(n - 1) * sd_upper.powi(2) + f64::from(n) * x_real.powi(2)) * f64::from(items.pow(2)) ;

    let pass_test1: bool = sum_squares_lower.ceil() <= sum_squares_upper.floor();

    if !pass_test1 {
        if show_rec {
            println!("Failed test 1");
            return false;
        };
        return false;
    };

    let integers_possible: Vec<u32> =
        (sum_squares_lower.ceil() as u32..=sum_squares_upper.floor() as u32).collect();

    let sd_predicted: Vec<f64> = integers_possible
        .iter()
        .map(|x| {
            (((*x as f64 / items.pow(2) as f64) - n as f64 * x_real.powi(2)) / (n as f64 - 1.0))
                .powf(0.5)
        })
        .collect();

    let sd_rec_rounded = reround(sd_predicted, digits_sd, rounding, threshold, symmetric);

    let sd = dustify(sd);

    let sd_rec_rounded: Vec<f64> = sd_rec_rounded.into_iter().flat_map(dustify).collect();

    let matches_sd: Vec<bool> = sd
        .iter()
        .zip(sd_rec_rounded.iter())
        .map(|(i, sdr)| is_near(*i, *sdr, EPS.powf(0.5)))
        .collect();

    let pass_test2: bool = matches_sd.iter().any(|&b| b);

    if !pass_test2 {
        if show_rec {
            println!("Failed test 2");
            return false;
        };
        return false;
    }

    let sum_parity = sum_real % 2.0;

    let matches_parity: Vec<bool> = integers_possible
        .iter()
        .map(|&n| n as f64 % 2.0 == sum_parity)
        .collect();

    let matches_sd_and_parity: Vec<bool> = matches_sd
        .iter()
        .zip(matches_parity)
        .map(|(s, p)| s & p)
        .collect();

    let pass_test3 = matches_sd_and_parity.iter().any(|&b| b);

    if !pass_test3 {
        if show_rec {
            println!("Failed test 3");
            return false;
        };
        return false;
    }

    if show_rec {
        println!("Passed all tests");
        true
    } else {
        true
    }
}

#[cfg(not(tarpaulin_include))]
#[pyfunction(signature = (xs, sds, ns, rounding = "up_or_down".to_string(), items=vec![1], percent = false, show_reason = false, threshold = 5.0, symmetric = false, tolerance = f64::EPSILON.powf(0.5)))]
#[allow(clippy::too_many_arguments)]
/// Determines the possibility of standard deviations from given means and sample sizes using the A-GRIMMER algorithm.
///
/// This function implements L. Jung's adaptation of A. Allard's A-GRIMMER algorithm for testing the possibility of standard deviations. It processes multiple sets of means, standard deviations, and sample sizes, returning a boolean vector indicating the possibility for each set.
///
/// # Arguments
/// * `xs` - A vector of strings representing the sample means. Trailing zeros are preserved by using strings.
/// * `sds` - A vector of strings representing the sample standard deviations. Trailing zeros are preserved by using strings.
/// * `ns` - A vector of unsigned integers representing the sample sizes.
/// * `rounding` - A string specifying the method of rounding to be used.
/// * `items` - A vector of unsigned integers representing the number of items. Default is a vector with a single element [1].
/// * `percent` - A boolean indicating whether to treat the means as percentages. Default is false.
/// * `show_reason` - A boolean indicating whether to print the reason for failure if the tests do not pass. Default is false.
/// * `threshold` - A floating-point number representing the rounding threshold. Default is 5.0.
/// * `symmetric` - A boolean indicating whether to use symmetric rounding. Default is false.
/// * `tolerance` - A floating-point number representing the rounding tolerance, usually the square root of machine epsilon. Default is `f64::EPSILON.powf(0.5)`.
///
/// # Returns
/// A vector of booleans where each element corresponds to a set of inputs, indicating whether the standard deviation is possible for that set.
///
/// # Panics
/// The function will panic if the lengths of `xs`, `sds`, and `ns` do not match.
pub fn grimmer(
    xs: Vec<String>,
    sds: Vec<String>,
    ns: Vec<u32>,
    rounding: String,
    items: Vec<u32>,
    percent: bool,
    show_reason: bool,
    threshold: f64,
    symmetric: bool,
    tolerance: f64,
) -> Vec<bool> {
    let bool_params = vec![percent, show_reason, symmetric];
    let xs: Vec<&str> = xs.iter().map(|s| &**s).collect();
    let sds: Vec<&str> = sds.iter().map(|s| &**s).collect();

    grimmer_rust(
        xs,
        sds,
        ns,
        items,
        bool_params,
        rounding.as_str(),
        threshold,
        tolerance,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn grimmer_rust(
    xs: Vec<&str>,
    sds: Vec<&str>,
    ns: Vec<u32>,
    items: Vec<u32>,
    bool_params: Vec<bool>,
    rounding: &str,
    threshold: f64,
    tolerance: f64,
) -> Vec<bool> {
    xs.iter()
        .zip(sds.iter())
        .zip(ns.iter())
        .zip(items.iter())
        .map(|(((x, sd), n), item)| {
            grimmer_scalar(
                x,
                sd,
                *n,
                *item,
                bool_params.clone(),
                rounding,
                threshold,
                tolerance,
            )
        })
        .collect()
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn grimmer_scalar_test_1() {
        let val = grimmer_scalar(
            "1.03",
            "0.41",
            40,
            1,
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
        assert!(!val)
    }

    #[test]
    fn grimmer_scalar_test_2() {
        let val = grimmer_scalar(
            "1.03",
            "0.41",
            40,
            1,
            vec![false, false, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
        assert!(!val)
    }

    #[test]
    fn grimmer_scalar_test_3() {
        let val = grimmer_scalar(
            "3.10",
            "1.37",
            10,
            1,
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
        assert!(val)
    }

    #[test]
    fn grimmer_scalar_test_4() {
        let val = grimmer_scalar(
            "2.57",
            "2.57",
            30,
            1,
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
        assert!(val)
    }

    #[test]
    #[should_panic]
    fn grimmer_scalar_test_5() {
        let _ = grimmer_scalar(
            "2.57",
            "2.57",
            30,
            2, // in current version, item > 1 is not covered, should return an
            // todo! panic error
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
    }

    #[test]
    #[should_panic]
    fn grimmer_scalar_test_6() {
        let _ = grimmer_scalar(
            "",
            "2.57",
            30,
            1,
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
    }

    #[test]
    #[should_panic]
    fn grimmer_scalar_test_7() {
        let _ = grimmer_scalar(
            "2.57",
            "",
            30,
            1,
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
    }

    #[test]
    fn grimmer_scalar_test_8() {
        let val = grimmer_scalar(
            "1.03",
            "0.41",
            40,
            1,
            vec![true, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
        assert!(!val)
    }

    #[test]
    fn grimmer_rust_test_1() {
        let val = grimmer_rust(
            vec!["1.03"],
            vec!["0.41"],
            vec![40],
            vec![1],
            vec![false, true, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        )[0];
        assert!(!val)
    }

    #[test]
    fn grimmer_rust_test_2() {
        let val = grimmer_rust(
            vec!["52.13"],
            vec!["2.26"],
            vec![30],
            vec![1],
            vec![false, false, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        )[0];
        assert!(val)
    }

    #[test]
    fn grimmer_rust_test_3() {
        let vals = grimmer_rust(
            vec!["1.03", "52.13", "9.42375"],
            vec!["0.41", "2.26", "3.86"],
            vec![40, 30, 59],
            vec![1, 1, 1],
            vec![false, false, false],
            "up_or_down",
            5.0,
            EPS.powf(0.5),
        );
        assert_eq!(vals, vec![false, true, false])
    }
}
