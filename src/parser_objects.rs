use std::collections::HashMap;

use pyo3::{
    exceptions::PyKeyError,
    prelude::*,
    types::{PyDict, PyList},
};

type VecArcs = Vec<Vec<Arcs>>;

#[derive(Debug)]
struct Arcs(Option<Vec<i32>>);

impl<'a, 'py> FromPyObject<'a, 'py> for Arcs {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if obj.is_instance_of::<PyList>() {
            let list: Borrowed<'a, 'py, PyList> = obj.cast()?;
            let arcs = list.extract()?;
            Ok(Self(Some(arcs)))
        } else {
            Ok(Self(None))
        }
    }
}

#[derive(Debug)]
pub struct TopoJSON {
    r#type: String,
    bbox: Vec<f32>,
    transform: Transform,
    objects: HashMap<String, Geometries>,
    arcs: VecArcs,
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
            .ok_or_else(|| PyKeyError::new_err("\"transform\" not found in the topojson"))?
            .extract()?;
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
struct Transform {
    scale: Vec<f32>,
    translate: Vec<f32>,
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
struct Geometries {
    r#type: String,
    geometries: Vec<Geometry>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Geometries {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
        let r#type = dict
            .get_item("type")?
            .ok_or_else(|| PyKeyError::new_err("\"type\" not found in one object"))?
            .extract()?;
        let geometries = dict
            .get_item("geometries")?
            .ok_or_else(|| PyKeyError::new_err("\"geometries\" not found in one object"))?
            .extract()?;
        Ok(Self { r#type, geometries })
    }
}

#[derive(Debug)]
struct Geometry {
    r#type: String,
    arcs: VecArcs,
    id: Option<String>,
    properties: Option<Properties>,
}

impl<'a, 'py> FromPyObject<'a, 'py> for Geometry {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let dict: Borrowed<'a, 'py, PyDict> = obj.cast()?;
        let r#type = dict
            .get_item("type")?
            .ok_or_else(|| PyKeyError::new_err("\"type\" not found in \"geometry\""))?
            .extract()?;
        let arcs = dict
            .get_item("arcs")?
            .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?
            .extract()?;
        let id = dict.get_item("id")?.map(|v| v.extract()).transpose()?;
        let properties = dict
            .get_item("properties")?
            .map(|v| v.extract())
            .transpose()?;

        Ok(Self {
            r#type,
            arcs,
            id,
            properties,
        })
    }
}

#[derive(Debug)]
struct Properties {
    name: String,
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
