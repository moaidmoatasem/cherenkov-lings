# Drill 04: Solution -- Test Real Behaviour via the Crucible
# By calling the real Crucible API, we test actual request/response behaviour.
# If the endpoint changes, returns a 500, or the contract breaks, THIS test fails.
# The fake would never catch that.
import requests

CRUCIBLE_URL = "http://localhost:8081"

def test_checkout_endpoint_accepts_valid_item():
    # ARRANGE
    payload = {"item_id": "item-1", "quantity": 1}

    # ACT -- calling the REAL Crucible, not a fake
    response = requests.post(f"{CRUCIBLE_URL}/checkout", json=payload, timeout=5)

    # ASSERT on the real response
    assert response.status_code == 200
    body = response.json()
    assert "order_id" in body or "status" in body
