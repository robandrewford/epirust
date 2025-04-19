"""Tests for example Jupyter notebooks."""
import pytest
from pathlib import Path
from .utils import (
    load_notebook,
    validate_notebook_structure,
    execute_notebook,
    validate_notebook_dependencies
)

NOTEBOOKS_DIR = Path(__file__).parents[2] / "examples"
NOTEBOOKS = [
    "dag_analysis.ipynb",
    "diagnostic_stats.ipynb",
    "propensity_matching.ipynb",
    "survival_analysis.ipynb",
    "high_performance.ipynb"
]

@pytest.fixture(scope="module")
def notebooks():
    """Load all notebooks."""
    return {
        name: load_notebook(str(NOTEBOOKS_DIR / name))
        for name in NOTEBOOKS
    }

@pytest.mark.parametrize("notebook_name", NOTEBOOKS)
def test_notebook_exists(notebook_name):
    """Test that each notebook file exists."""
    path = NOTEBOOKS_DIR / notebook_name
    assert path.exists(), f"Notebook {notebook_name} not found"
    assert path.is_file(), f"{notebook_name} is not a file"

@pytest.mark.parametrize("notebook_name", NOTEBOOKS)
def test_notebook_structure(notebooks, notebook_name):
    """Test notebook structure and content."""
    nb = notebooks[notebook_name]
    errors = validate_notebook_structure(nb)
    assert not errors, f"Structure validation failed for {notebook_name}:\n" + "\n".join(errors)

@pytest.mark.parametrize("notebook_name", NOTEBOOKS)
def test_notebook_dependencies(notebooks, notebook_name):
    """Test that all notebook dependencies are in requirements.txt."""
    nb = notebooks[notebook_name]
    missing = validate_notebook_dependencies(nb)
    assert not missing, (
        f"Missing dependencies for {notebook_name}:\n" + 
        "\n".join(f"- {dep}" for dep in missing)
    )

@pytest.mark.slow
@pytest.mark.parametrize("notebook_name", NOTEBOOKS)
def test_notebook_execution(notebooks, notebook_name):
    """Test that notebooks execute without errors."""
    nb = notebooks[notebook_name]
    results = execute_notebook(nb)
    assert not results["errors"], (
        f"Execution failed for {notebook_name}:\n" + 
        "\n".join(results["errors"])
    )

def test_notebook_coverage():
    """Test that all notebooks in examples/ are being tested."""
    actual_notebooks = set(
        p.name for p in NOTEBOOKS_DIR.glob("*.ipynb")
        if not p.name.startswith(".")
    )
    tested_notebooks = set(NOTEBOOKS)
    
    untested = actual_notebooks - tested_notebooks
    assert not untested, f"Untested notebooks found: {untested}"
    
    nonexistent = tested_notebooks - actual_notebooks
    assert not nonexistent, f"Test references nonexistent notebooks: {nonexistent}"