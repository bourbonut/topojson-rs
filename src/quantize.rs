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
        if n < 2. || n.is_nan() {
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

#[cfg(test)]
mod tests {
    use pyo3::Python;

    use crate::parser::{json_parse, request};

    use super::*;

    async fn quantize(
        actual_filetest: &str,
        transform: &f64,
        expected_filetest: &str,
    ) -> Result<(), String> {
        let topology = TopoJSON::try_from(json_parse(request(actual_filetest).await?)?)?;

        let expected_topology = TopoJSON::try_from(json_parse(request(expected_filetest).await?)?)?;
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
            TopoJSON::try_from(json_parse(request("test/topojson/polygon.json").await?)?)?;
        before.bbox.clear();
        let after = wrap_quantize(&before, &1e4)
            .map_err(|e| format!("Error during quantize operation: {}", e.to_string()))?;

        let expected_topology = TopoJSON::try_from(json_parse(
            request("test/topojson/polygon-q1e4.json").await?,
        )?)?;
        assert_eq!(after, expected_topology);
        assert_eq!(after.bbox, vec![0., 0., 10., 10.]);
        assert_eq!(before.bbox, Vec::<f64>::new());
        Ok(())
    }

    #[tokio::test]
    async fn test_quantize_6() -> Result<(), String> {
        Python::initialize();
        let topology = TopoJSON::try_from(json_parse(
            request("test/topojson/polygon-q1e4.json").await?,
        )?)?;
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
            TopoJSON::try_from(json_parse(request("test/topojson/polygon.json").await?)?)?;
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
