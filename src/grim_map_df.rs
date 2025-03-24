use core::f64;
use polars::series::Series;
use polars::datatypes::AnyValue;
use polars::{frame::DataFrame, prelude::DataType};
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyAnyMethods, PyString};
use pyo3::{pyfunction, FromPyObject, PyResult, Python};
use pyo3_polars::PyDataFrame;
use num::NumCast;
use thiserror::Error;
use crate::grim::grim_rust;

/// We want to give grim_map the ability to operate on dataframes passed in from python
/// Let's pseudocode out what we want to do 
///
/// The user calls grim_map from python. They may input some iterable, like a series, list, array,
/// what have you, and if they do, we want to use the grim_map functionality we've already defined
/// But if they put in a dataframe, we want to do something quite different
/// First, inside the dataframe, we distinguish that they've done this, and prepare to call a
/// different set of rust functions 
/// We'll demand that they input certain keyword arguments which are optional (in fact, useless) if
/// the function supplies the iterables directly 
/// These optional kwargs supply the names or indices of the columns that we want to use for the
/// analysis, with one being the xs and the other the sds, 
/// Do we perhaps need a third, super-duper special double-optional column of item numbers?
/// Probably not. 
///
/// Having been supplied either the names or indices of the columns (can we mix and match? I don't
/// see why not) we need to:
///     - check to make sure these aren't the same column. That seems like an easy way to shoot
///     yourself in the foot with this, especially if you are mixing and matching names and indices
///     for some godsforsaken reason, and we want to cut off that possibility at the outset
///     We'll give the user an error, and tell them that if they've thought about it long and hard
///     and eaten their vegetables, they acn urn on yet another optional kwarg which silences this
///     message. At that point, any mistake they make is their own fool fault.
///     - access these two columns and confirm that their types are compatible with our needs.
///     These can either strings or numerics, but it must be possible to convert them into
///     integers or floats, as needed. How do we check this properly? We can try to guarantee it
///     for our other needs, but at some point, we will need to simply go through every value and
///     see whether it parses into a numeric type. Otherwise, we would need to allow for any given
///     record to simply not be accepted, and alert the user to that fact. 
/// The grim_map_df can take either column indices or column names as inputs when selecting the x
/// and n columns. This enum allows for 
#[derive(FromPyObject, PartialEq)]
pub enum ColumnInput {
    Index(usize),
    Name(String),
    Default(usize),
}

/// Implements grim_map over the columns of a Python dataframe. 
///
/// Takes the provided dataframe as well as inputs indicating the columns to be used as xs and ns.
/// If one or more columns are not indicated, it will take the first column as xs and the second
/// column as ns by default. All other grim_map arguments can be provided as keyword arguments.
/// default respectively. 
#[allow(clippy::too_many_arguments)]
#[pyfunction(signature = (
    pydf, 
    x_col=ColumnInput::Index(0), 
    n_col=ColumnInput::Index(1), 
    bool_params = vec![false, false, false], 
    items = None, 
    rounding = vec!["up_or_down".to_string()], 
    threshold = 5.0, 
    tolerance = f64::EPSILON.powf(0.5),
    silence_default_warning = false,
    silence_numeric_warning = false,
))]
pub fn grim_map_df(
    py: Python, 
    pydf: PyDataFrame, 
    x_col: ColumnInput, 
    n_col: ColumnInput, 
    bool_params: Vec<bool>, // contains percent, show_rec, symmetric
    items: Option<Vec<u32>>, 
    rounding: Vec<String>, 
    threshold: f64, 
    tolerance: f64,
    silence_default_warning: bool,
    silence_numeric_warning: bool,
) -> PyResult<(Vec<bool>, Option<Vec<usize>>)>
{
    let df: DataFrame = pydf.into();
    let rounds: Vec<&str> = rounding.iter().map(|s| &**s).collect(); 

    let warnings = py.import("warnings").unwrap();
    if (x_col == ColumnInput::Index(0)) & (n_col == ColumnInput::Index(1)) & !silence_default_warning {
        warnings.call_method1(
            "warn",
            (PyString::new(py, "The columns `x_col` and `n_col` haven't been changed from their defaults. \n Please ensure that the first and second columns contain the xs and ns respectively. \n To silence this warning, set `silence_default_warning = True`."),),
        ).unwrap();
    };

    let xs: &Series = match x_col {
        ColumnInput::Name(name) => df.column(&name).unwrap().as_series().unwrap(),
        ColumnInput::Index(ind) => df.get_columns()[ind].as_series().unwrap(), 
        ColumnInput::Default(ind) => df.get_columns()[ind].as_series().unwrap(),
    };

    let ns: &Series= match n_col {
        ColumnInput::Name(name) => df.column(&name).unwrap().as_series().unwrap(),
        ColumnInput::Index(ind) => df.get_columns()[ind].as_series().unwrap(),
        ColumnInput::Default(ind) => df.get_columns()[ind].as_series().unwrap(),
    };

    let xs_result = match xs.dtype() {
        DataType::String => Ok(xs.iter().map(|x| x.to_string()).collect::<Vec<String>>()),
        DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::Float32
            | DataType::Float64 => Ok(xs.iter().map(|x| x.to_string()).collect::<Vec<String>>()),
        _ => Err("Input xs column is neither a string nor numeric type"),
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
        | DataType::Int64 => Ok({
            if !silence_numeric_warning {
                warnings.call_method1(
                    "warn", 
                    (PyString::new(py, "The column `x_col` is made up of numeric types instead of strings. \n Understand that you may be losing trailing zeros by using a purely numeric type. \n To silence this warning, set `silence_numeric_warning = True`."),),
                ).unwrap();
            }
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
        //match n_result {
        //    Ok(u) => {
        //        ns.push(*u);
        //        xs.push(*x)
        //    },
        //    Err(_) => ns_err_inds.push(i)
        //}
    }

    // since we can't set a default for items which is dependent on the size of another variable
    // known at runtime, we wait until now to turn the default option into a vector of 1s the same
    // length as the number of valid counts 
    let revised_items = match items {
        None => vec![1; xs.len()],
        Some(i) => i,
    };

    let res = grim_rust(xs, ns.clone(), bool_params, revised_items, rounds, threshold, tolerance);

    // if the length of ns_err_inds is 0, ie if no errors occurred, our error return is Option<None>.
    // Otherwise, our error return is Option<ns_err_inds>
    let err_output: Option<Vec<usize>> = match ns_err_inds.len() {
        0 => None,
        _ => Some(ns_err_inds),
    };

    Ok((res, err_output)) 
}

#[derive(Debug, Error)]
pub enum GrimMapDfError {
    #[error("bla bla")]
    BlaBla(),
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

fn coerce_to_u32<T: Copy + NumCast + PartialOrd + std::fmt::Debug>(value: T) -> Result<u32, NsParsingError> {
    if let Some(f) = NumCast::from(value) {
        let float: f64 = f;
        if float.fract() != 0.0 {
            return Err(NsParsingError::NotAnInteger(float));
        }
    }

    let as_i128 = NumCast::from(value).unwrap_or_default();
    if as_i128 < 0 {
        return Err(NsParsingError::NotPositive(as_i128));
    }

    let as_u128 = NumCast::from(value).unwrap_or_default();
    if as_u128 > u32::MAX as u128 {
        return Err(NsParsingError::TooLarge(as_u128));
    }

    NumCast::from(value).ok_or(NsParsingError::TooLarge(0)) // shouldn't hit
}


