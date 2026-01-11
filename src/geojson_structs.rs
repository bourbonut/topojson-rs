use crate::topojson_structs::Properties;
use pyo3::{prelude::*, types::PyDict};

#[derive(Debug, PartialEq)]
pub enum Feature {
    Collection(FeatureCollection),
    Item(FeatureItem),
}

impl<'py> IntoPyObject<'py> for Feature {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Feature::Item(feature) => Ok(feature.into_pyobject(py)?),
            Feature::Collection(feature) => Ok(feature.into_pyobject(py)?),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct FeatureCollection {
    pub features: Vec<FeatureItem>,
}

impl<'py> IntoPyObject<'py> for FeatureCollection {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item("type", "FeatureCollection")?;
        dict.set_item("features", self.features)?;
        Ok(dict)
    }
}

#[derive(Debug, PartialEq)]
pub struct FeatureItem {
    pub properties: Option<Properties>,
    pub geometry: FeatureGeometryType,
    pub id: Option<String>,
    pub bbox: Option<Vec<f64>>,
}

impl<'py> IntoPyObject<'py> for FeatureItem {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item("type", "Feature")?;
        match self.properties {
            Some(properties) => dict.set_item("properties", properties)?,
            None => dict.set_item("properties", PyDict::new(py))?,
        };
        dict.set_item("geometry", self.geometry)?;
        if let Some(id) = self.id {
            dict.set_item("id", id)?;
        }
        if let Some(bbox) = self.bbox {
            dict.set_item("bbox", bbox)?;
        };
        Ok(dict)
    }
}

#[derive(Debug, PartialEq)]
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

impl<'py> IntoPyObject<'py> for FeatureGeometryType {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            FeatureGeometryType::GeometryCollection { geometries } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "GeometryCollection")?;
                dict.set_item("geometries", geometries)?;
                Ok(dict)
            }
            FeatureGeometryType::Point { coordinates } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "Point")?;
                dict.set_item("coordinates", coordinates)?;
                Ok(dict)
            }
            FeatureGeometryType::MultiPoint { coordinates } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "MultiPoint")?;
                dict.set_item("coordinates", coordinates)?;
                Ok(dict)
            }
            FeatureGeometryType::LineString { coordinates } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "LineString")?;
                dict.set_item("coordinates", coordinates)?;
                Ok(dict)
            }
            FeatureGeometryType::MultiLineString { coordinates } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "MultiLineString")?;
                dict.set_item("coordinates", coordinates)?;
                Ok(dict)
            }
            FeatureGeometryType::Polygon { coordinates } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "Polygon")?;
                dict.set_item("coordinates", coordinates)?;
                Ok(dict)
            }
            FeatureGeometryType::MultiPolygon { coordinates } => {
                let dict = PyDict::new(py);
                dict.set_item("type", "MultiPolygon")?;
                dict.set_item("coordinates", coordinates)?;
                Ok(dict)
            }
        }
    }
}
