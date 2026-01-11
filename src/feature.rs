use crate::geojson_structs::{Feature, FeatureCollection, FeatureGeometryType, FeatureItem};
use crate::reverse::reverse;
use crate::topojson_structs::{Geometry, GeometryType, TopoJSON};
use crate::transform::{IdentityTransformer, ScaleTransformer, Transformer};

pub fn wrap_feature(topology: &TopoJSON, o: &Geometry) -> Feature {
    match &o.geometry {
        GeometryType::GeometryCollection { geometries } => {
            let features: Vec<FeatureItem> = geometries
                .into_iter()
                .map(|o| feature_item(&topology, &o))
                .collect();
            Feature::Collection(FeatureCollection { features })
        }
        _ => Feature::Item(feature_item(&topology, o)),
    }
}

pub fn object_func(topology: &TopoJSON, o: &Geometry) -> FeatureGeometryType {
    match &topology.transform {
        Some(transform) => Object::call(topology, o, ScaleTransformer::new(transform)),
        None => Object::call(topology, o, IdentityTransformer::new()),
    }
}

fn feature_item(topology: &TopoJSON, o: &Geometry) -> FeatureItem {
    let geometry = object_func(topology, &o);
    let id = o.id.clone();
    let bbox = o.bbox.clone();
    let properties = o.properties.clone();
    FeatureItem {
        id,
        bbox,
        properties,
        geometry,
    }
}

pub struct Object<'a, T>
where
    T: Transformer,
{
    arcs: &'a Vec<Vec<[i32; 2]>>,
    transformer: T,
}

impl<'a, T: Transformer> Object<'a, T> {
    pub fn call(topology: &TopoJSON, o: &Geometry, transformer: T) -> FeatureGeometryType {
        let mut object = Object {
            arcs: &topology.arcs,
            transformer,
        };
        object.geometry(o)
    }

    fn arc(&mut self, i: i32, points: &mut Vec<[f64; 2]>) {
        if !points.is_empty() {
            points.pop();
        }
        let a = &self.arcs[if i < 0 { !i } else { i } as usize];
        for (k, arc) in a.iter().enumerate() {
            points.push(self.transformer.call(&arc.map(|x| x as f64), k));
        }
        if i < 0 {
            reverse(points, a.len());
        }
    }

    #[inline]
    fn point(&mut self, p: &[f64; 2]) -> [f64; 2] {
        self.transformer.call(p, 0)
    }

    fn line(&mut self, arcs: &[i32]) -> Vec<[f64; 2]> {
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
    fn ring(&mut self, arcs: &[i32]) -> Vec<[f64; 2]> {
        let mut points = self.line(arcs);
        while points.len() < 4 {
            points.push(points[0].clone());
        }
        points
    }

    #[inline]
    fn polygon(&mut self, arcs: &[Vec<i32>]) -> Vec<Vec<[f64; 2]>> {
        arcs.iter().map(|arcs| self.ring(arcs)).collect()
    }

    fn geometry(&mut self, o: &Geometry) -> FeatureGeometryType {
        match &o.geometry {
            GeometryType::GeometryCollection { geometries } => {
                return FeatureGeometryType::GeometryCollection {
                    geometries: geometries.iter().map(|o| self.geometry(o)).collect(),
                };
            }
            GeometryType::Point { coordinates } => FeatureGeometryType::Point {
                coordinates: self.point(coordinates),
            },
            GeometryType::MultiPoint { coordinates } => FeatureGeometryType::MultiPoint {
                coordinates: coordinates.iter().map(|p| self.point(p)).collect(),
            },
            GeometryType::LineString { arcs } => FeatureGeometryType::LineString {
                coordinates: self.line(arcs),
            },
            GeometryType::MultiLineString { arcs } => FeatureGeometryType::MultiLineString {
                coordinates: arcs.iter().map(|arcs| self.line(arcs)).collect(),
            },
            GeometryType::Polygon { arcs } => FeatureGeometryType::Polygon {
                coordinates: self.polygon(arcs),
            },
            GeometryType::MultiPolygon { arcs } => FeatureGeometryType::MultiPolygon {
                coordinates: arcs.iter().map(|arcs| self.polygon(arcs)).collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::topojson_structs::{Properties, Transform};
    use std::collections::HashMap;

    use super::*;

    fn simple_topology(object: Geometry) -> TopoJSON {
        TopoJSON {
            bbox: vec![],
            transform: Some(Transform {
                scale: [1., 1.],
                translate: [0., 0.],
            }),
            objects: HashMap::from_iter([("foo".to_string(), object)]),
            arcs: vec![
                vec![[0, 0], [1, 0], [0, 1], [-1, 0], [0, -1]],
                vec![[0, 0], [1, 0], [0, 1]],
                vec![[1, 1], [-1, 0], [0, -1]],
                vec![[1, 1]],
                vec![[0, 0]],
            ],
        }
    }

    #[test]
    fn test_feature_1() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature_item) = wrap_feature(&t, &t.objects["foo"]) {
            assert!(matches!(
                feature_item.geometry,
                FeatureGeometryType::Polygon { .. }
            ));
        } else {
            panic!("Result should be variant of Feature::Item")
        }
    }

    #[test]
    fn test_feature_2() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Point {
                coordinates: [0., 0.],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::Point {
                    coordinates: [0., 0.]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_3() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::MultiPoint {
                coordinates: vec![[0., 0.]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::MultiPoint {
                    coordinates: vec![[0., 0.]]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_4() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::LineString { arcs: vec![0] },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::LineString {
                    coordinates: vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_5() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::MultiLineString {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::MultiLineString {
                    coordinates: vec![vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]]]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_6() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::LineString { arcs: vec![3] },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::LineString {
                    coordinates: vec![[1., 1.], [1., 1.]]
                },
                id: None,
                bbox: None
            })
        );

        let t = simple_topology(Geometry {
            geometry: GeometryType::MultiLineString {
                arcs: vec![vec![3], vec![4]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::MultiLineString {
                    coordinates: vec![vec![[1., 1.], [1., 1.]], vec![[0., 0.], [0., 0.]]]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_7() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::Polygon {
                    coordinates: vec![vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]]]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_8() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::MultiPolygon {
                arcs: vec![vec![vec![0]]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::MultiPolygon {
                    coordinates: vec![vec![vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]]]]
                },
                id: None,
                bbox: None
            })
        );
    }

    #[test]
    fn test_feature_9() {
        let topology = TopoJSON {
            bbox: vec![],
            transform: Some(Transform {
                scale: [1., 1.],
                translate: [0., 0.],
            }),
            objects: HashMap::from_iter([
                (
                    "foo".to_string(),
                    Geometry {
                        geometry: GeometryType::Polygon {
                            arcs: vec![vec![0]],
                        },
                        id: None,
                        properties: None,
                        bbox: None,
                    },
                ),
                (
                    "bar".to_string(),
                    Geometry {
                        geometry: GeometryType::Polygon {
                            arcs: vec![vec![0, 1]],
                        },
                        id: None,
                        properties: None,
                        bbox: None,
                    },
                ),
            ]),
            arcs: vec![vec![[0, 0], [1, 1]], vec![[1, 1], [-1, -1]]],
        };

        if let Feature::Item(feature) = wrap_feature(&topology, &topology.objects["foo"]) {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![[0., 0.], [1., 1.], [0., 0.], [0., 0.]]]
                );
            } else {
                panic!("FeatureGeometryType of 'foo' must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature of 'foo' must be variant of 'Item'.")
        }

        if let Feature::Item(feature) = wrap_feature(&topology, &topology.objects["bar"]) {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![[0., 0.], [1., 1.], [0., 0.], [0., 0.]]]
                );
            } else {
                panic!("FeatureGeometryType of 'bar' must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature of 'bar' must be variant of 'Item'.")
        }
    }

    #[test]
    fn test_feature_10() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    geometry: GeometryType::MultiPolygon {
                        arcs: vec![vec![vec![0]]],
                    },
                    id: None,
                    properties: None,
                    bbox: None,
                }],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                features: vec![FeatureItem {
                    properties: None,
                    geometry: FeatureGeometryType::MultiPolygon {
                        coordinates: vec![vec![vec![
                            [0., 0.],
                            [1., 0.],
                            [1., 1.],
                            [0., 1.],
                            [0., 0.]
                        ]]]
                    },
                    id: None,
                    bbox: None
                }]
            })
        );
    }

    #[test]
    fn test_feature_11() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    geometry: GeometryType::Point {
                        coordinates: [0., 0.],
                    },
                    id: None,
                    properties: None,
                    bbox: None,
                }],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                features: vec![FeatureItem {
                    properties: None,
                    geometry: FeatureGeometryType::Point {
                        coordinates: [0., 0.]
                    },
                    id: None,
                    bbox: None
                }]
            })
        );
    }

    #[test]
    fn test_feature_12() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    geometry: GeometryType::Point {
                        coordinates: [0., 0.],
                    },
                    id: Some("feature".to_string()),
                    properties: None,
                    bbox: None,
                }],
            },
            id: Some("collection".to_string()),
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                features: vec![FeatureItem {
                    properties: None,
                    geometry: FeatureGeometryType::Point {
                        coordinates: [0., 0.]
                    },
                    id: Some("feature".to_string()),
                    bbox: None
                }]
            })
        );
    }

    #[test]
    fn test_feature_13() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    geometry: GeometryType::Point {
                        coordinates: [0., 0.],
                    },
                    id: None,
                    properties: Some(Properties {
                        name: "feature".to_string(),
                    }),
                    bbox: None,
                }],
            },
            id: None,
            properties: Some(Properties {
                name: "collection".to_string(),
            }),
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                features: vec![FeatureItem {
                    properties: Some(Properties {
                        name: "feature".to_string()
                    }),
                    geometry: FeatureGeometryType::Point {
                        coordinates: [0., 0.]
                    },
                    id: None,
                    bbox: None
                }]
            })
        );
    }

    #[test]
    fn test_feature_14() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: Some("foo".to_string()),
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"]) {
            assert_eq!(feature.id, Some("foo".to_string()));
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
    }

    #[test]
    fn test_feature_15() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: Some(Properties {
                name: "property".to_string(),
            }),
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"]) {
            assert_eq!(
                feature.properties,
                Some(Properties {
                    name: "property".to_string()
                })
            );
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
    }

    #[test]
    fn test_feature_16() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"]) {
            assert_eq!(feature.id, None);
            assert_eq!(feature.properties, None);
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
    }

    #[test]
    fn test_feature_17() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"]) {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]]]
                );
            } else {
                panic!("Feature Geometry Type must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
    }

    #[test]
    fn test_feature_18() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![!0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"]) {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![[0., 0.], [0., 1.], [1., 1.], [1., 0.], [0., 0.]]]
                );
            } else {
                panic!("Feature Geometry Type must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
    }

    #[test]
    fn test_feature_19() {
        let t = simple_topology(Geometry {
            geometry: GeometryType::LineString { arcs: vec![1, 2] },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::LineString {
                    coordinates: vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.], [0., 0.]]
                },
                id: None,
                bbox: None
            })
        );

        let t = simple_topology(Geometry {
            geometry: GeometryType::Polygon {
                arcs: vec![vec![!2, !1]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"]);
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                properties: None,
                geometry: FeatureGeometryType::Polygon {
                    coordinates: vec![vec![[0., 0.], [0., 1.], [1., 1.], [1., 0.], [0., 0.]]]
                },
                id: None,
                bbox: None
            })
        );
    }
}
