use core::f64;
use thiserror::Error;
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
) {
    let digits_x = decimal_places_scalar(Some(x), ".");
    let digits_sd = decimal_places_scalar(Some(sd), ".");


    let x_num: f64 = x.parse().unwrap();
    let sd_num: u32 = sd.parse().unwrap();


    let x_unrounded = unround(x, rounding, f64::EPSILON.powf(0.5)).unwrap();

    let x_lower = x_unrounded.0;
    let x_upper = x_unrounded.1;


    let sd_unrounded = unround(sd, rounding, f64::EPSILON.powf(0.5)).unwrap();

    let sd_lower = sd_unrounded.0;
    let sd_upper = sd_unrounded.1;

    //let sd_rec_lower = reconstruct_sd_scalar(formula, x_lower, n, 0, 0);
    //let sd_rec_upper = reconstruct_sd_scalar(formula, x_upper, n, 0, 0);
    // right now, this will only support mean reconstruction, not other formulas

    

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

fn unround(x: &str, rounding: &str, threshold: f64) -> Result<(f64, f64, &'static str, &'static str), RoudingBoundError> {
    let digits = decimal_places_scalar(Some(x), ".");
    let p10: f64 = 10.0f64.powi(digits.unwrap() + 1);
    let d = 5.0 / p10;
    let d_var = threshold / p10;

    let x_num :f64 = x.parse().unwrap();

    rounding_bounds(rounding, x_num, d_var, d)
}








