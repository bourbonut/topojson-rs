use std::collections::{HashMap, HashSet};

use pyo3::PyResult;
use pyo3::exceptions::PyRuntimeError;

use crate::feature::Object;
use crate::geojson_structs::FeatureGeometryType;
use crate::stitch::stitch;
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};

fn planar_ring_area(ring: &Vec<Vec<f64>>) -> f64 {
    let mut i = 0;
    let n = ring.len();
    let b = ring.last().unwrap();
    let mut area: f64 = 0.;
    while i < n {
        let (a, b) = (b, &ring[i]);
        area += a[0] * b[1] - a[1] * b[0];
        i += 1;
    }
    area.abs()
}

#[derive(Default)]
struct MergeArcs<'a> {
    polygons_by_arcs: HashMap<usize, Vec<Vec<Vec<i32>>>>,
    polygons: Vec<&'a Vec<Vec<i32>>>,
    groups: Vec<Vec<Vec<Vec<i32>>>>,
}

impl<'a> MergeArcs<'a> {
    fn call(topology: &TopoJSON, objects: &Vec<Geometry>) {
        MergeArcs::default().merge(topology, objects)
    }

    fn merge(&mut self, topology: &TopoJSON, objects: &'a Vec<Geometry>) {
        objects.iter().for_each(|o| self.geometry(o));
        let mut visited_polygons = HashSet::new();
        let mut visited_polygons_by_arc = HashSet::new();

        for (i, polygon) in self.polygons.iter().enumerate() {
            if !visited_polygons.contains(&i) {
                let mut group = Vec::new();
                let mut neighbors: Vec<[usize; 2]> = Vec::new();
                let mut target_polygon = Some(polygon.to_vec());
                visited_polygons.insert(i);

                while let Some(polygon) = target_polygon {
                    target_polygon = None;

                    if let Some([arc, k]) = neighbors.pop() {
                        target_polygon = Some(self.polygons_by_arcs[&arc][k].to_vec());
                    }

                    for ring in polygon.iter() {
                        for &arc in ring.iter() {
                            let arc = if arc < 0 { -arc } else { arc } as usize;
                            for k in 0..self.polygons_by_arcs[&arc].len() {
                                if !visited_polygons_by_arc.contains(&[arc, k]) {
                                    visited_polygons_by_arc.insert([arc, k]);
                                    neighbors.push([arc, k]);
                                }
                            }
                        }
                    }

                    group.push(polygon);
                }
                self.groups.push(group);
            }
        }
    }

    fn geometry(&mut self, o: &'a Geometry) {
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                geometries.iter().for_each(|o| self.geometry(o))
            }
            GeometryType::Polygon { arcs } => self.extract(&arcs),
            GeometryType::MultiPolygon { arcs } => {
                arcs.iter().for_each(|polygons| self.extract(polygons))
            }
            _ => (),
        }
    }

    fn extract(&mut self, polygon: &'a Vec<Vec<i32>>) {
        polygon.iter().for_each(|ring| {
            ring.iter().for_each(|&arc| {
                let arc = if arc < 0 { -arc } else { arc } as usize;
                self.polygons_by_arcs
                    .entry(arc)
                    .and_modify(|polygons: &mut Vec<Vec<Vec<i32>>>| polygons.push(polygon.to_vec()))
                    .or_insert(vec![polygon.to_vec()]);
            });
        });
        self.polygons.push(polygon);
    }

    fn area(topology: &TopoJSON, ring: Vec<i32>) -> PyResult<f64> {
        if let FeatureGeometryType::Polygon { coordinates } = Object::call(
            topology,
            &Geometry {
                r#type: "Polygon".to_string(),
                geometry: GeometryType::Polygon { arcs: vec![ring] },
                id: None,
                properties: None,
                bbox: None,
            },
        )?
        .geometry
        {
            Ok(planar_ring_area(&coordinates[0]))
        } else {
            Err(PyRuntimeError::new_err("Cannot compute the area of ring."))
        }
    }
}
