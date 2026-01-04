import json
from time import perf_counter

import topojson
from pytopojson.bbox import BBox
from pytopojson.feature import Feature
from pytopojson.merge import Merge
from pytopojson.mesh import Mesh
from pytopojson.neighbors import Neighbors
from pytopojson.quantize import Quantize


def load_land():
    with open("./land-110m.json") as file:
        topology = json.load(file)
    return topology


def load_states():
    with open("./states-10m.json") as file:
        topology = json.load(file)
    return topology


def benchmark(name, py_func, rs_func):
    start = perf_counter()
    expected = py_func()
    end = perf_counter()
    t1 = (end - start) * 1_000

    start = perf_counter()
    actual = rs_func()
    end = perf_counter()
    t2 = (end - start) * 1_000

    is_same = actual == expected
    print(
        f"{name.title():>10}: ratio: {t1 / t2:.3f}, python: {t1:.3f} ms, rust: {t2:.3f} ms ({is_same})"
    )


topology = load_land()
land = topology["objects"]["land"]
benchmark(
    "feature",
    lambda: Feature()(topology, land),
    lambda: topojson.feature(topology, land),
)

topology = load_land()
land = topology["objects"]["land"]
benchmark(
    "mesh",
    lambda: Mesh()(topology, land),
    lambda: topojson.mesh(topology, land, filter=None),
)

topology = load_states()
objects = list(topology["objects"].values())
benchmark(
    "merge",
    lambda: Merge()(topology, objects),
    lambda: topojson.merge(topology, objects),
)

topology = load_states()
objects = list(topology["objects"].values())
benchmark(
    "neighbors",
    lambda: Neighbors()(objects),
    lambda: topojson.neighbors(objects),
)

topology = load_land()
benchmark(
    "bbox",
    lambda: BBox()(topology),
    lambda: topojson.bbox(topology),
)

topology = load_land()
topology.pop("transform")
benchmark(
    "quantize",
    lambda: Quantize()(topology, transform=1e4),
    lambda: topojson.quantize(topology, 1e4),
)
