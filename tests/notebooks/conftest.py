"""Pytest configuration for notebook tests."""
import pytest

def pytest_configure(config):
    """Configure custom markers."""
    config.addinivalue_line(
        "markers",
        "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )

@pytest.fixture(autouse=True)
def add_imports(doctest_namespace):
    """Add imports for notebook testing."""
    import numpy as np
    import pandas as pd
    import matplotlib.pyplot as plt
    import seaborn as sns
    
    doctest_namespace["np"] = np
    doctest_namespace["pd"] = pd
    doctest_namespace["plt"] = plt
    doctest_namespace["sns"] = sns