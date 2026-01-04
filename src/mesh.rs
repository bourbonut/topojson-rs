use std::collections::HashMap;
use std::rc::Rc;

use pyo3::types::{PyAnyMethods, PyFunction};
use pyo3::{Bound, PyResult};

use crate::feature::Object;
use crate::geojson_structs::FeatureGeometryType;
use crate::stitch::stitch;
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};

pub fn wrap_mesh(
    topology: &TopoJSON,
    object: Option<&Geometry>,
    filter: Option<&Bound<PyFunction>>,
) -> PyResult<FeatureGeometryType> {
    Object::call(topology, &MeshArcs::call(topology, object, filter))
}

struct ArcItem<'a> {
    i: i32,
    geometry: Rc<&'a Geometry>,
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
    geom: Option<Rc<&'a Geometry>>,
}

impl<'a> MeshArcs<'a> {
    fn call(
        topology: &TopoJSON,
        object: Option<&'a Geometry>,
        filter: Option<&'a Bound<PyFunction>>,
    ) -> Geometry {
        let arcs = match object {
            Some(object) => MeshArcs::default().extract(object, filter),
            None => (0..topology.arcs.len()).map(|x| x as i32).collect(),
        };
        Geometry {
            geometry: GeometryType::MultiLineString {
                arcs: stitch(topology, arcs),
            },
            id: None,
            properties: None,
            bbox: None,
        }
    }

    fn extract(mut self, object: &'a Geometry, filter: Option<&'a Bound<PyFunction>>) -> Vec<i32> {
        self.geometry(object);

        let geoms_by_arc =
            (0..=self.geoms_by_arc.max_index).filter_map(|k| self.geoms_by_arc.hmap.get(&k));
        match filter {
            Some(filter_func) => {
                geoms_by_arc.for_each(|geoms| {
                    match filter_func.call1((
                        geoms[0].geometry.as_ref(),
                        geoms.last().unwrap().geometry.as_ref(),
                    )) {
                        Ok(result) => {
                            if result.extract::<bool>().unwrap_or(false) {
                                self.arcs.push(geoms[0].i);
                            }
                        }
                        Err(_) => (),
                    }
                });
            }
            None => geoms_by_arc.for_each(|geoms| {
                self.arcs.push(geoms[0].i);
            }),
        };

        self.arcs
    }

    fn extract_0(&mut self, i: i32) {
        let j = if i < 0 { !i } else { i } as usize;
        let geom = self.geom.clone().expect("Undefined 'geom' during runtime");
        self.geoms_by_arc
            .hmap
            .entry(j)
            .and_modify(|vec| {
                vec.push(ArcItem {
                    i,
                    geometry: geom.clone(),
                })
            })
            .or_insert(vec![ArcItem {
                i,
                geometry: geom.clone(),
            }]);
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
        self.geom = Some(Rc::new(o));
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                geometries.iter().for_each(|o| self.geometry(o))
            }
            GeometryType::LineString { arcs } => self.extract_1(arcs),
            GeometryType::MultiLineString { arcs } => self.extract_2(arcs),
            GeometryType::Polygon { arcs } => self.extract_2(arcs),
            GeometryType::MultiPolygon { arcs } => self.extract_3(arcs),
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
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::LineString { arcs: vec![0] },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::LineString { arcs: vec![1] },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                        ],
                    },
                    id: None,
                    properties: None,
                    bbox: None,
                },
            )]),
            arcs: vec![vec![vec![1, 0], vec![2, 0]], vec![vec![0, 0], vec![1, 0]]],
        };
        assert_eq!(
            wrap_mesh(&topology, None, None)?,
            FeatureGeometryType::MultiLineString {
                coordinates: vec![vec![vec![0., 0.], vec![1., 0.], vec![2., 0.]]]
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
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::LineString { arcs: vec![0] },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::LineString { arcs: vec![1] },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                        ],
                    },
                    id: None,
                    properties: None,
                    bbox: None,
                },
            )]),
            arcs: vec![vec![vec![2, 0], vec![3, 0]], vec![vec![0, 0], vec![1, 0]]],
        };
        if let FeatureGeometryType::MultiLineString { coordinates } =
            wrap_mesh(&topology, None, None)?
        {
            for values in [
                vec![vec![2., 0.], vec![3., 0.]],
                vec![vec![0., 0.], vec![1., 0.]],
            ] {
                assert!(coordinates.contains(&values));
            }
        } else {
            panic!("Feature Geometry Type must be 'FeatureGeometryType::MultiLineString'");
        }
        Ok(())
    }
}
