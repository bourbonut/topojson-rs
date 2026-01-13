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
        Feature Land: ratio: 2.115, python:  2.112 ms, rust:  0.998 ms (True)
    Feature Counties: ratio: 2.527, python: 64.627 ms, rust: 25.576 ms (True)
           Mesh Land: ratio: 2.975, python:  2.708 ms, rust:  0.910 ms (True)
         Mesh States: ratio: 2.762, python:  6.796 ms, rust:  2.461 ms (True)
       Mesh Counties: ratio: 1.991, python: 65.155 ms, rust: 32.720 ms (True)
               Merge: ratio: 1.797, python: 23.230 ms, rust: 12.926 ms (True)
           Neighbors: ratio: 4.419, python: 21.022 ms, rust:  4.757 ms (True)
           Bbox Land: ratio: 5.521, python:  1.896 ms, rust:  0.344 ms (True)
       Bbox Counties: ratio: 2.710, python: 18.356 ms, rust:  6.773 ms (True)
            Quantize: ratio: 2.689, python: 49.849 ms, rust: 18.536 ms (True)
```

I want to emphasize that if we measure the effective times, there is a **huge difference**:
```
        Feature Land: ratio: 24.366, python:  1.992, rust:  0.082 ms
    Feature Counties: ratio: 41.921, python: 65.003, rust:  1.551 ms
           Mesh Land: ratio: 20.978, python:  2.822, rust:  0.135 ms
         Mesh States: ratio: 17.955, python:  6.409, rust:  0.357 ms
       Mesh Counties: ratio:  3.879, python: 75.584, rust: 19.486 ms
               Merge: ratio:  7.009, python: 23.833, rust:  3.400 ms
           Neighbors: ratio:  9.990, python: 20.587, rust:  2.061 ms
           Bbox Land: ratio: 63.149, python:  1.848, rust:  0.029 ms
       Bbox Counties: ratio: 52.910, python: 15.600, rust:  0.295 ms
            Quantize: ratio: 20.785, python: 51.425, rust:  2.474 ms
```

Here, I only measured the time in Rust for **computation** and excluded the **time spent binding** Python objects into Rust structures and vice versa. In other words, I manually wrapped each function with the following code:

```rs
// example for quantize
#[pyfunction]
pub fn quantize(topology: TopoJSON, transform: f64) -> PyResult<TopoJSON> {
    let start = Instant::now();
    let result = wrap_quantize(&topology, &transform);
    let end = Instant::now();
    println!("duration: {:?} ms", (end - start).as_nanos() as f64 * 1e-6);
    result
}
```
Then I compared Python code with Rust code `ratio = python_time / rust_time (duration)`.

In other words, the current bottleneck is binding Python objects into Rust structures.
