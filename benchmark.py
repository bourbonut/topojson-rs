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


def load_counties():
    with open("./counties-10m.json") as file:
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
        f"{name.title():>20}: ratio: {t1 / t2:.3f}, python: {t1:>6.3f} ms, rust: {t2:>6.3f} ms ({is_same})"
    )


topology = load_land()
obj = topology["objects"]["land"]
benchmark(
    "feature land",
    lambda: Feature()(topology, obj),
    lambda: topojson.feature(topology, obj),
)

topology = load_counties()
obj = topology["objects"]["counties"]
benchmark(
    "feature counties",
    lambda: Feature()(topology, obj),
    lambda: topojson.feature(topology, obj),
)

topology = load_land()
obj = topology["objects"]["land"]
benchmark(
    "mesh land",
    lambda: Mesh()(topology, obj),
    lambda: topojson.mesh(topology, obj, filter=None),
)

topology = load_counties()
obj = topology["objects"]["counties"]
benchmark(
    "mesh counties",
    lambda: Mesh()(topology, obj),
    lambda: topojson.mesh(topology, obj, filter=None),
)

topology = load_counties()
objects = topology["objects"]["counties"]["geometries"]
benchmark(
    "merge",
    lambda: Merge()(topology, objects),
    lambda: topojson.merge(topology, objects),
)

topology = load_counties()
objects = list(topology["objects"].values())
benchmark(
    "neighbors",
    lambda: Neighbors()(objects),
    lambda: topojson.neighbors(objects),
)

topology = load_land()
benchmark(
    "bbox land",
    lambda: BBox()(topology),
    lambda: topojson.bbox(topology),
)

topology = load_counties()
benchmark(
    "bbox counties",
    lambda: BBox()(topology),
    lambda: topojson.bbox(topology),
)

topology = load_counties()
topology.pop("transform")
benchmark(
    "quantize",
    lambda: Quantize()(topology, transform=1e4),
    lambda: topojson.quantize(topology, 1e4),
)
