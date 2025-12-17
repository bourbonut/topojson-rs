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

mod api;
mod feature;
mod geojson_structs;
mod reverse;
mod topojson_structs;
mod transform;

use pyo3::prelude::*;

#[pymodule]
fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(api::feature, m)?)?;
    Ok(())
}
