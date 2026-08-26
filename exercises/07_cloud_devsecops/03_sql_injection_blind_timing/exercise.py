"""
PRODUCTION STORY:
Sony Pictures SQLi Data Breach (2011)
Unsanitized user inputs in database query parameters allowed attackers to execute
SQL statements and extract credentials.
"""
import requests
import time

def test_sql_injection_vulnerability():
    # Anti-pattern: Sending raw string concatenation without verifying parameterized queries
    # TODO: Assert that input parameters are parameterized and do not induce timing delays
    user_id = "1"
    res = requests.get(f"http://localhost:8081/api/security/user-lookup?user_id={user_id}")
    assert res.status_code == 200
