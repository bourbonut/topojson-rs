use std::collections::HashMap;

use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyString};
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_string_or_map<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    Ok(Some(value.to_string()))
}

#[pyclass]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TopoJSON {
    #[pyo3(get)]
    pub bbox: Vec<f64>,
    #[pyo3(get)]
    pub transform: Option<Transform>,
    #[pyo3(get)]
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

#[pyclass]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Geometry {
    GeometryCollection {
        geometries: Vec<Geometry>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    Point {
        coordinates: [f64; 2],
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    MultiPoint {
        coordinates: Vec<[f64; 2]>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    LineString {
        arcs: Vec<i32>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    MultiLineString {
        arcs: Vec<Vec<i32>>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    Polygon {
        arcs: Vec<Vec<i32>>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    MultiPolygon {
        arcs: Vec<Vec<Vec<i32>>>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_string_or_map")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
}

#[pymethods]
impl TopoJSON {
    #[getter]
    fn transform(&self) -> Option<Transform> {
        self.transform.clone()
    }
}
