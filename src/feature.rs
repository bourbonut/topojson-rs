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
    use crate::topojson_structs::{Properties, Transform};
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
            assert!(matches!(
                feature_item.geometry.geometry,
                FeatureGeometryType::Polygon { .. }
            ));
        } else {
            panic!("Result should be variant of Feature::Item")
        }
        Ok(())
    }

    #[test]
    fn test_feature_2() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Point".to_string(),
            geometry: GeometryType::Point {
                coordinates: vec![0., 0.],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "Point".to_string(),
                    geometry: FeatureGeometryType::Point {
                        coordinates: vec![0., 0.]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_3() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "MultiPoint".to_string(),
            geometry: GeometryType::MultiPoint {
                coordinates: vec![vec![0., 0.]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "MultiPoint".to_string(),
                    geometry: FeatureGeometryType::MultiPoint {
                        coordinates: vec![vec![0., 0.]]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_4() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "LineString".to_string(),
            geometry: GeometryType::LineString { arcs: vec![0] },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "LineString".to_string(),
                    geometry: FeatureGeometryType::LineString {
                        coordinates: vec![
                            vec![0., 0.],
                            vec![1., 0.],
                            vec![1., 1.],
                            vec![0., 1.],
                            vec![0., 0.]
                        ]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_5() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "MultiLineString".to_string(),
            geometry: GeometryType::MultiLineString {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "MultiLineString".to_string(),
                    geometry: FeatureGeometryType::MultiLineString {
                        coordinates: vec![vec![
                            vec![0., 0.],
                            vec![1., 0.],
                            vec![1., 1.],
                            vec![0., 1.],
                            vec![0., 0.]
                        ]]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_6() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "LineString".to_string(),
            geometry: GeometryType::LineString { arcs: vec![3] },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "LineString".to_string(),
                    geometry: FeatureGeometryType::LineString {
                        coordinates: vec![vec![1., 1.], vec![1., 1.],]
                    }
                },
                id: None,
                bbox: None
            })
        );

        let t = simple_topology(Geometry {
            r#type: "MultiLineString".to_string(),
            geometry: GeometryType::MultiLineString {
                arcs: vec![vec![3], vec![4]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "MultiLineString".to_string(),
                    geometry: FeatureGeometryType::MultiLineString {
                        coordinates: vec![
                            vec![vec![1., 1.], vec![1., 1.],],
                            vec![vec![0., 0.], vec![0., 0.],]
                        ]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_7() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "Polygon".to_string(),
                    geometry: FeatureGeometryType::Polygon {
                        coordinates: vec![vec![
                            vec![0., 0.],
                            vec![1., 0.],
                            vec![1., 1.],
                            vec![0., 1.],
                            vec![0., 0.]
                        ]]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_8() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "MultiPolygon".to_string(),
            geometry: GeometryType::MultiPolygon {
                arcs: vec![vec![vec![0]]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "MultiPolygon".to_string(),
                    geometry: FeatureGeometryType::MultiPolygon {
                        coordinates: vec![vec![vec![
                            vec![0., 0.],
                            vec![1., 0.],
                            vec![1., 1.],
                            vec![0., 1.],
                            vec![0., 0.]
                        ]]]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_9() -> PyResult<()> {
        let topology = TopoJSON {
            r#type: "Topology".to_string(),
            bbox: vec![],
            transform: Some(Transform {
                scale: vec![1., 1.],
                translate: vec![0., 0.],
            }),
            objects: HashMap::from_iter([
                (
                    "foo".to_string(),
                    Geometry {
                        r#type: "Polygon".to_string(),
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
                        r#type: "Polygon".to_string(),
                        geometry: GeometryType::Polygon {
                            arcs: vec![vec![0, 1]],
                        },
                        id: None,
                        properties: None,
                        bbox: None,
                    },
                ),
            ]),
            arcs: vec![vec![vec![0, 0], vec![1, 1]], vec![vec![1, 1], vec![-1, -1]]],
        };

        if let Feature::Item(feature) = wrap_feature(&topology, &topology.objects["foo"])? {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![vec![0., 0.], vec![1., 1.], vec![0., 0.], vec![0., 0.]]]
                );
            } else {
                panic!("FeatureGeometryType of 'foo' must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature of 'foo' must be variant of 'Item'.")
        }

        if let Feature::Item(feature) = wrap_feature(&topology, &topology.objects["bar"])? {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![vec![0., 0.], vec![1., 1.], vec![0., 0.], vec![0., 0.]]]
                );
            } else {
                panic!("FeatureGeometryType of 'bar' must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature of 'bar' must be variant of 'Item'.")
        }

        Ok(())
    }

    #[test]
    fn test_feature_10() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "GeometryCollection".to_string(),
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    r#type: "MultiPolygon".to_string(),
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
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                r#type: "FeatureCollection".to_string(),
                features: vec![FeatureItem {
                    r#type: "Feature".to_string(),
                    properties: None,
                    geometry: FeatureGeometry {
                        r#type: "MultiPolygon".to_string(),
                        geometry: FeatureGeometryType::MultiPolygon {
                            coordinates: vec![vec![vec![
                                vec![0., 0.],
                                vec![1., 0.],
                                vec![1., 1.],
                                vec![0., 1.],
                                vec![0., 0.]
                            ]]]
                        }
                    },
                    id: None,
                    bbox: None
                }]
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_11() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "GeometryCollection".to_string(),
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    r#type: "Point".to_string(),
                    geometry: GeometryType::Point {
                        coordinates: vec![0., 0.],
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
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                r#type: "FeatureCollection".to_string(),
                features: vec![FeatureItem {
                    r#type: "Feature".to_string(),
                    properties: None,
                    geometry: FeatureGeometry {
                        r#type: "Point".to_string(),
                        geometry: FeatureGeometryType::Point {
                            coordinates: vec![0., 0.]
                        }
                    },
                    id: None,
                    bbox: None
                }]
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_12() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "GeometryCollection".to_string(),
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    r#type: "Point".to_string(),
                    geometry: GeometryType::Point {
                        coordinates: vec![0., 0.],
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
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                r#type: "FeatureCollection".to_string(),
                features: vec![FeatureItem {
                    r#type: "Feature".to_string(),
                    properties: None,
                    geometry: FeatureGeometry {
                        r#type: "Point".to_string(),
                        geometry: FeatureGeometryType::Point {
                            coordinates: vec![0., 0.]
                        }
                    },
                    id: Some("feature".to_string()),
                    bbox: None
                }]
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_13() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "GeometryCollection".to_string(),
            geometry: GeometryType::GeometryCollection {
                geometries: vec![Geometry {
                    r#type: "Point".to_string(),
                    geometry: GeometryType::Point {
                        coordinates: vec![0., 0.],
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
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Collection(FeatureCollection {
                r#type: "FeatureCollection".to_string(),
                features: vec![FeatureItem {
                    r#type: "Feature".to_string(),
                    properties: Some(Properties {
                        name: "feature".to_string()
                    }),
                    geometry: FeatureGeometry {
                        r#type: "Point".to_string(),
                        geometry: FeatureGeometryType::Point {
                            coordinates: vec![0., 0.]
                        }
                    },
                    id: None,
                    bbox: None
                }]
            })
        );
        Ok(())
    }

    #[test]
    fn test_feature_14() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: Some("foo".to_string()),
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"])? {
            assert_eq!(feature.id, Some("foo".to_string()));
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
        Ok(())
    }

    #[test]
    fn test_feature_15() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: Some(Properties {
                name: "property".to_string(),
            }),
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"])? {
            assert_eq!(
                feature.properties,
                Some(Properties {
                    name: "property".to_string()
                })
            );
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
        Ok(())
    }

    #[test]
    fn test_feature_16() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"])? {
            assert_eq!(feature.id, None);
            assert_eq!(feature.properties, None);
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
        Ok(())
    }

    #[test]
    fn test_feature_17() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"])? {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![
                        vec![0., 0.],
                        vec![1., 0.],
                        vec![1., 1.],
                        vec![0., 1.],
                        vec![0., 0.]
                    ]]
                );
            } else {
                panic!("Feature Geometry Type must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
        Ok(())
    }

    #[test]
    fn test_feature_18() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![!0]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        if let Feature::Item(feature) = wrap_feature(&t, &t.objects["foo"])? {
            if let FeatureGeometryType::Polygon { coordinates } = feature.geometry.geometry {
                assert_eq!(
                    coordinates,
                    vec![vec![
                        vec![0., 0.],
                        vec![0., 1.],
                        vec![1., 1.],
                        vec![1., 0.],
                        vec![0., 0.]
                    ]]
                );
            } else {
                panic!("Feature Geometry Type must be variant of 'Polygon'.")
            }
        } else {
            panic!("Feature must be variant of 'Item'.")
        }
        Ok(())
    }

    #[test]
    fn test_feature_19() -> PyResult<()> {
        let t = simple_topology(Geometry {
            r#type: "LineString".to_string(),
            geometry: GeometryType::LineString { arcs: vec![1, 2] },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "LineString".to_string(),
                    geometry: FeatureGeometryType::LineString {
                        coordinates: vec![
                            vec![0., 0.],
                            vec![1., 0.],
                            vec![1., 1.],
                            vec![0., 1.],
                            vec![0., 0.]
                        ]
                    }
                },
                id: None,
                bbox: None
            })
        );

        let t = simple_topology(Geometry {
            r#type: "Polygon".to_string(),
            geometry: GeometryType::Polygon {
                arcs: vec![vec![!2, !1]],
            },
            id: None,
            properties: None,
            bbox: None,
        });
        let feature = wrap_feature(&t, &t.objects["foo"])?;
        assert_eq!(
            feature,
            Feature::Item(FeatureItem {
                r#type: "Feature".to_string(),
                properties: None,
                geometry: FeatureGeometry {
                    r#type: "Polygon".to_string(),
                    geometry: FeatureGeometryType::Polygon {
                        coordinates: vec![vec![
                            vec![0., 0.],
                            vec![0., 1.],
                            vec![1., 1.],
                            vec![1., 0.],
                            vec![0., 0.]
                        ]]
                    }
                },
                id: None,
                bbox: None
            })
        );
        Ok(())
    }
}
