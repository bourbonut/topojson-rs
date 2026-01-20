use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, PartialEq)]
pub enum Feature {
    Collection(FeatureCollection),
    Item(FeatureItem),
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub struct FeatureCollection {
    pub features: Vec<FeatureItem>,
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub struct FeatureItem {
    pub properties: Option<String>,
    pub geometry: FeatureGeometryType,
    pub id: Option<String>,
    pub bbox: Option<Vec<f64>>,
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
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
