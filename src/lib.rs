use pyo3::prelude::*;

#[pyfunction]
pub fn epi2by2(a: f64, b: f64, c: f64, d: f64) -> PyResult<f64> {
    let risk_ratio = (a / (a + b)) / (c / (c + d));
    Ok(risk_ratio)
}

#[pymodule]
fn epirust(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(epi2by2, m)?)?;
    Ok(())
}