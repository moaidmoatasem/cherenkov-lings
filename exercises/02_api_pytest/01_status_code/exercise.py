import requests

def test_health_check_status_code():
    response = requests.get("http://localhost:8081/health")
    # TODO: Assert that the status code is 200
    pass
