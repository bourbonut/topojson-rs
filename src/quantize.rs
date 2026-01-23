use crate::bbox::bbox;
use crate::topojsons::{Geometry, TopoJSON, Transform};
use crate::untransform::ScaleUntransformer;

use pyo3::PyResult;
use pyo3::exceptions::PyRuntimeError;

pub fn wrap_quantize(topology: &TopoJSON, transform: &f64) -> PyResult<TopoJSON> {
    Quantize::call(topology, transform)
}

struct Quantize {
    r#box: Vec<f64>,
    transform: Option<Transform>,
    untransformer: ScaleUntransformer,
}

impl Quantize {
    fn call(topology: &TopoJSON, transform: &f64) -> PyResult<TopoJSON> {
        Quantize::new(topology, transform)?.quantize(topology)
    }

    fn quantize(mut self, topology: &TopoJSON) -> PyResult<TopoJSON> {
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
        if n < 2. || n.is_nan() {
            return Err(PyRuntimeError::new_err("'transform' must be larger than 2"));
        }
        let r#box = if topology.bbox.is_empty() {
            bbox(topology).to_vec()
        } else {
            topology.bbox.to_vec()
        };
        let x0 = r#box[0];
        let y0 = r#box[1];
        let x1 = r#box[2];
        let y1 = r#box[3];

        let transform = Transform {
            scale: [
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
            translate: [x0, y0],
        };
        let untransformer = ScaleUntransformer::new(&transform);

        Ok(Self {
            r#box,
            transform: Some(transform),
            untransformer,
        })
    }

    fn quantize_point(&mut self, point: &[f64; 2]) -> [f64; 2] {
        self.untransformer.call(point, 0)
    }

    fn quantize_geometry(&mut self, input: &Geometry) -> Geometry {
        match input {
            Geometry::GeometryCollection {
                geometries,
                id,
                properties,
                bbox,
            } => Geometry::GeometryCollection {
                geometries: geometries
                    .iter()
                    .map(|geometry| self.quantize_geometry(geometry))
                    .collect(),
                id: id.clone(),
                properties: properties.clone(),
                bbox: bbox.clone(),
            },
            Geometry::Point {
                coordinates,
                id,
                properties,
                bbox,
            } => Geometry::Point {
                coordinates: self.quantize_point(coordinates),
                id: id.clone(),
                properties: properties.clone(),
                bbox: bbox.clone(),
            },
            Geometry::MultiPoint {
                coordinates,
                id,
                properties,
                bbox,
            } => Geometry::MultiPoint {
                coordinates: coordinates
                    .iter()
                    .map(|point| self.quantize_point(point))
                    .collect(),
                id: id.clone(),
                properties: properties.clone(),
                bbox: bbox.clone(),
            },
            _ => input.clone(),
        }
    }

    fn quantize_arc(&mut self, input: &[[i32; 2]]) -> Vec<[i32; 2]> {
        let mut untransform = |i: usize| {
            let arc = &input[i];
            self.untransformer.call(&arc.map(|x| x as f64), i)
        };

        let mut output = vec![untransform(0)];
        for i in 1..input.len() {
            let p = untransform(i);
            if p[0] != 0. || p[1] != 0. {
                output.push(p);
            }
        }
        if output.len() == 1 {
            output.push([0., 0.]);
        }
        output.iter().map(|array| array.map(|x| x as i32)).collect()
    }
}

#[cfg(test)]
mod tests {
    use pyo3::Python;

    use crate::request::request;

    use super::*;

    async fn quantize(
        actual_filetest: &str,
        transform: &f64,
        expected_filetest: &str,
    ) -> Result<(), String> {
        let topology = serde_json::from_str::<TopoJSON>(&request(actual_filetest).await?).unwrap();
        let expected_topology =
            serde_json::from_str::<TopoJSON>(&request(expected_filetest).await?).unwrap();
        assert_eq!(
            wrap_quantize(&topology, transform)
                .map_err(|e| format!("Error during quantize operation: {}", e.to_string()))?,
            expected_topology
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_1() -> Result<(), String> {
        quantize(
            "test/topojson/polygon.json",
            &1e4,
            "test/topojson/polygon-q1e4.json",
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_2() -> Result<(), String> {
        quantize(
            "test/topojson/polygon.json",
            &1e5,
            "test/topojson/polygon-q1e5.json",
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_3() -> Result<(), String> {
        quantize(
            "test/topojson/empty.json",
            &1e4,
            "test/topojson/empty-q1e4.json",
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_4() -> Result<(), String> {
        quantize(
            "test/topojson/properties.json",
            &1e4,
            "test/topojson/properties-q1e4.json",
        )
        .await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_5() -> Result<(), String> {
        let mut before =
            serde_json::from_str::<TopoJSON>(&request("test/topojson/polygon.json").await?)
                .unwrap();
        before.bbox.clear();
        let after = wrap_quantize(&before, &1e4)
            .map_err(|e| format!("Error during quantize operation: {}", e.to_string()))?;

        let expected_topology =
            serde_json::from_str::<TopoJSON>(&request("test/topojson/polygon-q1e4.json").await?)
                .unwrap();
        assert_eq!(after, expected_topology);
        assert_eq!(after.bbox, vec![0., 0., 10., 10.]);
        assert_eq!(before.bbox, Vec::<f64>::new());
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_6() -> Result<(), String> {
        Python::initialize();
        let topology =
            serde_json::from_str::<TopoJSON>(&request("test/topojson/polygon-q1e4.json").await?)
                .unwrap();
        if let Err(py_runtime_error) = wrap_quantize(&topology, &1e4) {
            assert_eq!(
                py_runtime_error.to_string(),
                String::from("RuntimeError: Already quantized")
            );
            Ok(())
        } else {
            Err(String::from(
                "Quantized must return an error: 'Already quantized'",
            ))
        }
    }

    #[tokio::test]
    async fn test_quantize_7() -> Result<(), String> {
        Python::initialize();
        let topology =
            serde_json::from_str::<TopoJSON>(&request("test/topojson/polygon.json").await?)
                .unwrap();
        for transform in [0., 1.5, f64::NAN, -2.] {
            if let Err(py_runtime_error) = wrap_quantize(&topology, &transform) {
                assert_eq!(
                    py_runtime_error.to_string(),
                    String::from("RuntimeError: n must be larger than 2")
                );
            } else {
                return Err(format!(
                    "Quantized must return an error: 'n must be larger than 2' for transform value '{:?}'",
                    transform,
                ));
            }
        }
        Ok(())
    }
}
