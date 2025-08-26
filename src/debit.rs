use core::f64;
use thiserror::Error;
use crate::utils::{dustify, reround};
use crate::utils::{decimal_places_scalar, reconstruct_sd_scalar};
use pyo3::{pyfunction, PyResult, exceptions::PyValueError, PyErr};
use thiserror;

#[derive(Debug, Error, PartialEq)]
enum DebitError {
    #[error("The lengths of xs, sds, and ns are not equal: xs: {0}, sds: {1}, ns: {2}")]
    LengthError(usize, usize, usize)
}

impl From<DebitError> for PyErr {
    fn from(err: DebitError) -> PyErr {
        match err {
            DebitError::LengthError(xs_len, sds_len, ns_len) => {
                PyValueError::new_err(format!(
                    "The lengths of xs, sds, and ns are not equal: xs: {xs_len}, sds: {sds_len}, ns: {ns_len}",
                ))
            }
        }
    }
}

/// Computes consistency using the Descriptive Binary test (DEBIT) for means and standard
/// deviations. 
///
/// This function takes vectors of `xs`, `sds`, and `ns` along with several optional parameters
/// to calculate the debit consistency. It returns a vector of boolean values indicating
/// the consistency for each set of inputs.
///
/// # Arguments
///
/// * `xs` - A vector of strings representing the x values, which must be parsed into
/// floating-point numbers.
/// * `sds` - A vector of strings representing the standard deviation values, which must be parsed
/// into floating-point numbers.
/// * `ns` - A vector of unsigned integers representing the sample sizes.
/// * `formula` - A string slice that specifies the formula to use (default is "mean_n").
/// NOTE: only the mean_n formula is currently working. Other formulas will be implemented in later
/// versions
/// * `rounding` - A string slice that specifies the rounding method (default is "up_or_down").
/// * `threshold` - A floating-point number representing the threshold for rounding (default is 5.0).
/// * `symmetric` - A boolean indicating whether the rounding should be symmetric (default is false).
/// * `show_rec` - A boolean indicating whether to show the reconstructed values (default is false).
///
/// # Returns
///
/// Returns a `PyResult` containing a vector of boolean values. Each boolean indicates whether
/// the corresponding set of inputs is consistent according to the specified parameters.
///
/// # Errors
///
/// Returns a `PyValueError` if the lengths of `xs`, `sds`, and `ns` are not equal. This ensures
/// that each element in the vectors corresponds to a complete set of inputs for the calculation.
///
/// # Example
///
/// ```rust
/// let xs = vec!["0.36".to_string(), "0.11".to_string()];
/// let sds = vec!["0.11".to_string(), "0.31".to_string()];
/// let ns = vec![20, 40];
/// let result = debit(xs, sds, ns, "mean_n", "up_or_down", 5.0, false, false);
/// assert!(result.is_ok());
/// assert!(result == vec![false, true]) 
#[pyfunction(signature = (
    xs,
    sds,
    ns,
    formula = "mean_n",
    rounding = "up_or_down",
    threshold = 5.0,
    symmetric = false,
    show_rec = false
))]
#[allow(clippy::too_many_arguments)]
pub fn debit(
    xs: Vec<String>,
    sds: Vec<String>,
    ns: Vec<u32>,
    formula: &str,
    rounding: &str,
    threshold: f64,
    symmetric: bool,
    show_rec: bool
) -> PyResult<Vec<bool>> {

    match debit_rust(xs, sds, ns, formula, rounding, threshold, symmetric, show_rec) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.into())
    }
}

#[allow(clippy::too_many_arguments)]
fn debit_rust(
    xs: Vec<String>,
    sds: Vec<String>,
    ns: Vec<u32>,
    formula: &str,
    rounding: &str,
    threshold: f64,
    symmetric: bool,
    show_rec: bool
) -> Result<Vec<bool>, DebitError>  {

    if (formula != "mean_n") & (formula != "mean") {
        todo!("Formulas other than mean_n are not yet implemented")
    };

    if xs.len() != sds.len() || sds.len() != ns.len() {
        return Err(DebitError::LengthError(
            xs.len(), sds.len(), ns.len()));
    }
    Ok(xs.iter().zip(sds.iter()).zip(ns.iter()).map(|((x, sd), n)| 
        debit_scalar(
            x.as_str(), 
            sd.as_str(), 
            *n, 
            formula, 
            rounding, 
            threshold,
            symmetric, 
            show_rec
        )).collect())
}

// adjust to also support other formulas using group0 and group1
#[pyfunction(signature = (
    x, 
    sd, 
    n, 
    formula = "mean_n", 
    rounding = "up_or_down", 
    threshold = 5.0, 
    symmetric = false, 
    show_rec = false
))]
#[allow(clippy::too_many_arguments)]
pub fn debit_scalar(
    x: &str, 
    sd: &str, 
    n: u32, 
    formula: &str, 
    rounding: &str, 
    threshold: f64, 
    symmetric: bool, 
    show_rec: bool
) -> bool {
    let table = debit_table(
        x,
        sd, 
        n,
        formula, 
        rounding, 
        threshold,
        symmetric,
        show_rec,
    );

    match table {
        DebitTables::DebitTable(debit_table) => debit_table.consistency,
        DebitTables::DebitTableVerbose(debit_table_verbose) => debit_table_verbose.consistency,
    }
}

enum DebitTables {
    DebitTable(DebitTable),
    DebitTableVerbose(DebitTableVerbose)
}

impl DebitTables {
    fn new_debit_table(
        sd: String, 
        x: String, 
        n: u32, 
        consistency: bool
    ) -> Self {
        DebitTables::DebitTable(
            DebitTable::new(sd, x, n, consistency)
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_debit_table_verbose(
        sd: String,
        x: String,
        n: u32,
        consistency: bool,
        rounding: String,
        sd_lower: f64,
        sd_incl_lower: bool,
        sd_incl_upper: bool,
        sd_upper: f64,
        x_lower: String,
        x_incl_lower: bool,
        x_upper: String,
        x_incl_upper: bool,
    ) -> Self {
        DebitTables::DebitTableVerbose(DebitTableVerbose::new(
            sd, 
            x, 
            n, 
            consistency, 
            rounding, 
            sd_lower, 
            sd_incl_lower, 
            sd_incl_upper, 
            sd_upper,
            x_lower, 
            x_incl_lower, 
            x_upper, 
            x_incl_upper,
        ))
    }
}

#[allow(dead_code)]
struct DebitTable {
    sd: String,
    x: String, 
    n: u32, 
    consistency: bool
}

impl DebitTable {
    pub fn new(
        sd: String, 
        x: String, 
        n: u32, 
        consistency: bool
    ) -> Self {
        DebitTable { sd, x, n, consistency }
    }
}

#[allow(dead_code)]
struct DebitTableVerbose {
    sd: String,
    x: String,
    n: u32, 
    consistency: bool, 
    rounding: String, 
    sd_lower: f64, 
    sd_incl_lower: bool, 
    sd_incl_upper: bool, 
    sd_upper: f64, 
    x_lower: String, 
    x_incl_lower: bool, 
    x_upper: String, 
    x_incl_upper: bool
}

impl DebitTableVerbose {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        sd: String,
        x: String,
        n: u32,
        consistency: bool,
        rounding: String,
        sd_lower: f64,
        sd_incl_lower: bool,
        sd_incl_upper: bool,
        sd_upper: f64,
        x_lower: String,
        x_incl_lower: bool,
        x_upper: String,
        x_incl_upper: bool,
    ) -> Self {
        DebitTableVerbose {
            sd,
            x,
            n,
            consistency,
            rounding,
            sd_lower,
            sd_incl_lower,
            sd_incl_upper,
            sd_upper,
            x_lower,
            x_incl_lower,
            x_upper,
            x_incl_upper,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn debit_table(
    x: &str, 
    sd: &str, 
    n: u32, 
    formula: &str, 
    rounding: &str, 
    threshold: f64, 
    symmetric: bool, 
    show_rec: bool
) -> DebitTables {
    //let digits_x = decimal_places_scalar(Some(x), ".");
    let digits_sd = decimal_places_scalar(Some(sd), ".");

    //let x_num: f64 = x.parse().unwrap();
    //let sd_num: f64 = sd.parse().unwrap();

    let x_unrounded = unround(x, rounding, 5.0).unwrap();

    let x_lower = x_unrounded.lower.to_string();
    let x_upper = x_unrounded.upper.to_string();

    let sd_unrounded = unround(sd, rounding, 5.0).unwrap();

    let sd_lower = sd_unrounded.lower;
    let sd_upper = sd_unrounded.upper;

    let sd_rec_lower = reconstruct_sd_scalar(
        formula, 
        x_lower.as_str(), 
        n, 
        0, 
        0
    );
    let sd_rec_upper = reconstruct_sd_scalar(
        formula, 
        x_upper.as_str(), 
        n, 
        0, 
        0
    );
    
    let x_incl_lower = x_unrounded.incl_lower;
    let x_incl_upper = x_unrounded.incl_upper;

    let sd_incl_lower = sd_unrounded.incl_lower;
    let sd_incl_upper = sd_unrounded.incl_upper;
    // right now, this will only support mean reconstruction, not other formulas

    let mut sd_rec_lower = reround(
        vec![sd_rec_lower.unwrap()], 
        digits_sd.unwrap(), 
        rounding, 
        threshold, 
        symmetric
    );

    let mut sd_rec_upper = reround(
        vec![sd_rec_upper.unwrap()], 
        digits_sd.unwrap(), 
        rounding, 
        threshold, 
        symmetric
    );
    
    sd_rec_lower.append(&mut sd_rec_upper);

    let sd_lower_test = dustify(sd_lower);

    let sd_rec_both_test: Vec<f64> = sd_rec_lower.iter().flat_map(
        |x| 
        dustify(*x)
    ).collect();

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

    if show_rec {
        DebitTables::new_debit_table_verbose(
            sd.to_string(), 
            x.to_string(), 
            n, 
            consistency, 
            rounding.to_string(), 
            sd_lower, 
            sd_incl_lower, 
            sd_incl_upper, 
            sd_upper, 
            x_lower, 
            x_incl_lower, 
            x_upper, 
            x_incl_upper
        )
    } else {
        DebitTables::new_debit_table(
            sd.to_string(), 
            x.to_string(), 
            n, 
            consistency
        )
    }
}

#[derive(Debug, Error)]
pub enum RoundingBoundError {
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
) -> Result<(f64, f64, &'static str, &'static str), RoundingBoundError> {
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
            Err(RoundingBoundError::ZeroError)
        }
    } else {
        match rounding {
            "up_or_down" => Ok((x_num - d_var, x_num + d_var, "<=", "<=")),
            "up" => Ok((x_num - d_var, x_num + d_var, "<=", "<")), 
            "down" => Ok((x_num - d_var, x_num + d_var, "<", "<=")), 
            "even" => Ok((x_num - d, x_num + d, "<", "<")),
            "ceiling" => Ok((x_num - (2.0 * d), x_num, "<", "<=")), 
            "floor" => Ok((x_num, x_num + (2.0 * d), "<=", "<")),
            _ => Err(RoundingBoundError::RoundingError)
        }
    }
}

pub fn unround(
    x: &str, 
    rounding: &str,
    threshold: f64
) -> Result<UnroundReturn, RoundingBoundError> {
    let digits = decimal_places_scalar(Some(x), ".");
    let p10: f64 = 10.0f64.powi(digits.unwrap() + 1);
    let d = 5.0 / p10;
    let d_var = threshold / p10;

    let x_num :f64 = x.parse().unwrap();

    let bounds = rounding_bounds(
        rounding, 
        x_num, 
        d_var, 
        d
    ).unwrap();

    let lower = bounds.0;
    let upper = bounds.1;

    let sign_lower = bounds.2;
    let sign_upper = bounds.3;

    Ok(UnroundReturn::new(
        lower, 
        sign_lower == "<=", 
        sign_upper == "<=", 
        upper
    ))
}

pub struct UnroundReturn {
    pub lower: f64, 
    pub incl_lower: bool,
    pub incl_upper: bool,
    pub upper: f64,
}

impl UnroundReturn {
    pub fn new(
        lower: f64, 
        incl_lower: bool, 
        incl_upper: bool, 
        upper: f64
    ) -> Self {
        UnroundReturn {lower, incl_lower, incl_upper, upper}
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn debit_scalar_test_1() {
        assert!(!debit_scalar("0.36", "0.11", 20,  "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_2() {
        assert!(debit_scalar("0.11", "0.31", 40,  "mean_n", "up_or_down", 5.0, false, false))
    } 

    #[test]
    fn debit_scalar_test_3() {
        assert!(!debit_scalar("0.118974", "0.6784", 100, "mean_n", "up_or_down", 5.0, false, false))
    } 

    #[test]
    fn debit_scalar_test_4() {
        assert!(debit_scalar("0.11", "0.31", 40,  "mean_n", "trunc", 5.0, false, false))
    } 

    #[test]
    fn debit_scalar_test_5() {
        assert!(debit_scalar("0.11", "0.31", 40,  "mean_n", "anti_trunc", 5.0, false, false))
    } 
    
    // the below tests come from Scrutiny
    #[test]
    fn debit_scalar_test_6() {
        assert!(debit_scalar("0.53", "0.50", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_7() {
        assert!(debit_scalar("0.44", "0.50", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_8() {
        assert!(debit_scalar("0.77", "0.42", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_9() {
        assert!(!debit_scalar("0.19", "0.35", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_10() {
        assert!(debit_scalar("0.34", "0.47", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_11() {
        assert!(debit_scalar("0.93", "0.25", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_12() {
        assert!(debit_scalar("0.12", "0.33", 1683, "mean_n", "up_or_down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_13() {
        assert!(debit_scalar("0.12", "0.33", 1683, "mean_n", "up_or_down", 5.0, false, true))
    }

    #[test]
    fn debit_scalar_test_14() {
        assert!(!debit_scalar("0.19", "0.35", 1683, "mean_n", "up", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_15() {
        assert!(debit_scalar("0.34", "0.47", 1683, "mean_n", "down", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_16() {
        assert!(debit_scalar("0.93", "0.25", 1683, "mean_n", "even", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_17() {
        assert!(debit_scalar("0.12", "0.33", 1683, "mean_n", "ceiling", 5.0, false, false))
    }

    #[test]
    fn debit_scalar_test_18() {
        assert!(debit_scalar("0.12", "0.33", 1683, "mean_n", "floor", 5.0, false, false))
    }
    
    #[test]
    fn debit_test_1() {
        let xs = ["0.36", "0.11", "0.118974", "0.53","0.44", "0.77", "0.19", "0.34", "0.93", "0.12"];
        let sds = ["0.11", "0.31", "0.6784", "0.50", "0.50", "0.42", "0.35", "0.47", "0.25", "0.33"];
        let ns = vec![20, 40, 100, 1683, 1683, 1683, 1683, 1683, 1683, 1683];
        let formula = "mean_n";
        let rounding = "up_or_down";
        let threshold = 5.0;
        let symmetric = false;
        let show_rec = false;

        let xs_string: Vec<String> = xs.iter().map(|s| s.to_string()).collect();
        let sds_string: Vec<String> = sds.iter().map(|s| s.to_string()).collect();

        let vals: Vec<bool> = debit(xs_string, sds_string, ns, formula, rounding, threshold, symmetric, show_rec).unwrap();
        assert_eq!(vals, vec![false, true, false, true, true, true, false, true, true, true]);

    }

    #[test]
    fn debit_test_2() {
        let xs = ["0.36, 0.11", "0.118974", "0.53","0.44", "0.77", "0.19", "0.34", "0.93", "0.12"]; // not
        // the right length!
        let sds = ["0.11", "0.31", "0.6784", "0.50", "0.50", "0.42", "0.35", "0.47", "0.25", "0.33"];
        let ns = vec![20, 40, 100, 1683, 1683, 1683, 1683, 1683, 1683, 1683];
        let formula = "mean_n";
        let rounding = "up_or_down";
        let threshold = 5.0;
        let symmetric = false;
        let show_rec = false;

        let xs_string: Vec<String> = xs.iter().map(|s| s.to_string()).collect();
        let sds_string: Vec<String> = sds.iter().map(|s| s.to_string()).collect();

        // we extract the error
        debit(xs_string, sds_string, ns, formula, rounding, threshold, symmetric, show_rec).unwrap_err();
    }

    #[test]
    fn debit_rust_test_1() {
        let xs = ["0.36, 0.11", "0.118974", "0.53","0.44", "0.77", "0.19", "0.34", "0.93", "0.12"]; // not
        // the right length!
        let sds = ["0.11", "0.31", "0.6784", "0.50", "0.50", "0.42", "0.35", "0.47", "0.25", "0.33"];
        let ns = vec![20, 40, 100, 1683, 1683, 1683, 1683, 1683, 1683, 1683];
        let formula = "mean_n";
        let rounding = "up_or_down";
        let threshold = 5.0;
        let symmetric = false;
        let show_rec = false;

        let xs_string: Vec<String> = xs.iter().map(|s| s.to_string()).collect();
        let sds_string: Vec<String> = sds.iter().map(|s| s.to_string()).collect();

        let err = debit_rust(xs_string, sds_string, ns, formula, rounding, threshold, symmetric, show_rec).unwrap_err();

        assert_eq!(err, DebitError::LengthError(9, 10, 10));
    }
}

