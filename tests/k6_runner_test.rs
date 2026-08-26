use cherenkov_lings::feedback::{self, StaticAnalysisReport};
use cherenkov_lings::runner::{
    AnyRunner, DrillResponse, K6Runner, RunResult, parse_k6_summary_json,
};
use std::path::Path;
use std::sync::Arc;

#[test]
fn test_k6_runner_initialization_and_options() {
    let runner = K6Runner::new();
    assert_eq!(runner.k6_cmd(), "k6");

    let custom = K6Runner::with_k6_cmd("custom-k6-binary");
    assert_eq!(custom.k6_cmd(), "custom-k6-binary");
}

#[test]
fn test_any_runner_k6_wrapping() {
    let runner = K6Runner::new();
    let any_runner = AnyRunner::K6(Arc::new(runner));

    match any_runner {
        AnyRunner::K6(r) => {
            assert_eq!(r.k6_cmd(), "k6");
        }
        _ => panic!("Expected AnyRunner::K6"),
    }
}

#[test]
fn test_parse_k6_summary_json_drill01_pool_starvation() {
    // Drill 01 Exercise: instant 50 VUs blast exhausts connection pool -> http_req_failed fails
    let failed_json = r#"{
        "metrics": {
            "http_req_duration": {
                "type": "trend",
                "values": {
                    "avg": 850.4,
                    "p(95)": 1950.0,
                    "p(99)": 2800.0
                }
            },
            "http_req_failed": {
                "type": "rate",
                "values": {
                    "rate": 0.082,
                    "passes": 41,
                    "fails": 459
                },
                "thresholds": {
                    "rate<0.01": {
                        "ok": false
                    }
                }
            }
        }
    }"#;

    let report_failed = parse_k6_summary_json(failed_json).expect("Parse failed summary JSON");
    assert!(!report_failed.all_thresholds_passed);
    assert_eq!(report_failed.failed_thresholds.len(), 1);
    assert!(report_failed.failed_thresholds[0].contains("http_req_failed"));
    assert!(report_failed.failed_thresholds[0].contains("rate<0.01"));
    assert_eq!(report_failed.http_req_failed_rate, Some(0.082));

    // Drill 01 Solution: staged ramp-up keeps error rate < 1%
    let passed_json = r#"{
        "metrics": {
            "http_req_duration": {
                "type": "trend",
                "values": {
                    "avg": 120.5,
                    "p(95)": 350.0,
                    "p(99)": 600.0
                },
                "thresholds": {
                    "p(95)<2000": {
                        "ok": true
                    }
                }
            },
            "http_req_failed": {
                "type": "rate",
                "values": {
                    "rate": 0.002,
                    "passes": 1,
                    "fails": 499
                },
                "thresholds": {
                    "rate<0.01": {
                        "ok": true
                    }
                }
            }
        }
    }"#;

    let report_passed = parse_k6_summary_json(passed_json).expect("Parse passed summary JSON");
    assert!(report_passed.all_thresholds_passed);
    assert!(report_passed.failed_thresholds.is_empty());
    assert_eq!(report_passed.avg_duration_ms, 121);
    assert_eq!(report_passed.p95_duration_ms, Some(350.0));
    assert_eq!(report_passed.http_req_failed_rate, Some(0.002));
}

#[test]
fn test_parse_k6_summary_json_drill02_spike_profile_p99() {
    // Drill 02 Solution: Trend metric search_response_time p(99)<5000 and search_errors rate<0.05
    let json = r#"{
        "metrics": {
            "search_response_time": {
                "type": "trend",
                "values": {
                    "avg": 450.0,
                    "p(90)": 1100.0,
                    "p(95)": 1800.0,
                    "p(99)": 3900.0
                },
                "thresholds": {
                    "p(99)<5000": {
                        "ok": true
                    }
                }
            },
            "search_errors": {
                "type": "rate",
                "values": {
                    "rate": 0.01
                },
                "thresholds": {
                    "rate<0.05": {
                        "ok": true
                    }
                }
            }
        }
    }"#;

    let report = parse_k6_summary_json(json).expect("Parse spike summary");
    assert!(report.all_thresholds_passed);
    assert_eq!(report.p99_duration_ms, Some(3900.0));
    assert_eq!(report.http_req_failed_rate, Some(0.01));
}

#[test]
fn test_parse_k6_summary_json_drill03_chaos_sla_assertion() {
    // Drill 03 Solution: chaos_errors rate<0.05 and http_req_duration p(95)<3000
    let json = r#"{
        "metrics": {
            "chaos_errors": {
                "type": "rate",
                "values": {
                    "rate": 0.03
                },
                "thresholds": {
                    "rate<0.05": {
                        "ok": true
                    }
                }
            },
            "http_req_duration": {
                "type": "trend",
                "values": {
                    "avg": 620.0,
                    "p(95)": 1400.0
                },
                "thresholds": {
                    "p(95)<3000": {
                        "ok": true
                    }
                }
            }
        }
    }"#;

    let report = parse_k6_summary_json(json).expect("Parse chaos summary");
    assert!(report.all_thresholds_passed);
    assert_eq!(report.avg_duration_ms, 620);
    assert_eq!(report.p95_duration_ms, Some(1400.0));
    assert_eq!(report.http_req_failed_rate, Some(0.03));
}

#[test]
fn test_k6_feedback_matrix_evaluation() {
    let mock_failed_response = DrillResponse {
        id: "k6-req-1".to_string(),
        ok: true,
        passed: false,
        iterations: 1,
        passed_iterations: 0,
        failed_iterations: 1,
        total_duration_ms: 2500,
        runs: vec![RunResult {
            iteration: 1,
            passed: false,
            duration_ms: 2500,
            error: Some("http_req_failed: threshold 'rate<0.01' failed".to_string()),
        }],
        error: Some("http_req_failed: threshold 'rate<0.01' failed".to_string()),
    };

    let ast = StaticAnalysisReport {
        file_path: "exercises/04_perf_k6_js/01_database_pool_starvation/exercise.js".to_string(),
        locator_quality_score: 100.0,
        ..Default::default()
    };

    let scorecard = feedback::evaluate_feedback(
        &mock_failed_response,
        &ast,
        "High-Concurrency Load Testing (k6 JS)",
        "1.0.0",
        85.0,
        1000,
    );

    assert!(!scorecard.passed);
    assert_eq!(scorecard.correctness.score, 0.0);
    assert!(
        scorecard
            .diagnostics
            .iter()
            .any(|d| d.contains("threshold 'rate<0.01' failed"))
    );

    let mock_passed_response = DrillResponse {
        id: "k6-req-2".to_string(),
        ok: true,
        passed: true,
        iterations: 1,
        passed_iterations: 1,
        failed_iterations: 0,
        total_duration_ms: 500,
        runs: vec![RunResult {
            iteration: 1,
            passed: true,
            duration_ms: 500,
            error: None,
        }],
        error: None,
    };

    let scorecard_pass = feedback::evaluate_feedback(
        &mock_passed_response,
        &ast,
        "High-Concurrency Load Testing (k6 JS)",
        "1.0.0",
        85.0,
        1000,
    );

    assert!(scorecard_pass.passed);
    assert_eq!(scorecard_pass.correctness.score, 100.0);
    assert_eq!(scorecard_pass.total_score, 100.0);
}

#[tokio::test]
async fn test_k6_runner_nonexistent_file_drill_response() {
    let runner = K6Runner::new();
    let resp = runner
        .run_drill("non_existent_k6_script.js", "", 1, 5000)
        .await
        .expect("Drill execution");
    assert!(!resp.ok);
    assert!(!resp.passed);
    assert!(resp.error.is_some());
}

#[test]
fn test_all_k6_drill_files_exist_on_disk() {
    let drill_files = [
        "exercises/04_perf_k6_js/01_database_pool_starvation/exercise.js",
        "exercises/04_perf_k6_js/01_database_pool_starvation/solution.js",
        "exercises/04_perf_k6_js/01_database_pool_starvation/hints.md",
        "exercises/04_perf_k6_js/02_spike_profile_p99/exercise.js",
        "exercises/04_perf_k6_js/02_spike_profile_p99/solution.js",
        "exercises/04_perf_k6_js/02_spike_profile_p99/hints.md",
        "exercises/04_perf_k6_js/03_chaos_sla_assertion/exercise.js",
        "exercises/04_perf_k6_js/03_chaos_sla_assertion/solution.js",
        "exercises/04_perf_k6_js/03_chaos_sla_assertion/hints.md",
        "exercises/04_perf_k6_js/k6_runner.js",
    ];

    for file in drill_files {
        assert!(
            Path::new(file).exists(),
            "Expected k6 track file to exist: {}",
            file
        );
    }
}
