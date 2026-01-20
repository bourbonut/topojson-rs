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

Current benchmark (see `benchmark.py`, files available [here](https://github.com/topojson/us-atlas) and [fork of `pytopojson`](https://github.com/bourbonut/pytopojson) for correct results).
```
        Feature Land: ratio:  7.172, python:  3.650 ms, rust:  0.509 ms (True)
      Feature States: ratio:  8.602, python: 10.107 ms, rust:  1.175 ms (True)
    Feature Counties: ratio:  7.398, python: 83.120 ms, rust: 11.235 ms (True)
       Mesh Counties: ratio:  2.874, python: 87.193 ms, rust: 30.339 ms (True)
      Merge Counties: ratio:  3.047, python: 43.656 ms, rust: 14.326 ms (True)
           Bbox Land: ratio:  7.285, python:  3.244 ms, rust:  0.445 ms (True)
         Bbox States: ratio:  7.210, python:  7.185 ms, rust:  0.997 ms (True)
       Bbox Counties: ratio:  4.027, python: 35.674 ms, rust:  8.858 ms (True)
  Neighbors Counties: ratio:  2.819, python: 41.212 ms, rust: 14.622 ms (True)
       Quantize Land: ratio:  8.218, python:  4.735 ms, rust:  0.576 ms (True)
     Quantize States: ratio: 10.954, python: 18.021 ms, rust:  1.645 ms (True)
   Quantize Counties: ratio:  6.132, python: 66.919 ms, rust: 10.912 ms (True)
```
