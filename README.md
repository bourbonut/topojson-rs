# Topojson Client in Rust

Implementation of [`topojson-client`](https://github.com/topojson/topojson-client) in Rust for creating native Python extension.

I don't want to distribute the library on Pypi because I don't think it is yet ready for production due to some potential performance improvements to make and non-polished API. However if you want to test it, you can follow the manual instructions.

## Manual installation

Clone the repository, make a virtual environment and compile it

```bash
pip install maturin
maturin develop --release
```

Then don't forget to activate the environment.

## Example

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

## Benchmarks

Current benchmark (see `benchmark.py`, files available [here](https://github.com/topojson/us-atlas) and [fork of `pytopojson`](https://github.com/bourbonut/pytopojson) for correct results). `land`, `states` and `counties` are topojson files.
```
        Feature Land: read - ratio:  2.404, py:  1.376 ms, rs:  0.572 ms | compute - ratio: 18.471, py:  2.060 ms, rs:  0.112 ms | (True)
      Feature States: read - ratio:  1.811, py:  2.747 ms, rs:  1.517 ms | compute - ratio: 42.547, py:  6.641 ms, rs:  0.156 ms | (True)
    Feature Counties: read - ratio:  1.758, py: 19.583 ms, rs: 11.139 ms | compute - ratio: 29.984, py: 64.153 ms, rs:  2.140 ms | (True)
           Mesh Land: read - ratio:  2.304, py:  1.209 ms, rs:  0.525 ms | compute - ratio: 19.146, py:  2.867 ms, rs:  0.150 ms | (True)
         Mesh States: read - ratio:  1.755, py:  2.527 ms, rs:  1.440 ms | compute - ratio: 16.560, py:  6.413 ms, rs:  0.387 ms | (True)
       Mesh Counties: read - ratio:  1.516, py: 16.744 ms, rs: 11.043 ms | compute - ratio:  7.395, py: 64.420 ms, rs:  8.712 ms | (True)
          Merge Land: read - ratio:  2.260, py:  1.186 ms, rs:  0.525 ms | compute - ratio: 19.885, py:  3.861 ms, rs:  0.194 ms | (True)
        Merge States: read - ratio:  2.045, py:  2.758 ms, rs:  1.348 ms | compute - ratio: 13.230, py:  5.745 ms, rs:  0.434 ms | (True)
      Merge Counties: read - ratio:  2.276, py: 25.256 ms, rs: 11.097 ms | compute - ratio:  7.594, py: 33.058 ms, rs:  4.353 ms | (True)
           Bbox Land: read - ratio:  2.402, py:  1.236 ms, rs:  0.515 ms | compute - ratio: 87.579, py:  1.977 ms, rs:  0.023 ms | (True)
         Bbox States: read - ratio:  2.010, py:  2.626 ms, rs:  1.306 ms | compute - ratio: 77.495, py:  3.801 ms, rs:  0.049 ms | (True)
       Bbox Counties: read - ratio:  1.544, py: 16.799 ms, rs: 10.878 ms | compute - ratio: 60.739, py: 15.490 ms, rs:  0.255 ms | (True)
      Neighbors Land: read - ratio:  2.294, py:  1.154 ms, rs:  0.503 ms | compute - ratio:  1.765, py:  0.070 ms, rs:  0.040 ms | (True)
    Neighbors States: read - ratio:  1.953, py:  2.537 ms, rs:  1.299 ms | compute - ratio:  2.490, py:  0.516 ms, rs:  0.207 ms | (True)
  Neighbors Counties: read - ratio:  1.525, py: 16.842 ms, rs: 11.045 ms | compute - ratio:  5.178, py: 20.711 ms, rs:  4.000 ms | (True)
       Quantize Land: read - ratio:  2.368, py:  1.275 ms, rs:  0.538 ms | compute - ratio: 28.964, py:  2.864 ms, rs:  0.099 ms | (True)
     Quantize States: read - ratio:  1.977, py:  2.672 ms, rs:  1.352 ms | compute - ratio: 28.518, py:  7.413 ms, rs:  0.260 ms | (True)
   Quantize Counties: read - ratio:  3.385, py: 33.956 ms, rs: 10.033 ms | compute - ratio: 20.407, py: 49.777 ms, rs:  2.439 ms | (True)
```
