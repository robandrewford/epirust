use pyo3::prelude::*;
use rayon::prelude::*;

/// Parallel processing module
pub fn init_submodule(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "parallel")?;
    parent_module.add_submodule(submod)?;
    Ok(())
}

#[pyfunction]
fn parallel_sum(data: Vec<f64>) -> f64 {
    data.par_iter().sum()
} 