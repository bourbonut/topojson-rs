use pyo3::prelude::*;

#[pyclass]
#[derive(Debug, PartialEq)]
pub enum GeoJSON {
    Collection(FeatureCollection),
    Item(Feature),
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
pub struct FeatureCollection {
    #[pyo3(get)]
    pub features: Vec<Feature>,
}

#[pyclass]
#[derive(Debug, PartialEq, Clone)]
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
