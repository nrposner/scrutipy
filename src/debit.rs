use core::f64;
use thiserror::Error;
use crate::utils::{dustify, reround};
#[allow(unused_imports)]
use crate::utils::{decimal_places_scalar, reconstruct_sd_scalar};



// fn debit_scalar(x: &str, sd: &str, n: u32, formula: &str, rounding: &str, threshold: f64, symmetric: bool) {
    // //check_debit_inputs_all(x, sd)
   //  
    // out <- debit_table(
    // x = x, sd = sd, n = n,
    // formula = formula, rounding = rounding,
    // threshold = threshold, symmetric = symmetric
    // )
// 
  // return out.0
// }

#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
#[allow(unused_variables)]
fn debit_table(
    x: &str, 
    sd: &str, 
    n: u32, 
    group_0: bool, 
    group_1: bool, 
    formula: &str, 
    rounding: &str, 
    threshold: f64, 
    symmetric: bool, 
    show_rec: bool
) -> bool {
    let digits_x = decimal_places_scalar(Some(x), ".");
    let digits_sd = decimal_places_scalar(Some(sd), ".");


    let x_num: f64 = x.parse().unwrap();
    let sd_num: u32 = sd.parse().unwrap();


    let x_unrounded = unround(x, rounding, f64::EPSILON.powf(0.5)).unwrap();

    let x_lower = x_unrounded.lower.to_string();
    let x_upper = x_unrounded.upper.to_string();

    let sd_unrounded = unround(sd, rounding, f64::EPSILON.powf(0.5)).unwrap();

    let sd_lower = sd_unrounded.lower;
    let sd_upper = sd_unrounded.upper;

    let sd_rec_lower = reconstruct_sd_scalar(formula, x_lower.as_str(), n, 0, 0);
    let sd_rec_upper = reconstruct_sd_scalar(formula, x_upper.as_str(), n, 0, 0);
    
    let x_incl_lower = x_unrounded.incl_lower;
    let x_incl_upper = x_unrounded.incl_upper;

    let sd_incl_lower = sd_unrounded.incl_lower;
    let sd_incl_upper = sd_unrounded.incl_upper;
    // right now, this will only support mean reconstruction, not other formulas

    let mut sd_rec_lower = reround(vec![sd_rec_lower.unwrap()], digits_sd.unwrap(), rounding, threshold, symmetric);
    let mut sd_rec_upper = reround(vec![sd_rec_upper.unwrap()], digits_sd.unwrap(), rounding, threshold, symmetric);
    
    sd_rec_lower.append(&mut sd_rec_upper);

    let sd_lower_test = dustify(sd_lower);
    let sd_rec_both_test: Vec<_> = sd_rec_lower.iter().flat_map(|x| dustify(*x)).collect();
    // we just concatenate the latter into the former
    let sd_upper_test = dustify(sd_upper);

    // Determine consistency based on inclusion flags and test results
    let consistency = if sd_incl_lower && sd_incl_upper {
        sd_lower_test.iter().any(|&x| sd_rec_both_test.iter().any(|&y| x <= y)) &&
        sd_rec_both_test.iter().any(|&x| sd_upper_test.iter().any(|&y| x <= y))
    } else if sd_incl_lower && !sd_incl_upper {
        sd_lower_test.iter().any(|&x| sd_rec_both_test.iter().any(|&y| x <= y)) &&
        sd_rec_both_test.iter().any(|&x| sd_upper_test.iter().any(|&y| x < y))
    } else if !sd_incl_lower && sd_incl_upper {
        sd_lower_test.iter().any(|&x| sd_rec_both_test.iter().any(|&y| x < y)) &&
        sd_rec_both_test.iter().any(|&x| sd_upper_test.iter().any(|&y| x <= y))
    } else {
        sd_lower_test.iter().any(|&x| sd_rec_both_test.iter().any(|&y| x < y)) &&
        sd_rec_both_test.iter().any(|&x| sd_upper_test.iter().any(|&y| x < y))
    };

    consistency
    

}
    


#[derive(Debug, Error)]
pub enum RoudingBoundError {
    #[error("The input x is 0")]
    ZeroError,
    #[error("The rounding type provided is not valid")]
    RoundingError,
}

pub fn rounding_bounds(
    rounding: &str, 
    x_num:f64, 
    d_var: f64, 
    d: f64
) -> Result<(f64, f64, &'static str, &'static str), RoudingBoundError> {

    if rounding == "trunc" {
        if x_num > 0.0 {
            Ok((x_num, x_num + (2.0 * d), "<=", "<"))
        } else if x_num < 0.0 {
            Ok((x_num - (2.0 * d), x_num, "<", "<="))
        } else {
            Ok((x_num - (2.0 * d), x_num + (2.0 * d), "<",   "<"))
        }
    } else if rounding == "anti_trunc" {
        if x_num > 0.0 {
            Ok((x_num - (2.0 * d), x_num , "<=", "<"))
        } else if x_num < 0.0 {
            Ok((x_num, x_num + (2.0 * d), "<=", "<"))
        } else {
            Err(RoudingBoundError::ZeroError)
        }
    } else {
        match rounding {
            "up_or_down" => Ok((x_num - d_var, x_num + d_var, "<=", "<=")),
            "up" => Ok((x_num - d_var, x_num + d_var, "<=", "<")), 
            "down" => Ok((x_num - d_var, x_num + d_var, "<", "<=")), 
            "even" => Ok((x_num - d, x_num + d, "<", "<")),
            "ceiling" => Ok((x_num - (2.0 * d), x_num, "<", "<=")), 
            "floor" => Ok((x_num, x_num + (2.0 * d), "<=", "<")),
            _ => Err(RoudingBoundError::RoundingError)
        }
    }
}

fn unround(x: &str, rounding: &str, threshold: f64) -> Result<UnroundReturn, RoudingBoundError> {
    let digits = decimal_places_scalar(Some(x), ".");
    let p10: f64 = 10.0f64.powi(digits.unwrap() + 1);
    let d = 5.0 / p10;
    let d_var = threshold / p10;

    let x_num :f64 = x.parse().unwrap();

    let bounds = rounding_bounds(rounding, x_num, d_var, d).unwrap();

    let lower = bounds.0;
    let upper = bounds.1;

    let sign_lower = bounds.2;
    let sign_upper = bounds.3;

    Ok(UnroundReturn::new(lower, sign_lower == "<=", sign_upper == "<=", upper))



}

struct UnroundReturn {
    lower: f64, 
    incl_lower: bool,
    incl_upper: bool,
    upper: f64,
}

impl UnroundReturn {
    pub fn new(lower: f64, incl_lower: bool, incl_upper: bool, upper: f64) -> Self {
        UnroundReturn {lower, incl_lower, incl_upper, upper}
    }
}








