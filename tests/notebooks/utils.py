"""Utilities for testing Jupyter notebooks."""
import json
import nbformat
from nbconvert.preprocessors import ExecutePreprocessor
from pathlib import Path
from typing import Dict, List, Optional

def load_notebook(path: str) -> nbformat.NotebookNode:
    """Load a Jupyter notebook from path.
    
    Args:
        path: Path to the notebook file
        
    Returns:
        Loaded notebook as nbformat.NotebookNode
    """
    with open(path) as f:
        return nbformat.read(f, as_version=4)

def validate_notebook_structure(nb: nbformat.NotebookNode) -> List[str]:
    """Validate basic notebook structure and content.
    
    Args:
        nb: Notebook to validate
        
    Returns:
        List of validation errors, empty if valid
    """
    errors = []
    
    # Check for minimum cells
    if len(nb.cells) < 2:
        errors.append("Notebook must have at least 2 cells")
        
    # Check for title
    if not nb.cells[0].source.startswith("# "):
        errors.append("First cell must be markdown with title")
        
    # Check for imports in first code cell
    found_imports = False
    for cell in nb.cells:
        if cell.cell_type == "code" and any(
            line.startswith("import ") or line.startswith("from ") 
            for line in cell.source.split("\n")
        ):
            found_imports = True
            break
    if not found_imports:
        errors.append("No import statements found")
        
    # Check for markdown documentation
    markdown_count = sum(1 for cell in nb.cells if cell.cell_type == "markdown")
    if markdown_count < 2:
        errors.append("Insufficient markdown documentation")
        
    return errors

def execute_notebook(
    nb: nbformat.NotebookNode,
    timeout: int = 600,
    kernel_name: str = "python3"
) -> Dict[str, List[str]]:
    """Execute notebook and capture outputs/errors.
    
    Args:
        nb: Notebook to execute
        timeout: Cell execution timeout in seconds
        kernel_name: Jupyter kernel to use
        
    Returns:
        Dict with 'errors' and 'outputs' lists
    """
    ep = ExecutePreprocessor(timeout=timeout, kernel_name=kernel_name)
    results = {"errors": [], "outputs": []}
    
    try:
        ep.preprocess(nb)
    except Exception as e:
        results["errors"].append(f"Execution failed: {str(e)}")
        return results
        
    for cell in nb.cells:
        if cell.cell_type == "code":
            if hasattr(cell, "outputs"):
                for output in cell.outputs:
                    if output.output_type == "error":
                        results["errors"].append(
                            f"Cell error: {output.ename}: {output.evalue}"
                        )
                    elif output.output_type == "stream":
                        results["outputs"].append(output.text)
                        
    return results

def validate_notebook_dependencies(nb: nbformat.NotebookNode) -> List[str]:
    """Validate that all imported packages are in requirements.txt.
    
    Args:
        nb: Notebook to validate
        
    Returns:
        List of missing dependencies
    """
    # Get project requirements
    req_path = Path(__file__).parents[2] / "requirements.txt"
    requirements = []
    if req_path.exists():
        with open(req_path) as f:
            requirements = [
                line.split("==")[0].strip() 
                for line in f.readlines() 
                if line.strip() and not line.startswith("#")
            ]
    
    # Extract imports from notebook
    imports = set()
    for cell in nb.cells:
        if cell.cell_type == "code":
            for line in cell.source.split("\n"):
                if line.startswith("import "):
                    imports.add(line.split()[1].split(".")[0])
                elif line.startswith("from "):
                    imports.add(line.split()[1].split(".")[0])
                    
    # Find missing dependencies
    missing = []
    for imp in imports:
        if imp not in requirements and imp not in ["os", "sys", "time", "typing"]:
            missing.append(imp)
            
    return missing