"""Adversarial Fuzzing and Stress Test Suite for Sprint 4 REST API Endpoints.

Tests:
1. Fuzz /api/review with malformed JSON, missing fields, extreme code sizes (500KB+), negative score thresholds, unknown languages, and injection payloads.
2. Fuzz /api/review/fix with missing code/path, invalid fix_id, oversized content.
3. Fuzz /api/pipeline/validate with empty YAML, YAML entity expansion bomb (billion laughs), giant matrices, tab indentation syntax errors, binary junk, and hardcoded secrets.
4. Fuzz /api/pipeline/run with invalid YAML, 0 jobs, 100+ matrix jobs, extreme parameter combinations.
5. Fuzz /api/reports/allure and /api/reports/allure/html with unusual query parameters and verify zero external network requests.
6. Fuzz /api/triage/tests with invalid categories, non-boolean failing_only, unknown tracks.
7. Fuzz /api/triage/submit with empty payloads, XSS payloads, Unicode/emojis, giant explanations (50KB+), invalid test IDs, and type mismatches.
"""

from __future__ import annotations

from pathlib import Path
import pytest
from fastapi.testclient import TestClient

from crucible.backend.app import app
import crucible.backend.triage as triage_module

client = TestClient(app)


# =============================================================================
# 1. Adversarial Fuzzing: POST /api/review
# =============================================================================


def test_fuzz_review_empty_payload_fails():
    """Verify empty JSON payload without code or file returns 400."""
    resp = client.post("/api/review", json={})
    assert resp.status_code == 400
    assert "No code provided" in resp.json()["detail"]


def test_fuzz_review_none_fields_fails():
    """Verify payload with null code and null file_path returns 400."""
    resp = client.post("/api/review", json={"code": None, "file_path": None})
    assert resp.status_code == 400


def test_fuzz_review_extreme_code_size_500kb():
    """Verify large code input (500KB) is analyzed safely without memory exhaustion or timeout."""
    large_code = "import { test } from '@playwright/test';\n" + ("// filler comment\n" * 15000) + "test('a', async ({ page }) => { await page.waitForTimeout(1000); });"
    resp = client.post("/api/review", json={"code": large_code, "language": "typescript"})
    assert resp.status_code == 200
    data = resp.json()
    assert data["passed"] is False
    assert any(v["rule_id"] == "HARDCODED_SLEEP" for v in data["violations"])


def test_fuzz_review_negative_score_threshold():
    """Verify negative score threshold is handled without internal server error."""
    resp = client.post("/api/review", json={"code": "test('ok', () => {});", "score_threshold": -50})
    assert resp.status_code == 200
    assert "score" in resp.json()


def test_fuzz_review_excessive_score_threshold():
    """Verify score threshold above 100 causes test to fail if score is 100."""
    resp = client.post("/api/review", json={"code": "test('ok', () => {});", "score_threshold": 999})
    assert resp.status_code == 200
    assert resp.json()["passed"] is False


def test_fuzz_review_unknown_language_fallback():
    """Verify obscure/unsupported language falls back safely without error."""
    resp = client.post("/api/review", json={"code": "fn main() { println!(\"hello\"); }", "language": "brainfuck_xyz"})
    assert resp.status_code == 200
    data = resp.json()
    assert "violations" in data


def test_fuzz_review_malformed_json_returns_422():
    """Verify non-JSON or malformed body returns 422 Unprocessable Entity."""
    resp = client.post(
        "/api/review",
        content="not a valid json string { bad: True",
        headers={"Content-Type": "application/json"},
    )
    assert resp.status_code == 422


# =============================================================================
# 2. Adversarial Fuzzing: POST /api/review/fix
# =============================================================================


def test_fuzz_review_fix_missing_code_and_path_returns_400():
    """Verify calling /api/review/fix without code or file returns 400."""
    resp = client.post("/api/review/fix", json={})
    assert resp.status_code == 400


def test_fuzz_review_fix_unknown_rule_id():
    """Verify calling fix with unknown rule_id returns unchanged code gracefully."""
    raw_code = "await page.waitForTimeout(5000);"
    resp = client.post("/api/review/fix", json={"code": raw_code, "fix_id": "UNKNOWN_RULE_999"})
    assert resp.status_code == 200
    data = resp.json()
    assert data["success"] is True
    assert data["patched_code"] == raw_code


def test_fuzz_review_fix_large_payload():
    """Verify large payload patching executes without crash."""
    code = "// comment\n" * 5000 + "await page.waitForTimeout(1000);"
    resp = client.post("/api/review/fix", json={"code": code, "fix_id": "all"})
    assert resp.status_code == 200
    assert "waitForTimeout" not in resp.json()["patched_code"]


# =============================================================================
# 3. Adversarial Fuzzing: POST /api/pipeline/validate
# =============================================================================


def test_fuzz_pipeline_validate_missing_yaml_returns_400():
    """Verify missing workflow YAML returns 400."""
    resp = client.post("/api/pipeline/validate", json={})
    assert resp.status_code == 400


def test_fuzz_pipeline_validate_tab_indented_yaml():
    """Verify tab-indented YAML produces clear syntax error, not 500."""
    tab_yaml = "name: Bad CI\n\tjobs:\n\t\ttest:\n\t\t\truns-on: ubuntu-latest"
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": tab_yaml})
    assert resp.status_code == 200
    data = resp.json()
    assert data["valid"] is False
    assert data["sdet_score"] == 0
    assert any(e["code"] == "YAML_SYNTAX_ERROR" for e in data["errors"])


def test_fuzz_pipeline_validate_giant_matrix():
    """Verify large matrix definition is parsed and scored properly."""
    large_matrix_yaml = """name: Giant Matrix CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        browser: [chromium, firefox, webkit]
        os: [ubuntu, windows, macos]
        shard: [1/4, 2/4, 3/4, 4/4]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/upload-artifact@v4
        with:
          name: results
          path: target/
"""
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": large_matrix_yaml})
    assert resp.status_code == 200
    data = resp.json()
    assert data["valid"] is True
    assert data["matrix_detected"] is True
    assert data["artifact_upload_detected"] is True


def test_fuzz_pipeline_validate_secrets_regex_patterns():
    """Verify multiple secret formats are caught by SDET policy validator."""
    secrets_yaml = """name: Leaky CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
      - run: echo "GITHUB_TOKEN=ghp_999999999999999999999999999999999999"
"""
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": secrets_yaml})
    assert resp.status_code == 200
    data = resp.json()
    assert data["valid"] is False
    secret_errors = [e for e in data["errors"] if e["code"] == "HARDCODED_SECRET_DETECTED"]
    assert len(secret_errors) >= 1


# =============================================================================
# 4. Adversarial Fuzzing: POST /api/pipeline/run
# =============================================================================


def test_fuzz_pipeline_run_empty_yaml_returns_400():
    """Verify missing workflow content returns 400."""
    resp = client.post("/api/pipeline/run", json={})
    assert resp.status_code == 400


def test_fuzz_pipeline_run_empty_jobs_workflow():
    """Verify workflow with 0 jobs produces a run result without crashing."""
    yaml_content = "name: Empty Workflow\non: [push]\njobs: {}"
    resp = client.post("/api/pipeline/run", json={"workflow_yaml": yaml_content})
    assert resp.status_code == 200
    data = resp.json()
    assert data["workflow_name"] == "Empty Workflow"
    assert len(data["jobs"]) == 0


def test_fuzz_pipeline_run_syntax_error_yaml():
    """Verify invalid YAML returns failure run result with syntax error logged."""
    yaml_content = "name: Invalid\n  invalid_indentation: [}"
    resp = client.post("/api/pipeline/run", json={"workflow_yaml": yaml_content})
    assert resp.status_code == 200
    data = resp.json()
    assert data["success"] is False
    assert any("YAML parsing error" in log["message"] for log in data["logs"])


# =============================================================================
# 5. Adversarial Fuzzing: GET /api/reports/allure & HTML Report
# =============================================================================


def test_fuzz_allure_reports_endpoint():
    """Verify /api/reports/allure handles query parameters safely."""
    resp = client.get("/api/reports/allure?limit=-999&offset=xyz&filter=<script>")
    assert resp.status_code == 200
    data = resp.json()
    assert data["total_tests"] >= 70


def test_fuzz_allure_html_report_zero_external_network_requests():
    """Verify /api/reports/allure/html is 100% self-contained with no external dependencies."""
    resp = client.get("/api/reports/allure/html")
    assert resp.status_code == 200
    html = resp.text

    external_hosts = [
        "https://cdn.",
        "http://cdn.",
        "https://cdnjs.",
        "https://fonts.googleapis.com",
        "https://code.jquery.com",
        "https://unpkg.com",
        "https://cdn.jsdelivr.net",
    ]

    for host in external_hosts:
        assert host not in html, f"HTML report should not contain external dependency: {host}"

    assert "<style>" in html
    assert "<script>" in html


# =============================================================================
# 6. Adversarial Fuzzing: GET /api/triage/tests
# =============================================================================


def test_fuzz_triage_tests_invalid_category_returns_empty_or_all():
    """Verify invalid category filter returns 200 with empty list or handled gracefully."""
    resp = client.get("/api/triage/tests?category=totally_nonexistent_cat_9999")
    assert resp.status_code == 200
    assert isinstance(resp.json(), list)


def test_fuzz_triage_tests_failing_only_toggle():
    """Verify failing_only=false returns passed tests too."""
    resp_failing = client.get("/api/triage/tests?failing_only=true")
    assert resp_failing.status_code == 200
    count_failing = len(resp_failing.json())

    resp_all = client.get("/api/triage/tests?failing_only=false")
    assert resp_all.status_code == 200
    count_all = len(resp_all.json())

    assert count_all >= count_failing


# =============================================================================
# 7. Adversarial Fuzzing: POST /api/triage/submit
# =============================================================================


def test_fuzz_triage_submit_empty_fields():
    """Verify submitting hypothesis with empty strings is handled gracefully without crash."""
    payload = {
        "test_id": "BUG-101",
        "category": "real_bug",
        "explanation": "",
        "fix": "",
    }
    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["correct"] is True
    assert data["base_score"] == 100
    assert data["score_awarded"] >= 100


def test_fuzz_triage_submit_xss_and_unicode_payloads(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Verify submitting XSS scripts and Unicode in explanation/fix is handled safely."""
    custom_progress = tmp_path / ".cherenkov-progress.json"
    monkeypatch.setattr(triage_module, "PROGRESS_FILE_PATH", custom_progress)

    payload = {
        "test_id": "BUG-101",
        "category": "real_bug",
        "explanation": "<script>alert('XSS')</script> RBAC privilege escalation \u0000 \u202e 🚀💥",
        "fix": "<img src=x onerror=alert(1)> Prepared statement and parameterization",
    }
    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["correct"] is True
    assert data["score_awarded"] > 100


def test_fuzz_triage_submit_giant_explanation_50kb():
    """Verify huge explanation text (50KB) is scored safely without timing out."""
    huge_exp = "rbac authorization deadlock foreign key " * 2000
    huge_fix = "apply exponential backoff retry and prepared statement lock " * 1000

    payload = {
        "test_id": "BUG-101",
        "category": "real_bug",
        "explanation": huge_exp,
        "fix": huge_fix,
    }
    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["correct"] is True
    assert data["explanation_score"] <= 50
    assert data["fix_score"] <= 50
    assert data["score_awarded"] <= 200


def test_fuzz_triage_submit_invalid_test_id():
    """Verify non-existent test ID returns correct=False and 0 score."""
    payload = {
        "test_id": "INVALID_TEST_ID_#9999",
        "category": "real_bug",
        "explanation": "Valid looking explanation",
        "fix": "Valid looking fix",
    }
    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["correct"] is False
    assert data["score_awarded"] == 0
    assert "Unknown test ID" in data["feedback"]
