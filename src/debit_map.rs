use crate::grim_map_df::ColumnInput;
use core::f64;
use pyo3::types::PyAnyMethods;
use pyo3::{pyfunction, PyResult, Python, PyAny};
use pyo3_polars::PyDataFrame;
use pyo3::prelude::*;
use pyo3::exceptions::PyImportError;
use crate::debit_map_df::debit_map_pl;

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

    //let warnings = py.import("warnings").unwrap();
    //if (x_col == ColumnInput::Default(0)) & (n_col == ColumnInput::Default(1)) & !silence_default_warning {
    //    warnings.call_method1(
    //        "warn",
    //        (PyString::new(py, "The columns `x_col` and `n_col` haven't been changed from their defaults. \n Please ensure that the first and second columns contain the xs and ns respectively. \n To silence this warning, set `silence_default_warning = True`."),),
    //    ).unwrap();
    //};

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
