# epirust

A high-performance epidemiological analysis toolkit built in Rust, with bindings for Python and R. Inspired by R's `epiR`, `survival`, `epitools`, and `dagitty` packages.

## Features

- Fast and memory-safe epidemiological computations
- Python bindings via PyO3 and Maturin
- DAG parsing and adjustment set discovery
- Kaplan-Meier and Cox models (coming soon)
- Diagnostic accuracy stats (sensitivity, specificity, etc.)
- Propensity score matching and stratified analysis

## Install

```bash
make setup
make install
make build
```

## Run

```bash
make test
make lint
make provision-aws
```

## Notebooks

Explore interactive examples:

```bash
make notebook
```

## License

MIT © [Rob Ford](https://github.com/robandrewford)