use crate::feature_structs::{
    Feature, FeatureCollection, FeatureGeometry, FeatureGeometryType, FeatureItem,
};
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};
use crate::transform::transform;
use pyo3::prelude::PyResult;

fn wrap_feature(topology: TopoJSON, o: Geometry) -> PyResult<Feature> {
    match o.geometry {
        GeometryType::GeometryCollection { geometries } => {
            let features: Vec<Feature> = geometries
                .into_iter()
                .map(|o| feature_item(&topology, o))
                .collect::<PyResult<Vec<Feature>>>()?;
            Ok(Feature::Collection(FeatureCollection {
                r#type: "FeatureCollection".to_string(),
                features,
            }))
        }
        _ => feature_item(&topology, o),
    }
}

fn feature_item(topology: &TopoJSON, o: Geometry) -> PyResult<Feature> {
    let geometry = Object::call(topology, &o)?;
    let id = o.id;
    let bbox = o.bbox;
    let properties = o.properties;
    Ok(Feature::Item(FeatureItem {
        id,
        bbox,
        properties,
        geometry,
        r#type: String::from("Feature"),
    }))
}

struct Object<'a> {
    topology: &'a TopoJSON,
    arcs: &'a Vec<Vec<Vec<i32>>>,
    transform_point: Box<dyn FnMut(&[f32], usize) -> Vec<f32>>,
}

impl<'a> Object<'a> {
    fn call(topology: &TopoJSON, o: &Geometry) -> PyResult<FeatureGeometry> {
        let mut object = Object {
            topology: &topology,
            arcs: &topology.arcs,
            transform_point: transform(&topology.transform)?,
        };
        Ok(object.geometry(o))
    }

    fn point(&mut self, p: &[f32]) -> Vec<f32> {
        (self.transform_point)(&p, 0)
    }

    fn line(&mut self, arcs: &[i32]) -> Vec<Vec<f32>> {
        let mut points = Vec::new();
        for arc in arcs {
            self.arc(*arc, &mut points);
        }
        if points.len() < 2 {
            points.push(points[0].clone());
        }
        points
    }

    fn arc(&mut self, i: i32, points: &mut Vec<Vec<f32>>) {
        if !points.is_empty() {
            points.pop();
        }
        let a = &self.arcs[if i < 0 {
            self.arcs.len() - i as usize
        } else {
            i as usize
        }];
        for k in 0..a.len() {
            points.push((self.transform_point)(
                &a[k].iter().map(|x| *x as f32).collect::<Vec<f32>>(),
                k,
            ));
        }
        if i < 0 {
            points.reverse();
        }
    }

    fn ring(&mut self, arcs: &[i32]) -> Vec<Vec<f32>> {
        let mut points = self.line(arcs);
        while points.len() < 4 {
            points.push(points[0].clone());
        }
        points
    }

    fn polygon(&mut self, arcs: &[Vec<i32>]) -> Vec<Vec<Vec<f32>>> {
        arcs.iter().map(|ring| self.ring(ring)).collect()
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
                    coordinates: self.point(&coordinates),
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
                geometry: FeatureGeometryType::Line {
                    coordinates: self.line(&arcs),
                },
            },
            GeometryType::MultiLineString { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::MultiLineString {
                    coordinates: arcs.iter().map(|arc| self.line(arc)).collect(),
                },
            },
            GeometryType::Polygon { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::Polygon {
                    coordinates: self.polygon(&arcs),
                },
            },
            GeometryType::MultiPolygon { arcs } => FeatureGeometry {
                r#type,
                geometry: FeatureGeometryType::MultiPolygon {
                    coordinates: arcs.iter().map(|arc| self.polygon(arc)).collect(),
                },
            },
        }
    }
}
