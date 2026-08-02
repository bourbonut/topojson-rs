use std::collections::HashMap;

use pyo3::PyResult;

use crate::feature::object_func;
use crate::geojsons::FeatureGeometryType;
use crate::lambda::GeoVar;
use crate::stitch::stitch;
use crate::topojsons::{Geometry, TopoJSON};

pub fn wrap_mesh(
    topology: &TopoJSON,
    object: Option<&Geometry>,
    filter: Option<&GeoVar>,
) -> PyResult<FeatureGeometryType> {
    Ok(object_func(
        topology,
        &MeshArcs::call(topology, object, filter)?,
    ))
}

struct ArcItem<'a> {
    i: i32,
    #[allow(unused)]
    geometry: &'a Geometry,
}

#[derive(Default)]
struct GeometryByArcs<'a> {
    max_index: usize,
    hmap: HashMap<usize, Vec<ArcItem<'a>>>,
}

#[derive(Default)]
struct MeshArcs<'a> {
    arcs: Vec<i32>,
    geoms_by_arc: GeometryByArcs<'a>,
    geom: Option<&'a Geometry>,
}

impl<'a> MeshArcs<'a> {
    fn call(
        topology: &TopoJSON,
        object: Option<&'a Geometry>,
        filter: Option<&'a GeoVar>,
    ) -> PyResult<Geometry> {
        let arcs = match object {
            Some(object) => MeshArcs::default().extract(object, filter)?,
            None => (0..topology.arcs.len()).map(|x| x as i32).collect(),
        };
        Ok(Geometry::MultiLineString {
            arcs: stitch(topology, arcs),
            id: None,
            properties: None,
            bbox: None,
        })
    }

    fn extract(mut self, object: &'a Geometry, filter: Option<&'a GeoVar>) -> PyResult<Vec<i32>> {
        self.geometry(object);

        let geoms_by_arc =
            (0..=self.geoms_by_arc.max_index).filter_map(|k| self.geoms_by_arc.hmap.get(&k));
        match filter {
            Some(geo_var) => {
                for geoms in geoms_by_arc {
                    let geom1 = geoms.first().unwrap().geometry;
                    let geom2 = geoms.last().unwrap().geometry;
                    if geo_var.compare(geom1, geom2)?.as_bool() {
                        self.arcs.push(geoms[0].i);
                    }
                }
            }
            None => geoms_by_arc.for_each(|geoms| {
                self.arcs.push(geoms[0].i);
            }),
        };

        Ok(self.arcs)
    }

    fn extract_0(&mut self, i: i32) {
        let j = if i < 0 { !i } else { i } as usize;
        let geom = self.geom.expect("Undefined 'geom' during runtime");
        self.geoms_by_arc
            .hmap
            .entry(j)
            .or_default()
            .push(ArcItem { i, geometry: geom });
        self.geoms_by_arc.max_index = self.geoms_by_arc.max_index.max(j)
    }

    fn extract_1(&mut self, arcs: &[i32]) {
        arcs.iter().for_each(|&i| self.extract_0(i));
    }

    fn extract_2(&mut self, arcs: &[Vec<i32>]) {
        arcs.iter().for_each(|arcs| self.extract_1(arcs));
    }

    fn extract_3(&mut self, arcs: &[Vec<Vec<i32>>]) {
        arcs.iter().for_each(|arcs| self.extract_2(arcs));
    }

    fn geometry(&mut self, o: &'a Geometry) {
        self.geom = Some(o);
        match o {
            Geometry::GeometryCollection { geometries, .. } => {
                geometries.iter().for_each(|o| self.geometry(o))
            }
            Geometry::LineString { arcs, .. } => self.extract_1(arcs),
            Geometry::MultiLineString { arcs, .. } => self.extract_2(arcs),
            Geometry::Polygon { arcs, .. } => self.extract_2(arcs),
            Geometry::MultiPolygon { arcs, .. } => self.extract_3(arcs),
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_1() -> PyResult<()> {
        let topology = TopoJSON {
            arcs: Vec::new(),
            bbox: Vec::new(),
            objects: HashMap::new(),
            transform: None,
        };
        assert_eq!(
            wrap_mesh(&topology, None, None)?,
            FeatureGeometryType::MultiLineString {
                coordinates: Vec::new()
            }
        );
        Ok(())
    }

    #[test]
    fn test_mesh_2() -> PyResult<()> {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry::GeometryCollection {
                    geometries: vec![
                        Geometry::LineString {
                            arcs: vec![0],
                            id: None,
                            properties: None,
                            bbox: None,
                        },
                        Geometry::LineString {
                            arcs: vec![1],
                            id: None,
                            properties: None,
                            bbox: None,
                        },
                    ],
                    id: None,
                    properties: None,
                    bbox: None,
                },
            )]),
            arcs: vec![vec![[1, 0], [2, 0]], vec![[0, 0], [1, 0]]],
        };
        assert_eq!(
            wrap_mesh(&topology, None, None)?,
            FeatureGeometryType::MultiLineString {
                coordinates: vec![vec![[0., 0.], [1., 0.], [2., 0.]]]
            }
        );
        Ok(())
    }

    #[test]
    fn test_mesh_3() -> PyResult<()> {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry::GeometryCollection {
                    geometries: vec![
                        Geometry::LineString {
                            arcs: vec![0],
                            id: None,
                            properties: None,
                            bbox: None,
                        },
                        Geometry::LineString {
                            arcs: vec![1],
                            id: None,
                            properties: None,
                            bbox: None,
                        },
                    ],
                    id: None,
                    properties: None,
                    bbox: None,
                },
            )]),
            arcs: vec![vec![[2, 0], [3, 0]], vec![[0, 0], [1, 0]]],
        };
        if let FeatureGeometryType::MultiLineString { coordinates } =
            wrap_mesh(&topology, None, None)?
        {
            for values in [vec![[2., 0.], [3., 0.]], vec![[0., 0.], [1., 0.]]] {
                assert!(coordinates.contains(&values));
            }
        } else {
            panic!("Feature Geometry Type must be 'FeatureGeometryType::MultiLineString'");
        }
        Ok(())
    }
}
