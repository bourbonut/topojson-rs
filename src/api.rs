use crate::bbox::wrap_bbox;
use crate::feature::wrap_feature;
use crate::geojson_structs::{Feature, FeatureGeometryType};
use crate::merge::wrap_merge;
use crate::mesh::wrap_mesh;
use crate::neighbors::wrap_neighbors;
use crate::quantize::wrap_quantize;
use crate::topojson_structs::{Geometry, TopoJSON};
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict, PyFunction, PyList, PyString},
};

#[pyfunction]
pub fn feature(topology: &Bound<'_, PyDict>, o: &Bound<'_, PyAny>) -> PyResult<Feature> {
    let topology: TopoJSON = topology.extract()?;
    let feature = if o.is_instance_of::<PyString>() {
        let key: String = o.extract::<String>()?;
        let o = &topology.objects[&key];
        wrap_feature(&topology, o)?
    } else {
        let o: Geometry = o.extract()?;
        wrap_feature(&topology, &o)?
    };
    Ok(feature)
}

#[pyfunction]
pub fn merge(
    topology: &Bound<'_, PyDict>,
    objects: &Bound<'_, PyList>,
) -> PyResult<FeatureGeometryType> {
    let topology: TopoJSON = topology.extract()?;
    let objects: Vec<Geometry> = objects.extract()?;
    let feature_geometry = wrap_merge(&topology, &objects)?;
    Ok(feature_geometry)
}

#[pyfunction]
pub fn mesh(
    topology: &Bound<'_, PyDict>,
    object: Option<&Bound<'_, PyDict>>,
    filter: Option<&Bound<'_, PyFunction>>,
) -> PyResult<FeatureGeometryType> {
    let topology: TopoJSON = topology.extract()?;
    let object: Option<Geometry> = object.map(|o| o.extract()).transpose()?;
    wrap_mesh(&topology, object.as_ref(), filter)
}

#[pyfunction]
pub fn bbox(topology: &Bound<'_, PyDict>) -> PyResult<[f64; 4]> {
    let topology: TopoJSON = topology.extract()?;
    wrap_bbox(&topology)
}

#[pyfunction]
pub fn neighbors(objects: &Bound<'_, PyList>) -> PyResult<Vec<Vec<i32>>> {
    let objects: Vec<Geometry> = objects.extract()?;
    Ok(wrap_neighbors(&objects))
}

#[pyfunction]
pub fn quantize(topology: &Bound<'_, PyDict>, transform: f64) -> PyResult<TopoJSON> {
    let topology: TopoJSON = topology.extract()?;
    Ok(wrap_quantize(&topology, &transform)?)
}
