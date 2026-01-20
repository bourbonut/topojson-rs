mod api;
// mod bbox;
// mod bisect;
mod feature;
mod geojsons;
mod merge;
// mod mesh;
// mod neighbors;
// #[cfg(test)]
// mod parser;
// mod quantize;
mod reverse;
mod stitch;
mod topojsons;
mod transform;
// mod untransform;

use crate::geojsons::Feature;
use crate::topojsons::TopoJSON;

use std::fs;

use pyo3::{
    exceptions::{PyOSError, PyRuntimeError},
    prelude::*,
};

#[pyfunction]
fn read(file: &str) -> PyResult<TopoJSON> {
    let content = fs::read_to_string(file).map_err(PyOSError::new_err)?;
    serde_json::from_str::<TopoJSON>(&content).map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pymodule]
fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TopoJSON>()?;
    m.add_class::<Feature>()?;
    m.add_function(wrap_pyfunction!(read, m)?)?;
    m.add_function(wrap_pyfunction!(api::feature, m)?)?;
    m.add_function(wrap_pyfunction!(api::merge, m)?)?;
    // m.add_function(wrap_pyfunction!(api::mesh, m)?)?;
    // m.add_function(wrap_pyfunction!(api::bbox, m)?)?;
    // m.add_function(wrap_pyfunction!(api::neighbors, m)?)?;
    // m.add_function(wrap_pyfunction!(api::quantize, m)?)?;
    Ok(())
}
