use std::num::ParseFloatError;

use crate::utils::{decimal_places_scalar, dustify, reround};
use pyo3::{pyfunction, FromPyObject};
use thiserror::Error;

#[derive(FromPyObject)]
pub enum GRIMInput {
    Str(String),
    Num(f64), // this captures input integer and coerces it into a string if possible, in order to
              // deal with user error on the Python interface
}
/// reproducing scrutiny's grim_scalar() function, albeit with slightly different order of
/// arguments, because unlike R, Python requires that all the positional parameters be provided up
/// front before optional arguments with defaults
#[allow(clippy::too_many_arguments)]
#[pyfunction(signature = (x, n, rounding = "up_or_down".to_string(), items=1, percent = false, show_rec = false, threshold = 5.0, symmetric = false, tolerance = f64::EPSILON.powf(0.5)))]
pub fn grim_scalar(
    x: GRIMInput,
    n: u32,
    rounding: String,
    items: u32,
    percent: bool,
    show_rec: bool,
    threshold: f64,
    symmetric: bool,
    tolerance: f64,
) -> bool {
    let x: String = match x {
        GRIMInput::Str(s) => s,
        GRIMInput::Num(n) => format!("{n}"),
    };
    // accounting for the possibility that we might receive either a String or numeric type,
    // turning the numeric possibility into a String, which we later turn into a &str to
    // pass into grim_scalar_rust()

    //let round: &str = rounding.as_str();
                                                                     // turn Vec<String> to Vec<&str>
    let val = grim_scalar_rust(
        x.as_str(),
        n,
        vec![percent, show_rec, symmetric],
        items,
        rounding.as_str(),
        threshold,
        tolerance,
    );

    match val {
        Ok(r) => match r {
            GrimReturn::Bool(b) => b,
            GrimReturn::List(a, _, _, _, _, _, _, _) => a,
        },
        Err(_) => panic!(),
    }
}


pub enum GrimReturn {
    Bool(bool),
    List(bool, f64, Vec<f64>, Vec<f64>, f64, f64, f64, f64),
}


// vector wrapper for grim_scalar_rust
pub fn grim_rust(
    xs: Vec<&str>,
    ns: Vec<u32>,
    bool_params: Vec<bool>,
    items: Vec<u32>,
    rounding: &str,
    threshold: f64,
    tolerance: f64,
) -> Vec<bool> {

    let vals: Vec<Result<GrimReturn, GrimScalarError>> = xs
        .iter()
        .zip(ns.iter())
        .zip(items.iter())
        .map(|((x, num), item)| {
            grim_scalar_rust(
                x,
                *num,
                bool_params.clone(),
                *item,
                rounding,
                threshold,
                tolerance,
            )
        })
        .collect();

    vals.iter()
        .map(|grim_result| match grim_result {
            Ok(grim_return) => match grim_return {
                GrimReturn::Bool(b) => *b,
                GrimReturn::List(a, _, _, _, _, _, _, _) => *a,
            },
            Err(_) => panic!(),
        })
        .collect()
}

#[derive(Debug, Error)]
pub enum GrimScalarError {
    #[error("Could not parse x into a number")]
    ParseFloatError,
    #[error("Could not extract decimal places")]
    DecimalNullError(String),
}
/// Performs GRIM test of a single number
///
/// We test whether the provided mean is within a plausible rounding of any possible means given
/// the number of samples
pub fn grim_scalar_rust(
    x: &str,
    n: u32,
    bool_params: Vec<bool>, // includes percent, show_rec, and symmetric
    items: u32,
    rounding: &str,
    threshold: f64,
    tolerance: f64,
) -> Result<GrimReturn, GrimScalarError> {
    let percent: bool = bool_params[0];
    let show_rec: bool = bool_params[1];
    let symmetric: bool = bool_params[2];

    // Define key values from arguments
    let Ok(mut x_num): Result<f64, ParseFloatError> = x.parse() else {
        return Err(GrimScalarError::ParseFloatError)
    };
    let Some(mut digits): Option<i32> = decimal_places_scalar(Some(x), ".") else {
        return Err(GrimScalarError::DecimalNullError("".to_string()));
    };

    // the `percent` argument allows for easy conversion of percentages to decimal numbers
    if percent {
        x_num /= 100.0;
        digits += 2;
    };

    // prepare further objects for reconstructing original values
    let n_items = n * items;
    let rec_sum = x_num * f64::from(n_items) ;

    // now reconstruct the possible mean or percentage values ('granules'), controlling
    // for small differences introduced by spurious precision
    let rec_x_upper = dustify(rec_sum.ceil() / f64::from(n_items) );
    let rec_x_lower = dustify(rec_sum.floor() / f64::from(n_items) );

    // concatenate the mean vectors
    let conc: Vec<f64> = rec_x_upper
        .iter()
        .cloned()
        .chain(rec_x_lower.iter().cloned())
        .collect();

    // Round these "granules" using an internal helper function that also gets the number of decimal places 
    // as well as the `rounding`, `threshold`, and  `symmetric` arguments passed down to:
    let granules_rounded = reround(conc, digits, rounding, threshold, symmetric);

    // test if the reported mean or percentage `x_num` is within tolerance of for each reconstructed
    // values, 'granule'
    let bools: Vec<bool> = granules_rounded
        .clone()
        .into_iter()
        .map(|x| is_near(x, x_num, tolerance))
        .collect();

    // report whether any of the above comparisons returned true
    let consistency: bool = bools.iter().any(|&b| b);

    if !show_rec {
        Ok(GrimReturn::Bool(consistency))
    } else {
        let length_2ers = ["up_or_down", "up_from_or_down_from", "ceiling_or_floor"];

        if length_2ers.contains(&rounding) {
            Ok(GrimReturn::List(
                consistency,
                rec_sum,
                rec_x_upper,
                rec_x_lower,
                granules_rounded[0],
                granules_rounded[1],
                granules_rounded[4],
                granules_rounded[5],
            ))
        } else {
            Ok(GrimReturn::Bool(consistency))
        }
    }
}

/// Determine whether the two provided numbers are within a given tolerance of each other
pub fn is_near(num_1: f64, num_2: f64, tolerance: f64) -> bool {
    (num_1 - num_2).abs() <= tolerance
}

/// Automatically unpacks and tests the output of grim_scalar_rust and checks whether its main bool
/// result matches the expected bool
pub fn grim_tester(grim_result: Result<GrimReturn, GrimScalarError>, expected: bool) {
    match grim_result {
        Ok(grim_return) => match grim_return {
            GrimReturn::Bool(b) => match expected {
                true => assert!(b),
                false => assert!(!b),
            },
            GrimReturn::List(a, _, _, _, _, _, _, _) => assert!(!a),
        },
        Err(_) => panic!(),
    };
}

