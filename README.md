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
|                    |      Reading performances      |     Computation performances   |         |
|                    | ------------------------------ | ------------------------------ |         |
|   Function + Data  |  ratio |    python |      rust |  ratio |    python |      rust |  Same ? |
| ------------------ | ------------------------------ | ------------------------------ | ------- |
|       Feature Land |  2.499 |  1.359 ms |  0.544 ms | 19.277 |  2.154 ms |  0.112 ms |    True |
|     Feature States |  2.163 |  2.856 ms |  1.320 ms | 41.704 |  6.581 ms |  0.158 ms |    True |
|   Feature Counties |  1.729 | 19.520 ms | 11.289 ms | 39.676 | 87.367 ms |  2.202 ms |    True |
|          Mesh Land |  2.234 |  1.149 ms |  0.514 ms | 18.909 |  2.713 ms |  0.143 ms |    True |
|        Mesh States |  1.914 |  2.529 ms |  1.321 ms | 16.568 |  6.325 ms |  0.382 ms |    True |
|      Mesh Counties |  1.473 | 16.426 ms | 11.148 ms | 10.043 | 87.903 ms |  8.753 ms |    True |
|         Merge Land |  2.096 |  1.195 ms |  0.570 ms | 18.816 |  3.711 ms |  0.197 ms |    True |
|       Merge States |  1.923 |  2.631 ms |  1.368 ms | 13.350 |  5.605 ms |  0.420 ms |    True |
|     Merge Counties |  1.432 | 15.946 ms | 11.133 ms |  7.546 | 33.093 ms |  4.385 ms |    True |
|          Bbox Land |  1.980 |  3.121 ms |  1.576 ms | 27.834 |  2.555 ms |  0.092 ms |    True |
|        Bbox States |  1.145 |  5.276 ms |  4.608 ms | 25.082 |  5.070 ms |  0.202 ms |    True |
|      Bbox Counties |  1.556 | 16.742 ms | 10.759 ms | 61.178 | 15.506 ms |  0.253 ms |    True |
|     Neighbors Land |  2.354 |  1.197 ms |  0.508 ms |  0.798 |  0.071 ms |  0.089 ms |    True |
|   Neighbors States |  1.957 |  2.636 ms |  1.347 ms |  2.704 |  0.558 ms |  0.206 ms |    True |
| Neighbors Counties |  1.515 | 16.174 ms | 10.677 ms |  5.033 | 20.601 ms |  4.093 ms |    True |
|      Quantize Land |  2.359 |  1.201 ms |  0.509 ms | 30.167 |  2.851 ms |  0.095 ms |    True |
|    Quantize States |  1.861 |  2.492 ms |  1.339 ms | 27.742 |  7.166 ms |  0.258 ms |    True |
|  Quantize Counties |  1.711 | 16.913 ms |  9.883 ms | 20.389 | 48.865 ms |  2.397 ms |    True |
```
