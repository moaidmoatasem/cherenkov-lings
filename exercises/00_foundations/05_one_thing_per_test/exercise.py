# Drill 05: One Thing Per Test
# This single test checks FIVE different behaviours of the search API.
# When it fails, you have no idea WHICH of the five things broke.
# Manual QA equivalent: one test case with 20 steps and one "FAIL" result.
# TODO: Split this into 5 separate, focused tests -- one assertion each.
import requests

def test_search_api():
    # Tests empty query
    r = requests.get("http://localhost:8081/search?q=", timeout=5)
    assert r.status_code == 200
    # Tests valid query returns results
    r2 = requests.get("http://localhost:8081/search?q=Pay", timeout=5)
    assert r2.status_code == 200
    assert "results" in r2.json()
    # Tests result count
    assert len(r2.json().get("results", [])) > 0
    # Tests response has query field echoed back
    assert r2.json().get("query") == "Pay"
    # Tests content type
    assert "application/json" in r2.headers.get("content-type", "")
