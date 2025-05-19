use crate::grim_map_df::ColumnInput;
use core::f64;
use pyo3::types::PyAnyMethods;
use pyo3::{pyfunction, PyResult, Python, PyAny};
use pyo3::types::PyString;
use pyo3_polars::PyDataFrame;
use pyo3::prelude::*;
use pyo3::exceptions::PyImportError;
use crate::debit_map_df::debit_map_pl;

/// Computes consistency using the Descriptive Binary test (DEBIT) for means and standard
/// standard deviations
///
/// This function leverages the `polars` library to process a pandas DataFrame 
/// and compute consistency based on the provided formula and rounding methods.
/// It returns a tuple containing a list of boolean values indicating 
/// consistency and an optional list of indices where errors occurred.
///
/// Parameters:
/// - `pandas_df` (DataFrame): The input pandas DataFrame to be processed.
/// - `x_col` (ColumnInput): The column index or name for the x values. Defaults
/// to the first column.
/// - `sd_col` (ColumnInput): The column index or name for the standard 
/// deviation values. Defaults to the second column.
/// - `n_col` (ColumnInput): The column index or name for the sample sizes. 
/// Defaults to the third column.
/// - `formula` (str): The formula to use for computation. Defaults to "mean_n".
/// - `rounding` (str): The rounding method to apply. Defaults to "up_or_down".
/// - `threshold` (float): The threshold value for rounding. Defaults to 5.0.
/// - `show_rec` (bool): Whether to show reconstructed values. Defaults to False.
/// - `symmetric` (bool): Whether the rounding should be symmetric. Defaults to False.
/// - `silence_default_warning` (bool): Suppress warnings about default column 
/// usage. Defaults to False.
/// - `silence_numeric_warning` (bool): Suppress warnings about numeric issues. 
/// Defaults to False.
///
/// Returns:
/// - tuple: A tuple containing:
///     - List[bool]: A list indicating the consistency of each row.
///     - Optional[List[int]]: An optional list of indices where inconsistencies
///     were found.
///
/// Raises:
/// - ImportError: If the `polars` package is not installed.
///
/// Warnings:
/// - A warning is issued if the default column indices are used without 
/// modification, unless `silence_default_warning` is set to True.
/// - A warning is issued if the xs and sds columns are not in string form 
/// in order to preserve floating zeros, unless `silence_numerical_warning`
/// is set to True
///
/// Example:
/// ```python
/// import pandas as pd
/// from scrutipy import debit_map
///
/// df = pd.DataFrame({
///     'x': [0.36, 0.11],
///     'sd': [0.11, 0.31],
///     'n': [20, 40]
/// })
///
/// result, errors = debit_map(df)
/// print(result)  # Output: [True, False]
/// print(errors)  # Output: None (as no errors occurred)
#[cfg(not(tarpaulin_include))]
#[allow(clippy::too_many_arguments)]
#[pyfunction(signature = (
     pandas_df, 
     x_col=ColumnInput::Default(0), 
     sd_col=ColumnInput::Default(1),
     n_col=ColumnInput::Default(2), 
     formula = "mean_n".to_string(),
     rounding = "up_or_down".to_string(), 
     threshold = 5.0, 
     show_rec = false,
     symmetric = false,
     silence_default_warning = false,
     silence_numeric_warning = false,
))]
pub fn debit_map<'py>(
    py: Python<'py>,
    pandas_df: Bound<'py, PyAny>,
    x_col: ColumnInput,
    sd_col: ColumnInput,
    n_col: ColumnInput,
    formula: String,
    rounding: String,
    threshold: f64,
    show_rec: bool,
    symmetric: bool,
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
    if (x_col == ColumnInput::Default(0)) & (sd_col == ColumnInput::Default(1)) & (n_col == ColumnInput::Default(2)) & !silence_default_warning {
        warnings.call_method1(
            "warn",
            (PyString::new(py, "The columns `x_col` and `n_col` haven't been changed from their defaults. \n Please ensure that the first and second columns contain the xs and ns respectively. \n To silence this warning, set `silence_default_warning = True`."),),
        ).unwrap();
    };

    let pl_df_obj = polars
        .getattr("DataFrame")?
        .call1((pandas_df,))?; // This works if pandas_df is convertible
 
    let pydf: PyDataFrame = pl_df_obj.extract()?;

    debit_map_pl(
        py,
        pydf,
        x_col,
        sd_col,
        n_col,
        show_rec, 
        symmetric,
        formula,
        rounding,
        threshold,
        silence_default_warning,
        silence_numeric_warning,
    )
}
