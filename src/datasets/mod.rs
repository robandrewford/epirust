use std::path::{Path, PathBuf};
use reqwest;
use csv;
use serde::Deserialize;
use anyhow::Result;
use polars::prelude::*;
use pyo3::prelude::*;
use numpy::{PyArray1, PyArray2};
use crate::compute::simd;

#[derive(Debug)]
pub struct Dataset {
    name: String,
    url: String,
    citation: String,
    description: String,
}

pub fn get_available_datasets() -> Vec<Dataset> {
    vec![
        Dataset {
            name: String::from("framingham"),
            url: String::from("https://raw.githubusercontent.com/paulhendricks/datasets/master/data-raw/framingham/framingham.csv"),
            citation: String::from("Framingham Heart Study, National Heart, Lung, and Blood Institute"),
            description: String::from("Longitudinal study of cardiovascular disease risk factors"),
        },
        Dataset {
            name: String::from("wcgs"),
            url: String::from("https://raw.githubusercontent.com/vincentarelbundock/Rdatasets/master/csv/survival/wcgs.csv"),
            citation: String::from("Western Collaborative Group Study"),
            description: String::from("Survival data from WCGS study on CHD risk factors"),
        },
        Dataset {
            name: String::from("rossi"),
            url: String::from("https://raw.githubusercontent.com/vincentarelbundock/Rdatasets/master/csv/survival/rossi.csv"),
            citation: String::from("Rossi et al. (1980)"),
            description: String::from("Recidivism data from Rossi et al. prison release study"),
        },
    ]
}

pub async fn download_dataset(dataset: &Dataset, output_dir: &PathBuf) -> Result<()> {
    let response = reqwest::get(&dataset.url).await?;
    let content = response.text().await?;
    
    let mut output_path = output_dir.clone();
    output_path.push(format!("{}.csv", dataset.name));
    
    std::fs::write(output_path, content)?;
    Ok(())
}

pub fn load_dataset(name: &str, data_dir: &PathBuf) -> Result<DataFrame> {
    let mut path = data_dir.clone();
    path.push(format!("{}.csv", name));
    
    let df = CsvReader::from_path(path)?
        .infer_schema(None)
        .has_header(true)
        .finish()?;
    
    Ok(df)
}

#[derive(Debug)]
pub struct DatasetManager {
    data_dir: PathBuf,
    cache_dir: PathBuf,
}

impl DatasetManager {
    pub fn new() -> Self {
        let data_dir = PathBuf::from("data");
        let cache_dir = data_dir.join("raw");
        Self { data_dir, cache_dir }
    }

    pub async fn download_dataset(&self, config: &Dataset) -> Result<PathBuf> {
        let target_path = self.cache_dir.join(format!("{}.csv", config.name));
        
        if !target_path.exists() {
            println!("Downloading dataset: {}", config.name);
            let response = reqwest::get(&config.url).await?;
            let content = response.bytes().await?;
            std::fs::write(&target_path, content)?;
        }
        
        Ok(target_path)
    }

    pub async fn load_all_datasets(&self) -> Result<()> {
        for dataset in get_available_datasets() {
            self.download_dataset(&dataset).await?;
        }
        Ok(())
    }
}

// Test fixtures for unit tests
#[cfg(test)]
pub mod test_data {
    use super::*;
    
    pub fn create_test_dataset() -> Vec<Vec<f64>> {
        vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ]
    }
}

pub fn init_submodule(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let submod = PyModule::new(py, "datasets")?;
    parent_module.add_submodule(submod)?;
    Ok(())
} 