# Topojson Client in Rust

Implementation of [`topojson-client`](https://github.com/topojson/topojson-client) in Rust for creating native Python extension.

> [!WARNING]
> This project is under development

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

objects = list(topology["objects"].values())
neighbors = topojson.neighbors(objects)

bbox = topojson.bbox(topology)

quantize topojson.quantize(topology, 1e4)
```
