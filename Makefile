# Makefile for epirust - Rust-based Epidemiology Toolkit

PROJECT_NAME=epirust
VENV_DIR=.venv
PYTHON=$(VENV_DIR)/bin/python
PYTEST=$(VENV_DIR)/bin/pytest

.PHONY: setup install build test test-notebooks test-notebooks-fast test-all lint fmt provision-aws clean notebook datasets datasets-info datasets-download datasets-process

setup:
	uv venv
	source .venv/bin/activate && uv pip install -r requirements.txt
	$(PYTHON) -m pip install nbformat nbconvert jupyter pytest-xdist

install:
	cargo install --path .
	uv pip install maturin ruff pytest

build:
	maturin develop

test:
	$(PYTEST) tests/ --ignore=tests/notebooks

test-notebooks: setup-notebooks
	$(PYTEST) tests/notebooks/ -v --capture=no

test-notebooks-fast: setup-notebooks
	$(PYTEST) tests/notebooks/ -v -m "not slow" -n auto --capture=no

test-all: setup-notebooks
	$(PYTEST) tests/ -v -n auto --capture=no

setup-notebooks:
	@if [ ! -f $(VENV_DIR)/bin/jupyter ]; then \
		echo "Installing notebook dependencies..."; \
		$(PYTHON) -m pip install -q nbformat nbconvert jupyter pytest-xdist; \
	fi

lint:
	ruff check .

fmt:
	cargo fmt

provision-aws:
	cd aws && bash provision_resources.sh

clean:
	cargo clean
	find . -type d -name '__pycache__' -exec rm -r {} +
	find . -type f -name '*.pyc' -delete
	find . -type f -name '.pytest_cache' -delete
	find . -type f -name '.ipynb_checkpoints' -delete

notebook:
	source $(VENV_DIR)/bin/activate && jupyter notebook examples/

datasets-info:
	@$(PYTHON) -c "from epirust.datasets import list_datasets; print(list_datasets().to_markdown())"

datasets-download:
	@$(PYTHON) -c "from epirust.datasets import DATASETS, download_dataset; [download_dataset(d) for d in DATASETS]"

datasets-process:
	@$(PYTHON) -c "from epirust.datasets import DATASETS, process_dataset; [process_dataset(d) for d in DATASETS]"

datasets: datasets-download datasets-process
	@echo "All datasets downloaded and processed successfully"