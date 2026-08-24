from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)

def test_breaking_schema_change_detection():
    res = client.get("/api/pact/orders")
    assert res.status_code == 200
    data = res.json()
    
    # Assert total count metadata field and array presence
    assert "count" in data
    assert data["count"] == len(data["orders"])
    assert all("id" in o for o in data["orders"])
