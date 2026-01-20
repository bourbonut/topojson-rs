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

land = topology["objects"]["land"]

features = topojson.feature(topology, land)
mesh = topojson.mesh(topology, land, filter=None)

objects = topology["objects"]["counties"]["geometries"]
merge = topojson.merge(objects)

objects = list(topology["objects"].values())
neighbors = topojson.neighbors(objects)

bbox = topojson.bbox(topology)

quantize topojson.quantize(topology, 1e4)
```

## Benchmarks

Current benchmark (see `benchmark.py`, files available [here](https://github.com/topojson/us-atlas) and [fork of `pytopojson`](https://github.com/bourbonut/pytopojson) for correct results). `land`, `states` and `counties` are topojson files.
```
        Feature Land: ratio:  7.213, python:  3.599 ms, rust:  0.499 ms (True)
      Feature States: ratio:  7.153, python:  9.632 ms, rust:  1.347 ms (True)
    Feature Counties: ratio:  7.276, python: 83.732 ms, rust: 11.507 ms (True)
           Mesh Land: ratio:  8.274, python:  4.116 ms, rust:  0.498 ms (True)
         Mesh States: ratio:  6.495, python:  9.485 ms, rust:  1.460 ms (True)
       Mesh Counties: ratio:  2.862, python: 84.969 ms, rust: 29.691 ms (True)
      Merge Counties: ratio:  3.116, python: 43.853 ms, rust: 14.074 ms (True)
           Bbox Land: ratio:  8.497, python:  3.204 ms, rust:  0.377 ms (True)
         Bbox States: ratio:  6.991, python:  7.208 ms, rust:  1.031 ms (True)
       Bbox Counties: ratio:  3.969, python: 35.108 ms, rust:  8.846 ms (True)
  Neighbors Counties: ratio:  3.342, python: 47.247 ms, rust: 14.136 ms (True)
       Quantize Land: ratio:  9.775, python:  4.858 ms, rust:  0.497 ms (True)
     Quantize States: ratio:  7.931, python:  9.936 ms, rust:  1.253 ms (True)
   Quantize Counties: ratio:  5.368, python: 57.076 ms, rust: 10.633 ms (True)
```
