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


def feature_rust(filename, key):
    def wrapper():
        topology = topojson.read(filename)
        return topojson.feature(topology, topology.objects[key])

    return wrapper


def feature_python(filename, key):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        obj = topology["objects"][key]
        return Feature()(topology, obj)

    return wrapper


def mesh_rust(filename, key, filt=None):
    def wrapper():
        topology = topojson.read(filename)
        return topojson.mesh(topology, topology.objects[key], filter=filt)

    return wrapper


def mesh_python(filename, key, filt=None):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        obj = topology["objects"][key]
        return Mesh()(topology, obj, filter=filt)

    return wrapper


def merge_rust(filename, key):
    def wrapper():
        topology = topojson.read(filename)
        # return topology.merge(key)
        return topojson.merge(topology, topology.objects[key].geometries)

    return wrapper


def merge_python(filename, key):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        objects = topology["objects"][key]["geometries"]
        return Merge()(topology, objects)

    return wrapper


def bbox_rust(filename):
    def wrapper():
        topology = topojson.read(filename)
        return topojson.bbox(topology)

    return wrapper


def bbox_python(filename):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        return BBox()(topology)

    return wrapper


def neighbors_rust(filename):
    def wrapper():
        topology = topojson.read(filename)
        return topojson.neighbors(list(topology.objects.values()))

    return wrapper


def neighbors_python(filename):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        return Neighbors()(list(topology["objects"].values()))

    return wrapper


def quantize_rust(filename):
    def wrapper():
        topology = topojson.read(filename)
        topology.transform = None
        return topojson.quantize(topology, 1e4)

    return wrapper


def quantize_python(filename):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        topology.pop("transform")
        return Quantize()(topology, 1e4)

    return wrapper


benchmark(
    "feature land",
    feature_python("./land-110m.json", "land"),
    feature_rust("./land-110m.json", "land"),
)


benchmark(
    "feature counties",
    feature_python("./counties-10m.json", "counties"),
    feature_rust("./counties-10m.json", "counties"),
)

benchmark(
    "mesh counties",
    mesh_python("./counties-10m.json", "counties"),
    mesh_rust("./counties-10m.json", "counties"),
)


benchmark(
    "merge counties",
    merge_python("./counties-10m.json", "counties"),
    merge_rust("./counties-10m.json", "counties"),
)

benchmark(
    "bbox counties",
    bbox_python("./counties-10m.json"),
    bbox_rust("./counties-10m.json"),
)

benchmark(
    "neighbors counties",
    neighbors_python("./counties-10m.json"),
    neighbors_rust("./counties-10m.json"),
)

benchmark(
    "quantize counties",
    quantize_python("./counties-10m.json"),
    quantize_rust("./counties-10m.json"),
)

# topology = load_land()
# obj = topology["objects"]["land"]
# benchmark(
#     "mesh land",
#     lambda: Mesh()(topology, obj),
#     lambda: topojson.mesh(topology, obj, filter=None),
# )
#
#
# def filter_func(a, b):
#     return a != b
#
#
# topology = load_states()
# obj = topology["objects"]["states"]
# benchmark(
#     "mesh states",
#     lambda: Mesh()(topology, obj, filt=filter_func),
#     lambda: topojson.mesh(topology, obj, filter=filter_func),
# )
#
#
# def filter_func(a, b):
#     return a != b and int(int(a["id"]) / 1000) == int(int(b["id"]) / 1000)
#
#
# topology = load_counties()
# obj = topology["objects"]["counties"]
# benchmark(
#     "mesh counties",
#     lambda: Mesh()(topology, obj, filt=filter_func),
#     lambda: topojson.mesh(topology, obj, filter=filter_func),
# )
#
# topology = load_counties()
# objects = topology["objects"]["counties"]["geometries"]
# benchmark(
#     "merge",
#     lambda: Merge()(topology, objects),
#     lambda: topojson.merge(topology, objects),
# )
#
# topology = load_counties()
# objects = list(topology["objects"].values())
# benchmark(
#     "neighbors",
#     lambda: Neighbors()(objects),
#     lambda: topojson.neighbors(objects),
# )
#
# topology = load_land()
# benchmark(
#     "bbox land",
#     lambda: BBox()(topology),
#     lambda: topojson.bbox(topology),
# )
#
# topology = load_counties()
# benchmark(
#     "bbox counties",
#     lambda: BBox()(topology),
#     lambda: topojson.bbox(topology),
# )
#
# topology = load_counties()
# topology.pop("transform")
# benchmark(
#     "quantize",
#     lambda: Quantize()(topology, transform=1e4),
#     lambda: topojson.quantize(topology, 1e4),
# )
