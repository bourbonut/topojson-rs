use std::collections::HashMap;

use pyo3::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_map_into_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
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
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    Point {
        coordinates: [f64; 2],
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    MultiPoint {
        coordinates: Vec<[f64; 2]>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    LineString {
        arcs: Vec<i32>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    MultiLineString {
        arcs: Vec<Vec<i32>>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    Polygon {
        arcs: Vec<Vec<i32>>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
    MultiPolygon {
        arcs: Vec<Vec<Vec<i32>>>,
        id: Option<String>,
        #[serde(deserialize_with = "deserialize_map_into_string")]
        #[serde(default)]
        properties: Option<String>,
        bbox: Option<Vec<f64>>,
    },
}

impl Geometry {
    pub fn id(&self) -> Option<String> {
        match self {
            Geometry::GeometryCollection { id, .. } => id.clone(),
            Geometry::Point { id, .. } => id.clone(),
            Geometry::MultiPoint { id, .. } => id.clone(),
            Geometry::LineString { id, .. } => id.clone(),
            Geometry::MultiLineString { id, .. } => id.clone(),
            Geometry::Polygon { id, .. } => id.clone(),
            Geometry::MultiPolygon { id, .. } => id.clone(),
        }
    }

    pub fn properties(&self) -> Option<String> {
        match self {
            Geometry::GeometryCollection { properties, .. } => properties.clone(),
            Geometry::Point { properties, .. } => properties.clone(),
            Geometry::MultiPoint { properties, .. } => properties.clone(),
            Geometry::LineString { properties, .. } => properties.clone(),
            Geometry::MultiLineString { properties, .. } => properties.clone(),
            Geometry::Polygon { properties, .. } => properties.clone(),
            Geometry::MultiPolygon { properties, .. } => properties.clone(),
        }
    }

    pub fn bbox(&self) -> Option<Vec<f64>> {
        match self {
            Geometry::GeometryCollection { bbox, .. } => bbox.clone(),
            Geometry::Point { bbox, .. } => bbox.clone(),
            Geometry::MultiPoint { bbox, .. } => bbox.clone(),
            Geometry::LineString { bbox, .. } => bbox.clone(),
            Geometry::MultiLineString { bbox, .. } => bbox.clone(),
            Geometry::Polygon { bbox, .. } => bbox.clone(),
            Geometry::MultiPolygon { bbox, .. } => bbox.clone(),
        }
    }
}

#[pymethods]
impl TopoJSON {
    #[getter]
    fn transform(&self) -> Option<Transform> {
        self.transform.clone()
    }
}
