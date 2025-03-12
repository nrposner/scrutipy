pub mod grim;
pub mod grim_map;
pub mod grim_stats;
pub mod grim_tests;
pub mod grimmer;
pub mod rounding;
pub mod rounding_tests;
pub mod sd_binary;
pub mod utils;

#[allow(unused_imports)]
use utils::*;

//use pyo3::prelude::*;

//A Python module implemented in Rust.
//#[pymodule]
//fn scrutipy(m: &Bound<'_, PyModule>) -> PyResult<()> {
//    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
//    Ok(())
//}
