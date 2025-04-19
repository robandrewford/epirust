# epirust

A high-performance epidemiological analysis toolkit built in Rust, with bindings for Python and R. Inspired by R's `epiR`, `survival`, `epitools`, and `dagitty` packages.

## Features

- Fast and memory-safe epidemiological computations
  - SIMD vector operations (AVX2/AVX512/SSE2)
  - Parallel processing with Rayon
  - Memory-optimized data structures
  - GPU acceleration support
- Comprehensive statistical analysis
  - DAG parsing and adjustment set discovery
  - Kaplan-Meier and Cox models
  - Diagnostic accuracy stats
  - Propensity score matching
  - Disease transmission modeling
- High-performance optimizations
  - Automatic SIMD feature detection
  - Thread-safe data structures
  - Efficient memory allocation
  - Configurable data pipelines
- Python integration
  - Native PyO3 bindings
  - NumPy array support
  - Polars DataFrame integration
  - Async data loading

## Prerequisites

- Python 3.11+
- Rust toolchain
- uv (Python package installer)

## Install

First, install uv if you haven't already:

```bash
curl -LsSf https://astral.sh/uv/install.sh | sh
```

Then install and set up the project:

```bash
# Create virtual environment and install dependencies
uv venv
source .venv/bin/activate
uv pip install -e ".[dev]"

# Build Rust extensions
maturin develop
```

## Development

```bash
# Run tests
pytest

# Format code
black .
ruff check .
mypy .

# Run notebooks
jupyter lab
```

## Example Notebooks

> 🚀 **New to EpiRust?** Start with our comprehensive guide: [`getting_started.ipynb`](examples/getting_started.ipynb)
> 
> This notebook walks you through:
> - Basic setup and data handling
> - Essential statistical methods
> - Simple causal analysis
> - Survival analysis basics
> - Performance optimization tips

### Data Pipeline & Analysis
- `pipeline_config_demo.ipynb`: Data pipeline configuration and field mapping
- `diagnostic_stats.ipynb`: Diagnostic accuracy analysis (sensitivity, specificity, ROC curves)

### Causal Inference
- `dag_analysis.ipynb`: DAG-based causal inference and confounding analysis
- `propensity_matching.ipynb`: Propensity score matching and sensitivity analysis
- `causal_inference.ipynb`: Advanced causal methods (target trials, double robust estimation)

### Survival Analysis
- `survival_analysis.ipynb`: Comprehensive survival analysis techniques
  - Kaplan-Meier estimation
  - Cox proportional hazards
  - Time-dependent covariates
  - Competing risks

### Disease Modeling
- `disease_transmission.ipynb`: Disease transmission modeling
  - R0 estimation
  - Transmission networks
  - SEIR model fitting
  - Agent-based simulation
  - Intervention analysis

### Performance Optimization
- `high_performance_computing.ipynb`: Performance optimization techniques
  - SIMD operations
  - Parallel processing
  - Memory optimization
  - GPU acceleration
  - Benchmarking

## Available Datasets

- Framingham Heart Study: Longitudinal cardiovascular disease risk factors
- Western Collaborative Group Study (WCGS): Survival data on CHD risk factors
- Rossi Recidivism: Prison release study data

## License

MIT © [Rob Ford](https://github.com/robandrewford)
