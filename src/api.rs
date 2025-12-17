use crate::feature::wrap_feature;
use crate::geojson_structs::Feature;
use crate::topojson_structs::{Geometry, TopoJSON};
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict, PyString},
};

#[pyfunction]
pub fn feature(topology: &Bound<'_, PyDict>, o: &Bound<'_, PyAny>) -> PyResult<Feature> {
    let topology: TopoJSON = topology.extract()?;
    let feature = if o.is_instance_of::<PyString>() {
        let key: String = o.extract::<String>()?;
        let o = &topology.objects[&key];
        wrap_feature(&topology, o)?
    } else {
        let o: Geometry = o.extract()?;
        wrap_feature(&topology, &o)?
    };
    Ok(feature)
}
