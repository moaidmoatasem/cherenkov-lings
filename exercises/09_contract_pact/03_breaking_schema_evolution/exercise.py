"""
PRODUCTION STORY:
Additive vs Breaking API Evolution Failure (2020)
Removing a deprecated field without verifying consumer compatibility broke downstream reporting microservices.
Contract testing acts as a safety gate preventing non-backward-compatible schema changes.
"""
import requests

def test_breaking_schema_change_detection():
    # Anti-pattern: No validation for unexpected schema regressions
    # TODO: Use pact-python's `Verifier` to prove an additive-only contract
    # still passes against the live provider, and that a contract pinned to
    # a field the provider no longer returns is correctly rejected.
    res = requests.get("http://localhost:8081/api/pact/orders")
    assert res.status_code == 200
