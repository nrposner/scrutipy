use core::f64;
use polars::series::Series;
use polars::datatypes::AnyValue;
use polars::{frame::DataFrame, prelude::DataType};
use pyo3::exceptions::{PyTypeError, PyValueError, PyIndexError};
use pyo3::types::{PyAnyMethods, PyString};
use pyo3::{pyfunction, PyResult, Python};
use pyo3_polars::PyDataFrame;
use crate::debit::debit;
use crate::grim_map_df::{ColumnInput, coerce_string_to_u32, coerce_to_u32, NsParsingError};

#[pyfunction()]
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

    let sds: &Series = match sd_col {
        ColumnInput::Name(name) => df.column(&name).map_err(|_| PyValueError::new_err(format!(
            "The n_col column named '{}' not found in the provided dataframe. Available columns: {:?}", 
            name,
            df.get_column_names()
        )))?
            .as_series()
            .ok_or_else(|| PyTypeError::new_err(format!("The column '{}' could not be interpreted as a Series", name)))?,

        ColumnInput::Index(ind) | ColumnInput::Default(ind) => df.get_columns().get(ind).ok_or_else(|| PyIndexError::new_err(format!(
            "The sd_col column index '{}' is out of bounds for the provided dataframe, which has {} columns",
            ind,
            df.width()
        )))?
            .as_series()
            .ok_or_else(|| PyTypeError::new_err("Columns could not be interpreted as a Series"))?,
    };

    if sds.len() == 0 {
        return Err(PyTypeError::new_err("The sd_col column is empty"));
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
                    (PyString::new(py, "The column `sd_col` is made up of numeric types instead of strings. \n Understand that you may be losing trailing zeros by using a purely numeric type. \n To silence this warning, set `silence_numeric_warning = True`."),),
                ).unwrap();
            }
            xs.iter().map(|x| x.to_string()).collect::<Vec<String>>()}),
        _ => Err("Input sds column is neither a String nor numeric type"),
    };

    // if the data type of xs is neither a string nor a numeric type which we could plausibly
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

    //let sds_vec = match sds_result {
    //    Err(_) => return Err(PyTypeError::new_err("The sd_col column is composed of neither strings nor numeric types. Please check the input types and the documentation.")),
    //    Ok(vs) => vs,
    //};

    let xs_temp: Vec<&str> = xs_vec.iter().map(|s| &**s).collect();

    let mut xs: Vec<String> = Vec::new();
    let mut sds: Vec<String> = Vec::new();
    let mut ns: Vec<u32> = Vec::new();
    let mut ns_err_inds: Vec<usize> = Vec::new();
    let mut sds_err_inds: Vec<usize> = Vec::new();

    for (i, ((n_result, sds_result), x)) in ns_vec.iter().zip(sds_vec.iter()).zip(xs_temp.iter()).enumerate() {
        if let Ok(u) = n_result {
            ns.push(*u);
            xs.push(x.to_string());
            sds.push(sds_result.to_string())
            //sds.push(sds_result.parse::<f64>()?);
        } else {
            ns_err_inds.push(i);
            sds_err_inds.push(i);
        }
    }

    // since we can't set a default for items which is dependent on the size of another variable
    // known at runtime, we wait until now to turn the default option into a vector of 1s the same
    // length as the number of valid counts 
    
    //let revised_items = match items {
    //    None => vec![1; xs.len()],
    //    Some(i) => i,
    //};

    let res = debit(xs, sds, ns, formula.as_str(), rounding.as_str(), threshold, symmetric, show_rec);

    // let res = grim_rust(xs, ns.clone(), vec![percent, show_rec, symmetric], revised_items, rounding.as_str(), threshold, tolerance);

    // if the length of ns_err_inds is 0, ie if no errors occurred, our error return is Option<None>.
    // Otherwise, our error return is Option<ns_err_inds>
    let err_output: Option<Vec<usize>> = match ns_err_inds.len() {
        0 => None,
        _ => Some(ns_err_inds),
    };

    Ok((res, err_output)) 
}
