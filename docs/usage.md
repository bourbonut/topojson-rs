---
title: Usage
---

```py
import topojson

topology = topojson.read(file)

land = topology.objects["land"]

features = topojson.feature(topology, land)
features = topology.feature("land")

mesh = topojson.mesh(topology, land, filter=None)
mesh = topology.mesh("land", filter=None)

objects = topology.objects["counties"].geometries
merge = topojson.merge(objects)
merge = topology.merge("counties")

objects = list(topology["objects"].values())
neighbors = topojson.neighbors(objects)
neighbors = topology.neighbors(list(topology["objects"].keys()))

bbox = topojson.bbox(topology)
bbox = topology.compute_bbox()

quantize = topojson.quantize(topology, 1e4)
quantize = topology.quantize(1e4)
```
