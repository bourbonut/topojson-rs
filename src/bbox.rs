use std::array::from_fn;

use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};
use crate::transform::{IdentityTransformer, ScaleTransformer, Transformer};

pub fn wrap_bbox(topology: &TopoJSON) -> [f64; 4] {
    bbox(topology)
}

pub fn bbox(topology: &TopoJSON) -> [f64; 4] {
    match &topology.transform {
        Some(transform) => Bbox::call(topology, ScaleTransformer::new(transform)),
        None => Bbox::call(topology, IdentityTransformer::new()),
    }
}

pub struct Bbox<T>
where
    T: Transformer,
{
    transformer: T,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
}

impl<T: Transformer> Bbox<T> {
    fn call(topology: &TopoJSON, transformer: T) -> [f64; 4] {
        Bbox::new(transformer).bbox(topology)
    }

    fn new(transformer: T) -> Self {
        Self {
            transformer,
            x0: f64::INFINITY,
            x1: -f64::INFINITY,
            y0: f64::INFINITY,
            y1: -f64::INFINITY,
        }
    }

    pub fn bbox(mut self, topology: &TopoJSON) -> [f64; 4] {
        topology.arcs.iter().for_each(|arc_vec| {
            for (i, arc) in arc_vec.iter().enumerate() {
                let p = self.transformer.call(&from_fn(|i| arc[i] as f64), i);
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
        });

        topology.objects.values().for_each(|geometry| {
            self.geometry(geometry);
        });

        [self.x0, self.y0, self.x1, self.y1]
    }

    fn geometry(&mut self, o: &Geometry) {
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                geometries.iter().for_each(|o| self.geometry(o))
            }
            GeometryType::Point { coordinates } => self.point(coordinates),
            GeometryType::MultiPoint { coordinates } => {
                coordinates.iter().for_each(|p| self.point(p))
            }
            _ => (),
        }
    }

    fn point(&mut self, p: &[f64; 2]) {
        let p = self.transformer.call(p, 0);
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
    use pyo3::prelude::PyResult;
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
            wrap_bbox(&topology),
            [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY]
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_2() -> Result<(), String> {
        let topology = TopoJSON::try_from(json_parse(
            request("test/topojson/polygon-q1e4.json").await?,
        )?)?;
        assert_eq!(wrap_bbox(&topology), [0., 0., 10., 10.]);
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_3() -> Result<(), String> {
        let topology =
            TopoJSON::try_from(json_parse(request("test/topojson/polygon.json").await?)?)?;
        assert_eq!(wrap_bbox(&topology), [0., 0., 10., 10.]);
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_4() -> Result<(), String> {
        let topology = TopoJSON::try_from(json_parse(request("test/topojson/point.json").await?)?)?;
        assert_eq!(wrap_bbox(&topology), [0., 0., 10., 10.]);
        Ok(())
    }

    #[tokio::test]
    async fn test_bbox_5() -> Result<(), String> {
        let topology =
            TopoJSON::try_from(json_parse(request("test/topojson/points.json").await?)?)?;
        assert_eq!(wrap_bbox(&topology), [0., 0., 10., 10.]);
        Ok(())
    }
}
