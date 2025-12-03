use crate::topojson_structs::Transform;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;

struct Transformer {
    x0: f32,
    y0: f32,
    kx: f32,
    ky: f32,
    dx: f32,
    dy: f32,
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
    fn call(&mut self, input: &[f32], i: usize) -> Vec<f32> {
        if i == 0 {
            self.x0 = 0.;
            self.y0 = 0.;
        }
        let mut output: Vec<f32> = input.iter().cloned().collect();
        self.x0 += input[0];
        self.y0 += input[1];
        output[0] = self.x0 * self.kx + self.dx;
        output[1] = self.y0 * self.ky + self.dy;
        output
    }
}

pub fn transform(
    maybe_transform: &Option<Transform>,
) -> PyResult<Box<dyn FnMut(&[f32], usize) -> Vec<f32>>> {
    match maybe_transform {
        Some(transform) => {
            let mut transformer = Transformer::new(&transform)?;
            Ok(Box::new(move |input: &[f32], i: usize| {
                transformer.call(input, i)
            }))
        }
        None => Ok(Box::new(|input: &[f32], _: usize| {
            input.iter().cloned().collect::<Vec<f32>>()
        })),
    }
}
