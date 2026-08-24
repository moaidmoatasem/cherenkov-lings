from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)

def test_provider_honors_contract():
    res = client.get("/api/pact/orders")
    assert res.status_code == 200
    data = res.json()
    
    # Provider verification: Ensure all required fields exist with correct data types
    for order in data["orders"]:
        assert isinstance(order["id"], str)
        assert isinstance(order["total"], (int, float))
        assert order["status"] in ["COMPLETED", "PENDING", "CANCELLED"]
        assert order["currency"] == "USD"
