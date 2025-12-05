use crate::topojson_structs::Transform;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;

struct Transformer {
    x0: f64,
    y0: f64,
    kx: f64,
    ky: f64,
    dx: f64,
    dy: f64,
}

impl Transformer {
    fn new(transform: &Transform) -> PyResult<Self> {
        PyResult::Ok(Transformer {
            x0: 0.,
            y0: 0.,
            kx: *transform.scale.get(0).ok_or(PyIndexError::new_err(
                "\"transform.scale\" must be a list with at least two floating numbers.",
            ))?,
            ky: *transform.scale.get(1).ok_or(PyIndexError::new_err(
                "\"transform.scale\" must be a list with at least two floating numbers.",
            ))?,
            dx: *transform.translate.get(0).ok_or(PyIndexError::new_err(
                "\"transform.translate\" must be a list with at least two floating numbers.",
            ))?,
            dy: *transform.translate.get(1).ok_or(PyIndexError::new_err(
                "\"transform.translate\" must be a list with at least two floating numbers.",
            ))?,
        })
    }
}

impl Transformer {
    fn call(&mut self, input: &[f64], i: usize) -> Vec<f64> {
        if i == 0 {
            self.x0 = 0.;
            self.y0 = 0.;
        }
        let mut output: Vec<f64> = input.iter().cloned().collect();
        self.x0 += input.get(0).unwrap_or(&f64::NAN);
        self.y0 += input.get(1).unwrap_or(&f64::NAN);
        output[0] = self.x0 * self.kx + self.dx;
        output[1] = self.y0 * self.ky + self.dy;
        output
    }
}

pub fn transform(
    maybe_transform: &Option<Transform>,
) -> PyResult<Box<dyn FnMut(&[f64], usize) -> Vec<f64>>> {
    match maybe_transform {
        Some(transform) => {
            let mut transformer = Transformer::new(&transform)?;
            Ok(Box::new(move |input: &[f64], i: usize| {
                transformer.call(input, i)
            }))
        }
        None => Ok(Box::new(|input: &[f64], _: usize| {
            input.iter().cloned().collect::<Vec<f64>>()
        })),
    }
}
