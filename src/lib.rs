// mod transform;
// use pyo3::prelude::*;
// use pyo3::types::{PyFunction, PyList};
//
// #[pyfunction]
// fn example(transform: Option<&Bound<'_, PyFunction>>, x: i32, y: i32) -> PyResult<i32> {
//     match transform {
//         Some(transform) => {
//             let result = transform.call1((x, y))?;
//             result.extract::<i32>()
//         }
//         None => PyResult::Ok(20),
//     }
// }
//
// #[pyclass]
// struct Transformer {
//     #[pyo3(get, set)]
//     scale: [i32; 2],
//     #[pyo3(get, set)]
//     translate: [i32; 2],
// }
//
// #[pymethods]
// impl Transformer {
//     #[new]
//     fn new(scale: &Bound<'_, PyList>, translate: &Bound<'_, PyList>) -> PyResult<Self> {
//         PyResult::Ok(Transformer {
//             scale: [scale.get_item(0)?.extract()?, scale.get_item(1)?.extract()?],
//             translate: [
//                 translate.get_item(0)?.extract()?,
//                 translate.get_item(1)?.extract()?,
//             ],
//         })
//     }
// }
//
// #[pymodule]
// fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
//     m.add_function(wrap_pyfunction!(example, m)?)?;
//     m.add_class::<Transformer>()?;
//     Ok(())
// }

mod _feature;
mod geojson_structs;
mod reverse;
mod topojson_structs;
mod transform;

use _feature::wrap_feature;
use geojson_structs::Feature;
use pyo3::{
    prelude::*,
    types::{PyAny, PyDict, PyString},
};
use topojson_structs::{Geometry, TopoJSON};

#[pyfunction]
fn feature(topology: &Bound<'_, PyDict>, o: &Bound<'_, PyAny>) -> PyResult<Feature> {
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

#[pymodule]
fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(feature, m)?)?;
    Ok(())
}
