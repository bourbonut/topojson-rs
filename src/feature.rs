use crate::geojson_structs::{
    Feature, FeatureCollection, FeatureGeometry, FeatureGeometryType, FeatureItem,
};
use crate::reverse::reverse;
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};
use crate::transform::transform;
use pyo3::prelude::PyResult;

pub fn wrap_feature(topology: &TopoJSON, o: &Geometry) -> PyResult<Feature> {
    match &o.geometry {
        GeometryType::GeometryCollection { geometries } => {
            let features: Vec<FeatureItem> = geometries
                .into_iter()
                .map(|o| feature_item(&topology, &o))
                .collect::<PyResult<Vec<FeatureItem>>>()?;
            Ok(Feature::Collection(FeatureCollection {
                r#type: "FeatureCollection".to_string(),
                features,
            }))
        }
        _ => Ok(Feature::Item(feature_item(&topology, o)?)),
    }
}

fn feature_item(topology: &TopoJSON, o: &Geometry) -> PyResult<FeatureItem> {
    let geometry = Object::call(topology, &o)?;
    let id = o.id.clone();
    let bbox = o.bbox.clone();
    let properties = o.properties.clone();
    Ok(FeatureItem {
        id,
        bbox,
        properties,
        geometry,
        r#type: String::from("Feature"),
    })
}

pub struct Object<'a> {
    arcs: &'a Vec<Vec<Vec<i32>>>,
    transform_point: Box<dyn FnMut(&[f64], usize) -> Vec<f64>>,
}

impl<'a> Object<'a> {
    pub fn call(topology: &TopoJSON, o: &Geometry) -> PyResult<FeatureGeometry> {
        let mut object = Object {
            arcs: &topology.arcs,
            transform_point: transform(&topology.transform)?,
        };
        Ok(object.geometry(o))
    }

    fn arc(&mut self, i: i32, points: &mut Vec<Vec<f64>>) {
        if !points.is_empty() {
            points.pop();
        }
        let a = &self.arcs[if i < 0 { !i as usize } else { i as usize }];
        for (k, arc) in a.iter().enumerate() {
            points.push((self.transform_point)(
                &arc.iter().map(|&x| x as f64).collect::<Vec<f64>>(),
                k,
            ));
        }
        if i < 0 {
            reverse(points, a.len());
        }
    }

    #[inline]
    fn point(&mut self, p: &[f64]) -> Vec<f64> {
        (self.transform_point)(&p, 0)
    }

    fn line(&mut self, arcs: &[i32]) -> Vec<Vec<f64>> {
        let mut points = Vec::new();
        for &arc in arcs {
            self.arc(arc, &mut points);
        }
        if points.len() < 2 {
            points.push(points[0].clone());
        }
        points
    }

    #[inline]
    fn ring(&mut self, arcs: &[i32]) -> Vec<Vec<f64>> {
        let mut points = self.line(arcs);
        while points.len() < 4 {
            points.push(points[0].clone());
        }
        points
    }

    #[inline]
    fn polygon(&mut self, arcs: &[Vec<i32>]) -> Vec<Vec<Vec<f64>>> {
        arcs.iter().map(|arcs| self.ring(arcs)).collect()
    }

    fn geometry(&mut self, o: &Geometry) -> FeatureGeometry {
        let r#type = o.r#type.to_string();
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                return FeatureGeometry {
                    r#type,
                    geometry: FeatureGeometryType::GeometryCollection {
                        geometries: geometries.iter().map(|o| self.geometry(o)).collect(),
                    },
                };
            }
            GeometryType::Point { coordinates } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::Point {
                    coordinates: self.point(coordinates),
                },
            },
            GeometryType::MultiPoint { coordinates } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::MultiPoint {
                    coordinates: coordinates.iter().map(|p| self.point(p)).collect(),
                },
            },
            GeometryType::LineString { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::LineString {
                    coordinates: self.line(arcs),
                },
            },
            GeometryType::MultiLineString { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::MultiLineString {
                    coordinates: arcs.iter().map(|arcs| self.line(arcs)).collect(),
                },
            },
            GeometryType::Polygon { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::Polygon {
                    coordinates: self.polygon(arcs),
                },
            },
            GeometryType::MultiPolygon { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::MultiPolygon {
                    coordinates: arcs.iter().map(|arcs| self.polygon(arcs)).collect(),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::topojson_structs::Transform;
    use std::collections::HashMap;

    use super::*;

    fn simple_topology(object: Geometry) -> TopoJSON {
        TopoJSON {
            r#type: "Topology".to_string(),
            bbox: vec![],
            transform: Some(Transform {
                scale: vec![1., 1.],
                translate: vec![0., 0.],
            }),
            objects: HashMap::from_iter([("foo".to_string(), object)]),
            arcs: vec![
                vec![vec![0, 0], vec![1, 0], vec![0, 1], vec![-1, 0], vec![0, -1]],
                vec![vec![0, 0], vec![1, 0], vec![0, 1]],
                vec![vec![1, 1], vec![-1, 0], vec![0, -1]],
                vec![vec![1, 1]],
                vec![vec![0, 0]],
            ],
        }
    }

    #[test]
    fn test_feature_1() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature_item) = wrap_feature(&t, &t.objects["foo"])? {
            assert_eq!(feature_item.geometry.r#type, "Polygon");
            assert!(matches!(
                feature_item.geometry.geometry,
                FeatureGeometryType::Polygon { .. }
            ));
        } else {
            unreachable!("Result should be variant of Feature::Item")
        }
        Ok(())
    }
}
