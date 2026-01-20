// use crate::bbox::wrap_bbox;
use crate::feature::wrap_feature;
use crate::geojsons::{Feature, FeatureGeometryType};
use crate::merge::wrap_merge;
// use crate::mesh::wrap_mesh;
// use crate::neighbors::wrap_neighbors;
// use crate::quantize::wrap_quantize;
use crate::topojsons::{Geometry, TopoJSON, Transform};
use pyo3::exceptions::PyKeyError;
use pyo3::prelude::*;

#[pyfunction]
pub fn feature(topology: &TopoJSON, o: &Geometry) -> Feature {
    wrap_feature(topology, o)
}

#[pyfunction]
pub fn merge(topology: &TopoJSON, objects: Vec<Geometry>) -> FeatureGeometryType {
    wrap_merge(topology, objects.iter().collect::<Vec<_>>().as_slice())
}

// #[pyfunction]
// pub fn mesh(
//     topology: TopoJSON,
//     object: Option<&Bound<'_, PyDict>>,
//     filter: Option<&Bound<'_, PyFunction>>,
// ) -> PyResult<FeatureGeometryType> {
//     let object: Option<Geometry> = object.map(|o| o.extract()).transpose()?;
//     wrap_mesh(&topology, object.as_ref(), filter)
// }
//
// #[pyfunction]
// pub fn bbox(topology: TopoJSON) -> [f64; 4] {
//     wrap_bbox(&topology)
// }
//
// #[pyfunction]
// pub fn neighbors(objects: Vec<Geometry>) -> PyResult<Vec<Vec<i32>>> {
//     Ok(wrap_neighbors(&objects))
// }
//
// #[pyfunction]
// pub fn quantize(topology: TopoJSON, transform: f64) -> PyResult<TopoJSON> {
//     wrap_quantize(&topology, &transform)
// }

#[pymethods]
impl TopoJSON {
    #[getter]
    fn transform(&self) -> Option<Transform> {
        self.transform.clone()
    }

    fn feature(&self, key: &str) -> PyResult<Feature> {
        if let Some(o) = self.objects.get(key) {
            Ok(wrap_feature(self, o))
        } else {
            Err(PyKeyError::new_err(format!(
                "Key '{}' not found in 'objects'",
                key
            )))
        }
    }

    fn merge(&self, keys: Vec<String>) -> PyResult<FeatureGeometryType> {
        let objects: Vec<&Geometry> = keys
            .iter()
            .map(|key| {
                self.objects.get(key).ok_or(PyKeyError::new_err(format!(
                    "Key '{}' not found in 'objects'",
                    key
                )))
            })
            .collect::<PyResult<Vec<&Geometry>>>()?;
        Ok(wrap_merge(&self, &objects))
    }
}
