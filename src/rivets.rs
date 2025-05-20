use std::num::ParseFloatError;
use pyo3::FromPyObject;

use crate::{debit::unround, grim::{grim_scalar, GRIMInput, GrimScalarError}, utils::decimal_places_scalar};


pub fn rivets_t_test(x1: f64, sd1: f64, n1: f64, x2: f64, sd2: f64, n2: f64) -> f64 {
    let p_var = (((sd1.powi(2)) * (n1 - 1.0)) + ((sd2.powi(2)) * (n2 - 1.0))) / (n1 + n2 - 2.0);
    let p_se = ((p_var / n1) + (p_var / n2)).powf(0.5);

    (x1-x2) / p_se
}

#[derive(FromPyObject)]
pub enum RIVETSInput {
    Str(String),
    Num(f64)
}

impl RIVETSInput {
    fn to_grim(&self) -> GRIMInput {
        match self {
            RIVETSInput::Str(s) => GRIMInput::Str(s.to_string()),
            RIVETSInput::Num(n) => GRIMInput::Num(*n),
        }
    }

    fn to_some_str(&self) -> Option<String> {
        match self {
            RIVETSInput::Str(s) => Some(s.to_string()),
            RIVETSInput::Num(n) => Some(n.to_string()),
        }
    }
}

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments)]
pub fn rivets_possible_values(x: RIVETSInput, 
    n: u32, 
    items: u32, 
    dispersion: u32, 
    rounding: String, 
    threshold: f64, 
    symmetric: bool, 
    tolerance: f64, 
    digits: i32) -> Result<Vec<f64>, GrimScalarError> {

    // moved this up first to avoid a clone
    let x_unrounded = unround(&x.to_some_str().unwrap(), &rounding, threshold).unwrap();

    let x_lower = x_unrounded.lower;
    let x_upper = x_unrounded.upper;
    let x_incl_lower = x_unrounded.incl_lower;
    let x_incl_upper = x_unrounded.incl_upper;

    let grim_consistent = grim_scalar(x.to_grim(), n, rounding, items, false, false, threshold, symmetric, tolerance);

    if !grim_consistent {
        todo!()
    }

    let digits = decimal_places_scalar(x.to_some_str().as_deref(), ".");
    let p10 = 10.0f64.powi(digits.unwrap() + 1);

    let granularity = 1.0 / (n*items) as f64;


    let Ok(x): Result<f64, ParseFloatError> = x.to_some_str().unwrap().parse() else {
        return Err(GrimScalarError::ParseFloatError)
    };

    // let transgresses_at_lower = if x_incl_lower {"<"} else {"<="};
    //let transgresses_at_upper = if x_incl_upper {">"} else {">="};

    let mut x_possible = vec![x];

    let x_current = x;

    loop {
        let x_current = x_current - granularity;
        if transgresses_at_lower(x_current, x_lower, x_incl_lower) {
            break
        }
        x_possible.push(x_current)
    }

    loop {
        let x_current = x_current - granularity;
        if transgresses_at_upper(x_current, x_upper, x_incl_upper) {
            break
        }
        x_possible.push(x_current)
    }

    if x_possible.len() == 1 {
        x_possible.push(x_possible[0])
    }

    Ok(x_possible)

}

pub fn transgresses_at_lower(a: f64, b: f64, strict: bool) -> bool {
    if strict { a < b } else { a <= b }
}

pub fn transgresses_at_upper(a: f64, b: f64, strict: bool) -> bool {
    if strict { a > b } else { a >= b }
}



