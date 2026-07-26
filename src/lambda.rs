// def filter_func(a: Geometry, b: Geometry):
//     return a != b and int(int(a["id"]) / 1000) == int(int(b["id"]) / 1000)

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;

// Unfortunately, I didn't find a better optimized organization for `Eq`, `Neq`, `And` and `Or`
// variants due to pyo3's constraints.
#[pyclass]
#[derive(Clone)]
pub enum GeoElement {
    Ops(Vec<Ops>),
    Eq(Vec<GeoElement>),
    Neq(Vec<GeoElement>),
    And(Vec<GeoElement>),
    Or(Vec<GeoElement>),
}

#[pyclass]
#[derive(Clone)]
pub enum Ops {
    Attr(String),
    ParseI32(),
    ParseF32(),
    AddI32(i32),
    AddF32(f32),
    SubI32(i32),
    SubF32(f32),
    MulI32(i32),
    MulF32(f32),
    DivI32(i32),
    DivF32(f32),
}

pub fn element(attribute: String) -> GeoElement {
    GeoElement::new(attribute)
}

#[pymethods]
impl GeoElement {
    #[new]
    fn new(attribute: String) -> Self {
        Self::Ops(vec![Ops::Attr(attribute)])
    }

    pub fn parse_i32(&self) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::ParseI32());
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'parse_i32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn parse_f32(&self) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::ParseF32());
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'parse_f32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn add_i32(&self, value: i32) -> PyResult<()> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::AddI32(value));
                Ok(())
            }
            _ => Err(PyTypeError::new_err(
                "The method 'add_i32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn add_f32(&self, value: f32) -> PyResult<()> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::AddF32(value));
                Ok(())
            }
            _ => Err(PyTypeError::new_err(
                "The method 'add_f32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn sub_i32(&self, value: i32) -> PyResult<()> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::SubI32(value));
                Ok(())
            }
            _ => Err(PyTypeError::new_err(
                "The method 'sub_i32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn sub_f32(&self, value: f32) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::SubF32(value));
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'sub_f32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn mul_i32(&self, value: i32) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::MulI32(value));
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'mul_i32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn mul_f32(&self, value: f32) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::MulF32(value));
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'mul_f32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn div_i32(&self, value: i32) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::DivI32(value));
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'div_i32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn div_f32(&self, value: f32) -> PyResult<GeoElement> {
        match self {
            Self::Ops(ops) => {
                let mut cloned = ops.clone();
                cloned.push(Ops::DivF32(value));
                Ok(GeoElement::Ops(cloned))
            }
            _ => Err(PyTypeError::new_err(
                "The method 'div_f32' must be called only on 'GeoElement::Ops' variant",
            )),
        }
    }

    pub fn eq(&self, other: &GeoElement) -> Self {
        GeoElement::Eq(vec![self.clone(), other.clone()])
    }

    pub fn neq(&self, other: &GeoElement) -> Self {
        GeoElement::Neq(vec![self.clone(), other.clone()])
    }

    pub fn and(&self, other: &GeoElement) -> Self {
        GeoElement::And(vec![self.clone(), other.clone()])
    }

    pub fn or(&self, other: &GeoElement) -> Self {
        GeoElement::Or(vec![self.clone(), other.clone()])
    }
}
