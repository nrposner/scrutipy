use crate::grim::grim_scalar;
use crate::grimmer::grimmer;
use crate::grim_map_df::grim_map_df;
use pyo3::prelude::Bound;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[cfg(not(tarpaulin_include))]
#[pymodule(name = "scrutipy")]
fn scrutipy(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(grim_scalar, module)?)?;
    module.add_function(wrap_pyfunction!(grimmer, module)?)?;
    module.add_function(wrap_pyfunction!(grim_map_df, module)?)?;
    Ok(())
}
