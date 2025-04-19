use pyo3::prelude::*;
use pyo3::types::{PyList, PyDict};
use numpy::{PyArray1, PyArray2};
use crate::compute::survival::kaplan_meier::{KaplanMeier, KMResult};

#[pyclass]
struct PyKaplanMeier {
    inner: KaplanMeier,
}

#[pymethods]
impl PyKaplanMeier {
    #[new]
    fn new() -> PyResult<Self> {
        Ok(Self {
            inner: KaplanMeier::new().map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
            })?,
        })
    }

    fn fit(
        &self,
        py: Python,
        time: Vec<f64>,
        event: Vec<bool>
    ) -> PyResult<PyObject> {
        let result = self.inner.fit(&time, &event)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

        let dict = PyDict::new(py);
        dict.set_item("time", result.time)?;
        dict.set_item("survival", result.survival)?;
        dict.set_item("std_error", result.std_error)?;
        dict.set_item("n_risk", result.n_risk)?;
        dict.set_item("n_event", result.n_event)?;

        Ok(dict.into())
    }
}

pub fn register_survival_module(py: Python, parent_module: &PyModule) -> PyResult<()> {
    let m = PyModule::new(py, "survival")?;
    m.add_class::<PyKaplanMeier>()?;
    parent_module.add_submodule(m)?;
    Ok(())
} 