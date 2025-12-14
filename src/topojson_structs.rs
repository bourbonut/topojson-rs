use std::{collections::HashMap, usize};

use pyo3::{
    PyErr,
    exceptions::PyKeyError,
    prelude::*,
    types::{PyDict, PyList},
};

#[derive(Debug)]
pub struct Compact<T, const N: usize> {
    value: T,
    indices: [usize; N],
}

#[derive(Debug)]
pub struct TopoJSON {
    pub r#type: String,
    pub bbox: Vec<f64>,
    pub transform: Option<Transform>,
    pub objects: HashMap<String, Geometry>,
    pub arcs: Vec<Compact<i32, 2>>,
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
        // let arcs = dict
        //     .get_item("arcs")?
        //     .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in the topojson"))?
        //     .extract()?;
        let any = dict
            .get_item("arcs")?
            .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in the topojson"))?;
        let pyarcs: &Bound<'_, PyList> = any.cast()?;
        let mut arcs = Vec::new();
        for pylevel2 in pyarcs.iter() {
            let level2: &Bound<'_, PyList> = pylevel2.cast()?;
            for (i, pylevel3) in level2.iter().enumerate() {
                let level3: &Bound<'_, PyList> = pylevel3.cast()?;
                for (k, pylevel4) in level3.iter().enumerate() {
                    let value = pylevel4.extract()?;
                    arcs.push(Compact {
                        value,
                        indices: [i, k],
                    });
                }
            }
        }
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
    pub scale: Vec<f64>,
    pub translate: Vec<f64>,
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
    Point { coordinates: Vec<f64> },
    MultiPoint { coordinates: Vec<Compact<f64, 1>> },
    LineString { arcs: Vec<i32> },
    MultiLineString { arcs: Vec<Compact<i32, 1>> },
    Polygon { arcs: Vec<Compact<i32, 1>> },
    MultiPolygon { arcs: Vec<Compact<i32, 2>> },
}

#[derive(Debug)]
pub struct Geometry {
    pub r#type: String,
    pub geometry: GeometryType,
    pub id: Option<String>,
    pub properties: Option<Properties>,
    pub bbox: Option<Vec<f64>>,
}

// fn extract1<'a, T: FromPyObject<'a, 'a>>(dict: &Bound<'a, PyDict>) -> PyResult<Vec<Compact<T, 1>>>
// where
//     PyErr: From<<T as FromPyObject<'a, 'a>>::Error>,
// {
//     let any = dict
//         .get_item("coordinates")?
//         .ok_or_else(|| PyKeyError::new_err("\"coordinates\" not found in \"geometry\""))?;
//     let list: &Bound<'_, PyList> = any.cast()?;
//     let mut compacts = Vec::new();
//     for pylevel2 in list {
//         let level2: &Bound<'_, PyList> = pylevel2.cast()?;
//         for (i, pylevel3) in level2.iter().enumerate() {
//             compacts.push(Compact {
//                 value: pylevel3.extract()?,
//                 indices: [i],
//             });
//         }
//     }
//     Ok(compacts)
// }
//
// fn extract2<'a, T: FromPyObject<'a, 'a>>(dict: &Bound<'a, PyDict>) -> PyResult<Vec<Compact<T, 2>>>
// where
//     pyo3::PyErr: From<<T as pyo3::FromPyObject<'a, 'a>>::Error>,
// {
//     let any = dict
//         .get_item("coordinates")?
//         .ok_or_else(|| PyKeyError::new_err("\"coordinates\" not found in \"geometry\""))?;
//
//     let list: &Bound<'a, PyList> = any.cast()?;
//     let mut compacts = Vec::new();
//     for pylevel2 in list {
//         let level2: &Bound<'a, PyList> = pylevel2.cast()?;
//         for (i, pylevel3) in level2.iter().enumerate() {
//             let level3: &Bound<'a, PyList> = pylevel3.cast()?;
//             for (j, pylevel4) in level3.iter().enumerate() {
//                 compacts.push(Compact {
//                     value: pylevel4.extract()?,
//                     indices: [i, j],
//                 });
//             }
//         }
//     }
//     Ok(compacts)
// }

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
                coordinates: {
                    let any = dict.get_item("coordinates")?.ok_or_else(|| {
                        PyKeyError::new_err("\"coordinates\" not found in \"geometry\"")
                    })?;
                    let list: &Bound<'_, PyList> = any.cast()?;
                    let mut compacts = Vec::new();
                    for pylevel2 in list {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (i, pylevel3) in level2.iter().enumerate() {
                            compacts.push(Compact {
                                value: pylevel3.extract()?,
                                indices: [i],
                            });
                        }
                    }
                    compacts
                },
            },
            "LineString" => GeometryType::LineString {
                arcs: dict
                    .get_item("arcs")?
                    .ok_or_else(|| PyKeyError::new_err("\"arcs\" not found in \"geometry\""))?
                    .extract()?,
            },
            "MultiLineString" => GeometryType::MultiLineString {
                arcs: {
                    let any = dict.get_item("arcs")?.ok_or_else(|| {
                        PyKeyError::new_err("\"coordinates\" not found in \"geometry\"")
                    })?;
                    let list: &Bound<'_, PyList> = any.cast()?;
                    let mut compacts = Vec::new();
                    for pylevel2 in list {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (i, pylevel3) in level2.iter().enumerate() {
                            compacts.push(Compact {
                                value: pylevel3.extract()?,
                                indices: [i],
                            });
                        }
                    }
                    compacts
                },
            },
            "Polygon" => GeometryType::Polygon {
                arcs: {
                    let any = dict.get_item("arcs")?.ok_or_else(|| {
                        PyKeyError::new_err("\"coordinates\" not found in \"geometry\"")
                    })?;
                    let list: &Bound<'_, PyList> = any.cast()?;
                    let mut compacts = Vec::new();
                    for pylevel2 in list {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (i, pylevel3) in level2.iter().enumerate() {
                            compacts.push(Compact {
                                value: pylevel3.extract()?,
                                indices: [i],
                            });
                        }
                    }
                    compacts
                },
            },
            "MultiPolygon" => GeometryType::MultiPolygon {
                arcs: {
                    let any = dict.get_item("arcs")?.ok_or_else(|| {
                        PyKeyError::new_err("\"coordinates\" not found in \"geometry\"")
                    })?;

                    let list: &Bound<'_, PyList> = any.cast()?;
                    let mut compacts = Vec::new();
                    for pylevel2 in list {
                        let level2: &Bound<'_, PyList> = pylevel2.cast()?;
                        for (i, pylevel3) in level2.iter().enumerate() {
                            let level3: &Bound<'_, PyList> = pylevel3.cast()?;
                            for (j, pylevel4) in level3.iter().enumerate() {
                                compacts.push(Compact {
                                    value: pylevel4.extract()?,
                                    indices: [i, j],
                                });
                            }
                        }
                    }
                    compacts
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
