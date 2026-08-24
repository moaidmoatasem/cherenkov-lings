"""Interactive Root-Cause Triage Hypothesis Evaluation and Gamification Engine.

Evaluates student root-cause triage submissions against the chaotic test dataset,
generates contrastive architectural feedback, calculates multi-tier XP rewards,
and persists progress and badges to `.cherenkov-progress.json`.
"""

from __future__ import annotations

from datetime import datetime, timezone
import json
from pathlib import Path
from typing import Any

from crucible.backend.models import (
    ChaosTestResultItem,
    TriageResultResponse,
    TriageSubmissionRequest,
)
from crucible.backend.reports import generate_chaos_dataset

PROGRESS_FILE_PATH = Path(".cherenkov-progress.json")


def normalize_category(cat: str | None) -> str:
    """Normalize category strings to snake_case taxonomy."""
    if not cat:
        return "none"
    c = cat.strip().lower().replace(" ", "_").replace("-", "_")
    if "bug" in c or "defect" in c or "product" in c:
        return "real_bug"
    if "flake" in c or "infra" in c or "network" in c or "proxy" in c or "timeout" in c:
        return "flaky_infra"
    if "anti" in c or "pattern" in c or "sleep" in c or "xpath" in c or "locator" in c:
        return "anti_pattern"
    return c


def format_category_display(cat: str) -> str:
    """Format taxonomy category for human presentation."""
    if cat == "real_bug":
        return "Genuine Product Defect"
    if cat == "flaky_infra":
        return "Flaky Infrastructure Anomaly"
    if cat == "anti_pattern":
        return "Test Automation Anti-Pattern"
    return cat.replace("_", " ").title()


def generate_contrastive_feedback(
    learner_category: str,
    actual_category: str,
    test: ChaosTestResultItem,
) -> str:
    """Generate detailed contrastive feedback explaining why the chosen category is incorrect."""
    learner_disp = format_category_display(learner_category)
    actual_disp = format_category_display(actual_category)

    telemetry_clue = "None"
    if test.chaos_event and test.chaos_event.proxy_log:
        telemetry_clue = test.chaos_event.proxy_log
    elif test.error_message:
        telemetry_clue = test.error_message

    if actual_category == "real_bug" and learner_category == "flaky_infra":
        return (
            f"❌ **Misclassified as {learner_disp}** (Ground Truth: **{actual_disp}**)\n\n"
            f"• **Why it's NOT Flaky Infrastructure**: The Chaos Proxy telemetry shows clean L4 TCP connectivity and 0% packet loss. The failure is deterministic across all iterations.\n"
            f"• **Why it IS a Product Defect**: {test.root_cause_hint or 'The backend application raised an unhandled exception or business logic violation.'}\n"
            f"• **Telemetry Clue**: `{telemetry_clue}`\n\n"
            "💡 *Senior QA Rule*: Always verify proxy logs before attributing HTTP 500/SQL errors to network infrastructure flakiness."
        )

    if actual_category == "real_bug" and learner_category == "anti_pattern":
        return (
            f"❌ **Misclassified as {learner_disp}** (Ground Truth: **{actual_disp}**)\n\n"
            f"• **Why it's NOT an Anti-Pattern**: The test code uses standard assertions and locators; the failure is caused by an actual server-side logic bug or security flaw.\n"
            f"• **Why it IS a Product Defect**: {test.root_cause_hint or 'The API returned an unexpected status code violating the specification.'}\n"
            f"• **Telemetry Clue**: `{telemetry_clue}`"
        )

    if actual_category == "flaky_infra" and learner_category == "real_bug":
        return (
            f"❌ **Misclassified as {learner_disp}** (Ground Truth: **{actual_disp}**)\n\n"
            f"• **Why it's NOT a Product Defect**: The application code is functionally sound. The failure was caused by external environmental chaos.\n"
            f"• **Why it IS Flaky Infrastructure**: {test.root_cause_hint or 'Artificial proxy latency, packet loss, or connection drops triggered a transient timeout.'}\n"
            f"• **Telemetry Clue**: `{telemetry_clue}`\n\n"
            "💡 *Senior QA Rule*: When a test passes on retry or fails with gateway/read timeouts, check the L4/L7 proxy logs for injected latency spikes."
        )

    if actual_category == "flaky_infra" and learner_category == "anti_pattern":
        return (
            f"❌ **Misclassified as {learner_disp}** (Ground Truth: **{actual_disp}**)\n\n"
            f"• **Why it's NOT an Anti-Pattern**: The test is not using arbitrary sleeps or brittle selectors; the network buffer itself timed out or dropped connection.\n"
            f"• **Why it IS Flaky Infrastructure**: {test.root_cause_hint or 'Transient network jitter exceeded the socket read deadline.'}\n"
            f"• **Telemetry Clue**: `{telemetry_clue}`"
        )

    if actual_category == "anti_pattern" and learner_category == "real_bug":
        return (
            f"❌ **Misclassified as {learner_disp}** (Ground Truth: **{actual_disp}**)\n\n"
            f"• **Why it's NOT a Product Defect**: The production service is behaving properly. The test code contains a synchronization race or fragile locator anti-pattern.\n"
            f"• **Why it IS an Anti-Pattern**: {test.root_cause_hint or 'Brittle XPath, fixed millisecond sleep, or unawaited async promise in the test harness.'}\n"
            f"• **Telemetry Clue**: `{telemetry_clue}`\n\n"
            "💡 *Senior QA Rule*: If changing the UI styling or slightly increasing network latency causes test failures, the culprit is the test implementation, not the backend."
        )

    if actual_category == "anti_pattern" and learner_category == "flaky_infra":
        return (
            f"❌ **Misclassified as {learner_disp}** (Ground Truth: **{actual_disp}**)\n\n"
            f"• **Why it's NOT Flaky Infrastructure**: While network jitter exposed the failure, the root cause is that the test relied on a fixed millisecond sleep instead of web-first auto-waiting.\n"
            f"• **Why it IS an Anti-Pattern**: {test.root_cause_hint or 'Arbitrary sleep timeout expired prematurely before async DOM hydration finished.'}\n"
            f"• **Telemetry Clue**: `{telemetry_clue}`"
        )

    return (
        f"❌ **Hypothesis Incorrect**: Categorized as '{learner_disp}', but ground truth is '{actual_disp}'.\n\n"
        f"Root Cause Analysis: {test.root_cause_hint or 'Review stack trace and chaos telemetry.'}"
    )


def score_explanation(
    explanation: str, actual_category: str, test: ChaosTestResultItem
) -> tuple[int, list[str]]:
    """Score student explanation based on depth, domain keywords, and mechanism understanding."""
    if not explanation or len(explanation.strip()) < 10:
        return 10, ["Basic explanation provided (+10 XP)"]

    exp_lower = explanation.lower()
    score = 20
    reasons = ["Root-cause explanation submitted (+20 XP)"]

    keywords_by_cat = {
        "real_bug": [
            "rbac", "permission", "privilege", "deadlock", "lock", "foreign key",
            "constraint", "ssrf", "injection", "cors", "race condition", "null",
            "exception", "500", "database", "transaction", "unhandled"
        ],
        "flaky_infra": [
            "timeout", "latency", "jitter", "packet drop", "reset", "504", "502",
            "gateway", "dns", "connection", "proxy", "socket", "network", "pool"
        ],
        "anti_pattern": [
            "sleep", "waitfortimeout", "locator", "xpath", "css", "await",
            "floating", "promise", "assertion", "vacuous", "stale", "brittle", "dynamic"
        ],
    }

    matched_keywords = [kw for kw in keywords_by_cat.get(actual_category, []) if kw in exp_lower]
    if len(matched_keywords) >= 2:
        score += 20
        reasons.append(f"Technical mechanism keywords identified ({', '.join(matched_keywords[:3])}) (+20 XP)")
    elif len(matched_keywords) == 1:
        score += 10
        reasons.append(f"Domain keyword '{matched_keywords[0]}' identified (+10 XP)")

    if len(explanation.strip().split()) >= 20:
        score += 10
        reasons.append("Detailed architectural reasoning (+10 XP)")

    return min(50, score), reasons


def score_suggested_fix(
    fix: str, actual_category: str, test: ChaosTestResultItem
) -> tuple[int, list[str]]:
    """Score suggested remediation based on actionable architectural best practices."""
    if not fix or len(fix.strip()) < 5:
        return 10, ["Basic fix suggestion provided (+10 XP)"]

    fix_lower = fix.lower()
    score = 20
    reasons = ["Remediation proposal submitted (+20 XP)"]

    remediation_keywords = [
        "retry", "backoff", "await", "expect", "locator", "getbyrole", "getbytestid",
        "lock", "transaction", "prepared statement", "parameterized", "validate",
        "sanitize", "timeout", "pool", "circuit breaker", "awaitility", "index"
    ]

    matched = [kw for kw in remediation_keywords if kw in fix_lower]
    if len(matched) >= 2:
        score += 20
        reasons.append(f"Actionable engineering remedies ({', '.join(matched[:3])}) (+20 XP)")
    elif len(matched) == 1:
        score += 10
        reasons.append(f"Identified remediation pattern '{matched[0]}' (+10 XP)")

    if len(fix.strip().split()) >= 15:
        score += 10
        reasons.append("Comprehensive implementation steps (+10 XP)")

    return min(50, score), reasons


def load_gamification_progress(file_path: Path = PROGRESS_FILE_PATH) -> dict[str, Any]:
    """Load existing gamification progress or return default state."""
    if file_path.exists():
        try:
            return json.loads(file_path.read_text(encoding="utf-8"))
        except Exception:
            pass

    return {
        "total_xp": 0,
        "level_name": "Trainee",
        "streak_days": 0,
        "last_active_date": None,
        "flakiness_100_streak": 0,
        "perfect_locator_count": 0,
        "achievements": [],
        "completed_drills": {},
    }


def save_gamification_progress(state: dict[str, Any], file_path: Path = PROGRESS_FILE_PATH) -> None:
    """Save gamification state back to JSON file."""
    try:
        file_path.write_text(json.dumps(state, indent=2), encoding="utf-8")
    except Exception:
        pass


def evaluate_triage_submission(
    sub: TriageSubmissionRequest,
    dataset: list[ChaosTestResultItem] | None = None,
    progress_file: Path | None = None,
) -> TriageResultResponse:
    """Evaluate triage hypothesis, compute rewards, update progress, and return response."""
    if progress_file is None:
        progress_file = PROGRESS_FILE_PATH
    if dataset is None:
        dataset = generate_chaos_dataset()

    clean_id = sub.test_id.strip().lower()
    matched_test = next(
        (
            t
            for t in dataset
            if t.test_id.lower() == clean_id
            or t.name.lower() == clean_id
            or clean_id in t.name.lower()
        ),
        None,
    )

    learner_cat_raw = sub.category or sub.learner_category or ""
    learner_cat = normalize_category(learner_cat_raw)
    explanation = sub.explanation or sub.root_cause_explanation or ""
    fix_text = sub.fix or sub.suggested_fix or ""

    if not matched_test:
        return TriageResultResponse(
            test_id=sub.test_id,
            correct=False,
            actual_category="unknown",
            learner_category=learner_cat,
            score_awarded=0,
            base_score=0,
            explanation_score=0,
            fix_score=0,
            feedback=f"Unknown test ID '{sub.test_id}'. Choose a valid test from the Allure Chaos dataset (e.g., BUG-101, FLAKE-201, ANTI-301).",
            detailed_reasons=["Test ID not found in chaotic test dataset"],
        )

    actual_cat = matched_test.category
    is_correct = learner_cat == actual_cat

    if not is_correct:
        feedback = generate_contrastive_feedback(learner_cat, actual_cat, matched_test)
        return TriageResultResponse(
            test_id=matched_test.test_id,
            correct=False,
            actual_category=actual_cat,
            learner_category=learner_cat,
            score_awarded=0,
            base_score=0,
            explanation_score=0,
            fix_score=0,
            feedback=feedback,
            detailed_reasons=[
                f"Categorized as '{format_category_display(learner_cat)}', but ground truth is '{format_category_display(actual_cat)}'",
                f"Underlying Mechanism: {matched_test.root_cause_hint or 'See chaos telemetry logs'}",
            ],
        )

    # Correct diagnosis! Calculate scores
    base_score = 100
    exp_score, exp_reasons = score_explanation(explanation, actual_cat, matched_test)
    fix_score, fix_reasons = score_suggested_fix(fix_text, actual_cat, matched_test)
    total_score = base_score + exp_score + fix_score

    detailed_reasons = [f"Base Category Accuracy: +{base_score} XP"]
    detailed_reasons.extend(exp_reasons)
    detailed_reasons.extend(fix_reasons)

    feedback = (
        f"🎯 **Outstanding Diagnosis!** You correctly identified `{matched_test.name}` as a **{format_category_display(actual_cat)}**.\n\n"
        f"• **Root Cause Analysis**: {matched_test.root_cause_hint or 'Verified defect mechanism'}\n"
        f"• **Score Breakdown**: +{total_score} XP (Base: 100 XP, Explanation: +{exp_score} XP, Remediation: +{fix_score} XP)"
    )

    # Persist gamification updates
    progress_state = load_gamification_progress(progress_file)
    progress_state["total_xp"] = progress_state.get("total_xp", 0) + total_score

    # Check streak
    now_iso = datetime.now(timezone.utc).isoformat()
    if progress_state.get("streak_days", 0) == 0:
        progress_state["streak_days"] = 1
    progress_state["last_active_date"] = now_iso

    # Level calculation
    xp = progress_state["total_xp"]
    if xp >= 5000:
        progress_state["level_name"] = "Principal QA Architect"
    elif xp >= 2500:
        progress_state["level_name"] = "Senior SDET"
    elif xp >= 1000:
        progress_state["level_name"] = "SDET Engineer"
    elif xp >= 500:
        progress_state["level_name"] = "Junior SDET"
    else:
        progress_state["level_name"] = "Trainee"

    badge_unlocked = None
    existing_achievements = progress_state.get("achievements", [])
    if not any(a.get("id") == "first_triage" for a in existing_achievements if isinstance(a, dict)):
        badge_unlocked = "Triage Detective"
        existing_achievements.append(
            {
                "id": "first_triage",
                "name": "Triage Detective",
                "description": "Successfully diagnose your first chaotic test root-cause failure",
                "unlocked_at": now_iso,
            }
        )
        progress_state["achievements"] = existing_achievements
        detailed_reasons.append("🏆 Achievement Unlocked: 'Triage Detective' Badge!")

    save_gamification_progress(progress_state, progress_file)

    return TriageResultResponse(
        test_id=matched_test.test_id,
        correct=True,
        actual_category=actual_cat,
        learner_category=learner_cat,
        score_awarded=total_score,
        base_score=base_score,
        explanation_score=exp_score,
        fix_score=fix_score,
        feedback=feedback,
        badge_unlocked=badge_unlocked,
        detailed_reasons=detailed_reasons,
        updated_progress=progress_state,
    )
