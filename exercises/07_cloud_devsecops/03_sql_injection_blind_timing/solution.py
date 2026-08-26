from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)

def test_parameterized_query_neutralizes_sql_injection():
    # Test valid input
    res_valid = client.get("/api/security/user-lookup?user_id=1")
    assert res_valid.status_code == 200
    assert res_valid.json()["username"] == "alice_qa"

    # Test adversarial input with SQL injection payload
    res_sqli = client.get("/api/security/user-lookup?user_id=1%20OR%201=1")
    assert res_sqli.status_code == 200
    # Parameterized backend treats entire string as literal user ID
    assert res_sqli.json()["id"] == "1 OR 1=1"
