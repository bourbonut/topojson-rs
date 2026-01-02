use crate::bbox::Bbox;
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON, Transform};
use crate::untransform::untransform;

use pyo3::PyResult;
use pyo3::exceptions::PyRuntimeError;

fn wrap_quantize(topology: &TopoJSON, transform: &f64) -> PyResult<TopoJSON> {
    Ok(Quantize::new(topology, transform)?.call(topology)?)
}

struct Quantize {
    r#box: Vec<f64>,
    transform: Option<Transform>,
    untransform: Box<dyn FnMut(&[f64], usize) -> Vec<i32>>,
}

impl Quantize {
    fn call(mut self, topology: &TopoJSON) -> PyResult<TopoJSON> {
        let objects = topology
            .objects
            .iter()
            .map(|(key, input)| (key.to_string(), self.quantize_geometry(input)))
            .collect();
        let arcs = topology
            .arcs
            .iter()
            .map(|input| self.quantize_arc(input))
            .collect();
        Ok(TopoJSON {
            bbox: self.r#box,
            transform: self.transform,
            objects,
            arcs,
        })
    }

    fn new(topology: &TopoJSON, transform: &f64) -> PyResult<Quantize> {
        if topology.transform.is_some() {
            return Err(PyRuntimeError::new_err("Already quantized"));
        };

        let n = transform.floor();
        if n < 2. {
            return Err(PyRuntimeError::new_err("n must be larger than 2"));
        }
        let r#box = if topology.bbox.is_empty() {
            Bbox::call(topology)?.to_vec()
        } else {
            topology.bbox.to_vec()
        };
        let x0 = r#box[0];
        let y0 = r#box[1];
        let x1 = r#box[2];
        let y1 = r#box[3];

        let transform = Some(Transform {
            scale: vec![
                if x1 - x0 != 0. {
                    (x1 - x0) / (n - 1.)
                } else {
                    1.
                },
                if y1 - y0 != 0. {
                    (y1 - y0) / (n - 1.)
                } else {
                    1.
                },
            ],
            translate: vec![x0, y0],
        });

        let untransform = untransform(&transform)?;
        Ok(Self {
            r#box,
            transform,
            untransform,
        })
    }

    fn quantize_point(&mut self, point: &[f64]) -> Vec<i32> {
        (self.untransform)(point, 0)
    }

    fn quantize_geometry(&mut self, input: &Geometry) -> Geometry {
        let quantized_geometry = match &input.geometry {
            GeometryType::GeometryCollection { geometries } => GeometryType::GeometryCollection {
                geometries: geometries
                    .iter()
                    .map(|geometry| self.quantize_geometry(geometry))
                    .collect(),
            },
            GeometryType::Point { coordinates } => GeometryType::Point {
                coordinates: self
                    .quantize_point(coordinates)
                    .iter()
                    .map(|&x| x as f64)
                    .collect(),
            },
            GeometryType::MultiPoint { coordinates } => GeometryType::MultiPoint {
                coordinates: coordinates
                    .iter()
                    .map(|point| {
                        self.quantize_point(point)
                            .iter()
                            .map(|&x| x as f64)
                            .collect()
                    })
                    .collect(),
            },
            _ => return input.clone(),
        };

        Geometry {
            geometry: quantized_geometry,
            id: input.id.clone(),
            properties: input.properties.clone(),
            bbox: input.bbox.clone(),
        }
    }

    fn quantize_arc(&mut self, input: &[Vec<i32>]) -> Vec<Vec<i32>> {
        let mut untransform = |i: usize| {
            (self.untransform)(&input[i].iter().map(|&x| x as f64).collect::<Vec<f64>>(), i)
        };

        let mut output = vec![untransform(0)];
        for i in 1..input.len() {
            let p = untransform(i);
            if p[0] != 0 || p[1] != 0 {
                output.push(p);
            }
        }
        if output.len() == 1 {
            output.push(vec![0, 0]);
        }
        output
    }
}
