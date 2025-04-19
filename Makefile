# Makefile for epirust - Rust-based Epidemiology Toolkit

PROJECT_NAME=epirust

setup:
	uv venv
	source .venv/bin/activate && uv pip install -r requirements.txt

install:
	cargo install --path .
	uv pip install maturin ruff pytest

build:
	maturin develop

test:
	pytest tests/

lint:
	ruff check .

fmt:
	cargo fmt

provision-aws:
	cd aws && bash provision_resources.sh

clean:
	cargo clean
	find . -type d -name '__pycache__' -exec rm -r {} +

notebook:
	jupyter notebook notebooks/