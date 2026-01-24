use pyo3::exceptions::{PyOSError, PyRuntimeError};
use pyo3::prelude::*;
use serde::Serialize;
use std::fs;

#[pyclass]
#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum GeoJSON {
    FeatureCollection(FeatureCollection),
    Feature(Feature),
}

#[pymethods]
impl GeoJSON {
    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}

#[pyclass]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct FeatureCollection {
    #[pyo3(get)]
    pub features: Vec<Feature>,
}

#[pymethods]
impl FeatureCollection {
    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}

#[pyclass]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct Feature {
    #[pyo3(get)]
    pub properties: Option<String>,
    #[pyo3(get)]
    pub geometry: FeatureGeometryType,
    #[pyo3(get)]
    pub id: Option<String>,
    #[pyo3(get)]
    pub bbox: Option<Vec<f64>>,
}

#[pymethods]
impl Feature {
    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}

#[pyclass]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum FeatureGeometryType {
    GeometryCollection {
        geometries: Vec<FeatureGeometryType>,
    },
    Point {
        coordinates: [f64; 2],
    },
    MultiPoint {
        coordinates: Vec<[f64; 2]>,
    },
    LineString {
        coordinates: Vec<[f64; 2]>,
    },
    MultiLineString {
        coordinates: Vec<Vec<[f64; 2]>>,
    },
    Polygon {
        coordinates: Vec<Vec<[f64; 2]>>,
    },
    MultiPolygon {
        coordinates: Vec<Vec<Vec<[f64; 2]>>>,
    },
}

#[pymethods]
impl FeatureGeometryType {
    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}
