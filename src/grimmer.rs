use core::f64;

use crate::decimal_places_scalar;
use crate::grim::{grim_scalar_rust, is_near, GrimReturn};
use crate::rounding::rust_round;
use crate::utils::{dustify, reround};

// bool params in this case takes show_reason, default false, and symmetric, default false
// no getting around it, the original contains lots of arguments, even if we condense the bools
// into a vec, we still have 8
#[allow(clippy::too_many_arguments)]
pub fn grimmer_scalar(
    x: &str,
    sd: &str,
    n: u32,
    items: u32,
    bool_params: Vec<bool>,
    rounding: Vec<&str>,
    threshold: f64,
    tolerance: f64,
) -> bool {
    // in the original, items does not work as intended, message Jung about this once I have more
    // time
    if items > 1 {
        panic!()
    };

    let digits_sd = decimal_places_scalar(Some(sd), ".").unwrap();

    let show_reason: bool = bool_params[0];
    let symmetric: bool = bool_params[1];

    let grim_return = grim_scalar_rust(
        x,
        n,
        bool_params.clone(),
        items,
        rounding.clone(),
        threshold,
        tolerance,
    );

    let pass_grim = match grim_return {
        Ok(grim_return) => match grim_return {
            GrimReturn::Bool(b) => b,
            GrimReturn::List(a, _, _, _, _, _) => a,
        },
        Err(_) => panic!(),
    };

    let n_items = n * items;

    let x: f64 = x.parse().unwrap();

    let sum = x * n_items as f64;
    let sum_real = rust_round(sum, 0);
    let x_real = sum_real / n_items as f64;

    if !pass_grim {
        if show_reason {
            panic!("code this arm already jackass")
        };
        return false;
    };

    let p10 = 10.0f64.powi(digits_sd + 1i32);
    let p10_frac = 5.0 / p10;

    let sd: f64 = sd.parse().unwrap(); // why can't this be a ?

    let sd_lower = match sd < p10_frac {
        true => 0f64,
        false => sd - p10_frac,
    };

    let sd_upper = sd + p10_frac;

    let sum_squares_lower =
        ((n - 1) as f64 * sd_lower.powi(2) + n as f64 * x_real.powi(2)) * items.pow(2) as f64;
    let sum_squares_upper =
        ((n - 1) as f64 * sd_upper.powi(2) + n as f64 * x_real.powi(2)) * items.pow(2) as f64;

    let pass_test1: bool = sum_squares_lower.ceil() <= sum_squares_upper.floor();

    if !pass_test1 {
        if show_reason {
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

    // double check that this works as expected
    // instead of generated var_predicted, we go directly to sd_predictably

    let sd_rec_rounded = reround(sd_predicted, digits_sd, rounding, threshold, symmetric);

    // again, double check on reround, give it some robust testing

    let sd = dustify(sd);

    // the line below is doing the same in one line as the two lines below
    //let interim: Vec<f64> = sd_rec_rounded.into_iter().flatten().collect();
    //let sd_rec_rounded: Vec<Vec<f64>> = interim.iter().map(|x| dustify(*x)).collect();

    let sd_rec_rounded: Vec<f64> = sd_rec_rounded.into_iter().flat_map(dustify).collect();

    // checking whether the elements of sd are within a tolerance of their equivalent in
    // sd_rec_rounded
    // also assuming we should be flattening the latter

    let matches_sd: Vec<bool> = sd
        .iter()
        .zip(sd_rec_rounded.iter())
        .map(|(i, sdr)| is_near(*i, *sdr, f64::EPSILON.powf(0.5)))
        .collect();

    let pass_test2: bool = matches_sd.iter().any(|&b| b);

    if !pass_test2 {
        if show_reason {
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

    //let matches_parity = sum_real % 2.0 == (integers_possible % 2);

    let matches_sd_and_parity: Vec<bool> = matches_sd
        .iter()
        .zip(matches_parity)
        .map(|(s, p)| s & p)
        .collect();

    let pass_test3 = matches_sd_and_parity.iter().any(|&b| b);

    if !pass_test3 {
        if show_reason {
            println!("Failed test 3");
            return false;
        };
        return false;
    }

    if show_reason {
        println!("Passed all tests");
        true
    } else {
        true
    }

    // make absolutely double check that this is returning the expected result
    //
    //

    // likely not how it's meant to work, but I'm going to flatten
}
