# Drill 05: Solution -- One Thing Per Test
# Each test now has ONE reason to fail and ONE thing to fix.
# When test_search_echoes_query_in_response fails, you know exactly what is broken.
import requests

SEARCH_URL = "http://localhost:8081/search"

def test_search_returns_200_for_empty_query():
    r = requests.get(f"{SEARCH_URL}?q=", timeout=5)
    assert r.status_code == 200

def test_search_returns_200_for_valid_query():
    r = requests.get(f"{SEARCH_URL}?q=Pay", timeout=5)
    assert r.status_code == 200

def test_search_results_field_present_in_response():
    r = requests.get(f"{SEARCH_URL}?q=Pay", timeout=5)
    assert "results" in r.json()

def test_search_returns_at_least_one_result():
    r = requests.get(f"{SEARCH_URL}?q=Pay", timeout=5)
    assert len(r.json().get("results", [])) > 0

def test_search_echoes_query_in_response():
    r = requests.get(f"{SEARCH_URL}?q=Pay", timeout=5)
    assert r.json().get("query") == "Pay"
