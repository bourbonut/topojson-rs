mod api;
mod feature;
mod geojson_structs;
mod merge;
mod reverse;
mod stitch;
mod topojson_structs;
mod transform;

use pyo3::prelude::*;

#[pymodule]
fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(api::feature, m)?)?;
    m.add_function(wrap_pyfunction!(api::merge, m)?)?;
    Ok(())
}
