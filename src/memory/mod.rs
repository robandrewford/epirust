use pyo3::prelude::*;
use bumpalo::Bump;
use numpy::{PyArray1, PyArray2};
use std::alloc::{alloc, Layout};

/// Memory optimization module
pub fn init_submodule(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "memory")?;
    parent_module.add_submodule(submod)?;
    Ok(())
}

/// Aligns a slice to the specified byte boundary by adding padding
pub fn align_array<T: Copy + Default>(slice: &[T], alignment: usize) -> Vec<T> {
    let size = std::mem::size_of::<T>();
    let padding = (alignment - (slice.len() * size) % alignment) % alignment;
    let padded_len = slice.len() + padding / size;
    
    let mut aligned = Vec::with_capacity(padded_len);
    aligned.extend_from_slice(slice);
    aligned.resize(padded_len, T::default());
    aligned
}

#[pyfunction]
fn allocate_bump(size: usize) -> usize {
    let bump = Bump::new();
    let _data = bump.alloc_slice_fill_default(size);
    size
} 