// Example
//
// ```python
// def filter_func(a: Geometry, b: Geometry):
//     return a != b and int(int(a["id"]) / 1000) == int(int(b["id"]) / 1000)
// ```
//
// is equivalent to
//
// ```rust
// GeoVar::And(
//   GeoVar::Neq(GeoVar::NoChange, GeoVar::NoChange),
//   GeoVar::Eq(
//       GeoVar::Ops(vec![
//           Ops::Attr("id", int),
//           Ops::DivI32(1000),
//           Ops::Transform(Transform::ParseI32)
//       ]),
//       GeoVar::Ops(vec![
//           Ops::Attr("id", int),
//           Ops::DivI32(1000),
//           Ops::Transform(Transform::ParseI32)
//       ]),
//   )
// )
// ```

use std::collections::VecDeque;
use std::num::{ParseFloatError, ParseIntError};

use crate::topojsons::Geometry;
use pyo3::exceptions::{PyAttributeError, PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;

// Unfortunately, I didn't find a better optimized organization for `Eq`, `Neq`, `And` and `Or`
// variants due to pyo3's constraints.
#[pyclass]
#[derive(Clone)]
pub enum GeoVar {
    NoChange(),
    Ops(Vec<Ops>),
    Eq(Vec<GeoVar>),
    Neq(Vec<GeoVar>),
    And(Vec<GeoVar>),
    Or(Vec<GeoVar>),
}

#[pyclass]
#[derive(Clone, Debug)]
pub enum Transform {
    ParseI32,
    ParseF32,
    Length,
}

#[pyclass]
#[derive(Clone)]
pub enum Ops {
    Attr(String, Transform),
    Transform(Transform),
    AddI32(i32),
    AddF32(f32),
    SubI32(i32),
    SubF32(f32),
    MulI32(i32),
    MulF32(f32),
    DivI32(i32),
    DivF32(f32),
}

#[pymethods]
impl GeoVar {
    #[new]
    pub fn new() -> Self {
        Self::NoChange()
    }

    fn attribute(&self, attribute: String, transform: Transform) -> PyResult<GeoVar> {
        match self {
            Self::NoChange() => Ok(GeoVar::Ops(vec![Ops::Attr(attribute, transform)])),
            _ => Err(PyTypeError::new_err(
                "The method 'attribute' must be called only on 'Geovar::NoChange' variant",
            )),
        }
    }

    pub fn attribute_i32(&self, attribute: String) -> PyResult<GeoVar> {
        self.attribute(attribute, Transform::ParseI32)
    }

    pub fn attribute_f32(&self, attribute: String) -> PyResult<GeoVar> {
        self.attribute(attribute, Transform::ParseF32)
    }

    pub fn attribute_len(&self, attribute: String) -> PyResult<GeoVar> {
        self.attribute(attribute, Transform::Length)
    }

    fn transform(&self, transform: Transform) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::Transform(transform));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'transform' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn as_i32(&self) -> PyResult<GeoVar> {
        self.transform(Transform::ParseI32)
    }

    pub fn as_f32(&self) -> PyResult<GeoVar> {
        self.transform(Transform::ParseF32)
    }

    pub fn len(&self) -> PyResult<GeoVar> {
        self.transform(Transform::Length)
    }

    pub fn add_i32(&self, value: i32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::AddI32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'add_i32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn add_f32(&self, value: f32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::AddF32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'add_f32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn sub_i32(&self, value: i32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::SubI32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'sub_i32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn sub_f32(&self, value: f32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::SubF32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'sub_f32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn mul_i32(&self, value: i32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::MulI32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'mul_i32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn mul_f32(&self, value: f32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::MulF32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'mul_f32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn div_i32(&self, value: i32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::DivI32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'div_i32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn div_f32(&self, value: f32) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::DivF32(value));
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'div_f32' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn ops_eq(&self, other: &GeoVar) -> Self {
        GeoVar::Eq(vec![self.clone(), other.clone()])
    }

    pub fn ops_neq(&self, other: &GeoVar) -> Self {
        GeoVar::Neq(vec![self.clone(), other.clone()])
    }

    pub fn ops_and(&self, other: &GeoVar) -> Self {
        GeoVar::And(vec![self.clone(), other.clone()])
    }

    pub fn ops_or(&self, other: &GeoVar) -> Self {
        GeoVar::Or(vec![self.clone(), other.clone()])
    }
}

#[derive(Debug)]
pub(crate) enum Value {
    Float(f32),
    Int(i32),
    Bool(bool),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match [self, other] {
            [Self::Int(a), Self::Int(b)] => a == b,
            [Self::Float(a), Self::Float(b)] => a == b,
            [Self::Bool(a), Self::Bool(b)] => a == b,
            [Self::Float(a), Self::Int(b)] => *a == *b as f32,
            [Self::Int(a), Self::Float(b)] => *a as f32 == *b,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match [self, other] {
            [Self::Int(a), Self::Int(b)] => a != b,
            [Self::Float(a), Self::Float(b)] => a != b,
            [Self::Bool(a), Self::Bool(b)] => a != b,
            [Self::Float(a), Self::Int(b)] => *a != *b as f32,
            [Self::Int(a), Self::Float(b)] => *a as f32 != *b,
            _ => false,
        }
    }
}

impl Value {
    fn and(&self, other: &Self) -> bool {
        match [self, other] {
            [Self::Int(a), Self::Int(b)] => *a != 0 && *b != 0,
            [Self::Float(a), Self::Float(b)] => *a != 0.0 && *b != 0.0,
            [Self::Bool(a), Self::Bool(b)] => *a && *b,
            [Self::Float(a), Self::Int(b)] => *a != 0.0 && *b != 0,
            [Self::Int(a), Self::Float(b)] => *a != 0 && *b != 0.0,
            [Self::Bool(a), Self::Float(b)] => *a && *b != 0.0,
            [Self::Float(a), Self::Bool(b)] => *a != 0.0 && *b,
            [Self::Bool(a), Self::Int(b)] => *a && *b != 0,
            [Self::Int(a), Self::Bool(b)] => *a != 0 && *b,
        }
    }

    fn or(&self, other: &Self) -> bool {
        match [self, other] {
            [Self::Int(a), Self::Int(b)] => *a != 0 || *b != 0,
            [Self::Float(a), Self::Float(b)] => *a != 0.0 || *b != 0.0,
            [Self::Bool(a), Self::Bool(b)] => *a || *b,
            [Self::Float(a), Self::Int(b)] => *a != 0.0 || *b != 0,
            [Self::Int(a), Self::Float(b)] => *a != 0 || *b != 0.0,
            [Self::Bool(a), Self::Float(b)] => *a || *b != 0.0,
            [Self::Float(a), Self::Bool(b)] => *a != 0.0 || *b,
            [Self::Bool(a), Self::Int(b)] => *a || *b != 0,
            [Self::Int(a), Self::Bool(b)] => *a != 0 || *b,
        }
    }

    pub fn as_i32(self) -> Self {
        match self {
            Self::Int(x) => Self::Int(x),
            Self::Bool(x) => Self::Int(x as i32),
            Self::Float(x) => Self::Int(x as i32),
        }
    }

    pub fn as_f32(self) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f32),
            Self::Bool(x) => Self::Float(x as i32 as f32),
            Self::Float(x) => Self::Float(x),
        }
    }

    pub fn add_i32(self, other: i32) -> Self {
        match self {
            Self::Int(x) => Self::Int(x + other),
            Self::Float(x) => Self::Float(x + other as f32),
            Self::Bool(x) => Self::Int(x as i32 + other),
        }
    }

    pub fn add_f32(self, other: f32) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f32 + other),
            Self::Float(x) => Self::Float(x + other),
            Self::Bool(x) => Self::Float(x as i32 as f32 + other),
        }
    }

    pub fn sub_i32(self, other: i32) -> Self {
        match self {
            Self::Int(x) => Self::Int(x - other),
            Self::Float(x) => Self::Float(x - other as f32),
            Self::Bool(x) => Self::Int(x as i32 - other),
        }
    }

    pub fn sub_f32(self, other: f32) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f32 - other),
            Self::Float(x) => Self::Float(x - other),
            Self::Bool(x) => Self::Float(x as i32 as f32 - other),
        }
    }

    pub fn mul_i32(self, other: i32) -> Self {
        match self {
            Self::Int(x) => Self::Int(x * other),
            Self::Float(x) => Self::Float(x * other as f32),
            Self::Bool(x) => Self::Int(x as i32 * other),
        }
    }

    pub fn mul_f32(self, other: f32) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f32 * other),
            Self::Float(x) => Self::Float(x * other),
            Self::Bool(x) => Self::Float(x as i32 as f32 * other),
        }
    }

    pub fn div_i32(self, other: i32) -> Self {
        match self {
            Self::Int(x) => Self::Int(x / other),
            Self::Float(x) => Self::Float(x / other as f32),
            Self::Bool(x) => Self::Int(x as i32 / other),
        }
    }

    pub fn div_f32(self, other: f32) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f32 / other),
            Self::Float(x) => Self::Float(x / other),
            Self::Bool(x) => Self::Float(x as i32 as f32 / other),
        }
    }

    pub fn as_bool(self) -> bool {
        match self {
            Self::Bool(x) => x,
            Self::Int(x) => x != 0,
            Self::Float(x) => x != 0.0,
        }
    }
}

// GeoVar::And(
//   GeoVar::Neq(GeoVar::NoChange, GeoVar::NoChange),
//   GeoVar::Eq(
//       GeoVar::Ops(vec![
//           Ops::Attr("id", int),
//           Ops::DivI32(1000),
//           Ops::Transform(Transform::ParseI32)
//       ]),
//       GeoVar::Ops(vec![
//           Ops::Attr("id", int),
//           Ops::DivI32(1000),
//           Ops::Transform(Transform::ParseI32)
//       ]),
//   )
// )
pub(crate) fn geo_cmp(var: &GeoVar, geom1: &Geometry, geom2: &Geometry) -> PyResult<Value> {
    match var {
        GeoVar::Eq(vec) => {
            if let [var1, var2] = vec.as_slice() {
                match [var1, var2] {
                    [GeoVar::NoChange(), GeoVar::NoChange()] => Ok(Value::Bool(geom1 == geom2)),
                    [GeoVar::NoChange(), _] | [_, GeoVar::NoChange()] => Ok(Value::Bool(false)),
                    [GeoVar::Ops(ops1), GeoVar::Ops(ops2)] => {
                        Ok(Value::Bool(geo_ops(ops1, geom1)? == geo_ops(ops2, geom2)?))
                    }
                    _ => Ok(Value::Bool(
                        geo_cmp(var1, geom1, geom2)? == geo_cmp(var2, geom1, geom2)?,
                    )),
                }
            } else {
                return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
            }
        }
        GeoVar::Neq(vec) => {
            if let [var1, var2] = vec.as_slice() {
                match [var1, var2] {
                    [GeoVar::NoChange(), GeoVar::NoChange()] => Ok(Value::Bool(geom1 != geom2)),
                    [GeoVar::NoChange(), _] | [_, GeoVar::NoChange()] => Ok(Value::Bool(false)),
                    [GeoVar::Ops(ops1), GeoVar::Ops(ops2)] => {
                        Ok(Value::Bool(geo_ops(ops1, geom1)? != geo_ops(ops2, geom2)?))
                    }
                    _ => Ok(Value::Bool(
                        geo_cmp(var1, geom1, geom2)? != geo_cmp(var2, geom1, geom2)?,
                    )),
                }
            } else {
                return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
            }
        }
        GeoVar::And(vec) => {
            if let [var1, var2] = vec.as_slice() {
                match [var1, var2] {
                    [GeoVar::NoChange(), GeoVar::NoChange()] => Ok(Value::Bool(true)),
                    [GeoVar::NoChange(), _] | [_, GeoVar::NoChange()] => Ok(Value::Bool(true)),
                    [GeoVar::Ops(ops1), GeoVar::Ops(ops2)] => Ok(Value::Bool(
                        geo_ops(ops1, geom1)?.and(&geo_ops(ops2, geom2)?),
                    )),
                    _ => Ok(Value::Bool(
                        geo_cmp(var1, geom1, geom2)?.and(&geo_cmp(var2, geom1, geom2)?),
                    )),
                }
            } else {
                return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
            }
        }
        GeoVar::Or(vec) => {
            if let [var1, var2] = vec.as_slice() {
                match [var1, var2] {
                    [GeoVar::NoChange(), GeoVar::NoChange()] => Ok(Value::Bool(true)),
                    [GeoVar::NoChange(), _] | [_, GeoVar::NoChange()] => Ok(Value::Bool(true)),
                    [GeoVar::Ops(ops1), GeoVar::Ops(ops2)] => Ok(Value::Bool(
                        geo_ops(ops1, geom1)?.or(&geo_ops(ops2, geom2)?),
                    )),
                    _ => Ok(Value::Bool(
                        geo_cmp(var1, geom1, geom2)?.or(&geo_cmp(var2, geom1, geom2)?),
                    )),
                }
            } else {
                return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
            }
        }
        GeoVar::NoChange() => Ok(Value::Bool(true)),
        GeoVar::Ops(_) => Err(PyRuntimeError::new_err(
            "Cannot compare geometries without two distinct values.",
        )),
    }
}

fn geo_ops(ops: &Vec<Ops>, geom: &Geometry) -> PyResult<Value> {
    if ops.is_empty() {
        return Err(PyRuntimeError::new_err(
            "The vector of operations must never be empty.",
        ));
    }
    let mut queue = VecDeque::from_iter(ops);
    let op = queue.pop_front().unwrap();
    let mut value: Value = match op {
        Ops::Attr(attr, transform) => match (attr.as_str(), transform) {
            // ("geometries", Transform::Length) => match geom {
            //     Geometry::GeometryCollection { geometries, .. } => {
            //         Value::Int(geometries.len() as i32)
            //     }
            //     _ => {
            //         return Err(PyAttributeError::new_err(
            //             "The value is not a 'Geometry::GeometryCollection'",
            //         ));
            //     }
            // },
            // ("geometries", transform) => {
            //     return Err(PyRuntimeError::new_err(format!(
            //         "Unsupported conversion with 'geometries' attribute (found '{:?}')",
            //         transform
            //     )));
            // }
            ("id", Transform::ParseI32) => {
                if let Some(id) = geom.id() {
                    let value: i32 = id
                        .parse()
                        .map_err(|e: ParseIntError| PyValueError::new_err(e.to_string()))?;
                    Value::Int(value)
                } else {
                    return Err(PyTypeError::new_err(
                        "int() argument must be a string, a bytes-like object or a real number, not 'NoneType'",
                    ));
                }
            }
            ("id", Transform::ParseF32) => {
                if let Some(id) = geom.id() {
                    let value: f32 = id
                        .parse()
                        .map_err(|e: ParseFloatError| PyValueError::new_err(e.to_string()))?;
                    Value::Float(value)
                } else {
                    return Err(PyTypeError::new_err(
                        "float() argument must be a string or a real number, not 'NoneType'",
                    ));
                }
            }
            ("id", Transform::Length) => {
                if let Some(id) = geom.id() {
                    Value::Int(id.len() as i32)
                } else {
                    return Err(PyTypeError::new_err(
                        "int() argument must be a string, a bytes-like object or a real number, not 'NoneType'",
                    ));
                }
            }
            _ => return Err(PyRuntimeError::new_err("Not implemented yet")),
        },
        _ => {
            return Err(PyValueError::new_err("First operator must be 'Ops::Attr'."));
        }
    };
    while let Some(op) = queue.pop_front() {
        value = match *op {
            Ops::AddI32(other) => value.add_i32(other),
            Ops::AddF32(other) => value.add_f32(other),
            Ops::SubI32(other) => value.sub_i32(other),
            Ops::SubF32(other) => value.sub_f32(other),
            Ops::MulI32(other) => value.mul_i32(other),
            Ops::MulF32(other) => value.mul_f32(other),
            Ops::DivI32(other) => value.div_i32(other),
            Ops::DivF32(other) => value.div_f32(other),
            Ops::Transform(Transform::ParseI32) => value.as_i32(),
            Ops::Transform(Transform::ParseF32) => value.as_f32(),
            Ops::Transform(Transform::Length) => {
                return Err(PyValueError::new_err(format!(
                    "Cannot apply len() on '{:?}'",
                    value
                )));
            }
            Ops::Attr(_, _) => {
                return Err(PyAttributeError::new_err(format!(
                    "Cannot get any attribute on '{:?}'",
                    value
                )));
            }
        }
    }
    Ok(value)
}
