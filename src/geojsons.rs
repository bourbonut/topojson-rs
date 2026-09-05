use pyo3::exceptions::{PyOSError, PyRuntimeError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde::Serialize;
use std::fs;

#[pyclass(from_py_object)]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum GeoJSON {
    FeatureCollection(FeatureCollection),
    Feature(Feature),
}

#[pymethods]
impl GeoJSON {
    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        match self {
            Self::FeatureCollection(feature_collection) => feature_collection.to_dict(py),
            Self::Feature(feature) => feature.to_dict(py),
        }
    }

    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}

#[pyclass(from_py_object)]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct FeatureCollection {
    #[pyo3(get)]
    pub features: Vec<Feature>,
}

#[pymethods]
impl FeatureCollection {
    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        dict.set_item("type", "FeatureCollection")?;
        dict.set_item(
            "features",
            self.features
                .iter()
                .map(|feature| feature.to_dict(py))
                .collect::<PyResult<Vec<Bound<'py, PyDict>>>>()?,
        )?;
        Ok(dict)
    }

    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}

#[pyclass(from_py_object)]
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
    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict: Bound<'py, PyDict> = PyDict::new(py);
        dict.set_item("type", "Feature")?;
        dict.set_item("geometry", self.geometry.to_dict(py)?)?;
        if let Some(id) = self.id.as_ref() {
            dict.set_item("id", id)?;
        }
        if let Some(properties) = self.properties.as_ref() {
            dict.set_item("properties", properties)?;
        }
        if let Some(bbox) = self.bbox.as_ref() {
            dict.set_item("bbox", bbox)?;
        }
        Ok(dict)
    }

    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}

#[pyclass(from_py_object)]
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
    fn to_bytes(&self) -> PyResult<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        match self {
            Self::GeometryCollection { geometries } => {
                dict.set_item("type", "GeometryCollection")?;
                dict.set_item(
                    "geometries",
                    geometries
                        .iter()
                        .map(|geometry| geometry.to_dict(py))
                        .collect::<PyResult<Vec<Bound<'py, PyDict>>>>()?,
                )?;
            }
            Self::Point { coordinates } => {
                dict.set_item("type", "Point")?;
                dict.set_item("coordinates", coordinates)?;
            }
            Self::MultiPoint { coordinates } => {
                dict.set_item("type", "MultiPoint")?;
                dict.set_item("coordinates", coordinates)?;
            }
            Self::LineString { coordinates } => {
                dict.set_item("type", "LineString")?;
                dict.set_item("coordinates", coordinates)?;
            }
            Self::MultiLineString { coordinates } => {
                dict.set_item("type", "MultiLineString")?;
                dict.set_item("coordinates", coordinates)?;
            }
            Self::Polygon { coordinates } => {
                dict.set_item("type", "Polygon")?;
                dict.set_item("coordinates", coordinates)?;
            }
            Self::MultiPolygon { coordinates } => {
                dict.set_item("type", "MultiPolygon")?;
                dict.set_item("coordinates", coordinates)?;
            }
        }
        Ok(dict)
    }

    fn write(&self, file: &str) -> PyResult<()> {
        fs::write(
            file,
            serde_json::to_vec(self).map_err(|e| PyRuntimeError::new_err(e.to_string()))?,
        )
        .map_err(PyOSError::new_err)?;
        Ok(())
    }
}
