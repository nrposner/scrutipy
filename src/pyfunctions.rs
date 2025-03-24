use crate::grim::grim_scalar;
use crate::grimmer::grimmer;
use pyo3::prelude::Bound;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[cfg(not(tarpaulin_include))]
#[pymodule(name = "scrutipy_rs")]
fn scrutipy_rs(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(grim_scalar, module)?)?;
    module.add_function(wrap_pyfunction!(grimmer, module)?)?;
    Ok(())
}
