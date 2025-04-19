"""Dataset management module for epirust.

This module provides functions to download and manage popular epidemiology datasets.
Datasets are cached locally and provided in standardized formats.
"""
import os
import hashlib
import json
import pandas as pd
import requests
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Union, Any, Callable
from tqdm import tqdm
import logging

# Base paths
DATA_DIR = Path(__file__).parents[1] / "data"
RAW_DIR = DATA_DIR / "raw"
PROCESSED_DIR = DATA_DIR / "processed"
METADATA_FILE = DATA_DIR / "metadata.json"

def _process_framingham_mini(df: pd.DataFrame) -> pd.DataFrame:
    """Process Framingham mini dataset."""
    # Rename columns for clarity
    df = df.rename(columns={
        'sex': 'gender',
        'age': 'baseline_age',
        'education': 'education_years',
        'currentSmoker': 'is_current_smoker',
        'cigsPerDay': 'cigarettes_per_day',
        'BPMeds': 'on_bp_meds',
        'prevalentStroke': 'had_stroke',
        'prevalentHyp': 'has_hypertension',
        'diabetes': 'has_diabetes',
        'totChol': 'total_cholesterol',
        'sysBP': 'systolic_bp',
        'diaBP': 'diastolic_bp',
        'BMI': 'bmi',
        'heartRate': 'heart_rate',
        'glucose': 'blood_glucose',
        'TenYearCHD': 'chd_10year'
    })
    
    # Convert binary columns to boolean
    binary_cols = ['is_current_smoker', 'on_bp_meds', 'had_stroke', 
                  'has_hypertension', 'has_diabetes', 'chd_10year']
    for col in binary_cols:
        df[col] = df[col].astype(bool)
    
    return df

def _process_covid_counties(df: pd.DataFrame) -> pd.DataFrame:
    """Process COVID-19 counties dataset."""
    # Convert date to datetime
    df['date'] = pd.to_datetime(df['date'])
    
    # Ensure numeric columns are properly typed
    df['cases'] = pd.to_numeric(df['cases'], errors='coerce')
    df['deaths'] = pd.to_numeric(df['deaths'], errors='coerce')
    
    # Sort by location and date
    df = df.sort_values(['state', 'county', 'date'])
    
    # Calculate daily new cases and deaths
    df['new_cases'] = df.groupby(['state', 'county'])['cases'].diff()
    df['new_deaths'] = df.groupby(['state', 'county'])['deaths'].diff()
    
    return df

def _process_who_mortality(df: pd.DataFrame) -> pd.DataFrame:
    """Process WHO mortality dataset."""
    # Rename columns for clarity
    df = df.rename(columns={
        'Entity': 'country',
        'Year': 'year',
        'Life expectancy': 'life_expectancy'
    })
    
    # Convert year to integer
    df['year'] = df['year'].astype(int)
    
    # Sort by country and year
    df = df.sort_values(['country', 'year'])
    
    return df

DATASETS = {
    "framingham_mini": {
        "name": "Framingham Heart Study (Mini)",
        "url": "https://raw.githubusercontent.com/JWarmenhoven/Framingham/master/Data/framingham.csv",
        "description": "A subset of the Framingham Heart Study data focusing on cardiovascular risk factors.",
        "citation": "Dawber et al. (1951)",
        "size_mb": 0.1,
        "format": "csv",
        "processor": _process_framingham_mini
    },
    "covid_counties": {
        "name": "US Counties COVID-19 Data",
        "url": "https://raw.githubusercontent.com/nytimes/covid-19-data/master/us-counties-2020.csv",
        "description": "County-level data for COVID-19 cases and deaths in the United States by The New York Times.",
        "citation": "The New York Times (2020)",
        "size_mb": 2.5,
        "format": "csv",
        "processor": _process_covid_counties
    },
    "who_mortality": {
        "name": "WHO Mortality Data",
        "url": "https://raw.githubusercontent.com/owid/owid-datasets/master/datasets/Life%20expectancy%20-%20WHO%20(2019)/Life%20expectancy%20-%20WHO%20(2019).csv",
        "description": "Global mortality rates and life expectancy data from the World Health Organization.",
        "citation": "World Health Organization (2019)",
        "size_mb": 0.2,
        "format": "csv",
        "processor": _process_who_mortality
    }
}

def get_dataset_info(dataset_name: str) -> Dict:
    """Get information about a specific dataset.
    
    Args:
        dataset_name: Name of the dataset
        
    Returns:
        Dict containing dataset metadata
        
    Raises:
        ValueError: If dataset_name is not found
    """
    if dataset_name not in DATASETS:
        raise ValueError(f"Dataset {dataset_name} not found. Available datasets: {list(DATASETS.keys())}")
    return DATASETS[dataset_name]

def list_datasets() -> pd.DataFrame:
    """List all available datasets with their metadata.
    
    Returns:
        DataFrame containing dataset information
    """
    return pd.DataFrame.from_dict(DATASETS, orient='index')

def download_dataset(
    dataset_name: str,
    force: bool = False,
    show_progress: bool = True
) -> Path:
    """Download a dataset if not already present.
    
    Args:
        dataset_name: Name of the dataset to download
        force: If True, download even if already present
        show_progress: If True, show download progress bar
        
    Returns:
        Path to the downloaded dataset
        
    Raises:
        ValueError: If dataset not found
        RuntimeError: If download fails
    """
    info = get_dataset_info(dataset_name)
    
    # Create directories if needed
    RAW_DIR.mkdir(parents=True, exist_ok=True)
    PROCESSED_DIR.mkdir(parents=True, exist_ok=True)
    
    # Generate filename from URL
    ext = info['format']
    filename = f"{dataset_name}.{ext}"
    filepath = RAW_DIR / filename
    
    if filepath.exists() and not force:
        return filepath
        
    # Download with progress bar
    response = requests.get(info['url'], stream=True)
    response.raise_for_status()
    
    total = int(response.headers.get('content-length', 0))
    
    with open(filepath, 'wb') as f:
        if show_progress:
            with tqdm(total=total, unit='iB', unit_scale=True) as pbar:
                for data in response.iter_content(chunk_size=1024):
                    size = f.write(data)
                    pbar.update(size)
        else:
            f.write(response.content)
            
    return filepath

def load_dataset(
    dataset_name: str,
    processed: bool = True,
    force_download: bool = False
) -> pd.DataFrame:
    """Load a dataset into memory.
    
    Args:
        dataset_name: Name of the dataset to load
        processed: If True, load processed version if available
        force_download: If True, force new download
        
    Returns:
        DataFrame containing the dataset
        
    Raises:
        ValueError: If dataset not found
    """
    info = get_dataset_info(dataset_name)
    
    # Try processed version first
    if processed:
        proc_path = PROCESSED_DIR / f"{dataset_name}_processed.parquet"
        if proc_path.exists():
            return pd.read_parquet(proc_path)
    
    # Download raw if needed
    filepath = download_dataset(dataset_name, force=force_download)
    
    # Load based on format
    if info['format'] == 'csv':
        df = pd.read_csv(filepath)
    else:
        raise ValueError(f"Unsupported format: {info['format']}")
        
    return df

def process_dataset(
    dataset_name: str,
    force: bool = False
) -> pd.DataFrame:
    """Process a raw dataset and save processed version.
    
    Args:
        dataset_name: Name of the dataset to process
        force: If True, process even if processed version exists
        
    Returns:
        Processed DataFrame
        
    Raises:
        ValueError: If dataset not found
    """
    proc_path = PROCESSED_DIR / f"{dataset_name}_processed.parquet"
    
    if proc_path.exists() and not force:
        return pd.read_parquet(proc_path)
        
    # Load raw data
    df = load_dataset(dataset_name, processed=False)
    
    # Apply dataset-specific processing
    if dataset_name == "framingham_mini":
        df = _process_framingham_mini(df)
    elif dataset_name == "covid_counties":
        df = _process_covid_counties(df)
    elif dataset_name == "who_mortality":
        df = _process_who_mortality(df)
        
    # Save processed version
    df.to_parquet(proc_path, index=False)
    return df

def list_datasets() -> None:
    """Print information about available datasets."""
    for key, dataset in DATASETS.items():
        print(f"- {key}: {dataset['name']}, size {dataset['size_mb']} MB")
        print(f"  {dataset['description']}")
        print()

def download_dataset(name: str, cache_dir: str = ".cache/datasets") -> pd.DataFrame:
    """Download and process a dataset.
    
    Args:
        name: Name of the dataset to download
        cache_dir: Directory to cache downloaded files
        
    Returns:
        Processed pandas DataFrame
    """
    if name not in DATASETS:
        raise ValueError(f"Dataset {name} not found. Available datasets: {list(DATASETS.keys())}")
    
    dataset = DATASETS[name]
    cache_dir = Path(cache_dir)
    cache_dir.mkdir(parents=True, exist_ok=True)
    
    # Create cache filename
    cache_file = cache_dir / f"{name}.{dataset['format']}"
    
    # Download if not cached
    if not cache_file.exists():
        logging.info(f"Downloading {name} dataset...")
        response = requests.get(dataset['url'])
        response.raise_for_status()
        
        with open(cache_file, 'wb') as f:
            f.write(response.content)
        logging.info(f"Downloaded {name} dataset to {cache_file}")
    
    # Read and process
    df = pd.read_csv(cache_file)
    if 'processor' in dataset:
        df = dataset['processor'](df)
    
    return df