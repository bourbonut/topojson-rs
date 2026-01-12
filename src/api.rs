use crate::bbox::wrap_bbox;
use crate::feature::wrap_feature;
use crate::geojsons::{Feature, FeatureGeometryType};
use crate::merge::wrap_merge;
use crate::mesh::wrap_mesh;
use crate::neighbors::wrap_neighbors;
use crate::quantize::wrap_quantize;
use crate::topojsons::{Geometry, TopoJSON};
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict, PyFunction, PyString},
};

#[pyfunction]
pub fn feature(topology: TopoJSON, o: &Bound<'_, PyAny>) -> PyResult<Feature> {
    let feature = if o.is_instance_of::<PyString>() {
        let key: String = o.extract::<String>()?;
        let o = &topology.objects[&key];
        wrap_feature(&topology, o)
    } else {
        let o: Geometry = o.extract()?;
        wrap_feature(&topology, &o)
    };
    Ok(feature)
}

#[pyfunction]
pub fn merge(topology: TopoJSON, objects: Vec<Geometry>) -> FeatureGeometryType {
    wrap_merge(&topology, &objects)
}

#[pyfunction]
pub fn mesh(
    topology: TopoJSON,
    object: Option<&Bound<'_, PyDict>>,
    filter: Option<&Bound<'_, PyFunction>>,
) -> PyResult<FeatureGeometryType> {
    let object: Option<Geometry> = object.map(|o| o.extract()).transpose()?;
    wrap_mesh(&topology, object.as_ref(), filter)
}

#[pyfunction]
pub fn bbox(topology: TopoJSON) -> [f64; 4] {
    wrap_bbox(&topology)
}

#[pyfunction]
pub fn neighbors(objects: Vec<Geometry>) -> PyResult<Vec<Vec<i32>>> {
    Ok(wrap_neighbors(&objects))
}

#[pyfunction]
pub fn quantize(topology: TopoJSON, transform: f64) -> PyResult<TopoJSON> {
    wrap_quantize(&topology, &transform)
}
