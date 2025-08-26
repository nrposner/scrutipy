use core::f64;
use polars::{frame::DataFrame, series::Series};
use pyo3::{pyfunction, PyErr, PyResult, Python, 
    exceptions::{PyIndexError, PyTypeError, PyValueError}, 
    types::{PyAnyMethods, PyString}};
use pyo3_polars::PyDataFrame;
use thiserror::Error;
use crate::debit::debit;
use crate::grim_map_df::ColumnInput;
use crate::utils::{InputType, process_series_to_string, process_series_to_num};

#[derive(Debug, Error)]
pub enum DataFrameParseError {
    #[error("The column named '{0}' not found in the provided dataframe. Available columns: {1:?}")]
    ValueError(String, Vec<String>),    
    #[error("The column '{0}' could not be interpreted as a Series")]
    TypeError(String),
    #[error("The column at index {0} could not be interpreted as a Series")]
    TypeIndexError(usize),
    #[error("the column index '{0}' is out of bounds for the provided dataframe, which has {1} columns")]
    IndexError(usize, usize),
}

fn parse_col(df: &DataFrame, col: ColumnInput) -> Result<Series, DataFrameParseError>{
    let xs: &Series = match col {
        ColumnInput::Name(name) => df.column(&name).map_err(
            |_| DataFrameParseError::ValueError(
                name.clone(), 
                df.get_column_names().iter().map(|s| s.to_string()).collect()
            ))?.as_series()
            .ok_or(DataFrameParseError::TypeError(name))?,

        ColumnInput::Index(ind) | ColumnInput::Default(ind) => df.get_columns().get(ind).ok_or(
            DataFrameParseError::IndexError(ind, df.width())
        )?
            .as_series()
            .ok_or(DataFrameParseError::TypeIndexError(ind))?,
    };
    Ok(xs.clone())
}

fn parse_col_errors(df: &DataFrame, n_col: ColumnInput, err_name: String) -> Result<Series, PyErr> {
    parse_col(df, n_col)
    .map_err(|e| match e {
        DataFrameParseError::ValueError(name, cols) => {
            PyValueError::new_err(format!(
                "The {err_name} column named '{name}', not found in the provided dataframe. Available columns: {cols:?}",
            ))
        }
        DataFrameParseError::TypeError(name) => {
            PyTypeError::new_err(format!(
                "The {err_name} column '{name}' could not be interpreted as a Series",
            ))
        }
        DataFrameParseError::IndexError(ind, total) => {
            PyIndexError::new_err(format!(
                "The {err_name} column_index '{ind}' is out of bounds for the provided dataframe, which has {total} columns",
            ))
        }
        DataFrameParseError::TypeIndexError(ind) => {
            PyTypeError::new_err(format!(
                "The {err_name} column at index {ind} could not be interpreted as a Series",
            ))
        }
    })
}



#[allow(clippy::too_many_arguments)]
#[pyfunction(signature = (
    pydf, x_col = ColumnInput::Default(0), sd_col = ColumnInput::Default(1), n_col = ColumnInput::Default(2), show_rec = false, symmetric = false, formula = "mean_n".to_string(), rounding = "up_or_down".to_string(), threshold = 5.0, silence_default_warning = false, silence_numeric_warning = false
))]
#[cfg(not(tarpaulin_include))]
pub fn debit_map_pl(
    py: Python, 
    pydf: PyDataFrame, 
    x_col: ColumnInput, 
    sd_col: ColumnInput,
    n_col: ColumnInput, 
    show_rec: bool,
    symmetric: bool,
    formula: String,
    rounding: String, 
    threshold: f64, 
    silence_default_warning: bool,
    silence_numeric_warning: bool,
) -> PyResult<(Vec<bool>, Option<Vec<usize>>)> {
    let df: DataFrame = pydf.into();

    let warnings = py.import("warnings").unwrap();
    if (x_col == ColumnInput::Default(0)) & (sd_col == ColumnInput::Default(1)) & (n_col == ColumnInput::Default(2)) & !silence_default_warning {
        warnings.call_method1(
            "warn",
            (PyString::new(py, "The columns `x_col`, `sd_col`, and `n_col` haven't been changed from their defaults. \n Please ensure that the first and second columns contain the xs and ns respectively. \n To silence this warning, set `silence_default_warning = True`."),),
        ).unwrap();
    };

    let xs = parse_col_errors(&df, x_col, "x_col".to_string())?;

    if xs.is_empty() {
        return Err(PyTypeError::new_err("The x_col column is empty."));
    }

    let sds = parse_col_errors(&df, sd_col, "sd_col".to_string())?;

    if sds.is_empty() {
        return Err(PyTypeError::new_err("The sd_col column is empty"));
    }

    let ns = parse_col_errors(&df, n_col, "n_col".to_string())?;

    if ns.is_empty() {
        return Err(PyTypeError::new_err("The n_col column is empty."));
    }

    let xs_vec = process_series_to_string(py, xs, silence_numeric_warning, InputType::Xs)?;
    let sds_vec = process_series_to_string(py, sds, silence_numeric_warning, InputType::Sds)?;
    let ns_vec = process_series_to_num(ns)?;


    let xs_temp: Vec<&str> = xs_vec.iter().map(|s| &**s).collect();

    let mut xs: Vec<String> = Vec::new();
    let mut sds: Vec<String> = Vec::new();
    let mut ns: Vec<u32> = Vec::new();
    let mut err_inds: Vec<usize> = Vec::new();

    for (i, ((n_result, sds_result), x)) in ns_vec.iter().zip(sds_vec.iter()).zip(xs_temp.iter()).enumerate() {
        if let Ok(u) = n_result {
            ns.push(*u);
            xs.push(x.to_string());
            sds.push(sds_result.to_string())
        } else {
            err_inds.push(i);
        }
    }

    let res = debit(xs, sds, ns, formula.as_str(), rounding.as_str(), threshold, symmetric, show_rec)?;

    // if the length of err_inds is 0, ie if no errors occurred, our error return is Option<None>.
    // Otherwise, our error return is Option<ns_err_inds>
    let err_output: Option<Vec<usize>> = match err_inds.len() {
        0 => None,
        _ => Some(err_inds),
    };

    Ok((res, err_output)) 
}
