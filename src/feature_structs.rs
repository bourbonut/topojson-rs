use crate::topojson_structs::Properties;

#[derive(Debug)]
pub enum Feature {
    Collection(FeatureCollection),
    Item(FeatureItem),
}

#[derive(Debug)]
pub struct FeatureCollection {
    pub r#type: String,
    pub features: Vec<Feature>,
}

#[derive(Debug)]
pub struct FeatureItem {
    pub r#type: String,
    pub properties: Option<Properties>,
    pub geometry: FeatureGeometry,
    pub id: Option<String>,
    pub bbox: Option<Vec<f32>>,
}

#[derive(Debug)]
pub enum FeatureGeometryType {
    GeometryCollection {
        geometries: Vec<FeatureGeometry>,
    },
    Point {
        coordinates: Vec<f32>,
    },
    MultiPoint {
        coordinates: Vec<Vec<f32>>,
    },
    Line {
        coordinates: Vec<Vec<f32>>,
    },
    MultiLineString {
        coordinates: Vec<Vec<Vec<f32>>>,
    },
    Polygon {
        coordinates: Vec<Vec<Vec<f32>>>,
    },
    MultiPolygon {
        coordinates: Vec<Vec<Vec<Vec<f32>>>>,
    },
}

#[derive(Debug)]
pub struct FeatureGeometry {
    pub r#type: String,
    pub geometry: FeatureGeometryType,
}
