use crate::utils::{decimal_places_scalar, dustify};

pub fn grim_scalar(
    x: &str,
    n: u32,
    _bool_params: Vec<bool>, // includes percent, show_rec, and symmetric
    items: u32,
    _rounding: &str,
    _threshold: u16,
    _tolerance: f64,
) {
    let percent: bool = _bool_params[0];
    let _show_rec: bool = _bool_params[1];
    let _symmetric: bool = _bool_params[2];

    let mut x_num: f64 = match x.parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    let mut digits = decimal_places_scalar(Some(x), ".").unwrap();

    if percent {
        x_num = x_num / 100.0;
        digits += 2;
    };

    let n_items = n * items;

    let rec_sum = x_num * n_items as f64;

    let rec_x_upper = dustify(rec_sum.ceil() / n_items as f64);
    let rec_x_lower = dustify(rec_sum.floor() / n_items as f64);

    // round the grains using reround(), see reround.R
}

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
