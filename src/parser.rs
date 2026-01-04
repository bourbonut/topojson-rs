use std::collections::HashMap;

use crate::topojson_structs::{Geometry, GeometryType, Properties, TopoJSON, Transform};
use json::{JsonValue, object::Object};

const PREFIX_URL: &str =
    "https://raw.githubusercontent.com/topojson/topojson-client/refs/heads/master";

pub async fn request(filepath: &str) -> Result<String, String> {
    let url = format!("{PREFIX_URL}/{filepath}")
        .parse::<reqwest::Url>()
        .map_err(|e| format!("Cannot parse the URL: {}", e.to_string()))?;
    println!("url: {:?}", url.to_string());
    Ok(reqwest::get(url)
        .await
        .map_err(|e| format!("Cannot send a request: {}", e.to_string()))?
        .text()
        .await
        .map_err(|e| format!("Cannot get the text from the request: {}", e.to_string()))?)
}

fn array_from_item<T, F>(item: &JsonValue, f: F) -> Option<Vec<T>>
where
    F: FnMut(&JsonValue) -> Option<T>,
{
    if let JsonValue::Array(values) = item {
        Some(values.iter().filter_map(f).collect())
    } else {
        None
    }
}

fn array_from_key<T, F>(obj: &Object, key: &str, f: F) -> Vec<T>
where
    F: FnMut(&JsonValue) -> Option<T>,
{
    if let Some(item) = obj.get(key) {
        array_from_item(item, f).unwrap_or(Vec::new())
    } else {
        Vec::new()
    }
}

fn value_from_key<'a>(
    obj: &'a Object,
    key: &str,
    value: &'a JsonValue,
) -> Result<&'a JsonValue, String> {
    obj.get(key).ok_or(format!("No key '{key}' in {value:?}"))
}

fn bbox(obj: &Object) -> Vec<f64> {
    array_from_key(&obj, "bbox", |item| item.as_f64())
}

fn transform(obj: &Object) -> Option<Transform> {
    if let Some(JsonValue::Object(transform)) = obj.get("transform") {
        Some(Transform {
            scale: array_from_key(&transform, "scale", |item| item.as_f64()),
            translate: array_from_key(&transform, "translate", |item| item.as_f64()),
        })
    } else {
        None
    }
}

fn arcs(obj: &Object) -> Vec<Vec<Vec<i32>>> {
    if let Some(item) = obj.get("arcs") {
        array_from_item(item, |item| {
            array_from_item(item, |item| array_from_item(item, |item| item.as_i32()))
        })
        .unwrap_or(Vec::new())
    } else {
        Vec::new()
    }
}

fn objects(obj: &Object) -> Result<HashMap<String, Geometry>, String> {
    if let Some(JsonValue::Object(objects)) = obj.get("objects") {
        objects
            .iter()
            .map(|(key, value)| Ok((key.to_string(), geometry(value)?)))
            .collect()
    } else {
        Ok(HashMap::new())
    }
}

fn geometry(value: &JsonValue) -> Result<Geometry, String> {
    if let JsonValue::Object(obj) = value {
        let r#type = value_from_key(obj, "type", value)?
            .as_str()
            .ok_or(format!("'type' must be a string (value = {:?})", value))?;
        let geometry = match r#type {
            "GeometryCollection" => GeometryType::GeometryCollection {
                geometries: {
                    if let JsonValue::Array(geometries) = value_from_key(obj, "geometries", value)?
                    {
                        geometries
                            .iter()
                            .map(|value| Ok(geometry(value)?))
                            .collect::<Result<Vec<Geometry>, String>>()?
                    } else {
                        return Err(format!("geometries must be an array in {:?}", value));
                    }
                },
            },
            "Point" => GeometryType::Point {
                coordinates: array_from_item(value_from_key(obj, "coordinates", value)?, |item| {
                    item.as_f64()
                })
                .ok_or(format!("'Point' cannot be parsed (value = {:?})", value))?,
            },
            "MultiPoint" => GeometryType::MultiPoint {
                coordinates: array_from_item(value_from_key(obj, "coordinates", value)?, |item| {
                    array_from_item(item, |item| item.as_f64())
                })
                .ok_or(format!(
                    "'MultiPoint' cannot be parsed (value = {:?})",
                    value
                ))?,
            },
            "LineString" => GeometryType::LineString {
                arcs: array_from_item(value_from_key(obj, "arcs", value)?, |item| item.as_i32())
                    .ok_or(format!(
                        "'LineString' cannot be parsed (value = {:?})",
                        value
                    ))?,
            },
            "MultiLineString" => GeometryType::MultiLineString {
                arcs: array_from_item(value_from_key(obj, "arcs", value)?, |item| {
                    array_from_item(item, |item| item.as_i32())
                })
                .ok_or(format!(
                    "'MultiLineString' cannot be parsed (value = {:?})",
                    value
                ))?,
            },
            "Polygon" => GeometryType::Polygon {
                arcs: array_from_item(value_from_key(obj, "arcs", value)?, |item| {
                    array_from_item(item, |item| item.as_i32())
                })
                .ok_or(format!("'Polygon' cannot be parsed (value = {:?})", value))?,
            },
            "MultiPolygon" => GeometryType::MultiPolygon {
                arcs: array_from_item(value_from_key(obj, "arcs", value)?, |item| {
                    array_from_item(item, |item| array_from_item(item, |item| item.as_i32()))
                })
                .ok_or(format!(
                    "'MultiPolygon' cannot be parsed (value = {:?})",
                    value
                ))?,
            },
            unknown_type => return Err(format!("Unknown type '{unknown_type}' in 'objects'")),
        };
        let id = obj
            .get("id")
            .and_then(|id| id.as_str().map(|id| id.to_string()));
        let bbox = obj
            .get("bbox")
            .and_then(|item| array_from_item(item, |item| item.as_f64()));
        let properties = obj.get("properties").and_then(|item| {
            if let JsonValue::Object(properties) = item {
                properties.get("name").and_then(|name| {
                    name.as_str().map(|name_str| Properties {
                        name: name_str.to_string(),
                    })
                })
            } else {
                None
            }
        });
        Ok(Geometry {
            geometry,
            properties,
            id,
            bbox,
        })
    } else {
        Err(format!(
            "The following object cannot be parsed: {:?}",
            value
        ))
    }
}

impl TryFrom<JsonValue> for TopoJSON {
    type Error = String;

    fn try_from(parsed: JsonValue) -> Result<Self, Self::Error> {
        if let JsonValue::Object(obj) = parsed {
            let bbox = bbox(&obj);
            let transform = transform(&obj);
            let arcs = arcs(&obj);
            let objects = objects(&obj)?;
            Ok(TopoJSON {
                bbox,
                transform,
                arcs,
                objects,
            })
        } else {
            Err("Invalid json content.".to_string())
        }
    }
}

#[tokio::test]
async fn test_parsing() -> Result<(), String> {
    let json_content = request("test/topojson/polygon-q1e4.json").await?;
    let topology = TopoJSON::try_from(
        json::parse(&json_content)
            .map_err(|e| format!("Cannot parse the content through 'json': {}", e.to_string()))?,
    )?;
    assert_eq!(topology.bbox, vec![0., 0., 10., 10.]);
    if let Some(transform) = topology.transform {
        assert_eq!(transform.scale, vec![0.001000100010001, 0.001000100010001]);
        assert_eq!(transform.translate, vec![0., 0.],)
    } else {
        panic!("'transform' is not defined.")
    };
    assert_eq!(
        topology.arcs,
        vec![vec![
            vec![0, 0],
            vec![0, 9999],
            vec![9999, 0],
            vec![0, -9999],
            vec![-9999, 0]
        ]]
    );
    if let Some(object) = topology.objects.get("polygon") {
        assert!(object.bbox.is_none());
        assert!(object.id.is_none());
        assert!(object.properties.is_none());
        if let GeometryType::Polygon { arcs } = &object.geometry {
            assert_eq!(arcs, &vec![vec![0]]);
        } else {
            panic!("'polygon' object is not variant of 'GeometryType::Polygon'");
        }
    } else {
        panic!("'polygon' not found in 'objects'");
    }
    Ok(())
}
