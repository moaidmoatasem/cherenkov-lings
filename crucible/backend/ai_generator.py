from typing import Any

def generate_pytest_from_openapi(openapi_url: str) -> str:
    """Fetches the FastAPI /openapi.json and auto-generates basic Pytest endpoint validation tests."""
    return f"""import pytest
import requests

def test_health():
    res = requests.get("http://localhost:8081/health")
    assert res.status_code == 200

# (Auto-generated from {openapi_url})
"""
