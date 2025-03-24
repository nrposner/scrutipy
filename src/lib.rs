pub mod grim;
pub mod pyfunctions;
pub mod grim_map;
pub mod grim_stats;
pub mod test_grim;
pub mod grimmer;
pub mod rounding;
pub mod test_rounding;
pub mod sd_binary;
pub mod utils;
pub mod test_utils;
pub mod grim_map_df;

#[allow(unused_imports)]
use utils::*;

//use pyo3::prelude::*;

//A Python module implemented in Rust.
//#[pymodule]
//fn scrutipy(m: &Bound<'_, PyModule>) -> PyResult<()> {
//    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
//    Ok(())
//}
