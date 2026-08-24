"""
PRODUCTION STORY:
Capital One Cloud Metadata SSRF Breach (2019)
Server-Side Request Forgery allowed an attacker to query the AWS EC2 Instance Metadata Service
(169.254.169.254) and extract IAM role session credentials.
"""
import requests

def test_ssrf_protection():
    # Anti-pattern: Not asserting that internal metadata IPs are blocked
    # TODO: Verify that requests to cloud metadata services are intercepted with HTTP 403
    res = requests.post("http://localhost:8081/api/security/fetch-url", json={"url": "https://example.com"})
    assert res.status_code == 200
