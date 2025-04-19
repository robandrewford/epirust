use pyo3::prelude::*;
use ndarray::Array1;

/// Compute module for high-performance operations
pub fn init_submodule(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "compute")?;
    parent_module.add_submodule(submod)?;
    Ok(())
}

#[pyfunction]
fn vector_mean(data: Vec<f64>) -> f64 {
    let arr = Array1::from(data);
    arr.mean().unwrap_or(0.0)
} 