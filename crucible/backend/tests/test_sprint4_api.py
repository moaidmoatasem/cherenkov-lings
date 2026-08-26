"""Comprehensive Test Suite for Sprint 4 Backend REST API Endpoints.

Tests:
- POST /api/review (AST rule analysis, scoring, mentor critique, Socratic questions, diff)
- POST /api/review/fix (Automated code patching and diff generation)
- POST /api/pipeline/validate (Enterprise SDET policy enforcement: matrix, artifacts, secrets, concurrency)
- POST /api/pipeline/run (Parallel matrix execution simulation, steps, and logs)
- GET /api/reports/allure (Chaos test dataset summary and telemetry)
- GET /api/reports/allure/html (Interactive HTML report)
- GET /api/triage/tests (Chaotic test list and category filtering)
- POST /api/triage/submit (Hypothesis evaluation, contrastive feedback, XP rewards, badge unlock)
"""

from __future__ import annotations

import json
from pathlib import Path
import pytest
from fastapi.testclient import TestClient

from crucible.backend.app import app
import crucible.backend.triage as triage_module

client = TestClient(app)


# =============================================================================
# 1. POST /api/review Tests
# =============================================================================


def test_review_anti_patterns_detection_and_score():
    """Verify AST review engine identifies sleep, xpath, and missing await anti-patterns."""
    brittle_code = """import { test, expect } from '@playwright/test';

test('brittle checkout test', async ({ page }) => {
  await page.goto('http://localhost:8080/checkout');
  
  // Floating promise (missing await)
  page.locator('#item').click();

  // Brittle absolute XPath
  await page.locator('/html/body/div[2]/div/table/tr/td[1]').fill('ACC-101');

  // Hardcoded sleep anti-pattern
  await page.waitForTimeout(5000);

  expect(true).toBe(true); // Vacuous assertion
});
"""
    resp = client.post("/api/review", json={"code": brittle_code, "language": "typescript"})
    assert resp.status_code == 200
    data = resp.json()

    assert data["passed"] is False
    assert data["score"] < 70
    assert len(data["violations"]) >= 3

    rule_ids = [v["rule_id"] for v in data["violations"]]
    assert "HARDCODED_SLEEP" in rule_ids
    assert "FRAGILE_LOCATOR_ABSOLUTE_XPATH" in rule_ids
    assert "FLOATING_PROMISE_UNAWAITED_ACTION" in rule_ids
    assert "VACUOUS_ASSERTION" in rule_ids

    assert len(data["socratic_questions"]) >= 1
    assert "Senior QA Code Review" in data["mentor_critique"]
    assert data["suggested_diff"] is not None
    assert "--- a/" in data["suggested_diff"]


def test_review_clean_exemplary_code_passes_100():
    """Verify clean, resilient code gets score 100 and passes."""
    clean_code = """import { test, expect } from '@playwright/test';

test('resilient checkout test', async ({ page }) => {
  await page.goto('http://localhost:8080/checkout');
  await page.getByRole('button', { name: 'Add to Cart' }).click();
  await expect(page.getByTestId('order-status')).toBeVisible();
});
"""
    resp = client.post("/api/review", json={"code": clean_code, "language": "typescript"})
    assert resp.status_code == 200
    data = resp.json()

    assert data["passed"] is True
    assert data["score"] == 100
    assert len(data["violations"]) == 0
    assert "Exemplary test design" in data["mentor_critique"]


def test_review_file_path_resolution(tmp_path: Path):
    """Verify review endpoint can read and analyze an exercise file from disk."""
    test_file = tmp_path / "exercise.ts"
    test_file.write_text(
        """import { test, expect } from '@playwright/test';
test('file test', async ({ page }) => {
  await page.waitForTimeout(3000);
  expect(true).toBe(true);
});""",
        encoding="utf-8",
    )

    resp = client.post("/api/review", json={"file_path": str(test_file)})
    assert resp.status_code == 200
    data = resp.json()
    assert data["exercise_name"] == "exercise.ts"
    assert any(v["rule_id"] == "HARDCODED_SLEEP" for v in data["violations"])


def test_review_nonexistent_file_returns_400():
    """Verify reviewing a nonexistent file without code returns 400."""
    resp = client.post("/api/review", json={"file_path": "nonexistent/path/exercise.ts"})
    assert resp.status_code == 400


# =============================================================================
# 2. POST /api/review/fix Tests
# =============================================================================


def test_review_fix_applies_automated_patch():
    """Verify automated patch replaces hardcoded sleep with web-first locator assertion."""
    code = """import { test, expect } from '@playwright/test';

test('sample', async ({ page }) => {
  await page.waitForTimeout(5000);
});"""

    resp = client.post("/api/review/fix", json={"code": code, "fix_id": "all"})
    assert resp.status_code == 200
    data = resp.json()

    assert data["success"] is True
    assert "waitForTimeout" not in data["patched_code"]
    assert "toBeVisible" in data["patched_code"]
    assert "HARDCODED_SLEEP" in data["applied_fixes"]
    assert data["diff"] is not None


def test_review_fix_file_on_disk(tmp_path: Path):
    """Verify fix endpoint modifies file on disk when file_path is provided."""
    target_file = tmp_path / "exercise.ts"
    target_file.write_text(
        """test('disk fix', async ({ page }) => {
  await page.waitForTimeout(5000);
});""",
        encoding="utf-8",
    )

    resp = client.post("/api/review/fix", json={"file_path": str(target_file), "fix_id": "HARDCODED_SLEEP"})
    assert resp.status_code == 200
    patched_disk_content = target_file.read_text(encoding="utf-8")
    assert "waitForTimeout" not in patched_disk_content
    assert "toBeVisible" in patched_disk_content


# =============================================================================
# 3. POST /api/pipeline/validate Tests
# =============================================================================


def test_pipeline_validate_compliant_enterprise_workflow():
    """Verify compliant workflow with matrix and artifact upload gets high score and passes."""
    workflow_yaml = """name: Enterprise SDET Matrix
on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  test:
    name: Parallel Matrix E2E
    runs-on: ubuntu-latest
    timeout-minutes: 20
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest]
        shard: [1/2, 2/2]
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20.x
      - name: Run Playwright Tests
        run: npx playwright test --shard=${{ matrix.shard }}
      - name: Upload Test Results
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: allure-results-${{ matrix.os }}-${{ matrix.shard }}
          path: target/allure-results
"""
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": workflow_yaml})
    assert resp.status_code == 200
    data = resp.json()

    assert data["valid"] is True
    assert data["sdet_score"] >= 90
    assert data["matrix_detected"] is True
    assert data["artifact_upload_detected"] is True
    assert len(data["errors"]) == 0


def test_pipeline_validate_missing_matrix_and_artifacts_fails():
    """Verify missing matrix strategy and artifact uploads produces explicit SDET errors."""
    bad_yaml = """name: Naive CI
on: [push]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Tests
        run: pytest
"""
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": bad_yaml})
    assert resp.status_code == 200
    data = resp.json()

    assert data["valid"] is False
    assert data["matrix_detected"] is False
    assert data["artifact_upload_detected"] is False

    error_codes = [e["code"] for e in data["errors"]]
    assert "MISSING_MATRIX_STRATEGY" in error_codes
    assert "MISSING_ARTIFACT_UPLOAD" in error_codes


def test_pipeline_validate_hardcoded_secrets_flagged():
    """Verify hardcoded GitHub/AWS tokens trigger security violations."""
    leaky_yaml = """name: Insecure CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo "Deploying with ghp_123456789012345678901234567890123456"
"""
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": leaky_yaml})
    assert resp.status_code == 200
    data = resp.json()

    assert data["valid"] is False
    assert any(e["code"] == "HARDCODED_SECRET_DETECTED" for e in data["errors"])


def test_pipeline_validate_invalid_yaml_syntax():
    """Verify malformed YAML produces syntax error."""
    invalid_yaml = "name: CI\n  bad: [indentation"
    resp = client.post("/api/pipeline/validate", json={"workflow_yaml": invalid_yaml})
    assert resp.status_code == 200
    data = resp.json()

    assert data["valid"] is False
    assert data["sdet_score"] == 0
    assert any(e["code"] == "YAML_SYNTAX_ERROR" for e in data["errors"])


# =============================================================================
# 4. POST /api/pipeline/run Tests
# =============================================================================


def test_pipeline_run_simulates_matrix_jobs():
    """Verify pipeline runner simulates parallel execution across matrix dimensions."""
    workflow_yaml = """name: Matrix Suite
on: [push]
jobs:
  test:
    strategy:
      matrix:
        browser: [chromium, firefox]
    steps:
      - uses: actions/checkout@v4
      - name: Run Playwright Tests
        run: npx playwright test --project=${{ matrix.browser }}
      - uses: actions/upload-artifact@v4
"""
    resp = client.post("/api/pipeline/run", json={"workflow_yaml": workflow_yaml})
    assert resp.status_code == 200
    data = resp.json()

    assert data["workflow_name"] == "Matrix Suite"
    assert data["success"] is True
    assert len(data["jobs"]) == 2  # 2 matrix combinations: chromium, firefox

    job_browsers = [j["matrix_combination"].get("browser") for j in data["jobs"]]
    assert "chromium" in job_browsers
    assert "firefox" in job_browsers

    assert len(data["logs"]) >= 6
    assert data["duration_ms"] > 0


# =============================================================================
# 5. GET /api/reports/allure & GET /api/reports/allure/html Tests
# =============================================================================


def test_get_allure_report_summary():
    """Verify /api/reports/allure returns comprehensive chaos telemetry summary."""
    resp = client.get("/api/reports/allure")
    assert resp.status_code == 200
    data = resp.json()

    assert data["total_tests"] >= 70
    assert data["real_bugs"] >= 19
    assert data["flaky_infra"] >= 25
    assert data["anti_patterns"] >= 26
    assert data["pass_percentage"] >= 0.0

    # Verify tests list has telemetry
    assert len(data["tests"]) == data["total_tests"]
    sample_test = next(t for t in data["tests"] if t["test_id"] == "BUG-101")
    assert sample_test["category"] == "real_bug"
    assert sample_test["chaos_event"] is not None
    assert sample_test["chaos_event"]["layer"] == "L7"


def test_get_allure_html_report():
    """Verify /api/reports/allure/html serves interactive HTML report."""
    resp = client.get("/api/reports/allure/html")
    assert resp.status_code == 200
    assert "text/html" in resp.headers["content-type"]
    html_content = resp.text

    assert "Allure Chaos Test Report" in html_content
    assert "BUG-101" in html_content
    assert "FLAKE-201" in html_content
    assert "ANTI-301" in html_content


# =============================================================================
# 6. GET /api/triage/tests Tests
# =============================================================================


def test_get_triage_tests_and_filtering():
    """Verify /api/triage/tests retrieves chaotic test failures and filters by category."""
    # 1. All failing tests
    resp = client.get("/api/triage/tests")
    assert resp.status_code == 200
    tests = resp.json()
    assert len(tests) >= 50
    assert all(t["status"] in ("failed", "broken", "flaky") for t in tests)

    # 2. Filter by category=real_bug
    resp_bug = client.get("/api/triage/tests?category=real_bug")
    assert resp_bug.status_code == 200
    bugs = resp_bug.json()
    assert len(bugs) >= 19
    assert all(t["category"] == "real_bug" for t in bugs)

    # 3. Filter by category=flaky_infra
    resp_flake = client.get("/api/triage/tests?category=flaky_infra")
    assert resp_flake.status_code == 200
    flakes = resp_flake.json()
    assert len(flakes) >= 25
    assert all(t["category"] == "flaky_infra" for t in flakes)


# =============================================================================
# 7. POST /api/triage/submit Tests
# =============================================================================


def test_triage_submit_correct_hypothesis_awards_xp_and_badge(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Verify correct root-cause hypothesis awards XP and unlocks badge in progress file."""
    custom_progress = tmp_path / ".cherenkov-progress.json"
    monkeypatch.setattr(triage_module, "PROGRESS_FILE_PATH", custom_progress)

    payload = {
        "test_id": "BUG-101",
        "category": "real_bug",
        "explanation": "The authorization middleware failed to perform RBAC role validation allowing privilege escalation.",
        "fix": "Implement strict RBAC permission checks in middleware before handling role elevation requests.",
    }

    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()

    assert data["correct"] is True
    assert data["actual_category"] == "real_bug"
    assert data["score_awarded"] >= 140
    assert data["badge_unlocked"] == "Triage Detective"
    assert "Outstanding Diagnosis" in data["feedback"]

    # Verify progress state was saved
    assert custom_progress.exists()
    saved_state = json.loads(custom_progress.read_text(encoding="utf-8"))
    assert saved_state["total_xp"] == data["score_awarded"]
    assert any(a["id"] == "first_triage" for a in saved_state["achievements"])


def test_triage_submit_incorrect_hypothesis_returns_contrastive_feedback(tmp_path: Path, monkeypatch: pytest.MonkeyPatch):
    """Verify incorrect hypothesis returns contrastive feedback and 0 XP."""
    custom_progress = tmp_path / ".cherenkov-progress.json"
    monkeypatch.setattr(triage_module, "PROGRESS_FILE_PATH", custom_progress)

    payload = {
        "test_id": "BUG-101",
        "category": "flaky_infra",  # Misclassification: It is actually a real product defect
        "explanation": "The network proxy dropped packets causing the test to fail.",
        "fix": "Increase socket timeout.",
    }

    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()

    assert data["correct"] is False
    assert data["score_awarded"] == 0
    assert data["actual_category"] == "real_bug"
    assert data["learner_category"] == "flaky_infra"
    assert "Why it's NOT Flaky Infrastructure" in data["feedback"]
    assert "Why it IS a Product Defect" in data["feedback"]


def test_triage_submit_unknown_test_id():
    """Verify submitting hypothesis for non-existent test ID returns clear error response."""
    payload = {
        "test_id": "UNKNOWN-999",
        "category": "real_bug",
        "explanation": "Unknown",
        "fix": "None",
    }
    resp = client.post("/api/triage/submit", json=payload)
    assert resp.status_code == 200
    data = resp.json()
    assert data["correct"] is False
    assert "Unknown test ID" in data["feedback"]
