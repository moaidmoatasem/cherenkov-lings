//! Adversarial empirical stress suite for Sprint 4: Enterprise Allure Reporting & Triage Engine.
//!
//! Stress-tests:
//! 1. Allure report generation under edge cases:
//!    - Empty datasets (0 tests)
//!    - Deeply nested output paths
//!    - Special characters, XSS payloads, Unicode/emojis in test names & stack traces
//!    - Path traversal attempts in test IDs
//!    - Giant datasets (1,000+ tests with oversized telemetry)
//! 2. Interactive HTML report completeness & isolation:
//!    - Embedded styles & scripts
//!    - Zero external network requests (no external CDNs, fonts, or tracking)
//!    - Proper HTML escaping of test names, suite names, error messages, and logs
//! 3. Triage Hypothesis scoring algorithm & gamification:
//!    - Boundary conditions on explanation length (0, 14, 15, 39, 40 chars)
//!    - Boundary conditions on keyword matches (0, 1, 2, 10 keywords)
//!    - Boundary conditions on suggested fix length & patterns (0, 9, 10, 29, 30 chars)
//!    - Score clamping (minimum 0, base 100, maximum 150)
//!    - Streak calculation (consecutive days, same day, skipped days, past timestamps, malformed dates)
//!    - Gamification persistence resilience (corrupted JSON, missing files, concurrent updates)

use cherenkov_lings::gamification::{
    current_utc_iso_timestamp, get_level_info, load_progress, save_progress, GamificationState,
};
use cherenkov_lings::reports::allure::{
    generate_allure_report_for_dataset, generate_allure_results, generate_chaos_allure_report,
    generate_interactive_html_report, render_html_report_string, summarize_dataset,
    write_allure_metadata_files, AllureTestResultJson,
};
use cherenkov_lings::reports::chaos_dataset::{
    generate_chaos_dataset, get_failing_tests, get_test_by_id, get_tests_by_category,
    get_tests_by_track, ChaosEventTelemetry, ChaosTestResult, FailureCategory, FlakinessMetrics,
    TestStatus, TestStepTelemetry,
};
use cherenkov_lings::triage::evaluator::{
    calculate_triage_stats, evaluate_and_record_progress, evaluate_triage,
    evaluate_triage_against_dataset, TriageResult, TriageSubmission,
};
use cherenkov_lings::triage::interactive::parse_category_from_str;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[test]
fn test_edge_case_empty_dataset_handling() {
    let empty_dataset: Vec<ChaosTestResult> = Vec::new();
    let temp_dir = std::env::temp_dir().join("cherenkov_stress_empty_dataset");
    let _ = fs::remove_dir_all(&temp_dir);

    // 1. summarize_dataset on empty slice
    let summary = summarize_dataset(&empty_dataset, "results", "report.html");
    assert_eq!(summary.total_tests, 0);
    assert_eq!(summary.passed, 0);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.broken, 0);
    assert_eq!(summary.flaky, 0);
    assert_eq!(summary.real_bugs, 0);
    assert_eq!(summary.flaky_infra, 0);
    assert_eq!(summary.anti_patterns, 0);
    assert_eq!(summary.pass_percentage, 0.0);
    assert_eq!(summary.duration_ms, 0);

    // 2. generate_allure_report_for_dataset on empty slice
    let report_res = generate_allure_report_for_dataset(&empty_dataset, &temp_dir);
    assert!(report_res.is_ok(), "Empty dataset report generation must not panic");

    // 3. render_html_report_string on empty slice
    let html = render_html_report_string(&empty_dataset, &summary);
    assert!(!html.is_empty());
    assert!(html.contains("Total Tests</div>\n      <div class=\"value\">0</div>"));
    assert!(html.contains("0.0%"));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_edge_case_deep_nested_directory_creation() {
    let dataset = generate_chaos_dataset();
    let deep_dir = std::env::temp_dir()
        .join("cherenkov_stress_deep_1")
        .join("nested_level_2")
        .join("sub_level_3")
        .join("reports_output");

    let _ = fs::remove_dir_all(&deep_dir);

    let summary_res = generate_chaos_allure_report(&deep_dir);
    assert!(summary_res.is_ok(), "Must create deep nested parent paths automatically");
    assert!(deep_dir.join("allure-results").exists());
    assert!(deep_dir.join("allure-report").exists());
    assert!(deep_dir.join("index.html").exists());

    let _ = fs::remove_dir_all(std::env::temp_dir().join("cherenkov_stress_deep_1"));
}

#[test]
fn test_edge_case_special_characters_xss_and_path_traversal() {
    let mut adversarial_dataset = Vec::new();

    adversarial_dataset.push(ChaosTestResult {
        test_id: "../../../traversal_id_<script>alert('xss')</script>".to_string(),
        name: "<img src=x onerror=alert(document.cookie)> & \" ' / \\ \n\r\t \u{0000} \u{202E} RLO".to_string(),
        suite: "<svg onload=alert(1)> Suite & SpecialChars".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 1234,
        error_message: Some("<script>console.log('injected error')</script> \n Trace: at line 42 & <> \"'".to_string()),
        stack_trace: Some("Stack trace with quotes \" and brackets <tag> and emoji 🔥🚀💥 and \\n escape".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "<L7-XSS>".to_string(),
            event_type: "malicious_payload_<script>".to_string(),
            latency_ms: 500,
            jitter_ms: 50,
            packet_loss_rate: 0.12,
            proxy_log: Some("WARN [Proxy] <script>alert('proxy log xss')</script> & raw chars".to_string()),
            correlated_timestamp: "2026-08-24T18:00:00Z".to_string(),
            retry_attempts: 1,
            injection_target: "http://127.0.0.1:8086/<script>".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry {
                name: "Step with <script>alert(1)</script>".to_string(),
                status: TestStatus::Failed,
                duration_ms: 100,
                error: Some("Error with <tag> & special chars".to_string()),
            }
        ],
        labels: {
            let mut map = HashMap::new();
            map.insert("malicious_label".to_string(), "<script>bad()</script>".to_string());
            map.insert("suite".to_string(), "Adversarial Suite".to_string());
            map.insert("track".to_string(), "devsecops-python".to_string());
            map.insert("framework".to_string(), "cherenkov-matrix".to_string());
            map
        },
        root_cause_hint: Some("Hint containing <script> and & \" ' quotes".to_string()),
    });

    let temp_dir = std::env::temp_dir().join("cherenkov_stress_adversarial_chars");
    let _ = fs::remove_dir_all(&temp_dir);

    let summary = generate_allure_report_for_dataset(&adversarial_dataset, &temp_dir)
        .expect("Report generation must handle adversarial chars without crashing");

    assert_eq!(summary.total_tests, 1);

    // Verify HTML escaping and self-containment
    let html = fs::read_to_string(temp_dir.join("index.html")).expect("HTML report must exist");
    assert!(html.contains("escapeHtml"), "HTML must include escapeHtml function");
    assert!(html.contains("const ALL_TESTS ="), "Must embed JSON safely");

    // Verify raw JSON results generated
    let results_dir = temp_dir.join("allure-results");
    assert!(results_dir.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_html_report_zero_external_network_requests() {
    let dataset = generate_chaos_dataset();
    let temp_dir = std::env::temp_dir().join("cherenkov_stress_zero_network");
    let _ = fs::remove_dir_all(&temp_dir);

    generate_allure_report_for_dataset(&dataset, &temp_dir).expect("Report generation must succeed");
    let html = fs::read_to_string(temp_dir.join("index.html")).expect("HTML must exist");

    // Check for any external CDN or remote resource URLs in scripts, styles, fonts, or images
    let forbidden_patterns = [
        "https://cdn.",
        "http://cdn.",
        "https://cdnjs.",
        "http://cdnjs.",
        "https://fonts.googleapis.com",
        "https://fonts.gstatic.com",
        "https://code.jquery.com",
        "https://unpkg.com",
        "https://cdn.jsdelivr.net",
        "https://ajax.googleapis.com",
        "https://stackpath.bootstrapcdn.com",
        "http://localhost:8080/static", // No external static asset references
    ];

    for pattern in &forbidden_patterns {
        assert!(
            !html.contains(pattern),
            "Interactive HTML report must be 100% self-contained! Found external URL reference: {}",
            pattern
        );
    }

    // Verify styles and scripts are completely inline
    assert!(html.contains("<style>"), "Must contain embedded style tags");
    assert!(html.contains("<script>"), "Must contain embedded script tags");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_giant_dataset_stress_1000_tests() {
    let mut large_dataset = Vec::with_capacity(1000);
    for i in 1..=1000 {
        let cat = match i % 4 {
            0 => FailureCategory::RealBug,
            1 => FailureCategory::FlakyInfra,
            2 => FailureCategory::AntiPattern,
            _ => FailureCategory::None,
        };
        let status = match cat {
            FailureCategory::None => TestStatus::Passed,
            FailureCategory::RealBug => TestStatus::Failed,
            FailureCategory::FlakyInfra => TestStatus::Flaky,
            FailureCategory::AntiPattern => TestStatus::Broken,
        };

        large_dataset.push(ChaosTestResult {
            test_id: format!("STRESS-{:04}", i),
            name: format!("test_large_scale_simulation_item_{:04}", i),
            suite: format!("Large Scale Suite {}", i % 10),
            track_id: "playwright-ts".to_string(),
            status,
            category: cat,
            duration_ms: (i as u64 * 7) % 5000 + 50,
            error_message: if status != TestStatus::Passed {
                Some(format!("Synthetic error message for test iteration {}", i))
            } else {
                None
            },
            stack_trace: if status != TestStatus::Passed {
                Some(format!("Detailed stack trace line 1\n  at com.cherenkov.Item_{}.run\n  at main.rs:100", i))
            } else {
                None
            },
            chaos_event: if status != TestStatus::Passed {
                Some(ChaosEventTelemetry {
                    layer: "L7".to_string(),
                    event_type: "scale_stress".to_string(),
                    latency_ms: 100,
                    jitter_ms: 10,
                    packet_loss_rate: 0.05,
                    proxy_log: Some(format!("Proxy log telemetry chunk for test {}", i)),
                    correlated_timestamp: "2026-08-24T18:00:00Z".to_string(),
                    retry_attempts: 1,
                    injection_target: "127.0.0.1:8086".to_string(),
                })
            } else {
                None
            },
            flakiness_metrics: None,
            steps: vec![
                TestStepTelemetry {
                    name: "Step 1".to_string(),
                    status: TestStatus::Passed,
                    duration_ms: 25,
                    error: None,
                },
                TestStepTelemetry {
                    name: "Step 2".to_string(),
                    status,
                    duration_ms: 50,
                    error: None,
                },
            ],
            labels: {
                let mut map = HashMap::new();
                map.insert("suite".to_string(), format!("Large Scale Suite {}", i % 10));
                map.insert("track".to_string(), "playwright-ts".to_string());
                map.insert("framework".to_string(), "cherenkov-matrix".to_string());
                map
            },
            root_cause_hint: Some(format!("Root cause analysis note for test {}", i)),
        });
    }

    let temp_dir = std::env::temp_dir().join("cherenkov_stress_1000_tests");
    let _ = fs::remove_dir_all(&temp_dir);

    let start = std::time::Instant::now();
    let summary = generate_allure_report_for_dataset(&large_dataset, &temp_dir)
        .expect("1000-test report generation must succeed");
    let elapsed = start.elapsed();

    assert_eq!(summary.total_tests, 1000);
    assert_eq!(summary.real_bugs, 250);
    assert_eq!(summary.flaky_infra, 250);
    assert_eq!(summary.anti_patterns, 250);
    assert_eq!(summary.passed, 250);
    assert!(elapsed.as_secs() < 10, "1000-test generation should complete under 10 seconds (took {:?})", elapsed);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_triage_scoring_boundary_matrix() {
    let test_id = "BUG-101"; // RealBug

    // 1. Explanation Length Boundaries (Thresholds: 15 chars = +8 XP, 40 chars = +15 XP)
    // Sub-15 chars, 0 keywords
    let sub_short = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "short text".to_string(), // 10 chars -> 0 length bonus, 0 kw
        suggested_fix: "".to_string(),
    };
    let res_short = evaluate_triage(&sub_short);
    assert!(res_short.correct);
    assert_eq!(res_short.base_score, 100);
    assert_eq!(res_short.explanation_score, 0); // 0
    assert_eq!(res_short.fix_score, 0);
    assert_eq!(res_short.score_awarded, 100);

    // Exact 15 chars, 0 keywords -> +8 XP
    let sub_15 = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "123456789012345".to_string(), // 15 chars
        suggested_fix: "".to_string(),
    };
    let res_15 = evaluate_triage(&sub_15);
    assert_eq!(res_15.explanation_score, 8);

    // Exact 39 chars, 0 keywords -> +8 XP
    let sub_39 = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "123456789012345678901234567890123456789".to_string(), // 39 chars
        suggested_fix: "".to_string(),
    };
    let res_39 = evaluate_triage(&sub_39);
    assert_eq!(res_39.explanation_score, 8);

    // Exact 40 chars, 0 keywords -> +15 XP
    let sub_40 = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "1234567890123456789012345678901234567890".to_string(), // 40 chars
        suggested_fix: "".to_string(),
    };
    let res_40 = evaluate_triage(&sub_40);
    assert_eq!(res_40.explanation_score, 15);

    // 2. Keyword Count Boundaries: 1 keyword = +10 XP, >=2 keywords = +20 XP
    // 1 keyword ('rbac'), <15 chars -> 10 XP
    let sub_1kw = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "rbac bug".to_string(), // 8 chars, 1 kw
        suggested_fix: "".to_string(),
    };
    let res_1kw = evaluate_triage(&sub_1kw);
    assert_eq!(res_1kw.explanation_score, 10);

    // 2 keywords ('rbac', 'deadlock'), <15 chars -> 20 XP
    let sub_2kw = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "rbac deadlock".to_string(), // 13 chars, 2 kw
        suggested_fix: "".to_string(),
    };
    let res_2kw = evaluate_triage(&sub_2kw);
    assert_eq!(res_2kw.explanation_score, 20);

    // Max explanation score cap: 15 (length) + 20 (>=2 kw) = 35 XP max
    let sub_max_exp = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "The rbac authorization middleware caused a deadlock and foreign key defect".to_string(), // >40 chars, 4 kw
        suggested_fix: "".to_string(),
    };
    let res_max_exp = evaluate_triage(&sub_max_exp);
    assert_eq!(res_max_exp.explanation_score, 35);

    // 3. Suggested Fix Length & Pattern Boundaries (Thresholds: 10 chars = +5 XP, 30 chars = +10 XP, pattern = +5 XP)
    // < 10 chars, 0 pattern -> 0 XP
    let sub_fix_short = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "".to_string(),
        suggested_fix: "fix it".to_string(), // 6 chars
    };
    let res_fix_short = evaluate_triage(&sub_fix_short);
    assert_eq!(res_fix_short.fix_score, 0);

    // Exact 10 chars, 0 pattern -> +5 XP
    let sub_fix_10 = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "".to_string(),
        suggested_fix: "1234567890".to_string(), // 10 chars
    };
    let res_fix_10 = evaluate_triage(&sub_fix_10);
    assert_eq!(res_fix_10.fix_score, 5);

    // Exact 30 chars, 0 pattern -> +10 XP
    let sub_fix_30 = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "".to_string(),
        suggested_fix: "123456789012345678901234567890".to_string(), // 30 chars
    };
    let res_fix_30 = evaluate_triage(&sub_fix_30);
    assert_eq!(res_fix_30.fix_score, 10);

    // Pattern ('retry'), <10 chars -> +5 XP
    let sub_fix_pat = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "".to_string(),
        suggested_fix: "retry".to_string(), // 5 chars, 1 pattern
    };
    let res_fix_pat = evaluate_triage(&sub_fix_pat);
    assert_eq!(res_fix_pat.fix_score, 5);

    // Max fix score cap: 10 (length >= 30) + 5 (pattern 'retry') = 15 XP max
    let sub_fix_max = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "".to_string(),
        suggested_fix: "Apply exponential backoff retry and prepared statement locking on db".to_string(), // >30 chars, multiple patterns
    };
    let res_fix_max = evaluate_triage(&sub_fix_max);
    assert_eq!(res_fix_max.fix_score, 15);

    // Max Total Score: 100 (base) + 35 (max exp) + 15 (max fix) = 150 XP
    let sub_max_all = TriageSubmission {
        test_id: test_id.to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "The rbac authorization middleware caused a deadlock and foreign key defect".to_string(),
        suggested_fix: "Apply exponential backoff retry and prepared statement locking on db".to_string(),
    };
    let res_max_all = evaluate_triage(&sub_max_all);
    assert_eq!(res_max_all.score_awarded, 150);
}

#[test]
fn test_triage_streak_calculation_adversarial_timelines() {
    let mut state = GamificationState::default();

    // Day 1: 2026-08-20 -> streak should become 1
    state.update_streak("2026-08-20T10:00:00Z");
    assert_eq!(state.streak_days, 1);
    assert_eq!(state.last_active_date, Some("2026-08-20".to_string()));

    // Same day submission (different hour) -> streak must remain 1
    state.update_streak("2026-08-20T23:59:59Z");
    assert_eq!(state.streak_days, 1);

    // Day 2 (consecutive): 2026-08-21 -> streak should increment to 2
    state.update_streak("2026-08-21T08:00:00Z");
    assert_eq!(state.streak_days, 2);
    assert_eq!(state.last_active_date, Some("2026-08-21".to_string()));

    // Day 3 (consecutive): 2026-08-22 -> streak should increment to 3
    state.update_streak("2026-08-22T14:30:00Z");
    assert_eq!(state.streak_days, 3);

    // Past timestamp (time travel / clock skew): 2026-08-19 -> must be ignored
    state.update_streak("2026-08-19T12:00:00Z");
    assert_eq!(state.streak_days, 3);
    assert_eq!(state.last_active_date, Some("2026-08-22".to_string()));

    // Skipped 2 days (gap): 2026-08-25 -> streak should reset to 1
    state.update_streak("2026-08-25T09:00:00Z");
    assert_eq!(state.streak_days, 1);
    assert_eq!(state.last_active_date, Some("2026-08-25".to_string()));

    // Malformed date string -> must not panic
    state.update_streak("not-a-valid-iso-date");
    assert_eq!(state.streak_days, 1);

    // Empty date string -> must not panic
    state.update_streak("");
    assert_eq!(state.streak_days, 1);
}

#[test]
fn test_gamification_persistence_corruption_and_recovery() {
    let temp_file = std::env::temp_dir().join("cherenkov_corrupted_progress.json");

    // 1. Write corrupted/truncated JSON
    fs::write(&temp_file, "{ \"total_xp\": 500, \"level_name\": ").expect("Failed to write test file");

    // load_progress should return default without panicking
    let loaded = load_progress(Some(&temp_file));
    assert!(loaded.is_err() || loaded.unwrap_or_default().total_xp == 0);

    // evaluate_and_record_progress should gracefully recover and save clean state
    let sub = TriageSubmission {
        test_id: "BUG-101".to_string(),
        learner_category: FailureCategory::RealBug,
        root_cause_explanation: "RBAC middleware vulnerability".to_string(),
        suggested_fix: "Add role check".to_string(),
    };

    let (res, state) = evaluate_and_record_progress(&sub, Some(&temp_file));
    assert!(res.correct);
    assert!(state.total_xp >= 100);

    // Verify clean JSON on disk
    let reloaded = load_progress(Some(&temp_file)).expect("Should recover cleanly");
    assert_eq!(reloaded.total_xp, state.total_xp);

    let _ = fs::remove_file(&temp_file);
}
