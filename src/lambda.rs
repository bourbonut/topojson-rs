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
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError, PyZeroDivisionError};
use pyo3::prelude::*;

#[derive(Debug, Clone)]
pub enum GeoVarEnum {
    NoChange,
    Ops(Vec<Ops>),
    Eq([Box<GeoVarEnum>; 2]),
    Neq([Box<GeoVarEnum>; 2]),
    And([Box<GeoVarEnum>; 2]),
    Or([Box<GeoVarEnum>; 2]),
}

#[pyclass]
#[derive(Clone, Debug)]
pub enum Transform {
    AsI64,
    AsF64,
    Length,
}

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

impl GeoVarEnum {
    fn compare(&self, geom1: &Geometry, geom2: &Geometry) -> PyResult<Value> {
        match self {
            Self::Eq(vars) => {
                Self::cmp_vars(vars, geom1, geom2, |g1, g2| g1 == g2, |v1, v2| v1 == v2)
            }
            Self::Neq(vars) => {
                Self::cmp_vars(vars, geom1, geom2, |g1, g2| g1 != g2, |v1, v2| v1 != v2)
            }
            Self::And(vars) => {
                Self::cmp_vars(vars, geom1, geom2, |_, _| true, |v1, v2| v1.and(&v2))
            }
            Self::Or(vars) => Self::cmp_vars(vars, geom1, geom2, |_, _| true, |v1, v2| v1.or(&v2)),
            Self::NoChange => Ok(Value::Bool(true)),
            Self::Ops(_) => Err(PyRuntimeError::new_err(
                "Cannot compare geometries without two distinct values.",
            )),
        }
    }

    fn cmp_vars<G, V>(
        vars: &[Box<GeoVarEnum>; 2],
        geom1: &Geometry,
        geom2: &Geometry,
        geom_fn: G,
        value_fn: V,
    ) -> PyResult<Value>
    where
        G: FnOnce(&Geometry, &Geometry) -> bool,
        V: FnOnce(Value, Value) -> bool,
    {
        let [var1, var2] = vars;
        match [var1.as_ref(), var2.as_ref()] {
            [GeoVarEnum::NoChange, GeoVarEnum::NoChange] => Ok(Value::Bool(geom_fn(geom1, geom2))),
            [GeoVarEnum::NoChange, _] | [_, GeoVarEnum::NoChange] => Ok(Value::Bool(false)),
            [GeoVarEnum::Ops(ops1), GeoVarEnum::Ops(ops2)] => Ok(Value::Bool(value_fn(
                geom1.process(ops1)?,
                geom2.process(ops2)?,
            ))),
            _ => Ok(Value::Bool(value_fn(
                var1.as_ref().compare(geom1, geom2)?,
                var2.as_ref().compare(geom1, geom2)?,
            ))),
        }
    }
}

#[pyfunction]
pub fn var() -> GeoVar {
    GeoVar::new()
}

#[pyclass]
#[derive(Debug)]
pub struct GeoVar {
    inner: GeoVarEnum,
}

#[pymethods]
impl GeoVar {
    #[new]
    pub fn new() -> Self {
        Self {
            inner: GeoVarEnum::NoChange,
        }
    }

    pub fn __getitem__(&self, key: &str) -> PyResult<GeoVar> {
        match self.inner {
            GeoVarEnum::NoChange => Ok(GeoVar {
                inner: GeoVarEnum::Ops(vec![Ops::ItemGetter(key.to_string())]),
            }),
            _ => Err(PyTypeError::new_err(
                "The method 'attribute' must be called only on 'GeoVarEnum::NoChange' variant",
            )),
        }
    }

    fn transform(&self, transform: Transform) -> PyResult<GeoVar> {
        match &self.inner {
            GeoVarEnum::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::Transform(transform));
                Ok(GeoVar {
                    inner: GeoVarEnum::Ops(cloned),
                })
            }
            _ => Err(PyTypeError::new_err(
                "The method 'transform' must be called only on 'GeoVarEnum::Ops' variant",
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
        self.ops(other, |v| Ops::AddI64(v), |v| Ops::AddF64(v), "__add__")
    }

    pub fn __sub__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        self.ops(other, |v| Ops::SubI64(v), |v| Ops::SubF64(v), "__sub__")
    }

    pub fn __mul__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        self.ops(other, |v| Ops::MulI64(v), |v| Ops::MulF64(v), "__mul__")
    }

    pub fn __truediv__(&self, other: &Bound<'_, PyAny>) -> PyResult<GeoVar> {
        self.ops(other, |v| Ops::DivI64(v), |v| Ops::DivF64(v), "__div__")
    }

    pub fn __eq__(&self, other: &GeoVar) -> Self {
        self.cmp(other, |vars| GeoVarEnum::Eq(vars))
    }

    pub fn __ne__(&self, other: &GeoVar) -> Self {
        self.cmp(other, |vars| GeoVarEnum::Neq(vars))
    }

    pub fn __and__(&self, other: &GeoVar) -> Self {
        self.cmp(other, |vars| GeoVarEnum::And(vars))
    }

    pub fn __or__(&self, other: &GeoVar) -> Self {
        self.cmp(other, |vars| GeoVarEnum::Or(vars))
    }

    pub fn __str__(&self) -> String {
        format!("{:?}", self)
    }
}

impl GeoVar {
    pub(crate) fn compare(&self, geom1: &Geometry, geom2: &Geometry) -> PyResult<Value> {
        self.inner.compare(geom1, geom2)
    }

    fn cmp<F>(&self, other: &GeoVar, f: F) -> Self
    where
        F: FnOnce([Box<GeoVarEnum>; 2]) -> GeoVarEnum,
    {
        Self {
            inner: f([Box::new(self.inner.clone()), Box::new(other.inner.clone())]),
        }
    }

    fn ops<I, F>(
        &self,
        other: &Bound<'_, PyAny>,
        ops_i32: I,
        ops_f32: F,
        method_name: &str,
    ) -> PyResult<Self>
    where
        I: FnOnce(i64) -> Ops,
        F: FnOnce(f64) -> Ops,
    {
        match &self.inner {
            GeoVarEnum::Ops(ops) => {
                let mut cloned = ops.clone();
                if let Ok(extracted) = other.extract() {
                    cloned.push(ops_i32(extracted));
                } else if let Ok(extracted) = other.extract() {
                    cloned.push(ops_f32(extracted));
                } else {
                    return Err(PyTypeError::new_err("Expected an integer or a float."));
                }
                Ok(GeoVar {
                    inner: GeoVarEnum::Ops(cloned),
                })
            }
            _ => Err(PyTypeError::new_err(format!(
                "The method '{method_name}' must be called only on 'GeoVarEnum::Ops' variant"
            ))),
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

    fn op_i64<I, F>(self, other: i64, int_fn: I, float_fn: F) -> Self
    where
        I: FnOnce(i64, i64) -> i64,
        F: FnOnce(f64, f64) -> f64,
    {
        match self {
            Self::Int(x) => Self::Int(int_fn(x, other)),
            Self::Float(x) => Self::Float(float_fn(x, other as f64)),
            Self::Bool(x) => Self::Int(int_fn(x as i64, other)),
        }
    }

    fn op_f64<F>(self, other: f64, f: F) -> Self
    where
        F: FnOnce(f64, f64) -> f64,
    {
        let val = match self {
            Self::Int(x) => x as f64,
            Self::Float(x) => x,
            Self::Bool(x) => x as i64 as f64,
        };
        Self::Float(f(val, other))
    }

    fn add_i64(self, other: i64) -> Self {
        self.op_i64(other, |a, b| a + b, |a, b| a + b)
    }

    fn add_f64(self, other: f64) -> Self {
        self.op_f64(other, |a, b| a + b)
    }

    fn sub_i64(self, other: i64) -> Self {
        self.op_i64(other, |a, b| a - b, |a, b| a - b)
    }

    fn sub_f64(self, other: f64) -> Self {
        self.op_f64(other, |a, b| a - b)
    }

    fn mul_i64(self, other: i64) -> Self {
        self.op_i64(other, |a, b| a * b, |a, b| a * b)
    }

    fn mul_f64(self, other: f64) -> Self {
        self.op_f64(other, |a, b| a * b)
    }

    fn div_i64(self, other: i64) -> PyResult<Self> {
        match other {
            0 => Err(PyZeroDivisionError::new_err("division by zero")),
            _ => Ok(self.op_i64(other, |a, b| a / b, |a, b| a / b)),
        }
    }

    fn div_f64(self, other: f64) -> PyResult<Self> {
        match other {
            0.0 => Err(PyZeroDivisionError::new_err("division by zero")),
            _ => Ok(self.op_f64(other, |a, b| a / b)),
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
                Ops::DivI64(other) => value.div_i64(other)?,
                Ops::DivF64(other) => value.div_f64(other)?,
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
