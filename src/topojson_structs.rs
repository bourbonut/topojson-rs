use crate::intervec::InterVec;
use std::collections::HashMap;

use pyo3::{
    exceptions::PyKeyError,
    prelude::*,
    types::{PyDict, PyList},
};

#[derive(Debug, PartialEq)]
pub struct TopoJSON {
    #[allow(unused)]
    pub bbox: Vec<f64>,
    pub transform: Option<Transform>,
    pub objects: HashMap<String, Geometry>,
    pub arcs: InterVec<i32, 3>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for TopoJSON {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
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
        let any = dict
            .get_item("arcs")?
            .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in topojson"))?;
        let mut arcs = InterVec::new();
        let pyarcs: &Bound<'_, PyList> = any.cast()?;
        for (i, pylevel2) in pyarcs.iter().enumerate() {
            let level2: &Bound<'_, PyList> = pylevel2.cast()?;
            for (j, pylevel3) in level2.iter().enumerate() {
                let level3: &Bound<'_, PyList> = pylevel3.cast()?;
                for (k, value) in level3.iter().enumerate() {
                    arcs.push(value.extract()?, [i, j, k]);
                }
            }
        }
        Ok(Self {
            bbox,
            transform,
            objects,
            arcs,
        })
    }
}

impl<'py> IntoPyObject<'py> for TopoJSON {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        dict.set_item("type", "Topology")?;
        if !self.bbox.is_empty() {
            dict.set_item("bbox", self.bbox)?;
        }
        if let Some(transform) = self.transform {
            let transform_dict = PyDict::new(py);
            transform_dict.set_item("scale", transform.scale)?;
            transform_dict.set_item("translate", transform.translate)?;
            dict.set_item("transform", transform_dict)?;
        }
        if !self.arcs.is_empty() {
            dict.set_item("arcs", self.arcs)?;
        }
        if !self.objects.is_empty() {
            dict.set_item("objects", self.objects)?;
        }
        Ok(dict)
    }
}

#[derive(Debug, PartialEq)]
pub struct Transform {
    pub scale: [f64; 2],
    pub translate: [f64; 2],
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

#[derive(Debug, Clone, PartialEq)]
pub enum GeometryType {
    GeometryCollection { geometries: Vec<Geometry> },
    Point { coordinates: [f64; 2] },
    MultiPoint { coordinates: Vec<[f64; 2]> },
    LineString { arcs: Vec<i32> },
    MultiLineString { arcs: InterVec<i32, 2> },
    Polygon { arcs: InterVec<i32, 2> },
    MultiPolygon { arcs: InterVec<i32, 3> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Geometry {
    pub geometry: GeometryType,
    pub id: Option<String>,
    pub properties: Option<Properties>,
    pub bbox: Option<Vec<f64>>,
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
                arcs: {
                    let any = dict
                        .get_item("arcs")?
                        .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?;
                    let pyarcs: &Bound<'_, PyList> = any.cast()?;

                    let mut arcs = InterVec::new();
                    for (i, pylevel2) in pyarcs.iter().enumerate() {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (j, value) in level2.iter().enumerate() {
                            arcs.push(value.extract()?, [i, j]);
                        }
                    }
                    arcs
                },
            },
            "Polygon" => GeometryType::Polygon {
                arcs: {
                    let any = dict
                        .get_item("arcs")?
                        .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?;
                    let pyarcs: &Bound<'_, PyList> = any.cast()?;

                    let mut arcs = InterVec::new();
                    for (i, pylevel2) in pyarcs.iter().enumerate() {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (j, value) in level2.iter().enumerate() {
                            arcs.push(value.extract()?, [i, j]);
                        }
                    }
                    arcs
                },
            },
            "MultiPolygon" => GeometryType::MultiPolygon {
                arcs: {
                    let any = dict
                        .get_item("arcs")?
                        .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?;
                    let pyarcs: &Bound<'_, PyList> = any.cast()?;

                    let mut arcs = InterVec::new();
                    for (i, pylevel2) in pyarcs.iter().enumerate() {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (j, pylevel3) in level2.iter().enumerate() {
                            let level3: &Bound<'_, PyList> = pylevel3.cast()?;
                            for (k, value) in level3.iter().enumerate() {
                                arcs.push(value.extract()?, [i, j, k]);
                            }
                        }
                    }
                    arcs
                },
            },
            unknown_type => {
                return Err(PyKeyError::new_err(format!(
                    "Unknown type: {}",
                    unknown_type
                )));
            }
        };

        Ok(Self {
            bbox,
            geometry,
            id,
            properties,
        })
    }
}

impl<'py> IntoPyObject<'py> for Geometry {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        if let Some(id) = &self.id {
            dict.set_item("id", id)?;
        }
        if let Some(properties) = &self.properties {
            let properties_dict = PyDict::new(py);
            properties_dict.set_item("name", properties.name.clone())?;
            dict.set_item("properties", properties_dict)?;
        }
        if let Some(bbox) = &self.bbox {
            dict.set_item("bbox", bbox)?;
        }

        match self.geometry {
            GeometryType::GeometryCollection { geometries } => {
                dict.set_item("type", "GeometryCollection")?;
                dict.set_item("geometries", geometries)?;
            }
            GeometryType::Point { coordinates } => {
                dict.set_item("type", "Point")?;
                dict.set_item("coordinates", coordinates)?;
            }
            GeometryType::MultiPoint { coordinates } => {
                dict.set_item("type", "MultiPoint")?;
                dict.set_item("coordinates", coordinates)?;
            }
            GeometryType::LineString { arcs } => {
                dict.set_item("type", "LineString")?;
                dict.set_item("arcs", arcs)?;
            }
            GeometryType::MultiLineString { arcs } => {
                dict.set_item("type", "MultiLineString")?;
                dict.set_item("arcs", arcs)?;
            }
            GeometryType::Polygon { arcs } => {
                dict.set_item("type", "Polygon")?;
                dict.set_item("arcs", arcs)?;
            }
            GeometryType::MultiPolygon { arcs } => {
                dict.set_item("type", "MultiPolygon")?;
                dict.set_item("arcs", arcs)?;
            }
        }
        Ok(dict)
    }
}

impl<'py> IntoPyObject<'py> for &Geometry {
    type Target = PyDict;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dict = PyDict::new(py);
        if let Some(id) = &self.id {
            dict.set_item("id", id)?;
        }
        if let Some(properties) = &self.properties {
            let properties_dict = PyDict::new(py);
            properties_dict.set_item("name", properties.name.clone())?;
            dict.set_item("properties", properties_dict)?;
        }
        if let Some(bbox) = &self.bbox {
            dict.set_item("bbox", bbox)?;
        }

        match &self.geometry {
            GeometryType::GeometryCollection { geometries } => {
                dict.set_item("type", "GeometryCollection")?;
                dict.set_item("arcs", geometries)?;
            }
            GeometryType::Point { coordinates } => {
                dict.set_item("type", "Point")?;
                dict.set_item("coordinates", coordinates)?;
            }
            GeometryType::MultiPoint { coordinates } => {
                dict.set_item("type", "MultiPoint")?;
                dict.set_item("coordinates", coordinates)?;
            }
            GeometryType::LineString { arcs } => {
                dict.set_item("type", "LineString")?;
                dict.set_item("arcs", arcs)?;
            }
            GeometryType::MultiLineString { arcs } => {
                dict.set_item("type", "MultiLineString")?;
                dict.set_item("arcs", arcs)?;
            }
            GeometryType::Polygon { arcs } => {
                dict.set_item("type", "Polygon")?;
                dict.set_item("arcs", arcs)?;
            }
            GeometryType::MultiPolygon { arcs } => {
                dict.set_item("type", "MultiPolygon")?;
                dict.set_item("arcs", arcs)?;
            }
        }
        Ok(dict)
    }
}

#[derive(Debug, Clone, PartialEq)]
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
