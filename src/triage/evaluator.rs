use crate::gamification::{
    GamificationState, current_utc_iso_timestamp, get_level_info, load_progress, save_progress,
};
use crate::reports::chaos_dataset::{ChaosTestResult, FailureCategory, generate_chaos_dataset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Root-cause triage hypothesis submission by a learner
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriageSubmission {
    pub test_id: String,
    pub learner_category: FailureCategory,
    pub root_cause_explanation: String,
    pub suggested_fix: String,
}

/// Result of evaluating a learner's triage hypothesis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriageResult {
    pub test_id: String,
    pub correct: bool,
    pub actual_category: FailureCategory,
    pub learner_category: FailureCategory,
    pub score_awarded: u32,
    pub base_score: u32,
    pub explanation_score: u32,
    pub fix_score: u32,
    pub feedback: String,
    pub badge_unlocked: Option<String>,
    pub detailed_reasons: Vec<String>,
}

/// Aggregated triage performance statistics across multiple submissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TriageStats {
    pub total_attempts: usize,
    pub correct_count: usize,
    pub accuracy_pct: f64,
    pub xp_earned: u64,
    pub real_bug_correct: usize,
    pub real_bug_total: usize,
    pub flaky_infra_correct: usize,
    pub flaky_infra_total: usize,
    pub anti_pattern_correct: usize,
    pub anti_pattern_total: usize,
    pub category_breakdown: HashMap<String, usize>,
}

/// Primary evaluation entrypoint: Evaluates a learner hypothesis against the standard chaos dataset
pub fn evaluate_triage(submission: &TriageSubmission) -> TriageResult {
    let dataset = generate_chaos_dataset();
    evaluate_triage_against_dataset(submission, &dataset)
}

/// Evaluate a learner hypothesis against a provided dataset
pub fn evaluate_triage_against_dataset(
    submission: &TriageSubmission,
    dataset: &[ChaosTestResult],
) -> TriageResult {
    let clean_id = submission.test_id.trim().to_lowercase();
    let matched_test = dataset.iter().find(|t| {
        t.test_id.to_lowercase() == clean_id
            || t.name.to_lowercase() == clean_id
            || t.name.to_lowercase().contains(&clean_id)
    });

    let test = match matched_test {
        Some(t) => t,
        None => {
            return TriageResult {
                test_id: submission.test_id.clone(),
                correct: false,
                actual_category: FailureCategory::None,
                learner_category: submission.learner_category,
                score_awarded: 0,
                base_score: 0,
                explanation_score: 0,
                fix_score: 0,
                feedback: format!(
                    "Unknown test ID '{}'. Please choose a valid chaotic test from the dataset (e.g. BUG-101, FLAKE-201, ANTI-301).",
                    submission.test_id
                ),
                badge_unlocked: None,
                detailed_reasons: vec!["Test ID not found in chaotic test dataset".to_string()],
            };
        }
    };

    let actual_category = test.category;
    let is_correct = submission.learner_category == actual_category;

    if !is_correct {
        let failure_explanation =
            generate_contrastive_feedback(submission.learner_category, actual_category, test);
        return TriageResult {
            test_id: test.test_id.clone(),
            correct: false,
            actual_category,
            learner_category: submission.learner_category,
            score_awarded: 0,
            base_score: 0,
            explanation_score: 0,
            fix_score: 0,
            feedback: failure_explanation,
            badge_unlocked: None,
            detailed_reasons: vec![
                format!(
                    "Categorized as '{}', but ground truth is '{}'",
                    submission.learner_category.display_name(),
                    actual_category.display_name()
                ),
                format!(
                    "Underlying mechanism: {}",
                    test.root_cause_hint
                        .as_deref()
                        .unwrap_or("See stack trace telemetry")
                ),
            ],
        };
    }

    // Hypothesis is correct! Compute detailed scoring & bonuses
    let base_score = 100u32;
    let (exp_score, exp_reasons) =
        score_explanation(&submission.root_cause_explanation, actual_category, test);
    let (fix_score, fix_reasons) =
        score_suggested_fix(&submission.suggested_fix, actual_category, test);
    let total_score = base_score + exp_score + fix_score;

    let mut detailed_reasons = Vec::new();
    detailed_reasons.push(format!("Base Category Accuracy: +{} XP", base_score));
    detailed_reasons.extend(exp_reasons);
    detailed_reasons.extend(fix_reasons);

    let feedback = format!(
        "🎯 Outstanding diagnosis! You correctly identified '{}' as a {}.\n\nSenior SDET Breakdown:\n- Root Cause: {}\n- Telemetry Clue: {}\n\nXP Awarded: +{} XP (Base: 100, Explanation Bonus: +{}, Fix Bonus: +{})",
        test.name,
        actual_category.display_name(),
        test.root_cause_hint
            .as_deref()
            .unwrap_or("Verified defect mechanism"),
        extract_telemetry_clue(test),
        total_score,
        exp_score,
        fix_score
    );

    TriageResult {
        test_id: test.test_id.clone(),
        correct: true,
        actual_category,
        learner_category: submission.learner_category,
        score_awarded: total_score,
        base_score,
        explanation_score: exp_score,
        fix_score,
        feedback,
        badge_unlocked: None, // Checked and populated during state persistence
        detailed_reasons,
    }
}

/// Evaluate submission, persist XP reward to `.cherenkov-progress.json`, and unlock badges
pub fn evaluate_and_record_progress(
    submission: &TriageSubmission,
    progress_path: Option<&Path>,
) -> (TriageResult, GamificationState) {
    let mut result = evaluate_triage(submission);
    let mut state = load_progress(progress_path).unwrap_or_default();

    if result.correct {
        let ts = current_utc_iso_timestamp();
        state.update_streak(&ts);
        state.total_xp = state.total_xp.saturating_add(result.score_awarded as u64);
        state.level_name = get_level_info(state.total_xp).title.to_string();

        // Check for Triage-specific achievements
        if !state.has_achievement("first_triage") {
            let ach = state.try_unlock(
                "first_triage",
                "Triage Detective",
                "Successfully diagnose your first chaotic test root-cause failure",
                &ts,
            );
            if let Some(a) = ach {
                result.badge_unlocked = Some(a.name);
            }
        }

        let _ = save_progress(&state, progress_path);
    }

    (result, state)
}

/// Calculate batch triage statistics across multiple submissions
pub fn calculate_triage_stats(
    submissions: &[TriageSubmission],
    dataset: &[ChaosTestResult],
) -> TriageStats {
    let mut stats = TriageStats {
        total_attempts: submissions.len(),
        ..Default::default()
    };

    for sub in submissions {
        let res = evaluate_triage_against_dataset(sub, dataset);
        stats.xp_earned += res.score_awarded as u64;

        let clean_id = sub.test_id.trim().to_lowercase();
        let matched = dataset.iter().find(|t| {
            t.test_id.to_lowercase() == clean_id
                || t.name.to_lowercase() == clean_id
                || t.name.to_lowercase().contains(&clean_id)
        });

        if let Some(test) = matched {
            match test.category {
                FailureCategory::RealBug => {
                    stats.real_bug_total += 1;
                    if res.correct {
                        stats.real_bug_correct += 1;
                    }
                }
                FailureCategory::FlakyInfra => {
                    stats.flaky_infra_total += 1;
                    if res.correct {
                        stats.flaky_infra_correct += 1;
                    }
                }
                FailureCategory::AntiPattern => {
                    stats.anti_pattern_total += 1;
                    if res.correct {
                        stats.anti_pattern_correct += 1;
                    }
                }
                FailureCategory::None => {}
            }
        }

        if res.correct {
            stats.correct_count += 1;
            *stats
                .category_breakdown
                .entry(res.actual_category.to_string())
                .or_insert(0) += 1;
        }
    }

    stats.accuracy_pct = if stats.total_attempts > 0 {
        (stats.correct_count as f64 / stats.total_attempts as f64) * 100.0
    } else {
        0.0
    };

    stats
}

// =========================================================================
// Internal Scoring & Feedback Helpers
// =========================================================================

fn score_explanation(
    explanation: &str,
    category: FailureCategory,
    _test: &ChaosTestResult,
) -> (u32, Vec<String>) {
    let mut score = 0u32;
    let mut reasons = Vec::new();
    let lower_exp = explanation.to_lowercase();

    // 1. Length & depth bonus
    if lower_exp.len() >= 40 {
        score += 15;
        reasons.push("Comprehensive Root-Cause Analysis: +15 XP".to_string());
    } else if lower_exp.len() >= 15 {
        score += 8;
        reasons.push("Succinct Explanation: +8 XP".to_string());
    }

    // 2. Keyword relevance bonus
    let keywords = match category {
        FailureCategory::RealBug => vec![
            "deadlock",
            "rbac",
            "foreign key",
            "null pointer",
            "nullpointer",
            "contract",
            "integer overflow",
            "overflow",
            "jwt",
            "sql",
            "sqli",
            "double spend",
            "heap",
            "oom",
            "hallucination",
            "prompt injection",
            "contrast",
            "wcag",
            "thread pool",
            "biometric",
            "case sensitive",
            "idor",
            "stream",
            "500",
            "403",
            "defect",
            "logic",
            "schema",
            "vulnerability",
            "permission",
            "authorization",
        ],
        FailureCategory::FlakyInfra => vec![
            "proxy",
            "chaos",
            "latency",
            "jitter",
            "tcp",
            "reset",
            "rst",
            "502",
            "504",
            "bad gateway",
            "gateway timeout",
            "dns",
            "keepalive",
            "keep-alive",
            "socket",
            "timeout",
            "port exhaustion",
            "time_wait",
            "cold start",
            "ttft",
            "stall",
            "packet loss",
            "packet drop",
            "network",
            "failover",
            "redis",
            "429",
            "rate limit",
        ],
        FailureCategory::AntiPattern => vec![
            "sleep",
            "waitfortimeout",
            "stale",
            "element",
            "handle",
            "xpath",
            "locator",
            "assertion",
            "no assertions",
            "unwrap",
            "keyerror",
            "dictionary",
            "threshold",
            "sla",
            "exact match",
            "text match",
            "pollution",
            "leakage",
            "isolation",
            "async",
            "await",
            "unhandled",
            "promise",
            "ambiguous",
            "secret",
            "hardcoded",
            "seed",
            "temperature",
            "rampup",
            "hover",
            "animation",
            "opacity",
            "timestamp",
            "suppress",
            "regex",
            "swallow",
            "except",
        ],
        FailureCategory::None => vec!["pass", "healthy", "resilient"],
    };

    let mut matched_kw_count = 0;
    for kw in &keywords {
        if lower_exp.contains(kw) {
            matched_kw_count += 1;
        }
    }

    if matched_kw_count >= 2 {
        score += 20;
        reasons.push(format!(
            "Key Domain Concept Precision ({} terms): +20 XP",
            matched_kw_count
        ));
    } else if matched_kw_count == 1 {
        score += 10;
        reasons.push("Relevant Domain Term Identified: +10 XP".to_string());
    }

    (score.min(35), reasons)
}

fn score_suggested_fix(
    fix: &str,
    _category: FailureCategory,
    _test: &ChaosTestResult,
) -> (u32, Vec<String>) {
    let mut score = 0u32;
    let mut reasons = Vec::new();
    let lower_fix = fix.to_lowercase();

    if lower_fix.len() >= 30 {
        score += 10;
        reasons.push("Actionable Engineering Remediation Plan: +10 XP".to_string());
    } else if lower_fix.len() >= 10 {
        score += 5;
        reasons.push("Remediation Suggested: +5 XP".to_string());
    }

    let fix_patterns = [
        "retry",
        "waitfor",
        "locator",
        "parameterize",
        "prepared statement",
        "select for update",
        "lock",
        "jwt verify",
        "getbyrole",
        "exponential backoff",
        "clean",
        "teardown",
        "rollback",
        "seed",
        "relative",
        "defensive",
        "rate limiter",
        "keepalive",
        "circuit breaker",
    ];

    if fix_patterns.iter().any(|&p| lower_fix.contains(p)) {
        score += 5;
        reasons.push("SDET Best-Practice Solution Pattern: +5 XP".to_string());
    }

    (score.min(15), reasons)
}

fn extract_telemetry_clue(test: &ChaosTestResult) -> String {
    if let Some(ref chaos) = test.chaos_event {
        if let Some(ref log) = chaos.proxy_log {
            return log.clone();
        }
        if chaos.latency_ms > 0 {
            return format!(
                "Injected Latency {}ms (±{}ms) on layer {}",
                chaos.latency_ms, chaos.jitter_ms, chaos.layer
            );
        }
    }
    if let Some(ref err) = test.error_message {
        return err.clone();
    }
    "Clean execution baseline".to_string()
}

fn generate_contrastive_feedback(
    learner_cat: FailureCategory,
    actual_cat: FailureCategory,
    test: &ChaosTestResult,
) -> String {
    let hint = test
        .root_cause_hint
        .as_deref()
        .unwrap_or("Inspect stack trace");
    let telemetry = extract_telemetry_clue(test);

    match (learner_cat, actual_cat) {
        (FailureCategory::FlakyInfra, FailureCategory::RealBug) => {
            format!(
                "❌ Diagnostic Mismatch: You selected 'Flaky Infrastructure', but this is a Genuine Product Defect.\n\nClue: The failure did not occur due to proxy latency or dropped packets. The system under test returned a real defect ({hint}). Telemetry: {telemetry}"
            )
        }
        (FailureCategory::AntiPattern, FailureCategory::RealBug) => {
            format!(
                "❌ Diagnostic Mismatch: You selected 'Test Automation Anti-Pattern', but this is a Genuine Product Defect.\n\nClue: The test script is well-written and correctly asserts behavior. The application code itself failed ({hint}). Telemetry: {telemetry}"
            )
        }
        (FailureCategory::RealBug, FailureCategory::FlakyInfra) => {
            format!(
                "❌ Diagnostic Mismatch: You selected 'Genuine Product Defect', but this is a Flaky Infrastructure failure.\n\nClue: The backend application code is correct. The error was injected at the network/proxy layer (e.g. latency spike, TCP reset, synthetic 502). Telemetry: {telemetry}"
            )
        }
        (FailureCategory::AntiPattern, FailureCategory::FlakyInfra) => {
            format!(
                "❌ Diagnostic Mismatch: You selected 'Test Automation Anti-Pattern', but this is a Flaky Infrastructure failure.\n\nClue: The test structure is standard, but external network conditions (proxy jitter or socket resets) caused transient interruption. Telemetry: {telemetry}"
            )
        }
        (FailureCategory::RealBug, FailureCategory::AntiPattern) => {
            format!(
                "❌ Diagnostic Mismatch: You selected 'Genuine Product Defect', but this is a Test Automation Anti-Pattern.\n\nClue: The server/backend is functioning properly. The test itself is brittle (e.g. hardcoded sleep, stale element reference, missing await, or ambiguous locator). Telemetry: {telemetry}"
            )
        }
        (FailureCategory::FlakyInfra, FailureCategory::AntiPattern) => {
            format!(
                "❌ Diagnostic Mismatch: You selected 'Flaky Infrastructure', but this is a Test Automation Anti-Pattern.\n\nClue: While timing is involved, the root cause is within the test code (e.g. using `waitForTimeout(500)` rather than dynamic auto-retrying web assertions). Telemetry: {telemetry}"
            )
        }
        _ => {
            format!(
                "❌ Diagnostic Mismatch: Selected category '{}' does not match actual root cause '{}'. Hint: {hint}",
                learner_cat.display_name(),
                actual_cat.display_name()
            )
        }
    }
}
