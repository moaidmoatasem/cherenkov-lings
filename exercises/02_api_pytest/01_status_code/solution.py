import requests

def test_health_check_status_code():
    response = requests.get("http://localhost:8081/health")
    assert response.status_code == 200, f"Expected 200, got {response.status_code}"
