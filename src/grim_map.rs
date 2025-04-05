use crate::grim_map_df::{grim_map_pl, ColumnInput};
use core::f64;
use pyo3::types::PyAnyMethods;
use pyo3::{pyfunction, PyResult, Python, PyAny};
use pyo3_polars::PyDataFrame;
use pyo3::prelude::*;
use pyo3::exceptions::PyImportError;
use pyo3::types::PyString;
 
/// Run a GRIM consistency check on a pandas DataFrame.
///
/// Parameters
/// ----------
/// pandas_df : A pandas DataFrame containing at least two columns: one for reported means (`x_col`) and one for corresponding sample sizes (`n_col`).
/// x_col : The column containing reported means (index or name). Defaults to column 0.
/// n_col : The column containing sample sizes (index or name). Defaults to column 1.
/// percent : If `True`, values in `x_col` are interpreted as percentages (e.g., 25.3% instead of 0.253).
/// show_rec : ![not fully implemented!] If `True`, returns more verbose recommendation.
/// symmetric : If `True`, uses symmetric rounding when validating consistency.
/// items : Optional list of item counts. If not provided, defaults to all 1s.
/// rounding : A list of rounding strategies. Defaults to `["up_or_down"]`.
/// threshold : Threshold for rounding tolerance. Defaults to 5.0).
/// tolerance : Numerical epsilon used in float comparisons. Defaults to square root of 64-bit floating point machine epsilon
/// silence_default_warning : Suppresses warning about default column selection.
/// silence_numeric_warning : Suppresses warning about using numeric types in `x_col`.
/// 
/// Returns
/// ----------
/// tuple
///     (List of booleans indicating GRIM validity, Optional list of error row indices)
/// 
/// Example
/// ----------
/// >>> df = pd.DataFrame({"x": ["5.0", "5.27"], "n": [20, 20]})
/// >>> grim_map(df)
/// ([True, False], None
#[pyfunction(signature = (
     pandas_df, 
     x_col=ColumnInput::Default(0), 
     n_col=ColumnInput::Default(1), 
     percent = false,
     show_rec = false,
     symmetric = false,
     items = None, 
     rounding = "up_or_down".to_string(), 
     threshold = 5.0, 
     tolerance = f64::EPSILON.powf(0.5),
     silence_default_warning = false,
     silence_numeric_warning = false,
 ))]
#[allow(clippy::too_many_arguments)]
/// Runs a GRIM consistency check across a pandas DataFrame.
///
/// This function converts a pandas DataFrame to a Polars DataFrame under the hood,
/// then calls the core GRIM checking logic implemented in Rust.
///
/// GRIM (Granularity-Related Inconsistency of Means) evaluates whether reported
/// means are consistent with whole-number item counts, assuming uniform item structure.
///
/// # Parameters
/// - `pandas_df`: A pandas DataFrame containing at least two columns: one for reported means (`x_col`)
///   and one for corresponding sample sizes (`n_col`).
/// - `x_col`: The column containing reported means (index or name). Defaults to column 0.
/// - `n_col`: The column containing sample sizes (index or name). Defaults to column 1.
/// - `percent`: If `True`, values in `x_col` are interpreted as percentages (e.g., 25.3% instead of 0.253).
/// - `show_rec`: ![not fully implemented!] If `True`, returns more verbose recommendation.
/// - `symmetric`: If `True`, uses symmetric rounding when validating consistency.
/// - `items`: Optional list of item counts. If not provided, defaults to all 1s.
/// - `rounding`: A list of rounding strategies. Defaults to `["up_or_down"]`.
/// - `threshold`: Threshold for rounding tolerance. Defaults to 5.0).
/// - `tolerance`: Numerical epsilon used in float comparisons. Defaults to square root of 64-bit
/// floating point machine epsilon
/// - `silence_default_warning`: Suppresses warning about default column selection.
/// - `silence_numeric_warning`: Suppresses warning about using numeric types in `x_col`.
///
/// # Returns
/// A tuple of:
/// - `List[bool]`: Which rows pass or fail the GRIM test
/// - `Optional[List[int]]`: Indices of rows that failed to parse correctly, if any
///
/// # Example
/// ```python
/// df = pd.DataFrame({"x": ["5.0", "5.27"], "n": [20, 20]})
/// bools, errors = grim_map(df)
/// # ([True, False], None)
/// ```
pub fn grim_map<'py>(
     py: Python<'py>,
     pandas_df: Bound<'py, PyAny>,
     x_col: ColumnInput,
     n_col: ColumnInput,
     percent: bool,
     show_rec: bool,
     symmetric: bool,
     items: Option<Vec<u32>>,
     rounding: String,
     threshold: f64,
     tolerance: f64,
     silence_default_warning: bool,
     silence_numeric_warning: bool,
) -> PyResult<(Vec<bool>, Option<Vec<usize>>)> {
     let polars = py.import("polars").map_err(|_| {
        PyImportError::new_err(
            "The 'polars' package is required for this function but is not installed.\n\
                 You can install it with: pip install grim[polars]"
        )
    })?;

    let warnings = py.import("warnings").unwrap();
    if (x_col == ColumnInput::Default(0)) & (n_col == ColumnInput::Default(1)) & !silence_default_warning {
        warnings.call_method1(
            "warn",
            (PyString::new(py, "The columns `x_col` and `n_col` haven't been changed from their defaults. \n Please ensure that the first and second columns contain the xs and ns respectively. \n To silence this warning, set `silence_default_warning = True`."),),
        ).unwrap();
    };
     
    let pl_df_obj = polars
         .getattr("DataFrame")?
         .call1((pandas_df,))?; // This works if pandas_df is convertible
 
    let pydf: PyDataFrame = pl_df_obj.extract()?;
 
    grim_map_pl(
         py,
         pydf,
         x_col,
         n_col,
         percent, 
         show_rec, 
         symmetric,
         items,
         rounding,
         threshold,
         tolerance,
         silence_default_warning,
         silence_numeric_warning,
    )
}
