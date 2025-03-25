use crate::grim::grim_scalar;
use crate::grimmer::grimmer;
use crate::grim_map_df::grim_map_pl;
use pyo3::prelude::Bound;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use crate::grim_map::grim_map;

#[cfg(not(tarpaulin_include))]
#[pymodule(name = "scrutipy")]
fn scrutipy(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(grim_scalar, module)?)?;
    module.add_function(wrap_pyfunction!(grimmer, module)?)?;
    module.add_function(wrap_pyfunction!(grim_map_pl, module)?)?;
    module.add_function(wrap_pyfunction!(grim_map, module)?)?;
    Ok(())
}
