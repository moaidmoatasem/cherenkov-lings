use cherenkov_lings::feedback::{self, AntiPatternKind};
use cherenkov_lings::runner::{AnyRunner, DrillResponse, JvmRunner, RunResult};
use cherenkov_lings::watcher::should_ignore_path;
use std::path::Path;
use std::sync::Arc;

#[test]
fn test_jvm_runner_extract_class_name_all_drills() {
    let drills = [
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java",
            "com.cherenkov.drill01_idempotency.Exercise",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Solution.java",
            "com.cherenkov.drill01_idempotency.Solution",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Exercise.java",
            "com.cherenkov.drill02_jwt_auth.Exercise",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Solution.java",
            "com.cherenkov.drill02_jwt_auth.Solution",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
            "com.cherenkov.drill03_kafka_lag.Exercise",
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java",
            "com.cherenkov.drill03_kafka_lag.Solution",
        ),
    ];

    for (file_path, expected_class) in drills {
        let extracted = JvmRunner::extract_class_name(file_path);
        assert_eq!(
            extracted.as_deref(),
            Some(expected_class),
            "Failed resolving class from path: {}",
            file_path
        );

        // Also test Windows backslash representation
        let win_path = file_path.replace('/', "\\");
        let win_extracted = JvmRunner::extract_class_name(&win_path);
        assert_eq!(
            win_extracted.as_deref(),
            Some(expected_class),
            "Failed resolving class from Windows path: {}",
            win_path
        );
    }
}

#[test]
fn test_jvm_runner_parse_all_drill_surefire_reports() {
    let reports_dir = Path::new("exercises/02_api_restassured_java/target/surefire-reports");
    if !reports_dir.exists() {
        eprintln!("Skipping: Surefire reports directory not found");
        return;
    }

    // Drill 1 Exercise (Failure - 409 Conflict)
    let d1_ex_path = reports_dir.join("TEST-com.cherenkov.drill01_idempotency.Exercise.xml");
    if d1_ex_path.exists() {
        let report = JvmRunner::parse_surefire_report(&d1_ex_path).expect("Parse Drill 1 Exercise");
        assert_eq!(report.failures, 1);
        assert_eq!(report.errors, 0);
        let failure = report.test_cases[0].failure.as_ref().unwrap();
        assert!(
            failure.message.contains("409") || failure.message.contains("Expected status code")
        );
    }

    // Drill 1 Solution (Passes - 2 tests)
    let d1_sol_path = reports_dir.join("TEST-com.cherenkov.drill01_idempotency.Solution.xml");
    if d1_sol_path.exists() {
        let report =
            JvmRunner::parse_surefire_report(&d1_sol_path).expect("Parse Drill 1 Solution");
        assert_eq!(report.tests, 2);
        assert_eq!(report.failures, 0);
        assert_eq!(report.errors, 0);
    }

    // Drill 2 Exercise (Failure - 401 Unauthorized)
    let d2_ex_path = reports_dir.join("TEST-com.cherenkov.drill02_jwt_auth.Exercise.xml");
    if d2_ex_path.exists() {
        let report = JvmRunner::parse_surefire_report(&d2_ex_path).expect("Parse Drill 2 Exercise");
        assert_eq!(report.failures, 1);
        assert_eq!(report.errors, 0);
    }

    // Drill 2 Solution (Passes - 1 test)
    let d2_sol_path = reports_dir.join("TEST-com.cherenkov.drill02_jwt_auth.Solution.xml");
    if d2_sol_path.exists() {
        let report =
            JvmRunner::parse_surefire_report(&d2_sol_path).expect("Parse Drill 2 Solution");
        assert_eq!(report.tests, 1);
        assert_eq!(report.failures, 0);
        assert_eq!(report.errors, 0);
    }

    // Drill 3 Exercise (Failure - stale balance)
    let d3_ex_path = reports_dir.join("TEST-com.cherenkov.drill03_kafka_lag.Exercise.xml");
    if d3_ex_path.exists() {
        let report = JvmRunner::parse_surefire_report(&d3_ex_path).expect("Parse Drill 3 Exercise");
        assert_eq!(report.failures, 1);
        assert_eq!(report.errors, 0);
    }

    // Drill 3 Solution (Passes - 1 test)
    let d3_sol_path = reports_dir.join("TEST-com.cherenkov.drill03_kafka_lag.Solution.xml");
    if d3_sol_path.exists() {
        let report =
            JvmRunner::parse_surefire_report(&d3_sol_path).expect("Parse Drill 3 Solution");
        assert_eq!(report.tests, 1);
        assert_eq!(report.failures, 0);
        assert_eq!(report.errors, 0);
    }
}

#[test]
fn test_ast_feedback_on_actual_java_drill_files() {
    let d3_ex = Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
    );
    let d3_sol = Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java",
    );

    if !d3_ex.exists() || !d3_sol.exists() {
        eprintln!("Skipping: Drill 3 files not found");
        return;
    }

    // Exercise AST analysis
    let ast_ex = feedback::analyze_file(d3_ex).expect("Analyze Exercise.java");
    assert!(
        ast_ex.has_wait_for_timeout,
        "Exercise.java must trigger timing anti-pattern flag"
    );
    assert_eq!(ast_ex.anti_patterns.len(), 1);
    assert_eq!(
        ast_ex.anti_patterns[0].kind,
        AntiPatternKind::ThreadSleep {
            duration_ms: Some(100)
        }
    );
    assert_eq!(ast_ex.anti_patterns[0].line, 64);

    // Solution AST analysis
    let ast_sol = feedback::analyze_file(d3_sol).expect("Analyze Solution.java");
    assert!(
        !ast_sol.has_wait_for_timeout,
        "Solution.java must not contain Thread.sleep"
    );
    assert_eq!(ast_sol.anti_patterns.len(), 0);

    // Simulated test response for evaluation
    let mock_response = DrillResponse {
        id: "mock-1".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 2500,
        runs: vec![
            RunResult {
                iteration: 1,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 2,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 3,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 4,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 5,
                passed: true,
                duration_ms: 500,
                error: None,
            },
        ],
        error: None,
    };

    // Exercise feedback evaluation - must cap flakiness at 40.0 pts and fail
    let card_ex = feedback::evaluate_feedback(
        &mock_response,
        &ast_ex,
        "API Resilience & Security (REST Assured Java)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_ex.flakiness.score, 40.0);
    assert!(!card_ex.passed);
    assert!(
        card_ex
            .diagnostics
            .iter()
            .any(|d| d.contains("Thread.sleep(100ms)"))
    );

    // Solution feedback evaluation - 100.0 pts and passes
    let card_sol = feedback::evaluate_feedback(
        &mock_response,
        &ast_sol,
        "API Resilience & Security (REST Assured Java)",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_sol.flakiness.score, 100.0);
    assert_eq!(card_sol.total_score, 100.0);
    assert!(card_sol.passed);
}

#[test]
fn test_watcher_filter_on_maven_build_artifacts() {
    // Target directory files
    assert!(should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/target/classes/com/cherenkov/App.class"
    )));
    assert!(should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/target/test-classes/com/cherenkov/drill01_idempotency/Exercise.class"
    )));
    assert!(should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/target/surefire-reports/TEST-com.cherenkov.drill01_idempotency.Exercise.xml"
    )));
    assert!(should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/target/maven-status/maven-compiler-plugin/testCompile/default-testCompile/inputFiles.lst"
    )));

    // Windows backslash paths
    assert!(should_ignore_path(Path::new(
        r"exercises\02_api_restassured_java\target\test-classes\com\cherenkov\drill01_idempotency\Exercise.class"
    )));
    assert!(should_ignore_path(Path::new(
        r"exercises\02_api_restassured_java\target\surefire-reports\TEST-com.cherenkov.drill01_idempotency.Exercise.xml"
    )));

    // Standalone .class files
    assert!(should_ignore_path(Path::new("Exercise.class")));
    assert!(should_ignore_path(Path::new(
        "Solution$JwtRefreshFilter.class"
    )));

    // Temporary editor files
    assert!(should_ignore_path(Path::new("Exercise.java.tmp")));
    assert!(should_ignore_path(Path::new("Exercise.java~")));
    assert!(should_ignore_path(Path::new(".Exercise.java.swp")));

    // Legitimate Java source files MUST NOT be ignored
    assert!(!should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java"
    )));
    assert!(!should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Solution.java"
    )));
    assert!(!should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Exercise.java"
    )));
    assert!(!should_ignore_path(Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java"
    )));
}

#[test]
fn test_any_runner_jvm_wrapping() {
    let jvm_runner = JvmRunner::new("exercises/02_api_restassured_java");
    let any_runner = AnyRunner::Jvm(Arc::new(jvm_runner));

    match any_runner {
        AnyRunner::Jvm(r) => {
            assert_eq!(
                r.exercise_dir(),
                Path::new("exercises/02_api_restassured_java")
            );
        }
        _ => panic!("Expected AnyRunner::Jvm"),
    }
}
