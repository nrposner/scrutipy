 use crate::grim_map_df::{grim_map_df, ColumnInput};
 use core::f64;
 use pyo3::types::PyAnyMethods;
 use pyo3::{pyfunction, PyResult, Python, PyAny};
 use pyo3_polars::PyDataFrame;
 use pyo3::prelude::*;
 
 /// Transforms a pandas dataframe to polars and runs grim_map_df 
 #[pyfunction(signature = (
     pandas_df, 
     x_col=ColumnInput::Index(0), 
     n_col=ColumnInput::Index(1), 
     percent = false,
     show_rec = false,
     symmetric = false,
     items = None, 
     rounding = vec!["up_or_down".to_string()], 
     threshold = 5.0, 
     tolerance = f64::EPSILON.powf(0.5),
     silence_default_warning = false,
     silence_numeric_warning = false,
 ))]
 #[allow(clippy::too_many_arguments)]
 #[allow(dead_code)]
 pub fn grim_map<'py>(
     py: Python<'py>,
     pandas_df: Bound<'py, PyAny>,
     x_col: ColumnInput,
     n_col: ColumnInput,
     percent: bool,
     show_rec: bool,
     symmetric: bool,
     items: Option<Vec<u32>>,
     rounding: Vec<String>,
     threshold: f64,
     tolerance: f64,
     silence_default_warning: bool,
     silence_numeric_warning: bool,
 ) -> PyResult<(Vec<bool>, Option<Vec<usize>>)> {
     let polars = py.import("polars")?;
     let pl_df_obj = polars
         .getattr("DataFrame")?
         .call1((pandas_df,))?; // This works if pandas_df is convertible
 
     let pydf: PyDataFrame = pl_df_obj.extract()?;
 
     grim_map_df(
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
