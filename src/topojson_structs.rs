use std::collections::HashMap;

use pyo3::{exceptions::PyKeyError, prelude::*, types::PyDict};

#[derive(Debug)]
pub struct TopoJSON {
    pub r#type: String,
    pub bbox: Vec<f32>,
    pub transform: Option<Transform>,
    pub objects: HashMap<String, Geometry>,
    pub arcs: Vec<Vec<Vec<i32>>>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for TopoJSON {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
        let r#type = dict
            .get_item("type")?
            .ok_or_else(|| PyKeyError::new_err("\"type\" not found in the topojson"))?
            .extract()?;
        let bbox = dict
            .get_item("bbox")?
            .ok_or_else(|| PyKeyError::new_err("\"bbox\" not found in the topojson"))?
            .extract()?;
        let transform = dict
            .get_item("transform")?
            .map(|v| v.extract())
            .transpose()?;
        let objects = dict
            .get_item("objects")?
            .ok_or_else(|| PyKeyError::new_err("\"objects\" not found in the topojson"))?
            .extract()?;
        let arcs = dict
            .get_item("arcs")?
            .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in the topojson"))?
            .extract()?;
        Ok(Self {
            r#type,
            bbox,
            transform,
            objects,
            arcs,
        })
    }
}

#[derive(Debug)]
pub struct Transform {
    pub scale: Vec<f32>,
    pub translate: Vec<f32>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Transform {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
        let scale = dict
            .get_item("scale")?
            .ok_or_else(|| PyKeyError::new_err("\"scale\" not found in \"transform\""))?
            .extract()?;
        let translate = dict
            .get_item("translate")?
            .ok_or_else(|| PyKeyError::new_err("\"translate\" not found in \"transform\""))?
            .extract()?;
        Ok(Self { scale, translate })
    }
}

#[derive(Debug)]
pub enum GeometryType {
    GeometryCollection { geometries: Vec<Geometry> },
    Point { coordinates: Vec<f32> },
    MultiPoint { coordinates: Vec<Vec<f32>> },
    LineString { arcs: Vec<i32> },
    MultiLineString { arcs: Vec<Vec<i32>> },
    Polygon { arcs: Vec<Vec<i32>> },
    MultiPolygon { arcs: Vec<Vec<Vec<i32>>> },
}

#[derive(Debug)]
pub struct Geometry {
    pub r#type: String,
    pub geometry: GeometryType,
    pub id: Option<String>,
    pub properties: Option<Properties>,
    pub bbox: Option<Vec<f32>>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Geometry {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
        let r#type: String = dict
            .get_item("type")?
            .ok_or_else(|| PyKeyError::new_err("\"type\" not found in \"geometry\""))?
            .extract()?;
        let bbox = dict.get_item("bbox")?.map(|v| v.extract()).transpose()?;
        let id = dict.get_item("id")?.map(|v| v.extract()).transpose()?;
        let properties = dict
            .get_item("properties")?
            .map(|v| v.extract())
            .transpose()?;

        let geometry = match r#type.as_str() {
            "GeometryCollection" => GeometryType::GeometryCollection {
                geometries: dict
                    .get_item("geometries")?
                    .ok_or_else(|| PyKeyError::new_err("\"geometries\" not found in \"geometry\""))?
                    .extract()?,
            },
            "Point" => GeometryType::Point {
                coordinates: dict
                    .get_item("coordinates")?
                    .ok_or_else(|| {
                        PyKeyError::new_err("\"coordinates\" not found in \"geometry\"")
                    })?
                    .extract()?,
            },
            "MultiPoint" => GeometryType::MultiPoint {
                coordinates: dict
                    .get_item("coordinates")?
                    .ok_or_else(|| {
                        PyKeyError::new_err("\"coordinates\" not found in \"geometry\"")
                    })?
                    .extract()?,
            },
            "LineString" => GeometryType::LineString {
                arcs: dict
                    .get_item("arcs")?
                    .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?
                    .extract()?,
            },
            "MultiLineString" => GeometryType::MultiLineString {
                arcs: dict
                    .get_item("arcs")?
                    .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?
                    .extract()?,
            },
            "Polygon" => GeometryType::Polygon {
                arcs: dict
                    .get_item("arcs")?
                    .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?
                    .extract()?,
            },
            "MultiPolygon" => GeometryType::MultiPolygon {
                arcs: dict
                    .get_item("arcs")?
                    .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?
                    .extract()?,
            },
            unknown_type => {
                return Err(PyKeyError::new_err(format!(
                    "Unknown type: {}",
                    unknown_type
                )));
            }
        };

        Ok(Self {
            r#type,
            bbox,
            geometry,
            id,
            properties,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Properties {
    pub name: String,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Properties {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
        let name = dict
            .get_item("name")?
            .ok_or_else(|| PyKeyError::new_err("\"name\" not found in \"properties\""))?
            .extract()?;
        Ok(Self { name })
    }
}

impl<'py> IntoPyObject<'py> for Properties {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item("name", self.name)?;
        Ok(dict)
    }
}
