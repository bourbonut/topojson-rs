import json
from math import isclose
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


def check_struct(obj, attributes):
    return all(map(lambda attr: hasattr(obj, attr), attributes))


def check_none(obj, key):
    check = obj.get(key) is None
    assert check, f"{key} found in {obj['type']}"
    return check


def compare(actual, expected):
    if actual is None:
        assert expected is None
    elif isinstance(actual, list):
        assert isinstance(expected, list)
        result = all(map(compare, actual, expected))
        assert result
        return result
    elif isinstance(actual, dict):
        assert isinstance(expected, dict)
        check = True
        for key in actual:
            assert key in expected
            check = check and compare(actual[key], expected[key])
        assert check
        return check
    elif isinstance(actual, float):
        assert isinstance(expected, (int, float))
        result = isclose(actual, expected, abs_tol=1e-6)
        assert result, f"{actual}, {expected}"
        return result
    elif isinstance(actual, int):
        assert isinstance(expected, int)
        result = actual == expected
        assert result, f"{actual}, {expected}"
        return result
    elif isinstance(actual, str):
        assert isinstance(expected, str)
        try:
            actual_dict = json.loads(actual)
            expected_dict = json.loads(expected)
            result = actual_dict == expected_dict
            assert result, f"{actual_dict}, {expected_dict}"
            return result
        except json.decoder.JSONDecodeError:
            result = actual == expected
            assert result, f"{actual}, {expected}"
            return result
    # TopoJSON
    elif check_struct(actual, ["bbox", "transform", "objects", "arcs"]):
        bbox_check = (
            check_none(expected, "bbox")
            if actual.bbox is None
            else compare(actual.bbox, expected["bbox"])
        )
        transform_check = (
            check_none(expected, "transform")
            if actual.transform is None
            else compare(actual.transform, expected["transform"])
        )
        objects_check = (
            check_none(expected, "objects")
            if actual.objects is None
            else compare(actual.objects, expected["objects"])
        )
        arcs_check = (
            check_none(expected, "arcs")
            if actual.arcs is None
            else compare(actual.arcs, expected["arcs"])
        )
        result = bbox_check and transform_check and objects_check and arcs_check
        assert result
        return result
    # Transform
    elif check_struct(actual, ["scale", "translate"]):
        scale_check = compare(actual.scale, expected["scale"])
        translate_check = compare(actual.translate, expected["translate"])
        return scale_check and translate_check
    # Geometry from TopoJSON ("GeometryCollection")
    elif check_struct(actual, ["geometries", "id", "properties", "bbox"]):
        assert expected["type"] == "GeometryCollection"
        geometries_check = compare(actual.geometries, expected["geometries"])
        properties_check = (
            expected.get("properties") == {} or check_none(expected, "properties")
            if actual.properties is None
            else compare(actual.properties, json.dumps(expected["properties"]))
        )
        bbox_check = (
            check_none(expected, "bbox")
            if actual.bbox is None
            else compare(actual.bbox, expected["bbox"])
        )
        id_check = (
            check_none(expected, "id")
            if actual.id is None
            else compare(actual.id, expected["id"])
        )
        result = geometries_check and properties_check and bbox_check and id_check
        assert result
        return result
    # Geometry from TopoJSON ("Point", "MultiPoint")
    elif check_struct(actual, ["coordinates", "id", "properties", "bbox"]):
        assert expected["type"] in ["Point", "MultiPoint"]
        coordinates_check = compare(actual.coordinates, expected["coordinates"])
        properties_check = (
            expected.get("properties") == {} or check_none(expected, "properties")
            if actual.properties is None
            else compare(actual.properties, json.dumps(expected["properties"]))
        )
        bbox_check = (
            check_none(expected, "bbox")
            if actual.bbox is None
            else compare(actual.bbox, expected["bbox"])
        )
        id_check = (
            check_none(expected, "id")
            if actual.id is None
            else compare(actual.id, expected["id"])
        )
        result = coordinates_check and properties_check and bbox_check and id_check
        assert result
        return result
    # Geometry from TopoJSON ("LineString", "MultiLineString", "Polygon", "MultiPolygon")
    elif check_struct(actual, ["arcs", "id", "properties", "bbox"]):
        assert expected["type"] in [
            "LineString",
            "MultiLineString",
            "Polygon",
            "MultiPolygon",
        ]
        arcs_check = compare(actual.arcs, expected["arcs"])
        properties_check = (
            expected.get("properties") == {} or check_none(expected, "properties")
            if actual.properties is None
            else compare(actual.properties, json.dumps(expected["properties"]))
        )
        bbox_check = (
            check_none(expected, "bbox")
            if actual.bbox is None
            else compare(actual.bbox, expected["bbox"])
        )
        id_check = (
            check_none(expected, "id")
            if actual.id is None
            else compare(actual.id, expected["id"])
        )
        result = arcs_check and properties_check and bbox_check and id_check
        assert result
        return result
    # FeatureItem
    elif check_struct(actual, ["properties", "geometry", "id", "bbox"]):
        assert expected["type"] in ["Feature"]
        geometry_check = compare(actual.geometry, expected["geometry"])
        properties_check = (
            expected.get("properties") == {} or check_none(expected, "properties")
            if actual.properties is None
            else compare(actual.properties, json.dumps(expected["properties"]))
        )
        bbox_check = (
            check_none(expected, "bbox")
            if actual.bbox is None
            else compare(actual.bbox, expected["bbox"])
        )
        id_check = (
            check_none(expected, "id")
            if actual.id is None
            else compare(actual.id, expected["id"])
        )
        result = geometry_check and properties_check and bbox_check and id_check
        assert result
        return result
    # FeatureGeometryType
    elif check_struct(actual, ["coordinates"]):
        assert expected["type"] in [
            "GeometryCollection",
            "Point",
            "MultiPoint",
            "LineString",
            "MultiLineString",
            "Polygon",
            "MultiPolygon",
        ]
        result = compare(actual.coordinates, expected["coordinates"])
        assert result
        return result
    # FeatureCollection from Feature
    elif check_struct(actual[0], ["features"]):
        actual = actual[0]
        assert expected["type"] == "FeatureCollection"
        result = compare(actual.features, expected["features"])
        assert result
        return result
    # FeatureItem from Feature
    elif check_struct(actual[0], ["properties", "geometry", "id", "bbox"]):
        actual = actual[0]
        assert expected["type"] in ["Feature"]
        geometry_check = compare(actual.geometry, expected["geometry"])
        properties_check = (
            check_none(expected, "properties")
            if actual.properties is None
            else compare(actual.properties, json.dumps(expected["properties"]))
        )
        bbox_check = (
            check_none(expected, "bbox")
            if actual.bbox is None
            else compare(actual.bbox, expected["bbox"])
        )
        id_check = (
            check_none(expected, "id")
            if actual.id is None
            else compare(actual.id, expected["id"])
        )
        result = geometry_check and properties_check and bbox_check and id_check
        assert result
        return result
    else:
        raise TypeError(f"Unknow type {type(actual)}")


def benchmark(name, py_read_file, rs_read_file, py_func, rs_func):
    start = perf_counter()
    topology = py_read_file()
    end = perf_counter()
    r1 = (end - start) * 1_000

    start = perf_counter()
    expected = py_func(topology)
    end = perf_counter()
    t1 = (end - start) * 1_000

    start = perf_counter()
    topology = rs_read_file()
    end = perf_counter()
    r2 = (end - start) * 1_000

    start = perf_counter()
    actual = rs_func(topology)
    end = perf_counter()
    t2 = (end - start) * 1_000

    is_same = compare(actual, expected)
    print(
        f"{name.title():>20}: read - ratio: {r1 / r2:>6.3f}, py: {r1:>6.3f} ms, rs: {r2:>6.3f} ms | compute - ratio: {t1 / t2:>6.3f}, py: {t1:>6.3f} ms, rs: {t2:>6.3f} ms | ({is_same})"
    )


def rs_load_file(filename):
    def wrapper():
        return topojson.read(filename)

    return wrapper


def py_load_file(filename):
    def wrapper():
        with open(filename) as file:
            topology = json.load(file)
        return topology

    return wrapper


def feature_rust(key):
    def wrapper(topology):
        return topojson.feature(topology, topology.objects[key])

    return wrapper


def feature_python(key):
    def wrapper(topology):
        obj = topology["objects"][key]
        return Feature()(topology, obj)

    return wrapper


def mesh_rust(key, filt=None):
    def wrapper(topology):
        return topojson.mesh(topology, topology.objects[key], filter=filt)

    return wrapper


def mesh_python(key, filt=None):
    def wrapper(topology):
        obj = topology["objects"][key]
        return Mesh()(topology, obj, filter=filt)

    return wrapper


def merge_rust(key):
    def wrapper(topology):
        return topojson.merge(topology, topology.objects[key].geometries)

    return wrapper


def merge_python(key):
    def wrapper(topology):
        objects = topology["objects"][key]["geometries"]
        return Merge()(topology, objects)

    return wrapper


def bbox_rust():
    def wrapper(topology):
        return topojson.bbox(topology)

    return wrapper


def bbox_python():
    def wrapper(topology):
        return BBox()(topology)

    return wrapper


def neighbors_rust():
    def wrapper(topology):
        return topojson.neighbors(
            [topology.objects[key] for key in ["counties", "nation", "states"]]
        )

    return wrapper


def neighbors_python():
    def wrapper(topology):
        return Neighbors()(
            [topology["objects"][key] for key in ["counties", "nation", "states"]]
        )

    return wrapper


def quantize_rust():
    def wrapper(topology):
        topology.transform = None
        return topojson.quantize(topology, 1e4)

    return wrapper


def quantize_python():
    def wrapper(topology):
        topology.pop("transform")
        return Quantize()(topology, 1e4)

    return wrapper


benchmark(
    "feature land",
    py_load_file("./land-110m.json"),
    rs_load_file("./land-110m.json"),
    feature_python("land"),
    feature_rust("land"),
)

benchmark(
    "feature states",
    py_load_file("./states-10m.json"),
    rs_load_file("./states-10m.json"),
    feature_python("states"),
    feature_rust("states"),
)

benchmark(
    "feature counties",
    py_load_file("./counties-10m.json"),
    rs_load_file("./counties-10m.json"),
    feature_python("counties"),
    feature_rust("counties"),
)

benchmark(
    "mesh land",
    py_load_file("./land-110m.json"),
    rs_load_file("./land-110m.json"),
    mesh_python("land"),
    mesh_rust("land"),
)

benchmark(
    "mesh states",
    py_load_file("./states-10m.json"),
    rs_load_file("./states-10m.json"),
    mesh_python("states"),
    mesh_rust("states"),
)

benchmark(
    "mesh counties",
    py_load_file("./counties-10m.json"),
    rs_load_file("./counties-10m.json"),
    mesh_python("counties"),
    mesh_rust("counties"),
)

benchmark(
    "merge land",
    py_load_file("./land-110m.json"),
    rs_load_file("./land-110m.json"),
    merge_python("land"),
    merge_rust("land"),
)

benchmark(
    "merge states",
    py_load_file("./states-10m.json"),
    rs_load_file("./states-10m.json"),
    merge_python("states"),
    merge_rust("states"),
)


benchmark(
    "merge counties",
    py_load_file("./counties-10m.json"),
    rs_load_file("./counties-10m.json"),
    merge_python("counties"),
    merge_rust("counties"),
)

benchmark(
    "bbox land",
    py_load_file("./land-110m.json"),
    rs_load_file("./land-110m.json"),
    bbox_python(),
    bbox_rust(),
)

benchmark(
    "bbox states",
    py_load_file("./states-10m.json"),
    rs_load_file("./states-10m.json"),
    bbox_python(),
    bbox_rust(),
)

benchmark(
    "bbox counties",
    py_load_file("./counties-10m.json"),
    rs_load_file("./counties-10m.json"),
    bbox_python(),
    bbox_rust(),
)

benchmark(
    "neighbors counties",
    py_load_file("./counties-10m.json"),
    rs_load_file("./counties-10m.json"),
    neighbors_python(),
    neighbors_rust(),
)

benchmark(
    "quantize land",
    py_load_file("./land-110m.json"),
    rs_load_file("./land-110m.json"),
    quantize_python(),
    quantize_rust(),
)

benchmark(
    "quantize states",
    py_load_file("./states-10m.json"),
    rs_load_file("./states-10m.json"),
    quantize_python(),
    quantize_rust(),
)

benchmark(
    "quantize counties",
    py_load_file("./counties-10m.json"),
    rs_load_file("./counties-10m.json"),
    quantize_python(),
    quantize_rust(),
)

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
