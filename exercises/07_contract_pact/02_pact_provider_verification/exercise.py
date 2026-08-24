"""
PRODUCTION STORY:
Silent API Field Rename Outage (2019)
A backend team renamed a response field from `order_id` to `id`, breaking mobile app checkouts in production.
Provider contract verification prevents breaking changes from reaching production.
"""
import requests

def test_provider_honors_contract():
    # Anti-pattern: Testing provider without validating required consumer fields
    # TODO: Verify provider response strictly matches required contract schema
    res = requests.get("http://localhost:8081/api/pact/orders")
    assert res.status_code == 200
