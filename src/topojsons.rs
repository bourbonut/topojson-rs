use std::collections::HashMap;

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyString};
use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TopoJSON {
    #[pyo3(get)]
    pub bbox: Vec<f64>,
    #[pyo3(get)]
    pub transform: Option<Transform>,
    pub objects: HashMap<String, Geometry>,
    #[pyo3(get)]
    pub arcs: Vec<Vec<[i32; 2]>>,
}

#[pyclass]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Transform {
    #[pyo3(get)]
    pub scale: [f64; 2],
    #[pyo3(get)]
    pub translate: [f64; 2],
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    GeometryCollection {
        geometries: Vec<Geometry>,
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
    Point {
        coordinates: [f64; 2],
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
    MultiPoint {
        coordinates: Vec<[f64; 2]>,
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
    LineString {
        arcs: Vec<i32>,
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
    MultiLineString {
        arcs: Vec<Vec<i32>>,
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
    Polygon {
        arcs: Vec<Vec<i32>>,
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
    MultiPolygon {
        arcs: Vec<Vec<Vec<i32>>>,
        id: Option<String>,
        properties: Option<Properties>,
        bbox: Option<Vec<f64>>,
    },
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Properties(serde_json::Value);

impl<'py> IntoPyObject<'py> for Properties {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let bytes =
            serde_json::to_vec(&self.0).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        PyString::from_bytes(py, &bytes)
    }
}

#[pymethods]
impl TopoJSON {
    #[getter]
    fn transform(&self) -> Option<Transform> {
        self.transform.clone()
    }
}
