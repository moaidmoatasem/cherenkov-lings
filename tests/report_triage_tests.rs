use cherenkov_lings::gamification::{load_progress, GamificationState};
use cherenkov_lings::reports::allure::{
    generate_chaos_allure_report, summarize_dataset, AllureCategoryDef, AllureTestResultJson,
};
use cherenkov_lings::reports::chaos_dataset::{
    generate_chaos_dataset, get_failing_tests, get_test_by_id, get_tests_by_category,
    get_tests_by_track, FailureCategory, TestStatus,
};
use cherenkov_lings::triage::evaluator::{
    calculate_triage_stats, evaluate_and_record_progress, evaluate_triage, TriageSubmission,
};
use cherenkov_lings::triage::interactive::parse_category_from_str;
use std::collections::HashSet;
use std::fs;

#[test]
fn test_chaos_dataset_size_and_taxonomy_balance() {
    let dataset = generate_chaos_dataset();
    assert!(
        dataset.len() >= 60,
        "Dataset must contain at least 60 chaotic tests, found {}",
        dataset.len()
    );

    let mut real_bugs = 0;
    let mut flaky_infra = 0;
    let mut anti_patterns = 0;
    let mut passed = 0;

    let mut tracks = HashSet::new();

    for t in &dataset {
        tracks.insert(t.track_id.clone());
        match t.category {
            FailureCategory::RealBug => real_bugs += 1,
            FailureCategory::FlakyInfra => flaky_infra += 1,
            FailureCategory::AntiPattern => anti_patterns += 1,
            FailureCategory::None => passed += 1,
        }
    }

    assert!(
        real_bugs >= 15,
        "Expected >= 15 Genuine Product Defects, got {}",
        real_bugs
    );
    assert!(
        flaky_infra >= 15,
        "Expected >= 15 Flaky Infrastructure failures, got {}",
        flaky_infra
    );
    assert!(
        anti_patterns >= 15,
        "Expected >= 15 Test Automation Anti-Patterns, got {}",
        anti_patterns
    );
    assert!(
        passed >= 10,
        "Expected >= 10 Resilient Passing tests, got {}",
        passed
    );

    // Verify all curriculum tracks are represented
    let expected_tracks = [
        "playwright-ts",
        "restassured-java",
        "k6-js",
        "maestro-mobile",
        "devsecops-python",
        "genai-qa",
        "jmeter",
        "contract-pact",
        "a11y-axe",
        "foundations",
        "tool-decisions",
    ];

    for tr in &expected_tracks {
        assert!(
            tracks.contains(*tr),
            "Expected track '{}' to be represented in dataset",
            tr
        );
    }
}

#[test]
fn test_chaos_dataset_telemetry_integrity() {
    let dataset = generate_chaos_dataset();
    let mut test_ids = HashSet::new();

    for t in &dataset {
        // Unique IDs
        assert!(
            test_ids.insert(t.test_id.clone()),
            "Duplicate test ID detected: {}",
            t.test_id
        );

        // Positive duration
        assert!(t.duration_ms > 0, "Test duration must be positive");

        // Failing tests must have error message, stack trace, and chaos event
        if t.status != TestStatus::Passed && t.status != TestStatus::Skipped {
            assert!(
                t.error_message.is_some(),
                "Failing test {} must have an error message",
                t.test_id
            );
            assert!(
                t.stack_trace.is_some(),
                "Failing test {} must have a stack trace",
                t.test_id
            );
            assert!(
                t.chaos_event.is_some(),
                "Failing test {} must have correlated chaos event telemetry",
                t.test_id
            );
            assert!(
                t.root_cause_hint.is_some(),
                "Failing test {} must have a root cause hint for senior QA evaluation",
                t.test_id
            );
        }

        // Labels must include framework, track, and suite
        assert!(t.labels.contains_key("framework"));
        assert!(t.labels.contains_key("track"));
        assert!(t.labels.contains_key("suite"));
    }
}

#[test]
fn test_allure_json_generation_fidelity() {
    let temp_dir = std::env::temp_dir().join("cherenkov_allure_test_run");
    let _ = fs::remove_dir_all(&temp_dir);

    let summary = generate_chaos_allure_report(&temp_dir).expect("Report generation must succeed");
    assert!(summary.total_tests >= 60);

    let results_dir = temp_dir.join("allure-results");
    assert!(results_dir.exists(), "allure-results dir must exist");

    // Read result JSON files
    let result_files: Vec<_> = fs::read_dir(&results_dir)
        .expect("Cannot read results dir")
        .flatten()
        .filter(|e| e.file_name().to_string_lossy().ends_with("-result.json"))
        .collect();

    assert_eq!(
        result_files.len(),
        summary.total_tests,
        "Every test execution must produce an Allure result JSON file"
    );

    // Verify first result JSON parses into valid schema
    let first_file = &result_files[0];
    let content = fs::read_to_string(first_file.path()).expect("Cannot read result JSON");
    let parsed: AllureTestResultJson =
        serde_json::from_str(&content).expect("Result JSON must adhere to Allure schema");

    assert!(!parsed.uuid.is_empty());
    assert!(!parsed.name.is_empty());
    assert!(!parsed.full_name.is_empty());
    assert_eq!(parsed.stage, "finished");
    assert!(parsed.stop >= parsed.start);
    assert!(!parsed.labels.is_empty());
    assert!(!parsed.parameters.is_empty());

    // Verify categories.json
    let categories_file = results_dir.join("categories.json");
    assert!(categories_file.exists());
    let cat_content = fs::read_to_string(categories_file).expect("Cannot read categories.json");
    let cats: Vec<AllureCategoryDef> =
        serde_json::from_str(&cat_content).expect("categories.json must be valid");
    assert_eq!(cats.len(), 3);

    // Verify environment.properties
    let env_file = results_dir.join("environment.properties");
    assert!(env_file.exists());
    let env_content = fs::read_to_string(env_file).expect("Cannot read environment.properties");
    assert!(env_content.contains("Cherenkov-Lings"));
    assert!(env_content.contains("ZeroCloud=Enabled"));

    // Verify executor.json
    let exec_file = results_dir.join("executor.json");
    assert!(exec_file.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_allure_html_report_self_contained() {
    let temp_dir = std::env::temp_dir().join("cherenkov_html_report_test");
    let _ = fs::remove_dir_all(&temp_dir);

    let summary = generate_chaos_allure_report(&temp_dir).expect("Report generation must succeed");
    let primary_html = temp_dir.join("index.html");
    let secondary_html = temp_dir.join("allure-report").join("index.html");

    assert!(primary_html.exists(), "Primary index.html must exist");
    assert!(secondary_html.exists(), "allure-report/index.html must exist");

    let html = fs::read_to_string(&primary_html).expect("Cannot read primary HTML");
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("CHERENKOV-LINGS ALLURE REPORT"));
    assert!(html.contains("KPI Cards"));
    assert!(html.contains(&format!("Total Tests</div>\n      <div class=\"value\">{}", summary.total_tests)));
    assert!(html.contains("Root-Cause Taxonomy Breakdown"));
    assert!(html.contains("Correlated L4/L7 Chaos Telemetry"));
    assert!(html.contains("Triage Solver"));
    assert!(html.contains("cherenkov-lings triage --test-id"));

    // Ensure zero cloud / no external HTTP scripts
    assert!(!html.contains("https://cdn."));
    assert!(!html.contains("http://code.jquery.com"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_allure_report_summary_metrics_consistency() {
    let dataset = generate_chaos_dataset();
    let summary = summarize_dataset(&dataset, "target/allure-results", "target/allure-report/index.html");

    assert_eq!(
        summary.passed + summary.failed + summary.broken + summary.flaky + summary.skipped,
        summary.total_tests
    );
    assert_eq!(
        summary.real_bugs + summary.flaky_infra + summary.anti_patterns + summary.passed,
        summary.total_tests
    );
    assert!(summary.pass_percentage > 0.0 && summary.pass_percentage < 100.0);
    assert!(summary.duration_ms > 0);
}

#[test]
fn test_triage_evaluator_real_bug_correct_hypothesis() {
    let sub = TriageSubmission {
        test_id: "BUG-101".to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "The authorization middleware failed to validate RBAC role privileges, granting admin access to standard user.".to_string(),
        suggested_fix: "Add enforce_role('admin') check to /api/v1/auth/elevate endpoint before issuing elevated JWT token.".to_string(),
    };

    let result = evaluate_triage(&sub);
    assert!(result.correct, "Hypothesis should be marked correct");
    assert_eq!(result.actual_category, FailureCategory::RealBug);
    assert!(
        result.score_awarded >= 100,
        "Base score 100 plus explanation bonuses expected"
    );
    assert!(result.feedback.contains("Outstanding diagnosis"));
}

#[test]
fn test_triage_evaluator_flaky_infra_correct_hypothesis() {
    let sub = TriageSubmission {
        test_id: "FLAKE-201".to_string(),
        learner_category: FailureCategory::FlakyInfra,
        root_cause_explanation: "Chaos proxy injected an artificial latency spike of 3500ms which exceeded the client's 2000ms socket timeout.".to_string(),
        suggested_fix: "Increase client HTTP timeout threshold or configure exponential backoff retry on transient socket timeouts.".to_string(),
    };

    let result = evaluate_triage(&sub);
    assert!(result.correct, "Hypothesis should be marked correct");
    assert_eq!(result.actual_category, FailureCategory::FlakyInfra);
    assert!(result.score_awarded >= 100);
}

#[test]
fn test_triage_evaluator_anti_pattern_correct_hypothesis() {
    let sub = TriageSubmission {
        test_id: "ANTI-301".to_string(),
        learner_category: FailureCategory::AntiPattern,
        root_cause_explanation: "Test used brittle hardcoded waitForTimeout(500) sleep which failed when server latency took 620ms.".to_string(),
        suggested_fix: "Replace hardcoded sleep with auto-retrying web assertion expect(locator).toHaveText('John Doe').".to_string(),
    };

    let result = evaluate_triage(&sub);
    assert!(result.correct, "Hypothesis should be marked correct");
    assert_eq!(result.actual_category, FailureCategory::AntiPattern);
    assert!(result.score_awarded >= 100);
}

#[test]
fn test_triage_evaluator_incorrect_hypothesis_feedback() {
    let sub = TriageSubmission {
        test_id: "BUG-101".to_string(),
        learner_category: FailureCategory::FlakyInfra, // Wrong category!
        root_cause_explanation: "Network timeout occurred.".to_string(),
        suggested_fix: "Retry request.".to_string(),
    };

    let result = evaluate_triage(&sub);
    assert!(!result.correct, "Hypothesis should be marked incorrect");
    assert_eq!(result.score_awarded, 0);
    assert_eq!(result.actual_category, FailureCategory::RealBug);
    assert!(result.feedback.contains("Diagnostic Mismatch"));
    assert!(result.feedback.contains("Genuine Product Defect"));
}

#[test]
fn test_triage_evaluator_bonus_scoring_quality() {
    let sparse_sub = TriageSubmission {
        test_id: "BUG-102".to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "deadlock".to_string(),
        suggested_fix: "fix".to_string(),
    };

    let detailed_sub = TriageSubmission {
        test_id: "BUG-102".to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "Unordered lock acquisition across concurrent transfer threads caused a PostgreSQL database transaction deadlock.".to_string(),
        suggested_fix: "Acquire locks in a deterministic order by sorting account IDs before executing SELECT FOR UPDATE, or implement retry logic.".to_string(),
    };

    let sparse_res = evaluate_triage(&sparse_sub);
    let detailed_res = evaluate_triage(&detailed_sub);

    assert!(sparse_res.correct);
    assert!(detailed_res.correct);
    assert!(
        detailed_res.score_awarded > sparse_res.score_awarded,
        "Detailed analysis with keywords should award higher score ({} > {})",
        detailed_res.score_awarded,
        sparse_res.score_awarded
    );
}

#[test]
fn test_triage_evaluator_gamification_progress_update() {
    let temp_progress = std::env::temp_dir().join("test_triage_gamification_progress.json");
    let _ = fs::remove_file(&temp_progress);

    let sub = TriageSubmission {
        test_id: "ANTI-302".to_string(),
        learner_category: FailureCategory::AntiPattern,
        root_cause_explanation: "Cached stale element handle was invalidated when React component re-rendered DOM during state update.".to_string(),
        suggested_fix: "Use auto-retrying page.locator() query selector instead of caching raw element handle.".to_string(),
    };

    let (result, state) = evaluate_and_record_progress(&sub, Some(&temp_progress));
    assert!(result.correct);
    assert!(state.total_xp >= 100);
    assert!(
        state.has_achievement("first_triage"),
        "Should unlock first_triage badge"
    );

    // Verify persistence file was written
    assert!(temp_progress.exists());
    let loaded: GamificationState = load_progress(Some(&temp_progress)).expect("Must load progress");
    assert_eq!(loaded.total_xp, state.total_xp);

    let _ = fs::remove_file(&temp_progress);
}

#[test]
fn test_triage_stats_calculation() {
    let dataset = generate_chaos_dataset();
    let submissions = vec![
        TriageSubmission {
            test_id: "BUG-101".to_string(),
            learner_category: FailureCategory::RealBug,
            root_cause_explanation: "RBAC privilege escalation".to_string(),
            suggested_fix: "Fix auth check".to_string(),
        },
        TriageSubmission {
            test_id: "FLAKE-201".to_string(),
            learner_category: FailureCategory::FlakyInfra,
            root_cause_explanation: "Proxy latency spike".to_string(),
            suggested_fix: "Increase timeout".to_string(),
        },
        TriageSubmission {
            test_id: "ANTI-301".to_string(),
            learner_category: FailureCategory::RealBug, // Wrong
            root_cause_explanation: "Server error".to_string(),
            suggested_fix: "Fix server".to_string(),
        },
    ];

    let stats = calculate_triage_stats(&submissions, &dataset);
    assert_eq!(stats.total_attempts, 3);
    assert_eq!(stats.correct_count, 2);
    assert!((stats.accuracy_pct - 66.666).abs() < 0.1);
    assert_eq!(stats.real_bug_correct, 1);
    assert_eq!(stats.flaky_infra_correct, 1);
    assert_eq!(stats.anti_pattern_correct, 0);
    assert_eq!(stats.anti_pattern_total, 1);
}

#[test]
fn test_triage_category_parser_variations() {
    assert_eq!(parse_category_from_str("1"), Some(FailureCategory::RealBug));
    assert_eq!(parse_category_from_str("real_bug"), Some(FailureCategory::RealBug));
    assert_eq!(parse_category_from_str("bug"), Some(FailureCategory::RealBug));
    assert_eq!(parse_category_from_str("defect"), Some(FailureCategory::RealBug));

    assert_eq!(parse_category_from_str("2"), Some(FailureCategory::FlakyInfra));
    assert_eq!(parse_category_from_str("flaky_infra"), Some(FailureCategory::FlakyInfra));
    assert_eq!(parse_category_from_str("flaky"), Some(FailureCategory::FlakyInfra));
    assert_eq!(parse_category_from_str("proxy"), Some(FailureCategory::FlakyInfra));

    assert_eq!(parse_category_from_str("3"), Some(FailureCategory::AntiPattern));
    assert_eq!(parse_category_from_str("anti_pattern"), Some(FailureCategory::AntiPattern));
    assert_eq!(parse_category_from_str("antipattern"), Some(FailureCategory::AntiPattern));

    assert_eq!(parse_category_from_str("invalid_xyz"), None);
}

#[test]
fn test_query_helpers_by_category_track_and_failing() {
    let bug = get_test_by_id("BUG-101");
    assert!(bug.is_some());
    assert_eq!(bug.unwrap().category, FailureCategory::RealBug);

    let failing = get_failing_tests();
    assert!(failing.len() >= 45);
    for f in &failing {
        assert!(f.status != TestStatus::Passed);
    }

    let bugs = get_tests_by_category(FailureCategory::RealBug);
    assert!(bugs.len() >= 15);
    for b in &bugs {
        assert_eq!(b.category, FailureCategory::RealBug);
    }

    let playwright_tests = get_tests_by_track("playwright-ts");
    assert!(!playwright_tests.is_empty());
    for p in &playwright_tests {
        assert_eq!(p.track_id, "playwright-ts");
    }
}
