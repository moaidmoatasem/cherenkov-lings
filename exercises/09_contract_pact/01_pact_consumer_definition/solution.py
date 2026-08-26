from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)

def test_consumer_contract_definition():
    res = client.get("/api/pact/orders")
    assert res.status_code == 200
    data = res.json()
    
    # Assert structural contract schema expectations
    assert "orders" in data
    assert isinstance(data["orders"], list)
    assert len(data["orders"]) > 0
    order = data["orders"][0]
    assert "id" in order and "total" in order and "status" in order
