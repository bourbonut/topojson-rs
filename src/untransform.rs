use crate::topojson_structs::Transform;
use pyo3::exceptions::PyIndexError;
use pyo3::prelude::*;

struct Untransformer {
    x0: f64,
    y0: f64,
    kx: f64,
    ky: f64,
    dx: f64,
    dy: f64,
}

impl Untransformer {
    fn new(transform: &Transform) -> PyResult<Self> {
        PyResult::Ok(Self {
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

impl Untransformer {
    fn call(&mut self, input: &[f64], i: usize) -> Vec<i32> {
        if i == 0 {
            self.x0 = 0.;
            self.y0 = 0.;
        }
        let mut output: Vec<f64> = input.iter().cloned().collect();
        let x1 = ((input.get(0).unwrap_or(&f64::NAN) - self.dx) / self.kx).round();
        let y1 = ((input.get(1).unwrap_or(&f64::NAN) - self.dy) / self.ky).round();
        output[0] = x1 - self.x0;
        output[1] = y1 - self.y0;
        self.x0 = x1;
        output.iter().map(|&x| x as i32).collect()
    }
}

pub fn untransform(
    maybe_transform: &Option<Transform>,
) -> PyResult<Box<dyn FnMut(&[f64], usize) -> Vec<i32>>> {
    match maybe_transform {
        Some(transform) => {
            let mut transformer = Untransformer::new(&transform)?;
            Ok(Box::new(move |input: &[f64], i: usize| {
                transformer.call(input, i)
            }))
        }
        None => Ok(Box::new(|input: &[f64], _: usize| {
            input.iter().map(|&x| x as i32).collect::<Vec<i32>>()
        })),
    }
}
