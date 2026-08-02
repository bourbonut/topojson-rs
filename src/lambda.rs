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
//           Ops:ItemGetter("id"),
//           Ops::Transform(Transform::ParseI64)
//           Ops::DivI64(1000),
//           Ops::Transform(Transform::ParseI64)
//       ]),
//       GeoVar::Ops(vec![
//           Ops:ItemGetter("id"),
//           Ops::Transform(Transform::ParseI64)
//           Ops::DivI64(1000),
//           Ops::Transform(Transform::ParseI64)
//       ]),
//   )
// )
// ```

use std::collections::VecDeque;
use std::num::{ParseFloatError, ParseIntError};

use crate::topojsons::Geometry;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyFloat, PyInt};

// Unfortunately, I didn't find a better optimized organization for `Eq`, `Neq`, `And` and `Or`
// variants due to pyo3's constraints.
#[pyclass]
#[derive(Debug, Clone)]
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
    AsI64,
    AsF64,
    Length,
}

#[pyclass]
#[derive(Debug, Clone)]
pub enum Ops {
    ItemGetter(String),
    Transform(Transform),
    AddI64(i64),
    AddF64(f64),
    SubI64(i64),
    SubF64(f64),
    MulI64(i64),
    MulF64(f64),
    DivI64(i64),
    DivF64(f64),
}

#[pymethods]
impl GeoVar {
    #[new]
    pub fn new() -> Self {
        Self::NoChange()
    }

    fn __getitem__(&self, attribute: String) -> PyResult<GeoVar> {
        match self {
            Self::NoChange() => Ok(GeoVar::Ops(vec![Ops::ItemGetter(attribute)])),
            _ => Err(PyTypeError::new_err(
                "The method 'attribute' must be called only on 'Geovar::NoChange' variant",
            )),
        }
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

    pub fn int(&self) -> PyResult<GeoVar> {
        self.transform(Transform::AsI64)
    }

    pub fn float(&self) -> PyResult<GeoVar> {
        self.transform(Transform::AsF64)
    }

    pub fn len(&self) -> PyResult<GeoVar> {
        self.transform(Transform::Length)
    }

    pub fn __add__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                if let Ok(value) = other.cast::<PyInt>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::AddI64(extracted));
                } else if let Ok(value) = other.cast::<PyFloat>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::AddF64(extracted));
                } else {
                    return Err(PyTypeError::new_err("Expected an integer or a float."));
                }
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method '__add__' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                if let Ok(value) = other.cast::<PyInt>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::SubI64(extracted));
                } else if let Ok(value) = other.cast::<PyFloat>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::SubF64(extracted));
                } else {
                    return Err(PyTypeError::new_err("Expected an integer or a float."));
                }
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method '__sub__' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                if let Ok(value) = other.cast::<PyInt>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::MulI64(extracted));
                } else if let Ok(value) = other.cast::<PyFloat>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::MulF64(extracted));
                } else {
                    return Err(PyTypeError::new_err("Expected an integer or a float."));
                }
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method '__mul__' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                if let Ok(value) = other.cast::<PyInt>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::DivI64(extracted));
                } else if let Ok(value) = other.cast::<PyFloat>() {
                    let extracted = value.extract()?;
                    cloned.push(Ops::DivF64(extracted));
                } else {
                    return Err(PyTypeError::new_err("Expected an integer or a float."));
                }
                Ok(GeoVar::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method '__div__' must be called only on 'Geovar::Ops' variant",
            )),
        }
    }

    pub fn __eq__(&self, other: &GeoVar) -> Self {
        GeoVar::Eq(vec![self.clone(), other.clone()])
    }

    pub fn __ne__(&self, other: &GeoVar) -> Self {
        GeoVar::Neq(vec![self.clone(), other.clone()])
    }

    pub fn __and__(&self, other: &GeoVar) -> Self {
        GeoVar::And(vec![self.clone(), other.clone()])
    }

    pub fn __or__(&self, other: &GeoVar) -> Self {
        GeoVar::Or(vec![self.clone(), other.clone()])
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

impl GeoVar {
    pub(crate) fn compare(&self, geom1: &Geometry, geom2: &Geometry) -> PyResult<Value> {
        match self {
            Self::Eq(vec) => {
                if let [var1, var2] = vec.as_slice() {
                    match [var1, var2] {
                        [Self::NoChange(), Self::NoChange()] => Ok(Value::Bool(geom1 == geom2)),
                        [Self::NoChange(), _] | [_, Self::NoChange()] => Ok(Value::Bool(false)),
                        [Self::Ops(ops1), Self::Ops(ops2)] => {
                            Ok(Value::Bool(geom1.process(ops1)? == geom2.process(ops2)?))
                        }
                        _ => Ok(Value::Bool(
                            var1.compare(geom1, geom2)? == var2.compare(geom1, geom2)?,
                        )),
                    }
                } else {
                    return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
                }
            }
            Self::Neq(vec) => {
                if let [var1, var2] = vec.as_slice() {
                    match [var1, var2] {
                        [Self::NoChange(), Self::NoChange()] => Ok(Value::Bool(geom1 != geom2)),
                        [Self::NoChange(), _] | [_, Self::NoChange()] => Ok(Value::Bool(false)),
                        [Self::Ops(ops1), Self::Ops(ops2)] => {
                            Ok(Value::Bool(geom1.process(ops1)? != geom2.process(ops2)?))
                        }
                        _ => Ok(Value::Bool(
                            var1.compare(geom1, geom2)? != var2.compare(geom1, geom2)?,
                        )),
                    }
                } else {
                    return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
                }
            }
            Self::And(vec) => {
                if let [var1, var2] = vec.as_slice() {
                    match [var1, var2] {
                        [Self::NoChange(), Self::NoChange()] => Ok(Value::Bool(true)),
                        [Self::NoChange(), _] | [_, Self::NoChange()] => Ok(Value::Bool(true)),
                        [Self::Ops(ops1), Self::Ops(ops2)] => {
                            Ok(Value::Bool(geom1.process(ops1)?.and(&geom2.process(ops2)?)))
                        }
                        _ => Ok(Value::Bool(
                            var1.compare(geom1, geom2)?
                                .and(&var2.compare(geom1, geom2)?),
                        )),
                    }
                } else {
                    return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
                }
            }
            Self::Or(vec) => {
                if let [var1, var2] = vec.as_slice() {
                    match [var1, var2] {
                        [Self::NoChange(), Self::NoChange()] => Ok(Value::Bool(true)),
                        [Self::NoChange(), _] | [_, Self::NoChange()] => Ok(Value::Bool(true)),
                        [Self::Ops(ops1), Self::Ops(ops2)] => {
                            Ok(Value::Bool(geom1.process(ops1)?.or(&geom2.process(ops2)?)))
                        }
                        _ => Ok(Value::Bool(
                            var1.compare(geom1, geom2)?.or(&var2.compare(geom1, geom2)?),
                        )),
                    }
                } else {
                    return Err(PyRuntimeError::new_err("Cannot compare more than two vars"));
                }
            }
            Self::NoChange() => Ok(Value::Bool(true)),
            Self::Ops(_) => Err(PyRuntimeError::new_err(
                "Cannot compare geometries without two distinct values.",
            )),
        }
    }
}

#[derive(Debug)]
pub(crate) enum Value {
    Float(f64),
    Int(i64),
    Bool(bool),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match [self, other] {
            [Self::Int(a), Self::Int(b)] => a == b,
            [Self::Float(a), Self::Float(b)] => a == b,
            [Self::Bool(a), Self::Bool(b)] => a == b,
            [Self::Float(a), Self::Int(b)] => *a == *b as f64,
            [Self::Int(a), Self::Float(b)] => *a as f64 == *b,
            _ => false,
        }
    }

    fn ne(&self, other: &Self) -> bool {
        match [self, other] {
            [Self::Int(a), Self::Int(b)] => a != b,
            [Self::Float(a), Self::Float(b)] => a != b,
            [Self::Bool(a), Self::Bool(b)] => a != b,
            [Self::Float(a), Self::Int(b)] => *a != *b as f64,
            [Self::Int(a), Self::Float(b)] => *a as f64 != *b,
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

    fn as_i64(self) -> Self {
        match self {
            Self::Int(x) => Self::Int(x),
            Self::Bool(x) => Self::Int(x as i64),
            Self::Float(x) => Self::Int(x as i64),
        }
    }

    fn as_f64(self) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f64),
            Self::Bool(x) => Self::Float(x as i64 as f64),
            Self::Float(x) => Self::Float(x),
        }
    }

    fn add_i64(self, other: i64) -> Self {
        match self {
            Self::Int(x) => Self::Int(x + other),
            Self::Float(x) => Self::Float(x + other as f64),
            Self::Bool(x) => Self::Int(x as i64 + other),
        }
    }

    fn add_f64(self, other: f64) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f64 + other),
            Self::Float(x) => Self::Float(x + other),
            Self::Bool(x) => Self::Float(x as i64 as f64 + other),
        }
    }

    fn sub_i64(self, other: i64) -> Self {
        match self {
            Self::Int(x) => Self::Int(x - other),
            Self::Float(x) => Self::Float(x - other as f64),
            Self::Bool(x) => Self::Int(x as i64 - other),
        }
    }

    fn sub_f64(self, other: f64) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f64 - other),
            Self::Float(x) => Self::Float(x - other),
            Self::Bool(x) => Self::Float(x as i64 as f64 - other),
        }
    }

    fn mul_i64(self, other: i64) -> Self {
        match self {
            Self::Int(x) => Self::Int(x * other),
            Self::Float(x) => Self::Float(x * other as f64),
            Self::Bool(x) => Self::Int(x as i64 * other),
        }
    }

    fn mul_f64(self, other: f64) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f64 * other),
            Self::Float(x) => Self::Float(x * other),
            Self::Bool(x) => Self::Float(x as i64 as f64 * other),
        }
    }

    fn div_i64(self, other: i64) -> Self {
        match self {
            Self::Int(x) => Self::Int(x / other),
            Self::Float(x) => Self::Float(x / other as f64),
            Self::Bool(x) => Self::Int(x as i64 / other),
        }
    }

    fn div_f64(self, other: f64) -> Self {
        match self {
            Self::Int(x) => Self::Float(x as f64 / other),
            Self::Float(x) => Self::Float(x / other),
            Self::Bool(x) => Self::Float(x as i64 as f64 / other),
        }
    }

    fn value_type(&self) -> &str {
        match self {
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::Bool(_) => "bool",
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

impl Geometry {
    fn transform_id(&self, transform: &Transform) -> PyResult<Value> {
        let id_str = self.id().ok_or_else(|| {
            PyTypeError::new_err(
                "argument must be a string, a bytes-like object or a real number, not 'NoneType'",
            )
        })?;

        match transform {
            Transform::AsI64 => id_str
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|e: ParseIntError| PyValueError::new_err(e.to_string())),

            Transform::AsF64 => id_str
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|e: ParseFloatError| PyValueError::new_err(e.to_string())),

            Transform::Length => Ok(Value::Int(id_str.len() as i64)),
        }
    }

    fn transform_bbox(&self, transform: &Transform) -> PyResult<Value> {
        let bbox = self.bbox().ok_or_else(|| {
            PyTypeError::new_err(
                "argument must be a string, a bytes-like object or a real number, not 'NoneType'",
            )
        })?;

        match transform {
            Transform::AsI64 => Err(PyTypeError::new_err(
                "int() argument must be a string, a bytes-like object or a real number, not 'list'",
            )),
            Transform::AsF64 => Err(PyTypeError::new_err(
                "float() argument must be a string or a real number, not 'list'",
            )),
            Transform::Length => Ok(Value::Int(bbox.len() as i64)),
        }
    }

    fn transform_properties(&self, transform: &Transform) -> PyResult<Value> {
        let properties_str = self.properties().ok_or_else(|| {
            PyTypeError::new_err(
                "argument must be a string, a bytes-like object or a real number, not 'NoneType'",
            )
        })?;

        match transform {
            Transform::AsI64 => properties_str
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|e: ParseIntError| PyValueError::new_err(e.to_string())),

            Transform::AsF64 => properties_str
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|e: ParseFloatError| PyValueError::new_err(e.to_string())),

            Transform::Length => Ok(Value::Int(properties_str.len() as i64)),
        }
    }

    fn preprocess(&self, queue: &mut VecDeque<&Ops>) -> PyResult<Value> {
        let op = queue.pop_front().ok_or_else(|| {
            PyRuntimeError::new_err("The vector of operations must never be empty.")
        })?;

        let key = match op {
            Ops::ItemGetter(k) => k,
            _ => {
                return Err(PyValueError::new_err(
                    "First operator must be 'Ops::ItemGetter'. Use 'my_var[my_key]'.",
                ));
            }
        };

        let transform = match queue.pop_front() {
            Some(Ops::Transform(t)) => t,
            _ => {
                return Err(PyValueError::new_err(concat!(
                    "A 'Ops::ItemGetter' must be followed by a 'Ops::Transform' operator. ",
                    "Use 'obj.int()', 'obj.float()' or 'obj.len()'.",
                )));
            }
        };

        match key.as_str() {
            "id" => self.transform_id(transform),
            "properties" => self.transform_properties(transform),
            "bbox" => self.transform_bbox(transform),
            _ => Err(PyValueError::new_err(format!(
                "Unknown or not implemented key {:?}",
                key
            ))),
        }
    }

    fn process(&self, ops: &Vec<Ops>) -> PyResult<Value> {
        let mut queue = VecDeque::from_iter(ops);
        let mut value = self.preprocess(&mut queue)?;
        while let Some(op) = queue.pop_front() {
            value = match *op {
                Ops::AddI64(other) => value.add_i64(other),
                Ops::AddF64(other) => value.add_f64(other),
                Ops::SubI64(other) => value.sub_i64(other),
                Ops::SubF64(other) => value.sub_f64(other),
                Ops::MulI64(other) => value.mul_i64(other),
                Ops::MulF64(other) => value.mul_f64(other),
                Ops::DivI64(other) => value.div_i64(other),
                Ops::DivF64(other) => value.div_f64(other),
                Ops::Transform(Transform::AsI64) => value.as_i64(),
                Ops::Transform(Transform::AsF64) => value.as_f64(),
                Ops::Transform(Transform::Length) => {
                    return Err(PyTypeError::new_err(format!(
                        "object of type '{}' has no len()",
                        value.value_type()
                    )));
                }
                Ops::ItemGetter(_) => {
                    return Err(PyTypeError::new_err(format!(
                        "'{}' object is not subcriptable",
                        value.value_type()
                    )));
                }
            }
        }
        Ok(value)
    }
}
