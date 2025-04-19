use pyo3::prelude::*;

pub mod datasets;
pub mod compute;
pub mod stats;
mod simd;
mod parallel;
mod memory;

/// EpiRust: High-performance epidemiological analysis toolkit
#[pymodule]
fn epirust(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Initialize submodules
    compute::init_submodule(py, m)?;
    simd::init_submodule(py, m)?;
    parallel::init_submodule(py, m)?;
    memory::init_submodule(py, m)?;

    Ok(())
}

#[pyfunction]
pub fn epi2by2(a: f64, b: f64, c: f64, d: f64) -> PyResult<f64> {
    let risk_ratio = (a / (a + b)) / (c / (c + d));
    Ok(risk_ratio)
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum EpiRustError {
    #[error("Computation error: {0}")]
    ComputeError(String),
    
    #[error("Data error: {0}")]
    DataError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EpiRustError>;