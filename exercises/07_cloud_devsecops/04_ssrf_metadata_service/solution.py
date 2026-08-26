from fastapi.testclient import TestClient
from crucible.backend.app import app

client = TestClient(app)

def test_ssrf_blocks_cloud_metadata_service():
    # Test valid external URL
    res_valid = client.post("/api/security/fetch-url", json={"url": "https://playwright.dev"})
    assert res_valid.status_code == 200
    assert res_valid.json()["status"] == "fetched"

    # Test blocked AWS / Cloud metadata IP (169.254.169.254)
    res_ssrf = client.post("/api/security/fetch-url", json={"url": "http://169.254.169.254/latest/meta-data/"})
    assert res_ssrf.status_code == 403
    assert res_ssrf.json()["error"] == "SSRF_ATTEMPT_PREVENTED"
