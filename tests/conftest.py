"""Shared pytest fixtures."""

import pytest


@pytest.fixture
def temp_font_dir(tmp_path):
    """Create a temporary directory for font testing."""
    return tmp_path / "fonts"
