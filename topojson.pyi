from collections.abc import Callable
from typing import Optional

class TopoJSON:
    """
    TopoJSON is a JSON format for encoding geographic data structures into a shared topology.

    Notes
    -----
    See [topojson-specification](https://github.com/topojson/topojson-specification)
    """

    bbox: list[float]
    transform: Optional[Transform]
    objects: dict[str, Geometry]
    arcs: list[list[list[int]]]

    def feature(self, key: str) -> GeoJSON:
        """
        Returns the GeoJSON Feature or FeatureCollection for the specified
        object in the given topology. If the object is a
        `Geometry_GeometryCollection`, a `GeoJSON_FeatureCollection` is
        returned, and each geometry in the collection is mapped to a Feature.
        Otherwise, a Feature is returned. The returned feature is a shallow
        copy of the source object: they may share identifiers, bounding boxes,
        properties and coordinates.

        Parameters
        ----------
        key : str
            Key to access the object by doing `topology.objects[key]`

        Returns
        -------
        GeoJSON
            GeoJSON Feature or FeatureCollection

        Raises
        ------
        KeyError
            When `key` is not found in `objects`

        Examples
        --------
        - A point is mapped to a feature with a geometry object of type "Point".
        - Likewise for line strings, polygons, and other simple geometries.
        - A null geometry object (of type null in TopoJSON) is mapped to a feature
          with a null geometry object.
        - A geometry collection of points is mapped to a feature collection of
          features, each with a point geometry.
        - A geometry collection of geometry collections is mapped to a feature
          collection of features, each with a geometry collection.
        """

    def merge(self, key: str) -> FeatureGeometryType_MultiLineString:
        """
        Returns the GeoJSON MultiPolygon geometry object representing the union
        for the specified array of Polygon and MultiPolygon objects in the
        given topology. Interior borders shared by adjacent polygons are
        removed. See Merging States for an example. The returned geometry is a
        shallow copy of the source object: they may share coordinates.

        Parameters
        ----------
        key : str
            Key to access the object by doing `topology.objects[key]`. The
            object must be a `Geometry_GeometryCollection`.

        Returns
        -------
        FeatureGeometryType_MultiLineString
            GeoJSON MultiPolygon geometry object

        Raises
        ------
        KeyError
            When `key` is not found in `objects`
        TypeError
            Selected object is not a `Geometry_GeometryCollection`.
        """

    def mesh(
        self,
        key: Optional[str] = None,
        filter: Optional[Callable[[Geometry, Geometry], bool]] = None,
    ) -> FeatureGeometryType_MultiLineString:
        """
        Returns the GeoJSON MultiLineString geometry object representing the
        mesh for the specified object in the given topology. This is useful for
        rendering strokes in complicated objects efficiently, as edges that are
        shared by multiple features are only stroked once. If object is not
        specified, a mesh of the entire topology is returned. The returned
        geometry is a shallow copy of the source object: they may share
        coordinates.

        Parameters
        ----------
        key : str
            Key to access the object by doing `topology.objects[key]`.
        filter : Optional[Callable[[Geometry, Geometry], bool]]
            The filter function is called once for each candidate arc and takes
            two arguments, a and b, two geometry objects that share that arc.
            Each arc is only included in the resulting mesh if the filter
            function returns true. For typical map topologies the geometries a
            and b are adjacent polygons and the candidate arc is their
            boundary. If an arc is only used by a single geometry then a and b
            are identical.

        Returns
        -------
        FeatureGeometryType_MultiLineString
            GeoJSON MultiLineString geometry object

        Warnings
        --------
        Currently, `filter` argument does not change the result because it is
        avoided due to performance issues.

        Raises
        ------
        KeyError
            When `key` is not found in `objects`
        """

    def compute_bbox(self) -> list[float]:
        """
        Returns the computed bounding box of the specified topology $[x_0, y_0,
        x_1, y_1]$ where $x_0$ is the minimum x-value, $y_0$ is the minimum
        y-value, x_1 is the maximum x-value, and $y_1$ is the maximum y-value.
        If the topology has no points and no arcs, the returned bounding box is
        $[\\infty, \\infty, -\\infty, -\\infty]$.

        (This method ignores the existing topology.bbox, if any.)

        Returns
        -------
        list[float]
            Computed bounding box
        """

    def neighbors(self, keys: list[str]) -> list[list[int]]:
        """
        Returns an array representing the set of neighboring objects for each
        object in the specified objects array. The returned array has the same
        number of elements as the input array; each element i in the returned
        array is the array of indexes for neighbors of object i in the input
        array. For example, if the specified objects array contains the
        features foo and bar, and these features are neighbors, the returned
        array will be `[[1], [0]]`, indicating that foo is a neighbor of bar
        and vice versa. Each array of neighbor indexes for each object is
        guaranteed to be sorted in ascending order.

        Parameters
        ----------
        keys : list[str]
            List of keys for accessing the object by doing
            `topology.objects[key]`.

        Returns
        -------
        list[list[int]]
            Neighboring objects

        Raises
        ------
        KeyError
            When `key` is not found in `objects`
        """

    def quantize(self, transform: float) -> TopoJSON:
        """
        Returns a shallow copy of the specified topology with quantized and
        delta-encoded arcs according to the specified transform object. If the
        topology is already quantized, an error is thrown. See also
        topoquantize.

        The corresponding transform object is first computed using the bounding
        box of the topology. The quantization number `transform` must be a
        positive integer greater than one which determines the maximum number
        of expressible values per dimension in the resulting quantized
        coordinates; typically, a power of ten is chosen such as 1e4, 1e5 or
        1e6. If the topology does not already have a topology.bbox, one is
        computed using topojson.bbox.


        Parameters
        ----------
        transform : float
            The quantization number `transform`.

        Returns
        -------
        TopoJSON
            Shallow copy with quantized and delta-encoded arcs.

        Raises
        ------
        RuntimeError
            If topology is already quantized or transform is smaller than 2.
        """

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class Transform:
    """
    The purpose of the transform is to quantize positions for more efficient
    serialization, by representing positions as integers rather than floats
    """

    scale: list[float]
    translate: list[float]

type Geometry = (
    Geometry_Point
    | Geometry_MultiPoint
    | Geometry_LineString
    | Geometry_MultiLineString
    | Geometry_Polygon
    | Geometry_MultiPolygon
    | Geometry_GeometryCollection
)
Geometry.__doc__ = """A Geometry is a enumerator object"""

class Geometry_Point:
    """
    A geometry describes as a point.
    """

    coordinates: list[float]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

class Geometry_MultiPoint:
    """
    A geometry describes as serie of points.
    """

    coordinates: list[list[float]]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

class Geometry_LineString:
    """
    A geometry describes as serie of arc indexes.
    """

    arcs: list[int]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

class Geometry_MultiLineString:
    """
    A geometry describes as multiple series of series of arc indexes.
    """

    arcs: list[list[int]]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

class Geometry_Polygon:
    """
    A geometry describes as multiple rings of series of arc indexes.
    """

    arcs: list[list[int]]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

class Geometry_MultiPolygon:
    """
    A geometry describes as multiple polygons of rings of series of arc
    indexes.
    """

    arcs: list[list[list[int]]]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

class Geometry_GeometryCollection:
    """
    A geometry describes as a collection of geometries.
    """

    geometries: list[Geometry]
    id: Optional[str]
    properties: Optional[str]
    bbox: Optional[list[float]]

type GeoJSON = GeoJSON_FeatureCollection | GeoJSON_Feature
GeoJSON.__doc__ = """
    A GeoJSON object represents a Feature or collection of Features.

    Notes
    -----
    See [geojson-specification](https://datatracker.ietf.org/doc/html/rfc7946)
"""

class GeoJSON_FeatureCollection:
    """
    A collection of features.
    """

    features: list[GeoJSON_Feature]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class GeoJSON_Feature:
    """
    A feature represents points, curves, and surfaces in coordinate space.
    """

    properties: Optional[str]
    geometry: FeatureGeometryType
    id: Optional[str]
    bbox: Optional[list[float]]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

type FeatureGeometryType = (
    FeatureGeometryType_GeometryCollection
    | FeatureGeometryType_Point
    | FeatureGeometryType_MultiPoint
    | FeatureGeometryType_LineString
    | FeatureGeometryType_MultiLineString
    | FeatureGeometryType_Polygon
    | FeatureGeometryType_MultiPolygon
)
FeatureGeometryType.__doc__ = """Geometry type of a feature"""

class FeatureGeometryType_GeometryCollection:
    """
    A feature geometry describes as a collection of feature geometries.
    """

    geometries: list[FeatureGeometryType]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class FeatureGeometryType_Point:
    """
    A feature geometry describes as a position.
    """

    coordinates: list[float]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class FeatureGeometryType_MultiPoint:
    """
    A feature geometry describes as a serie of positions.
    """

    coordinates: list[list[float]]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class FeatureGeometryType_LineString:
    """
    A feature geometry describes as a serie of positions.
    """

    coordinates: list[list[float]]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class FeatureGeometryType_MultiLineString:
    """
    A feature geometry describes as multiple series of series of positions.
    """

    coordinates: list[list[list[float]]]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class FeatureGeometryType_Polygon:
    """
    A feature geometry describes as multiple rings of series of positions.
    """

    coordinates: list[list[list[float]]]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

class FeatureGeometryType_MultiPolygon:
    """
    A feature geometry describes as multiple polygons of rings of series of positions.
    """

    coordinates: list[list[list[list[float]]]]

    def write(self, file: str):
        """
        Write expression to json.

        Parameters
        ----------
        file : str
            Path to a file

        Raises
        ------
        RuntimeError
            When serialization fails
        OsError
            When the file cannot be written
        """

def read(file: str) -> TopoJSON:
    """
    Read into a TopoJSON from a JSON file.

    Parameters
    ----------
    file : str
        Path to a file

    Returns
    -------
    TopoJSON
        TopoJSON object

    Raises
    ------
    OsError
        Unable to find, open or read the file.
    RuntimeError
        Unable to deserialize the file.
    """

def feature(topology: TopoJSON, o: Geometry) -> GeoJSON:
    """
    Returns the GeoJSON Feature or FeatureCollection for the specified
    object in the given topology. If the object is a
    `Geometry_GeometryCollection`, a `GeoJSON_FeatureCollection` is
    returned, and each geometry in the collection is mapped to a Feature.
    Otherwise, a Feature is returned. The returned feature is a shallow
    copy of the source object: they may share identifiers, bounding boxes,
    properties and coordinates.

    Parameters
    ----------
    topology : TopoJSON
        Topology object
    o : Geometry
        Geometry

    Returns
    -------
    GeoJSON
        GeoJSON Feature or FeatureCollection

    Examples
    --------
    - A point is mapped to a feature with a geometry object of type "Point".
    - Likewise for line strings, polygons, and other simple geometries.
    - A null geometry object (of type null in TopoJSON) is mapped to a feature
      with a null geometry object.
    - A geometry collection of points is mapped to a feature collection of
      features, each with a point geometry.
    - A geometry collection of geometry collections is mapped to a feature
      collection of features, each with a geometry collection.
    """

def merge(topology: TopoJSON, o: Geometry) -> FeatureGeometryType_MultiLineString:
    """
    Returns the GeoJSON MultiPolygon geometry object representing the union for
    the specified array of Polygon and MultiPolygon objects in the given
    topology. Interior borders shared by adjacent polygons are removed. See
    Merging States for an example. The returned geometry is a shallow copy of
    the source object: they may share coordinates.

    Parameters
    ----------
    topology : TopoJSON
        Topology object
    o : Geometry
        Geometry

    Returns
    -------
    FeatureGeometryType_MultiLineString
        GeoJSON MultiPolygon geometry object

    Raises
    ------
    KeyError
        When `key` is not found in `objects`
    TypeError
        Selected object is not a `Geometry_GeometryCollection`.
    """

def mesh(
    topology: TopoJSON,
    object: Optional[Geometry] = None,
    filter: Optional[Callable[[Geometry, Geometry], bool]] = None,
) -> FeatureGeometryType_MultiLineString:
    """
    Returns the GeoJSON MultiLineString geometry object representing the
    mesh for the specified object in the given topology. This is useful for
    rendering strokes in complicated objects efficiently, as edges that are
    shared by multiple features are only stroked once. If object is not
    specified, a mesh of the entire topology is returned. The returned
    geometry is a shallow copy of the source object: they may share
    coordinates.

    Parameters
    ----------
    topology : TopoJSON
        TopoJSON object containing the object to mesh
    object : Optional[Geometry]
        Specfied geometry to mesh
    filter : Optional[Callable[[Geometry, Geometry], bool]]
        The filter function is called once for each candidate arc and takes
        two arguments, a and b, two geometry objects that share that arc.
        Each arc is only included in the resulting mesh if the filter
        function returns true. For typical map topologies the geometries a
        and b are adjacent polygons and the candidate arc is their
        boundary. If an arc is only used by a single geometry then a and b
        are identical.

    Returns
    -------
    FeatureGeometryType_MultiLineString
        GeoJSON MultiLineString geometry object

    Warnings
    --------
    Currently, `filter` argument does not change the result because it is
    avoided due to performance issues.

    Raises
    ------
    KeyError
        When `key` is not found in `objects`
    """

def bbox(topology: TopoJSON) -> list[float]:
    """
    Returns the computed bounding box of the specified topology $[x_0, y_0,
    x_1, y_1]$ where $x_0$ is the minimum x-value, $y_0$ is the minimum
    y-value, x_1 is the maximum x-value, and $y_1$ is the maximum y-value.
    If the topology has no points and no arcs, the returned bounding box is
    $[\\infty, \\infty, -\\infty, -\\infty]$.

    (This method ignores the existing topology.bbox, if any.)

    Parameters
    ----------
    topology : TopoJSON
        TopoJSON object

    Returns
    -------
    list[float]
        Computed bounding box
    """

def neighbors(objects: list[Geometry]) -> list[list[int]]:
    """
    Returns an array representing the set of neighboring objects for each
    object in the specified objects array. The returned array has the same
    number of elements as the input array; each element i in the returned
    array is the array of indexes for neighbors of object i in the input
    array. For example, if the specified objects array contains the
    features foo and bar, and these features are neighbors, the returned
    array will be `[[1], [0]]`, indicating that foo is a neighbor of bar
    and vice versa. Each array of neighbor indexes for each object is
    guaranteed to be sorted in ascending order.

    Parameters
    ----------
    objects : list[Geometry]
        List of neighboring objects

    Returns
    -------
    list[list[int]]
        Neighboring objects

    Raises
    ------
    KeyError
        When `key` is not found in `objects`
    """

def quantize(topology: TopoJSON, transform: float) -> TopoJSON:
    """
    Returns a shallow copy of the specified topology with quantized and
    delta-encoded arcs according to the specified transform object. If the
    topology is already quantized, an error is thrown. See also
    topoquantize.

    The corresponding transform object is first computed using the bounding
    box of the topology. The quantization number `transform` must be a
    positive integer greater than one which determines the maximum number
    of expressible values per dimension in the resulting quantized
    coordinates; typically, a power of ten is chosen such as 1e4, 1e5 or
    1e6. If the topology does not already have a topology.bbox, one is
    computed using topojson.bbox.


    Parameters
    ----------
    topology : TopoJSON
        TopoJSON object to quantize
    transform : float
        The quantization number `transform`.

    Returns
    -------
    TopoJSON
        Shallow copy with quantized and delta-encoded arcs.

    Raises
    ------
    RuntimeError
        If topology is already quantized or transform is smaller than 2.
    """
