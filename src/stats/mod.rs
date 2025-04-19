use pyo3::prelude::*;
use statrs::distribution::{ContinuousCDF, Normal};

/// Statistical analysis module
pub fn init_submodule(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "stats")?;
    parent_module.add_submodule(submod)?;
    Ok(())
}

#[pyfunction]
fn normal_cdf(x: f64, mean: f64, std_dev: f64) -> f64 {
    let normal = Normal::new(mean, std_dev).unwrap();
    normal.cdf(x)
}

#[pyfunction]
fn epi2by2(exposed_cases: u32, exposed_controls: u32, unexposed_cases: u32, unexposed_controls: u32) -> PyResult<f64> {
    let risk_ratio = (exposed_cases as f64 / (exposed_cases + exposed_controls) as f64) /
                    (unexposed_cases as f64 / (unexposed_cases + unexposed_controls) as f64);
    Ok(risk_ratio)
} 