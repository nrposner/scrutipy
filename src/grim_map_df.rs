use core::f64;
use polars::series::Series;
use polars::datatypes::AnyValue;
use polars::{frame::DataFrame, prelude::DataType};
use pyo3::exceptions::{PyTypeError, PyValueError, PyIndexError};
use pyo3::types::{PyAnyMethods, PyString};
use pyo3::{pyfunction, FromPyObject, PyResult, Python};
use pyo3_polars::PyDataFrame;
use num::NumCast;
use thiserror::Error;
use crate::grim::grim_rust;

/// Implements grim_map over the columns of a Python dataframe. 
///
/// Takes the provided dataframe as well as inputs indicating the columns to be used as xs and ns.
/// If one or more columns are not indicated, it will take the first column as xs and the second
/// column as ns by default. All other grim_map arguments can be provided as keyword arguments.
/// default respectively. 
#[allow(clippy::too_many_arguments)]
#[cfg(not(tarpaulin_include))] // since this function is only meant to be called from Python and
// requires certain PyO3 types which are tedious to recreate within Rust, I find it acceptable to
// exclude this function from internal Rust testing in exchange for rigorous testing on the Python
// end, reflecting actual user experience 
#[pyfunction(signature = (
    pydf, 
    x_col=ColumnInput::Index(0), 
    n_col=ColumnInput::Index(1), 
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
pub fn grim_map_pl(
    py: Python, 
    pydf: PyDataFrame, 
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
) -> PyResult<(Vec<bool>, Option<Vec<usize>>)>
{
    let df: DataFrame = pydf.into();
    //let rounds: Vec<&str> = rounding.iter().map(|s| &**s).collect(); 

    let warnings = py.import("warnings").unwrap();
    if (x_col == ColumnInput::Default(0)) & (n_col == ColumnInput::Default(1)) & !silence_default_warning {
        warnings.call_method1(
            "warn",
            (PyString::new(py, "The columns `x_col` and `n_col` haven't been changed from their defaults. \n Please ensure that the first and second columns contain the xs and ns respectively. \n To silence this warning, set `silence_default_warning = True`."),),
        ).unwrap();
    };

    let xs: &Series = match x_col {
        ColumnInput::Name(name) => df.column(&name).map_err(|_| PyValueError::new_err(format!(
            "The x_col column named '{}' not found in the provided dataframe. Available columns: {:?}",
            name,
            df.get_column_names()
        )))?
            .as_series()
            .ok_or_else(|| PyTypeError::new_err(format!("The column '{}' could not be interpreted as a Series", name)))?,

        ColumnInput::Index(ind) | ColumnInput::Default(ind) => df.get_columns().get(ind).ok_or_else(|| PyIndexError::new_err(format!(
            "The x_col column index '{}' is out of bounds for the provided dataframe, which has {} columns",
            ind,
            df.width()
        )))?
            .as_series()
            .ok_or_else(|| PyTypeError::new_err("Column could not be interpreted as a Series"))?,
    };
    if xs.len() == 0 {
        return Err(PyTypeError::new_err("The x_col column is empty."));
    }

    let ns: &Series = match n_col {
        ColumnInput::Name(name) => df.column(&name).map_err(|_| PyValueError::new_err(format!(
            "The n_col column named '{}' not found in the provided dataframe. Available columns: {:?}", 
            name, 
            df.get_column_names()
        )))?
            .as_series()
            .ok_or_else(|| PyTypeError::new_err(format!("The column '{}' could not be interpreted as a Series", name)))?,

        ColumnInput::Index(ind) | ColumnInput::Default(ind) => df.get_columns().get(ind).ok_or_else(|| PyIndexError::new_err(format!(
            "The n_col column index '{}' is out of bounds for the provided dataframe, which has {} columns", 
            ind, 
            df.width()
        )))?
            .as_series()
            .ok_or_else(|| PyTypeError::new_err("Column could not be interpreted as a Series"))?,
    };

    if ns.len() == 0 {
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

    let mut xs: Vec<&str> = Vec::new();
    let mut ns: Vec<u32> = Vec::new();
    let mut ns_err_inds: Vec<usize> = Vec::new();

    for (i, (n_result, x)) in ns_vec.iter().zip(xs_temp.iter()).enumerate() {

        if let Ok(u) = n_result {
            ns.push(*u);
            xs.push(*x);
        } else {
            ns_err_inds.push(i)
        };
    }

    // since we can't set a default for items which is dependent on the size of another variable
    // known at runtime, we wait until now to turn the default option into a vector of 1s the same
    // length as the number of valid counts 
    let revised_items = match items {
        None => vec![1; xs.len()],
        Some(i) => i,
    };

    let res = grim_rust(xs, ns.clone(), vec![percent, show_rec, symmetric], revised_items, rounding.as_str(), threshold, tolerance);

    // if the length of ns_err_inds is 0, ie if no errors occurred, our error return is Option<None>.
    // Otherwise, our error return is Option<ns_err_inds>
    let err_output: Option<Vec<usize>> = match ns_err_inds.len() {
        0 => None,
        _ => Some(ns_err_inds),
    };

    Ok((res, err_output)) 
}

/// Converts x_col and n_col inputs to either usize or String in order to attempt column extraction
#[derive(FromPyObject, PartialEq)]
pub enum ColumnInput {
    Index(usize), // first check to see whether we can coerce this into a usize 
    Name(String), // otherwise keep as string 
    Default(usize), // only accessible directly in the code, for the purposes of determining
    // whether default options have been changed
}

#[derive(Debug, Error, PartialEq)]
pub enum NsParsingError {
    #[error("Value {0} is not numeric")]
    NotNumeric(String),
    #[error("Value {0} is not an integer")]
    NotAnInteger(f64), // float with decimal part
    #[error("Value {0} is negative or 0")]
    NotPositive(i128), // negative or zero integer
    #[error("Value {0} is too large")]
    TooLarge(u128),    // doesn't fit in u32
}

fn coerce_string_to_u32(s: Series) -> Vec<Result<u32, NsParsingError>>{
    s.iter()
    .map(|val| {
        let s = val.to_string();
        s.parse::<u32>()
            .map_err(|_| NsParsingError::NotNumeric(s))
    })
    .collect::<Vec<Result<u32, NsParsingError>>>()
}

fn coerce_to_u32<T: Copy + NumCast + std::fmt::Debug>(value: T) -> Result<u32, NsParsingError> {
    let float: f64 = NumCast::from(value).ok_or(NsParsingError::NotAnInteger(0.0))?;

    if !float.is_finite() {
        return Err(NsParsingError::NotAnInteger(float));
    }

    if float.fract() != 0.0 {
        return Err(NsParsingError::NotAnInteger(float));
    }

    if float < 0.0 {
        return Err(NsParsingError::NotPositive(float as i128));
    }

    if float > u32::MAX as f64 {
        return Err(NsParsingError::TooLarge(float as u128));
    }

    Ok(float as u32)
}

