use crate::utils::{decimal_places_scalar, dustify, reround};
use pyo3::prelude::Bound;
use pyo3::prelude::*;
use pyo3::{pyfunction, wrap_pyfunction, FromPyObject};

#[derive(FromPyObject)]
enum GRIMInput {
    Str(String),
    Num(f64), // ideally, this will also capture an input integer and coerce it into and f64.
              // Make a test case on the Python end to confirm this
}
/// reproducing scrutiny's grim_scalar() function, albeit with slightly different order of
/// arguments, because unlike R, Python requires that all the positional parameters be provided up
/// front before optional arguments with defaults
#[pyfunction(signature = (x, n, rounding, items=1, percent = false, show_rec = false, threshold = 5.0, symmetric = false, tolerance = f64::EPSILON.powf(0.5)))]
#[allow(clippy::too_many_arguments)]
fn grim_scalar(
    x: GRIMInput,
    n: u32,
    rounding: Vec<String>,
    items: u32,
    percent: bool,
    show_rec: bool,
    threshold: f64,
    symmetric: bool,
    tolerance: f64,
) -> bool {
    let x: String = match x {
        GRIMInput::Str(s) => s,
        GRIMInput::Num(n) => format!("{}", n),
    };
    // accounting for the possibility that we might receive either a String or numeric type,
    // turning the numeric possibility into a String, which we later turn into a &str to
    // pass into grim_scalar_rust()

    let rounds: Vec<&str> = rounding.iter().map(|s| &**s).collect(); // idiomatic way to
                                                                     // turn Vec<String> to Vec<&str>
    let val = grim_scalar_rust(
        x.as_str(),
        n,
        vec![percent, show_rec, symmetric],
        items,
        rounds,
        threshold,
        tolerance,
    );

    match val {
        Ok(r) => match r {
            GrimReturn::Bool(b) => b,
            GrimReturn::List(a, _, _, _, _, _) => a,
        },
        Err(_) => panic!(),
    }
}

pub enum GrimReturn {
    Bool(bool),
    List(bool, f64, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>),
    //
    //
    //
    //List(
    // bool,
    //f64,
    //Vec<f64>,
    //Vec<f64>,
    //Vec<f64>,
    //Vec<f64>,
    //Vec<f64>,
    //Vec<f64>,
    //),
    //
}

// vector wrapper for grim_scalar_rust
pub fn grim_rust(
    xs: Vec<&str>,
    ns: Vec<u32>,
    bool_params: Vec<bool>,
    items: Vec<u32>,
    rounding: Vec<&str>,
    threshold: f64,
    tolerance: f64,
) -> Vec<bool> {
    //fn grim(xs: &[f64], nums: &[f64], items: &[f64]) -> Vec<f64> {
    let vals: Vec<Result<GrimReturn, std::num::ParseFloatError>> = xs
        .iter()
        .zip(ns.iter())
        .zip(items.iter())
        .map(|((x, num), item)| {
            grim_scalar_rust(
                x,
                *num,
                bool_params.clone(),
                *item,
                rounding.clone(),
                threshold,
                tolerance,
            )
        })
        .collect();

    vals.iter()
        .map(|grim_result| match grim_result {
            Ok(grim_return) => match grim_return {
                GrimReturn::Bool(b) => *b,
                GrimReturn::List(a, _, _, _, _, _) => *a,
            },
            Err(_) => panic!(),
        })
        .collect()
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
    rounding: Vec<&str>,
    threshold: f64,
    tolerance: f64,
) -> Result<GrimReturn, std::num::ParseFloatError> {
    let percent: bool = bool_params[0];
    let show_rec: bool = bool_params[1];
    let symmetric: bool = bool_params[2];

    let mut x_num: f64 = x.parse()?;

    let mut digits: i32 = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        x_num /= 100.0;
        digits += 2;
    };

    let n_items = n * items;

    let rec_sum = x_num * n_items as f64;

    let rec_x_upper = dustify(rec_sum.ceil() / n_items as f64);
    let rec_x_lower = dustify(rec_sum.floor() / n_items as f64);

    let grains_rounded = reround(
        vec![rec_x_upper.clone(), rec_x_lower.clone()],
        digits,
        rounding.clone(),
        threshold,
        symmetric,
    );

    let flat: Vec<f64> = grains_rounded.clone().into_iter().flatten().collect();

    // what's the return type here? is it a vec of bools? Let's run grim with some sample data and
    // check. Or are we checking whether any single one of these is true??
    let bools: Vec<bool> = flat
        .into_iter()
        .map(|x| is_near(x, x_num, tolerance))
        .collect();

    let grain_is_x: bool = bools.iter().any(|&b| b);

    if !show_rec {
        Ok(GrimReturn::Bool(grain_is_x))
    } else {
        let consistency: bool = grain_is_x;

        let length_2ers = ["up_or_down", "up_from_or_down_from", "ceiling_or_floor"];

        if rounding.iter().any(|r| length_2ers.contains(r)) {
            Ok(GrimReturn::List(
                consistency,
                rec_sum,
                rec_x_upper,
                rec_x_lower,
                grains_rounded[0].clone(),
                grains_rounded[1].clone(),
                //grains_rounded[4].clone(),
                //grains_rounded[5].clone(),
            ))
        } else {
            Ok(GrimReturn::Bool(true))
        }
    }
}

/// Determine whether the two provided numbers are within a given tolerance of each other
pub fn is_near(num_1: f64, num_2: f64, tolerance: f64) -> bool {
    (num_1 - num_2).abs() <= tolerance
}

/// Automatically unpacks and tests the output of grim_scalar_rust and checks whether its main bool
/// result matches the expected bool
pub fn grim_tester(grim_result: Result<GrimReturn, std::num::ParseFloatError>, expected: bool) {
    match grim_result {
        Ok(grim_return) => match grim_return {
            GrimReturn::Bool(b) => match expected {
                true => assert!(b),
                false => assert!(!b),
            },
            GrimReturn::List(a, _, _, _, _, _) => assert!(!a),
        },
        Err(_) => panic!(),
    };
}

//#[pymodule]
// for some reason this causes an error when using cargo test --lib
// commenting out for the moment, let's see if this has any effect on the porting to python
#[allow(dead_code)]
fn scrutipy(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(grim_scalar, module)?)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use core::f64;

    use super::*;

    #[test]
    pub fn grim_scalar_rust_test_1() {
        let val = grim_scalar_rust(
            "5.19",
            40,
            vec![false, false, false],
            1,
            vec!["up_or_down"],
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, false)
    }

    #[test]
    pub fn grim_scalar_rust_test_2() {
        let val = grim_scalar_rust(
            "5.18",
            40,
            vec![false, false, false],
            1,
            vec!["up_or_down"],
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, true);
    }

    #[test]
    pub fn grim_scalar_rust_test_3() {
        let val = grim_scalar_rust(
            "5.19",
            40,
            vec![false, false, false],
            2,
            vec!["up_or_down"],
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, true);
    }

    #[test]
    pub fn grim_scalar_rust_test_4() {
        let val = grim_scalar_rust(
            "5.19",
            20,
            vec![false, true, false],
            1,
            vec!["up_or_down"],
            5.0,
            f64::EPSILON.powf(0.5),
        );
        grim_tester(val, false);
    }

    #[test]
    pub fn grim_rust_test_1() {
        let xs = vec![
            "7.22", "4.74", "5.23", "2.57", "6.77", "2.68", "7.01", "7.38", "3.14", "6.89", "5.00",
            "0.24",
        ];

        let ns = vec![32, 25, 29, 24, 27, 28, 29, 26, 27, 31, 25, 28];

        let items = vec![1; 12]; //presumably all 1s?

        let bools = grim_rust(
            xs,
            ns,
            vec![false, false, false],
            items,
            vec!["up_or_down"],
            5.0,
            f64::EPSILON.powf(0.5),
        );

        assert_eq!(
            bools,
            vec![true, false, false, false, false, true, false, true, false, false, true, false]
        );
    }
}
// round the grains using reround(), see reround.R

// what is the return type? not a single value, but a list of values, depending on the
// conditions

// let's start with a nice simple one GRIM
//
// let's pseudocode this out
//
// we take in a number, eg 5.19
// a sample n associated with that number
// a number of items, by default 1
// and then some defaults and keyword arguments we can deal with
//
// we need to check that 'items' is a number, and that the
// percent keyword is a bool
//
// we expect x to actually come in as a string
// instead of as a number, because we need to preserve trailing 0s
//
// we then turn x into a number in a separate variable, and
// record the number of decimal places separately
//
// possibly convert these into percents if need be
//
// we create n_items, which is the sample size times the number if items
// still not quite clear on what 'items' is doing
// and rec_sum, n_items times the numerical value of x, which if
// x is a mean should be the sum of all the original values
//
// then use the dustify() function to generate an upper and lower bound
// for the possible mean or percent vales
//
// round them using a specialized internal function
//
// then check if the reported value is close to either of the reconstructed values, the upper or
// lower bound
//
// i guess dustify is actually really simple, it just fuzzes the value to within 1e-12 and returns
// the values as  a vector
//
//
