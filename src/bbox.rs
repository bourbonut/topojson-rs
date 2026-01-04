use core::f64;

use pyo3::PyResult;

use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};
use crate::transform::transform;

pub fn wrap_bbox(topology: &TopoJSON) -> PyResult<[f64; 4]> {
    Ok(Bbox::call(topology)?)
}

pub struct Bbox {
    transform: Box<dyn FnMut(&[f64], usize) -> Vec<f64>>,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    key: String,
}

impl Bbox {
    fn new(topology: &TopoJSON) -> PyResult<Self> {
        Ok(Self {
            transform: transform(&topology.transform)?,
            x0: f64::INFINITY,
            x1: -f64::INFINITY,
            y0: f64::INFINITY,
            y1: -f64::INFINITY,
            key: String::new(),
        })
    }

    pub fn call(topology: &TopoJSON) -> PyResult<[f64; 4]> {
        let mut bbox = Bbox::new(topology)?;
        topology.arcs.iter().for_each(|arc_vec| {
            for (i, arc) in arc_vec.iter().enumerate() {
                let p = (bbox.transform)(&arc.iter().map(|&x| x as f64).collect::<Vec<f64>>(), i);
                if p[0] < bbox.x0 {
                    bbox.x0 = p[0];
                }
                if p[0] > bbox.x1 {
                    bbox.x1 = p[0];
                }
                if p[1] < bbox.y0 {
                    bbox.y0 = p[1];
                }
                if p[1] > bbox.y1 {
                    bbox.y1 = p[1];
                }
            }
        });

        topology.objects.iter().for_each(|(key, geometry)| {
            bbox.key = key.to_string();
            bbox.geometry(geometry);
        });

        Ok([bbox.x0, bbox.y0, bbox.x1, bbox.y1])
    }

    fn geometry(&mut self, o: &Geometry) {
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                geometries.iter().for_each(|o| self.geometry(o))
            }
            GeometryType::Point { coordinates } => self.point(&coordinates),
            GeometryType::MultiPoint { coordinates } => {
                coordinates.iter().for_each(|p| self.point(p))
            }
            _ => (),
        }
    }

    fn point(&mut self, p: &Vec<f64>) {
        let p = (self.transform)(p, 0);
        if p[0] < self.x0 {
            self.x0 = p[0];
        }
        if p[0] > self.x1 {
            self.x1 = p[0];
        }
        if p[1] < self.y0 {
            self.y0 = p[1];
        }
        if p[1] > self.y1 {
            self.y1 = p[1];
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{json_parse, request};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_bbox_1() -> PyResult<()> {
        let bbox = vec![1., 2., 3., 4.];
        let topology = TopoJSON {
            bbox,
            transform: None,
            objects: HashMap::new(),
            arcs: Vec::new(),
        };
        assert_eq!(
            wrap_bbox(&topology)?,
            [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_2() -> Result<(), String> {
        let topology = TopoJSON::try_from(json_parse(
            request("test/topojson/polygon-q1e4.json").await?,
        )?)?;
        assert_eq!(
            wrap_bbox(&topology)
                .map_err(|e| format!("Error during bbox operation: {}", e.to_string()))?,
            [0., 0., 10., 10.]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_3() -> Result<(), String> {
        let topology =
            TopoJSON::try_from(json_parse(request("test/topojson/polygon.json").await?)?)?;
        assert_eq!(
            wrap_bbox(&topology)
                .map_err(|e| format!("Error during bbox operation: {}", e.to_string()))?,
            [0., 0., 10., 10.]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_4() -> Result<(), String> {
        let topology = TopoJSON::try_from(json_parse(request("test/topojson/point.json").await?)?)?;
        assert_eq!(
            wrap_bbox(&topology)
                .map_err(|e| format!("Error during bbox operation: {}", e.to_string()))?,
            [0., 0., 10., 10.]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_5() -> Result<(), String> {
        let topology =
            TopoJSON::try_from(json_parse(request("test/topojson/points.json").await?)?)?;
        assert_eq!(
            wrap_bbox(&topology)
                .map_err(|e| format!("Error during bbox operation: {}", e.to_string()))?,
            [0., 0., 10., 10.]
        );
        Ok(())
    }
}
