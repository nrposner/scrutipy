use core::f64;
use polars::series::Series;
use polars::datatypes::AnyValue;
use polars::{frame::DataFrame, prelude::DataType};
use pyo3::exceptions::{PyTypeError, PyValueError, PyIndexError};
use pyo3::types::{PyAnyMethods, PyString};
use pyo3::{pyfunction, PyErr, PyResult, Python};
use pyo3_polars::PyDataFrame;
use thiserror::Error;
use crate::debit::debit;
use crate::grim_map_df::{ColumnInput, coerce_string_to_u32, coerce_to_u32, NsParsingError};

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
                "The {} column named '{}', not found in the provided dataframe. Available columns: {:?}",
                err_name, name, cols,
            ))
        }
        DataFrameParseError::TypeError(name) => {
            PyTypeError::new_err(format!(
                "The {} column '{}' could not be interpreted as a Series",
                err_name, name,
            ))
        }
        DataFrameParseError::IndexError(ind, total) => {
            PyIndexError::new_err(format!(
                "The {} column_index '{}' is out of bounds for the provided dataframe, which has {} columns",
                err_name, ind, total,
            ))
        }
        DataFrameParseError::TypeIndexError(ind) => {
            PyTypeError::new_err(format!(
                "The {} column at index {} could not be interpreted as a Series",
                err_name, ind,
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
) -> PyResult<(Vec<bool>, Option<Vec<usize>>)>
{
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


    let xs_result = match xs.dtype() {
        DataType::String => Ok(
            xs.str().unwrap()
                .into_iter()
                .map(|opt| opt.unwrap_or("").to_string())
                .collect::<Vec<String>>()
        ),
        DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Float32
            | DataType::Float64 => Ok({
            if !silence_numeric_warning {
                warnings.call_method1(
                    "warn", 
                    (PyString::new(py, "The column `x_col` is made up of numeric types instead of strings. \n Understand that you may be losing trailing zeros by using a purely numeric type. \n To silence this warning, set `silence_numeric_warning = True`."),),
                ).unwrap();
            }
            xs.iter().map(|x| x.to_string()).collect::<Vec<String>>()}),
        _ => Err("Input xs column is neither a String nor numeric type"),
    };

    // if the data type of xs is neither a string nor a numeric type which we could plausibly
    // convert into a string (albeit while possibly losing some trailing zeros) we return early
    // with an error, as there's nowhere for the program to progress from here. 
    let xs_vec = match xs_result {
        Ok(xs) => xs,
        Err(_) => return Err(PyTypeError::new_err("The x_col column is composed of neither strings nor numeric types. Please check the input types and the documentation.")),
    };

    let sds_result = match sds.dtype() {
        DataType::String => Ok(
            sds.str().unwrap()
                .into_iter()
                .map(|opt| opt.unwrap_or("").to_string())
                .collect::<Vec<String>>()
        ),
        DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Float32
            | DataType::Float64 => Ok({
            if !silence_numeric_warning {
                warnings.call_method1(
                    "warn", 
                    (PyString::new(py, "The column `sd_col` is made up of numeric types instead of strings. \n Understand that you may be losing trailing zeros by using a purely numeric type. \n To silence this warning, set `silence_numeric_warning = True`."),),
                ).unwrap();
            }
            sds.iter().map(|sd| sd.to_string()).collect::<Vec<String>>()}),
        _ => Err("Input sds column is neither a String nor numeric type"),
    };

    // if the data type of sds is neither a string nor a numeric type which we could plausibly
    // convert into a string (albeit while possibly losing some trailing zeros) we return early
    // with an error, as there's nowhere for the program to progress from here. 
    let sds_vec = match sds_result {
        Ok(sds) => sds,
        Err(_) => return Err(PyTypeError::new_err("The sd_col column is composed of neither strings nor numeric types. Please check the input types and the documentation.")),
    };

    let ns_result = match ns.dtype() {
        DataType::String => Ok(coerce_string_to_u32(ns.clone())),
        DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64 
        | DataType::Float32
        | DataType::Float64 => Ok({
            ns.iter()
                .map(|val| match val {
                    AnyValue::UInt8(n) => coerce_to_u32(n),
                    AnyValue::UInt16(n) => coerce_to_u32(n),
                    AnyValue::UInt32(n) => coerce_to_u32(n),
                    AnyValue::UInt64(n) => coerce_to_u32(n),
                    AnyValue::Int8(n) => coerce_to_u32(n),
                    AnyValue::Int16(n) => coerce_to_u32(n),
                    AnyValue::Int32(n) => coerce_to_u32(n),
                    AnyValue::Int64(n) => coerce_to_u32(n),
                    AnyValue::Float32(f) => coerce_to_u32(f),
                    AnyValue::Float64(f) => coerce_to_u32(f),
                    _ => Err(NsParsingError::NotAnInteger(val.to_string().parse().unwrap_or(f64::NAN))),
                })
                .collect::<Vec<Result<u32, NsParsingError>>>()
            }),
            _ => Err(NsParsingError::NotNumeric),

    };

    // if the ns column is made up of neither strings nor any plausible numeric type, we return
    // early with an error. There is nowhere for the program to progress from here. 
    let ns_vec = match ns_result {
        Err(_) => return Err(PyTypeError::new_err("The n_col column is composed of neither strings nor numeric types. Please check the input types and the documentation.")),
        Ok(vs) => vs,
    };

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
            //sds.push(sds_result.parse::<f64>()?);
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
