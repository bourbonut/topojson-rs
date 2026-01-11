mod api;
mod bbox;
mod bisect;
mod feature;
mod geojson_structs;
// mod merge;
// mod mesh;
mod neighbors;
#[cfg(test)]
mod parser;
mod quantize;
mod reverse;
// mod stitch;
mod topojson_structs;
mod transform;
mod untransform;

use pyo3::prelude::*;

#[pymodule]
fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(api::feature, m)?)?;
    // m.add_function(wrap_pyfunction!(api::merge, m)?)?;
    // m.add_function(wrap_pyfunction!(api::mesh, m)?)?;
    m.add_function(wrap_pyfunction!(api::bbox, m)?)?;
    m.add_function(wrap_pyfunction!(api::neighbors, m)?)?;
    m.add_function(wrap_pyfunction!(api::quantize, m)?)?;
    Ok(())
}
