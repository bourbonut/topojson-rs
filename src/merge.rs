use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, RandomState};

use crate::feature::object_func;
use crate::geojson_structs::FeatureGeometryType;
use crate::stitch::stitch;
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};

pub fn wrap_merge(topology: &TopoJSON, objects: &Vec<Geometry>) -> FeatureGeometryType {
    object_func(topology, &MergeArcs::call(topology, objects))
}

fn planar_ring_area(ring: &Vec<[f64; 2]>) -> f64 {
    let mut i = 0;
    let n = ring.len();
    let b = ring.last().unwrap();
    let mut area: f64 = 0.;
    while i < n {
        let a = b;
        let b = &ring[i];
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
    fn call(topology: &TopoJSON, objects: &Vec<Geometry>) -> Geometry {
        MergeArcs::default().merge(topology, objects)
    }

    fn merge(&mut self, topology: &TopoJSON, objects: &'a Vec<Geometry>) -> Geometry {
        objects.iter().for_each(|o| self.geometry(o));
        let mut visited_polygons = HashSet::new();
        let state = RandomState::new();

        for polygon in self.polygons.iter() {
            let hash = state.hash_one(polygon);
            if !visited_polygons.contains(&hash) {
                let mut group = Vec::new();
                let mut neighbors = vec![polygon.to_vec()];
                visited_polygons.insert(hash);

                while let Some(polygon) = neighbors.pop() {
                    for ring in polygon.iter() {
                        for &arc in ring.iter() {
                            let arc = if arc < 0 { !arc } else { arc } as usize;
                            for polygon in self.polygons_by_arcs[&arc].iter() {
                                let hash = state.hash_one(polygon);
                                if !visited_polygons.contains(&hash) {
                                    visited_polygons.insert(hash);
                                    neighbors.push(polygon.to_vec());
                                }
                            }
                        }
                    }

                    group.push(polygon);
                }
                self.groups.push(group);
            }
        }

        let mut global_arcs = Vec::new();
        for polygons in self.groups.iter() {
            let mut arcs = Vec::new();
            polygons.iter().for_each(|polygon| {
                polygon.iter().for_each(|ring| {
                    ring.iter().for_each(|&arc| {
                        let index = if arc < 0 { !arc } else { arc } as usize;
                        if self.polygons_by_arcs[&index].len() < 2 {
                            arcs.push(arc);
                        }
                    });
                });
            });

            let mut arcs = stitch(topology, arcs);

            let n = arcs.len();
            if n > 1 {
                let mut k = self.area(topology, &arcs[0]);
                for i in 1..n {
                    let ki = self.area(topology, &arcs[i]);
                    if ki > k {
                        arcs.swap(0, i);
                        k = ki;
                    }
                }
            }

            if !arcs.is_empty() {
                global_arcs.push(arcs);
            }
        }
        Geometry {
            geometry: GeometryType::MultiPolygon { arcs: global_arcs },
            id: None,
            properties: None,
            bbox: None,
        }
    }

    fn geometry(&mut self, o: &'a Geometry) {
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                geometries.iter().for_each(|o| self.geometry(o))
            }
            GeometryType::Polygon { arcs } => self.extract(&arcs),
            GeometryType::MultiPolygon { arcs } => {
                arcs.iter().for_each(|polygon| self.extract(polygon))
            }
            _ => (),
        }
    }

    fn extract(&mut self, polygon: &'a Vec<Vec<i32>>) {
        polygon.iter().for_each(|ring| {
            ring.iter().for_each(|&arc| {
                let arc = if arc < 0 { !arc } else { arc } as usize;
                self.polygons_by_arcs
                    .entry(arc)
                    .and_modify(|polygons: &mut Vec<Vec<Vec<i32>>>| polygons.push(polygon.to_vec()))
                    .or_insert(vec![polygon.to_vec()]);
            });
        });
        self.polygons.push(polygon);
    }

    fn area(&self, topology: &TopoJSON, ring: &Vec<i32>) -> f64 {
        if let FeatureGeometryType::Polygon { coordinates } = object_func(
            topology,
            &Geometry {
                geometry: GeometryType::Polygon {
                    arcs: vec![ring.to_vec()], // TODO: remove `to_vec`
                },
                id: None,
                properties: None,
                bbox: None,
            },
        ) {
            planar_ring_area(&coordinates[0])
        } else {
            unreachable!(
                "Object function with 'GeometryType::Polygon' must return 'FeatureGeometryType::Polygon'"
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_1() {
        let topology = TopoJSON {
            objects: HashMap::new(),
            bbox: Vec::new(),
            transform: None,
            arcs: Vec::new(),
        };
        let merge = wrap_merge(&topology, &Vec::new());
        assert_eq!(
            merge,
            FeatureGeometryType::MultiPolygon {
                coordinates: Vec::new()
            }
        );
    }

    //
    // +----+----+            +----+----+
    // |    |    |            |         |
    // |    |    |    ==>     |         |
    // |    |    |            |         |
    // +----+----+            +----+----+
    //
    #[test]
    fn test_merge_2() {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![0, 1]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![-1, 2]],
                                },
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
            arcs: vec![
                vec![[1, 1], [1, 0]],
                vec![[1, 0], [0, 0], [0, 1], [1, 1]],
                vec![[1, 1], [2, 1], [2, 0], [1, 0]],
            ],
        };
        if let GeometryType::GeometryCollection { geometries } =
            &topology.objects["collection"].geometry
        {
            let merge = wrap_merge(&topology, &geometries);
            assert_eq!(
                merge,
                FeatureGeometryType::MultiPolygon {
                    coordinates: vec![vec![vec![
                        [1., 0.],
                        [0., 0.],
                        [0., 1.],
                        [1., 1.],
                        [2., 1.],
                        [2., 0.],
                        [1., 0.]
                    ]]]
                }
            );
        } else {
            panic!("Topology must have a collection of geometries.")
        }
    }

    //
    // +----+ +----+            +----+ +----+
    // |    | |    |            |    | |    |
    // |    | |    |    ==>     |    | |    |
    // |    | |    |            |    | |    |
    // +----+ +----+            +----+ +----+
    //
    #[test]
    fn test_merge_3() {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![0]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![1]],
                                },
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
            arcs: vec![
                vec![[0, 0], [0, 1], [1, 1], [1, 0], [0, 0]],
                vec![[2, 0], [2, 1], [3, 1], [3, 0], [2, 0]],
            ],
        };
        if let GeometryType::GeometryCollection { geometries } =
            &topology.objects["collection"].geometry
        {
            let merge = wrap_merge(&topology, &geometries);
            assert_eq!(
                merge,
                FeatureGeometryType::MultiPolygon {
                    coordinates: vec![
                        vec![vec![[0., 0.], [0., 1.], [1., 1.], [1., 0.], [0., 0.]]],
                        vec![vec![[2., 0.], [2., 1.], [3., 1.], [3., 0.], [2., 0.]]]
                    ]
                }
            );
        } else {
            panic!("Topology must have a collection of geometries.")
        }
    }

    //
    // +-----------+            +-----------+
    // |           |            |           |
    // |   +---+   |    ==>     |           |
    // |   |   |   |            |           |
    // |   +---+   |            |           |
    // |           |            |           |
    // +-----------+            +-----------+
    //
    #[test]
    fn test_merge_4() {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![0], vec![1]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![-2]],
                                },
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
            arcs: vec![
                vec![[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],
                vec![[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]],
            ],
        };
        if let GeometryType::GeometryCollection { geometries } =
            &topology.objects["collection"].geometry
        {
            let merge = wrap_merge(&topology, &geometries);
            assert_eq!(
                merge,
                FeatureGeometryType::MultiPolygon {
                    coordinates: vec![vec![vec![[0., 0.], [0., 3.], [3., 3.], [3., 0.], [0., 0.]]]]
                }
            );
        } else {
            panic!("Topology must have a collection of geometries.")
        }
    }

    //
    // +-----------+-----------+            +-----------+-----------+
    // |           |           |            |                       |
    // |   +---+   |   +---+   |    ==>     |   +---+       +---+   |
    // |   |   |   |   |   |   |            |   |   |       |   |   |
    // |   +---+   |   +---+   |            |   +---+       +---+   |
    // |           |           |            |                       |
    // +-----------+-----------+            +-----------+-----------+
    //
    #[test]
    fn test_merge_5() {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![0, 1], vec![2]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![-1, 3], vec![4]],
                                },
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
            arcs: vec![
                vec![[3, 3], [3, 0]],
                vec![[3, 0], [0, 0], [0, 3], [3, 3]],
                vec![[1, 1], [2, 1], [2, 2], [1, 2], [1, 1]],
                vec![[3, 3], [6, 3], [6, 0], [3, 0]],
                vec![[4, 1], [5, 1], [5, 2], [4, 2], [4, 1]],
            ],
        };
        if let GeometryType::GeometryCollection { geometries } =
            &topology.objects["collection"].geometry
        {
            // Special case: since `HashMap` are unordered, the coordinates may be unordered too.
            // So instead of checking if coordinates are the same, the test checks if sub parts of
            // coordinates are present in final result
            if let FeatureGeometryType::MultiPolygon { coordinates } =
                wrap_merge(&topology, &geometries)
            {
                for subpart in [
                    vec![
                        [3., 0.],
                        [0., 0.],
                        [0., 3.],
                        [3., 3.],
                        [6., 3.],
                        [6., 0.],
                        [3., 0.],
                    ],
                    vec![[1., 1.], [2., 1.], [2., 2.], [1., 2.], [1., 1.]],
                    vec![[4., 1.], [5., 1.], [5., 2.], [4., 2.], [4., 1.]],
                ] {
                    assert!(coordinates[0].contains(&subpart));
                }
            } else {
                panic!("Feature Geometry Type must be 'MultiPolygon'.")
            }
        } else {
            panic!("Topology must have a collection of geometries.")
        }
    }

    //
    // +-------+-------+            +-------+-------+
    // |       |       |            |               |
    // |   +---+---+   |    ==>     |   +---+---+   |
    // |   |       |   |            |   |       |   |
    // |   +---+---+   |            |   +---+---+   |
    // |       |       |            |               |
    // +-------+-------+            +-------+-------+
    //
    #[test]
    fn test_merge_6() {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![0, 1, 2, 3]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![-3, 4, -1, 5]],
                                },
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
            arcs: vec![
                vec![[2, 3], [2, 2]],
                vec![[2, 2], [1, 2], [1, 1], [2, 1]],
                vec![[2, 1], [2, 0]],
                vec![[2, 0], [0, 0], [0, 3], [2, 3]],
                vec![[2, 1], [3, 1], [3, 2], [2, 2]],
                vec![[2, 3], [4, 3], [4, 0], [2, 0]],
            ],
        };
        if let GeometryType::GeometryCollection { geometries } =
            &topology.objects["collection"].geometry
        {
            let merge = wrap_merge(&topology, &geometries);
            assert_eq!(
                merge,
                FeatureGeometryType::MultiPolygon {
                    coordinates: vec![vec![
                        vec![
                            [2., 0.],
                            [0., 0.],
                            [0., 3.],
                            [2., 3.],
                            [4., 3.],
                            [4., 0.],
                            [2., 0.]
                        ],
                        vec![
                            [2., 2.],
                            [1., 2.],
                            [1., 1.],
                            [2., 1.],
                            [3., 1.],
                            [3., 2.],
                            [2., 2.]
                        ]
                    ]]
                }
            );
        } else {
            panic!("Topology must have a collection of geometries.")
        }
    }

    //
    // +-------+-------+            +-------+-------+
    // |       |       |            |               |
    // |   +---+---+   |    ==>     |               |
    // |   |   |   |   |            |               |
    // |   +---+---+   |            |               |
    // |       |       |            |               |
    // +-------+-------+            +-------+-------+
    //
    #[test]
    fn test_merge_7() {
        let topology = TopoJSON {
            bbox: Vec::new(),
            transform: None,
            objects: HashMap::from_iter([(
                "collection".to_string(),
                Geometry {
                    geometry: GeometryType::GeometryCollection {
                        geometries: vec![
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![0, 1, 2, 3]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![-3, 4, -1, 5]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![6, -2]],
                                },
                                id: None,
                                properties: None,
                                bbox: None,
                            },
                            Geometry {
                                geometry: GeometryType::Polygon {
                                    arcs: vec![vec![-7, -5]],
                                },
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
            arcs: vec![
                vec![[2, 3], [2, 2]],
                vec![[2, 2], [1, 2], [1, 1], [2, 1]],
                vec![[2, 1], [2, 0]],
                vec![[2, 0], [0, 0], [0, 3], [2, 3]],
                vec![[2, 1], [3, 1], [3, 2], [2, 2]],
                vec![[2, 3], [4, 3], [4, 0], [2, 0]],
                vec![[2, 2], [2, 1]],
            ],
        };
        if let GeometryType::GeometryCollection { geometries } =
            &topology.objects["collection"].geometry
        {
            let merge = wrap_merge(&topology, &geometries);
            assert_eq!(
                merge,
                FeatureGeometryType::MultiPolygon {
                    coordinates: vec![vec![vec![
                        [2., 0.],
                        [0., 0.],
                        [0., 3.],
                        [2., 3.],
                        [4., 3.],
                        [4., 0.],
                        [2., 0.]
                    ]]]
                }
            );
        } else {
            panic!("Topology must have a collection of geometries.")
        }
    }
}
