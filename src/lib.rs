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

mod parser_objects;
use pyo3::{prelude::*, types::PyDict};

#[pyfunction]
fn dict_to_rust(py_dict: &Bound<'_, PyDict>) -> PyResult<()> {
    let geometry: parser_objects::TopoJSON = py_dict.extract()?;
    println!("{:?}", geometry);
    Ok(())
}

#[pymodule]
fn topojson(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(dict_to_rust, m)?)?;
    Ok(())
}
