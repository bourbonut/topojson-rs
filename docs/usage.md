---
title: Usage
---

```py
import topojson

topology = topojson.read(file) # (1)!
topology.write(file) # (2)!

land = topology.objects["land"]

features = topojson.feature(topology, land) # (3)!
features = topology.feature("land") # (4)!
features.write(file) # (5)!

mesh = topojson.mesh(topology, land, filter=None) # (6)!
mesh = topology.mesh("land", filter=None) # (7)!
mesh.write(file) # (8)!

objects = topology.objects["counties"].geometries
merge = topojson.merge(objects) # (9)!
merge = topology.merge("counties") # (10)!
merge.write(file) # (11)!

objects = list(topology["objects"].values())
neighbors = topojson.neighbors(objects) # (12)!
neighbors = topology.neighbors(list(topology["objects"].keys())) # (13)!

bbox = topojson.bbox(topology) # (14)!
bbox = topology.compute_bbox() # (15)!

quantize = topojson.quantize(topology, 1e4) # (16)!
quantize = topology.quantize(1e4) # (17)!
quantize.write(file) # (18)!
```

1. See [topojson.read][topojson.read]
2. See [TopoJSON.write][topojson.TopoJSON.write]
3. See [topojson.feature][topojson.feature]
4. See [TopoJSON.feature][topojson.TopoJSON.feature]
5. See [GeoJSON_FeatureCollection.write][topojson.GeoJSON_FeatureCollection.write] and [GeoJSON_Feature.write][topojson.GeoJSON_Feature.write]
6. See [topojson.mesh][topojson.mesh]
7. See [TopoJSON.mesh][topojson.TopoJSON.mesh]
8. See [FeatureGeometryType_MultiLineString.write][topojson.FeatureGeometryType_MultiLineString.write]
9. See [topojson.merge][topojson.merge]
10. See [TopoJSON.merge][topojson.TopoJSON.merge]
11. See [FeatureGeometryType_MultiLineString.write][topojson.FeatureGeometryType_MultiLineString.write]
12. See [topojson.neighbors][topojson.neighbors]
13. See [TopoJSON.neighbors][topojson.TopoJSON.neighbors]
14. See [topojson.bbox][topojson.bbox]
15. See [TopoJSON.compute_bbox][topojson.TopoJSON.compute_bbox]
16. See [topojson.quantize][topojson.quantize]
17. See [TopoJSON.quantize][topojson.TopoJSON.quantize]
18. See [TopoJSON.write][topojson.TopoJSON.write]
