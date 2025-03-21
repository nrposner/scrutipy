use std::f64;

use polars::series::Series;
use polars::datatypes::AnyValue;
use polars::{frame::DataFrame, prelude::DataType};
use pyo3::{pyfunction, FromPyObject};
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
#[derive(FromPyObject)]
#[allow(dead_code)]
pub enum ColumnInput {
    Name(String),
    Index(usize),
}

//best approach here, make a struct to imitate the dataframe that we're expecting? Then we could
//downcast directly into that struct, which we will also make into a pyclass, and we can get around
//all this business 
// We want to define a Dataframe type, whose contents are a list of column names and a set of
// Series
// And each of those Series is a Vec, but a Vec that can contain a flexible data type, though all
// must contain the same type


#[allow(unused_variables)]
#[pyfunction(signature = (pydf, x_col=ColumnInput::Index(0), n_col=ColumnInput::Index(1)))]
pub fn grim_map_df(pydf: PyDataFrame, x_col: ColumnInput, n_col: ColumnInput) -> (Vec<bool>, Option<Vec<usize>>){
    let df: DataFrame = pydf.into();

    let xs: &Series = match x_col {
        ColumnInput::Name(name) => df.column(&name).unwrap().as_series().unwrap(), //df[&*name].clone().get(),
        ColumnInput::Index(ind) => df.get_columns()[ind].as_series().unwrap(), //df[ind].clone().get(),
    };

    let ns: &Series= match n_col {
        ColumnInput::Name(name) => df.column(&name).unwrap().as_series().unwrap(),//df[&*name].clone(),
        ColumnInput::Index(ind) => df.get_columns()[ind].as_series().unwrap(),//df[ind].clone(),
    };


    // check the datatype of xs and ns, if string type, attempt to parse, and if any single element
    // fails, return an error along with an indication of which index caused the error 
    // if a numeric type, coerce as necessary, we want to ensure that everything could be turned
    // into a 
    //
    // do we actually need to account for the possibility that the xs column will be numeric in
    // addition to string? Yes, but in that event we should pass a warning to the user, letting
    // them know that by using numeric data directly, they may be losing necessary information from
    // trailing zeros, and that they thus should take results as possibly incorrect 
   

    //let values: Vec<String> = xs.utf8()?
    //.into_no_null_iter()
    //.map(|s| s.to_string())
    //.collect();
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

    let xs_vec = match xs_result {
        Ok(xs) => xs,
        Err(_) => panic!(),
    };

    let ns_result = match ns.dtype() {
        DataType::String => Ok(coerce_string_to_u32(ns.clone())),//Ok(ns.iter().map(|n| n.to_string()).collect::<Vec<String>>()),
        DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64 => Ok({
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
            .collect::<Vec<Result<u32, NsParsingError>>>()}),
        _ => Err(NsParsingError::NotNumeric),

    };


    let ns_vec = match ns_result {
        Err(_) => panic!(),
        Ok(vs) => vs,
    };

    // think more clearly about what we want this design to do:
    //  What do we want to do if one of these rows has an error? Do we want to skip over it and
    //  just do the others? Yes, but we also want to preserve the index of the erroneous row so
    //  that we can communicate that to the user 
    //  above, we're just going to pull out the result and pass up the error (currently a panic) to
    //  the user if it just failed completely
    //
    //  And now that it's in Vec<Result> form, we use it down below. If the internal result
    //  unwraps, then we run grim_map, or possibly just run grim_rust directly here 
    //  otherwise, we pass an error? Or we pass nothing, but preserve the error index?? 
    //
    //  In the inner loop, let's append the index into a counter vector, which we'll later pass
    //  back as an Option<Vec<usize>> 


    // now that we have these as vectors of Strings for xs and vectors of Results of u32s, we can
    // pass them into grim map
    // we may need to extend for vectors of items and so on, but for the time being we'll protoype
    // with just this
    
    let xs_temp: Vec<&str> = xs_vec.iter().map(|s| &**s).collect();

    let mut xs: Vec<&str> = Vec::new();
    let mut ns: Vec<u32> = Vec::new();
    let mut ns_err_inds: Vec<usize> = Vec::new();

    for (i, (n_result, x)) in ns_vec.iter().zip(xs_temp.iter()).enumerate() {
        match n_result {
            Ok(u) => {
                ns.push(*u);
                xs.push(*x)
            },
            Err(_) => ns_err_inds.push(i)
        }
    }


    let res = grim_rust(xs, ns.clone(), vec![false; 3], vec![1; ns.len()], vec!["up_or_down"], 5.0, f64::EPSILON.powf(0.5));

    let err_output: Option<Vec<usize>> = match ns_err_inds.len() {
        0 => Some(ns_err_inds),
        _ => None,
    };

    (res, err_output)

    // let's even add on to this by zipping in xs, so we also just collect the xs that are part of
    // valid rows

    //xs.iter().zip(ns_vec.iter()).map(|(x, n)| {
    //    match n.unwrap() {
    //        Ok(num) => grim_map()
    //    }
    //})

    // for the numeric types, coerce into a function that turns them into strings, but also note
    // that some important trailing zeros may have been lost 



    
    //let xs = ;
    //let ns = _;
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

/// run through a polars series and ensure that every element can be turned into an unsigned
/// integer type
/// If not, return an error (???) and note the indices in the error
/// Don't just stop at the first one, go all the way through to make sure there aren't more of them 
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







