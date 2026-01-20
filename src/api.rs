use crate::bbox::wrap_bbox;
use crate::feature::wrap_feature;
use crate::geojsons::{Feature, FeatureGeometryType};
use crate::merge::wrap_merge;
use crate::mesh::wrap_mesh;
use crate::neighbors::wrap_neighbors;
use crate::quantize::wrap_quantize;
use crate::topojsons::{Geometry, TopoJSON, Transform};
use pyo3::exceptions::{PyKeyError, PyTypeError};
use pyo3::prelude::*;
use pyo3::types::PyFunction;

#[pyfunction]
pub fn feature(topology: &TopoJSON, o: &Geometry) -> Feature {
    wrap_feature(topology, o)
}

#[pyfunction]
pub fn merge(topology: &TopoJSON, objects: Vec<Geometry>) -> FeatureGeometryType {
    wrap_merge(topology, objects.iter().collect::<Vec<_>>().as_slice())
}

#[pyfunction]
pub fn mesh(
    topology: &TopoJSON,
    object: Option<Geometry>,
    filter: Option<&Bound<'_, PyFunction>>,
) -> PyResult<FeatureGeometryType> {
    wrap_mesh(topology, object.as_ref(), filter)
}

#[pyfunction]
pub fn bbox(topology: &TopoJSON) -> [f64; 4] {
    wrap_bbox(topology)
}

#[pyfunction]
pub fn neighbors(objects: Vec<Geometry>) -> Vec<Vec<i32>> {
    wrap_neighbors(objects.iter().collect::<Vec<_>>().as_slice())
}

#[pyfunction]
pub fn quantize(topology: &TopoJSON, transform: f64) -> PyResult<TopoJSON> {
    wrap_quantize(&topology, &transform)
}

#[pymethods]
impl TopoJSON {
    #[getter(transform)]
    fn transform(&self) -> Option<Transform> {
        self.transform.clone()
    }

    #[setter(transform)]
    fn set_transform(&mut self, new_transform: Option<Transform>) {
        self.transform = new_transform;
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

    fn merge(&self, key: &str) -> PyResult<FeatureGeometryType> {
        if let Geometry::GeometryCollection { geometries, .. } = self.objects.get(key).ok_or(
            PyKeyError::new_err(format!("Key '{}' not found in 'objects'", key)),
        )? {
            Ok(wrap_merge(
                &self,
                geometries.iter().collect::<Vec<_>>().as_slice(),
            ))
        } else {
            Err(PyTypeError::new_err(format!(
                "The type of geometry '{}' must be 'GeometryCollection'",
                key
            )))
        }
    }

    fn mesh(
        &self,
        key: Option<&str>,
        filter: Option<&Bound<'_, PyFunction>>,
    ) -> PyResult<FeatureGeometryType> {
        match key {
            Some(key) => {
                if let Some(obj) = self.objects.get(key) {
                    wrap_mesh(self, Some(obj), filter)
                } else {
                    Err(PyKeyError::new_err(format!(
                        "Key '{}' not found in 'objects'",
                        key
                    )))
                }
            }
            None => wrap_mesh(self, None, filter),
        }
    }

    fn bbox(&self) -> [f64; 4] {
        wrap_bbox(self)
    }

    fn neighbors(&self, keys: Vec<String>) -> PyResult<Vec<Vec<i32>>> {
        let objects: Vec<&Geometry> = keys
            .iter()
            .map(|key| {
                self.objects.get(key).ok_or(PyKeyError::new_err(format!(
                    "Key '{}' not found in 'objects'",
                    key
                )))
            })
            .collect::<PyResult<Vec<&Geometry>>>()?;
        Ok(wrap_neighbors(&objects))
    }

    fn quantize(&self, transform: f64) -> PyResult<TopoJSON> {
        wrap_quantize(self, &transform)
    }
}
