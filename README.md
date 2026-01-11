# Topojson Client in Rust

Implementation of [`topojson-client`](https://github.com/topojson/topojson-client) in Rust for creating native Python extension.

I don't want to distribute it the library on Pypi because I don't think it is yet ready for production due to some potential performance improvements to make and non-polished API. However if you want to test it, you can follow the manual instructions.

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

Current benchmark (see `benchmark.py`, files available [here](https://github.com/topojson/us-atlas) and [fork of `pytopojson`](https://github.com/bourbonut/pytopojson) for correct results).
```
        Feature Land: ratio: 2.199, python:  2.270 ms, rust:  1.032 ms (True)
    Feature Counties: ratio: 1.924, python: 66.036 ms, rust: 34.328 ms (True)
           Mesh Land: ratio: 3.098, python:  2.754 ms, rust:  0.889 ms (True)
       Mesh Counties: ratio: 2.070, python: 69.451 ms, rust: 33.555 ms (True)
               Merge: ratio: 1.958, python: 25.163 ms, rust: 12.854 ms (True)
           Neighbors: ratio: 4.376, python: 21.461 ms, rust:  4.904 ms (True)
           Bbox Land: ratio: 5.435, python:  1.864 ms, rust:  0.343 ms (True)
       Bbox Counties: ratio: 2.430, python: 16.045 ms, rust:  6.603 ms (True)
            Quantize: ratio: 2.655, python: 49.822 ms, rust: 18.763 ms (True)
```
