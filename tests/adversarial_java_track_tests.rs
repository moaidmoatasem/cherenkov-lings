#![allow(dead_code, unused_imports)]

// src/runner.rs references crate::pipeline (the in-process CI simulator
// backing the ci-pipeline track), so the module must exist in this test
// crate too for the #[path] include below to resolve.
#[path = "../src/pipeline/mod.rs"]
mod pipeline;

#[path = "../src/runner.rs"]
mod runner;

#[path = "../src/feedback.rs"]
mod feedback;

#[path = "../src/watcher.rs"]
mod watcher;

use feedback::*;
use runner::{
    AnyRunner, DrillResponse, JvmRunner, RunResult, SurefireFailure, SurefireReport,
    SurefireTestCase, parse_surefire_xml,
};
use std::path::Path;
use std::sync::Arc;

// =========================================================================
// 1. PATH-TO-CLASS RESOLUTION ADVERSARIAL TESTS
// =========================================================================

#[test]
fn test_extract_class_name_standard_and_windows_paths() {
    let cases = vec![
        // Standard Maven directory structures
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java",
            Some("com.cherenkov.drill01_idempotency.Exercise"),
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Solution.java",
            Some("com.cherenkov.drill01_idempotency.Solution"),
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Exercise.java",
            Some("com.cherenkov.drill02_jwt_auth.Exercise"),
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Solution.java",
            Some("com.cherenkov.drill02_jwt_auth.Solution"),
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
            Some("com.cherenkov.drill03_kafka_lag.Exercise"),
        ),
        (
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java",
            Some("com.cherenkov.drill03_kafka_lag.Solution"),
        ),
        // Windows backslashes
        (
            r"exercises\02_api_restassured_java\src\test\java\com\cherenkov\drill01_idempotency\Exercise.java",
            Some("com.cherenkov.drill01_idempotency.Exercise"),
        ),
        (
            r"C:\projects\cherenkov\exercises\02_api_restassured_java\src\test\java\org\example\MyTest.java",
            Some("org.example.MyTest"),
        ),
        // Paths with spaces and version numbers
        (
            r"C:\My Documents\Workspace v1.2.3\exercises\02_api_restassured_java\src\test\java\com\cherenkov\CustomTest.java",
            Some("com.cherenkov.CustomTest"),
        ),
        // Subpaths rooted at src/test/java or test/java or main/java
        (
            "src/test/java/com/corp/service/UserApiTest.java",
            Some("com.corp.service.UserApiTest"),
        ),
        (
            "src/main/java/com/corp/service/UserApi.java",
            Some("com.corp.service.UserApi"),
        ),
        (
            "test/java/com/corp/service/HelperTest.java",
            Some("com.corp.service.HelperTest"),
        ),
        (
            "main/java/com/corp/service/Helper.java",
            Some("com.corp.service.Helper"),
        ),
        // Fully-qualified class names directly passed
        (
            "com.cherenkov.drill01_idempotency.Exercise",
            Some("com.cherenkov.drill01_idempotency.Exercise"),
        ),
        (
            "org.junit.jupiter.api.Test",
            Some("org.junit.jupiter.api.Test"),
        ),
        // Relative slash class names
        (
            "com/cherenkov/drill01_idempotency/Exercise.java",
            Some("com.cherenkov.drill01_idempotency.Exercise"),
        ),
        (
            "com/cherenkov/drill01_idempotency/Exercise",
            Some("com.cherenkov.drill01_idempotency.Exercise"),
        ),
        // Deeply nested package hierarchy
        (
            "src/test/java/a/b/c/d/e/f/g/DeepTest.java",
            Some("a.b.c.d.e.f.g.DeepTest"),
        ),
        // Simple class name without path
        ("SimpleTest", Some("SimpleTest")),
        ("SimpleTest.java", Some("SimpleTest")),
        // Empty string
        ("", None),
    ];

    for (input, expected) in cases {
        let actual = JvmRunner::extract_class_name(input);
        assert_eq!(
            actual.as_deref(),
            expected,
            "Failed extracting class name from input '{}': expected {:?}, got {:?}",
            input,
            expected,
            actual
        );
    }
}

#[test]
fn test_extract_class_name_disk_fallback_with_package_header() {
    let p1 = "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java";
    if Path::new(p1).exists() {
        assert_eq!(
            JvmRunner::extract_class_name(p1),
            Some("com.cherenkov.drill01_idempotency.Exercise".to_string())
        );
    }
}

// =========================================================================
// 2. SUREFIRE XML PARSER ADVERSARIAL TESTS (EMPTY, CORRUPT, MULTILINE CDATA)
// =========================================================================

#[test]
fn test_surefire_parser_empty_and_minimal_xml() {
    // Empty XML string -> Error
    assert!(parse_surefire_xml("").is_err());
    assert!(parse_surefire_xml("   \n\t  ").is_err());
    assert!(parse_surefire_xml("<notasuite></notasuite>").is_err());

    // Minimal valid testsuite with 0 tests
    let minimal = r#"<testsuite name="EmptySuite" time="0.0" tests="0" errors="0" skipped="0" failures="0"></testsuite>"#;
    let report = parse_surefire_xml(minimal).expect("Parse minimal suite");
    assert_eq!(report.name, "EmptySuite");
    assert_eq!(report.tests, 0);
    assert_eq!(report.failures, 0);
    assert_eq!(report.errors, 0);
    assert_eq!(report.test_cases.len(), 0);
}

#[test]
fn test_surefire_parser_multiline_cdata_with_nested_xml_entities() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.cherenkov.drill01_idempotency.Exercise" time="2.344" tests="1" errors="0" skipped="0" failures="1">
  <testcase name="testCheckoutWithStaticKey" classname="com.cherenkov.drill01_idempotency.Exercise" time="1.713">
    <failure message="1 expectation failed.&#10;Expected status code &lt;200&gt; but was &lt;409&gt;.&#10;" type="java.lang.AssertionError"><![CDATA[java.lang.AssertionError: 
1 expectation failed.
Expected status code <200> but was <409>.

	at java.base/jdk.internal.reflect.DirectConstructorHandleAccessor.newInstance(DirectConstructorHandleAccessor.java:62)
	at io.restassured.internal.ResponseSpecificationImpl.statusCode(ResponseSpecificationImpl.groovy:143)
	at com.cherenkov.drill01_idempotency.Exercise.testCheckoutWithStaticKey(Exercise.java:51)
]]></failure>
  </testcase>
</testsuite>"#;

    let report = parse_surefire_xml(xml).expect("Parse complex CDATA failure");
    assert_eq!(report.failures, 1);
    assert_eq!(report.test_cases.len(), 1);

    let tc = &report.test_cases[0];
    assert_eq!(tc.name, "testCheckoutWithStaticKey");
    assert_eq!(tc.classname, "com.cherenkov.drill01_idempotency.Exercise");

    let failure = tc.failure.as_ref().expect("Failure object present");
    // Escaped entities &#10; and &lt; &gt; must be properly decoded
    assert!(
        failure
            .message
            .contains("Expected status code <200> but was <409>")
    );
    assert_eq!(failure.failure_type, "java.lang.AssertionError");
    assert!(failure.stack_trace.contains("Exercise.java:51"));
    assert!(failure.stack_trace.contains("java.lang.AssertionError:"));
}

#[test]
fn test_surefire_parser_multiple_explicit_closing_test_cases() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.cherenkov.TestSuite" time="5.120" tests="3" errors="1" skipped="0" failures="1">
  <testcase name="testSuccess" classname="com.cherenkov.TestSuite" time="0.500"></testcase>
  <testcase name="testAssertionFailure" classname="com.cherenkov.TestSuite" time="1.200">
    <failure message="expected 200 got 500" type="java.lang.AssertionError"><![CDATA[Stack trace line 1
Stack trace line 2]]></failure>
  </testcase>
  <testcase name="testExceptionError" classname="com.cherenkov.TestSuite" time="2.100">
    <error message="NullPointerException occurred" type="java.lang.NullPointerException"><![CDATA[java.lang.NullPointerException: object is null
	at com.cherenkov.TestSuite.testExceptionError(TestSuite.java:88)]]></error>
  </testcase>
</testsuite>"#;

    let report = parse_surefire_xml(xml).expect("Parse multi-case suite");
    assert_eq!(report.tests, 3);
    assert_eq!(report.failures, 1);
    assert_eq!(report.errors, 1);
    assert_eq!(report.test_cases.len(), 3);

    // Case 1: Success
    assert_eq!(report.test_cases[0].name, "testSuccess");
    assert!(report.test_cases[0].failure.is_none());
    assert!(report.test_cases[0].error.is_none());

    // Case 2: Failure
    assert_eq!(report.test_cases[1].name, "testAssertionFailure");
    let f = report.test_cases[1].failure.as_ref().unwrap();
    assert_eq!(f.message, "expected 200 got 500");
    assert_eq!(f.failure_type, "java.lang.AssertionError");

    // Case 3: Error
    assert_eq!(report.test_cases[2].name, "testExceptionError");
    let e = report.test_cases[2].error.as_ref().unwrap();
    assert_eq!(e.message, "NullPointerException occurred");
    assert_eq!(e.failure_type, "java.lang.NullPointerException");
    assert!(e.stack_trace.contains("TestSuite.java:88"));
}

#[test]
fn test_surefire_parser_corrupt_and_unusual_formatting() {
    // Single quotes in XML attributes
    let single_quote_xml = "<testsuite name='SingleQuoteSuite' time='1.5' tests='1' errors='0' skipped='0' failures='0'><testcase name='case1' classname='pkg.Case' time='1.5'></testcase></testsuite>";
    let report = parse_surefire_xml(single_quote_xml).expect("Parse single quote XML");
    assert_eq!(report.name, "SingleQuoteSuite");
    assert_eq!(report.tests, 1);
    assert_eq!(report.test_cases.len(), 1);

    // Self-closing failure tag inside testcase
    let self_closing_failure = r#"<testsuite name="SelfClosing" tests="1" errors="0" skipped="0" failures="1">
      <testcase name="c1" classname="pkg.C"><failure message="abrupt failure" type="Fail"/></testcase>
    </testsuite>"#;
    let r2 = parse_surefire_xml(self_closing_failure).expect("Parse self-closing failure");
    assert_eq!(r2.failures, 1);
    assert_eq!(
        r2.test_cases[0].failure.as_ref().unwrap().message,
        "abrupt failure"
    );
}

// =========================================================================
// 3. FLAKINESS CALCULATIONS WITH SIMULATED INTERMITTENT PASSES / FAILS
// =========================================================================

#[test]
fn test_flakiness_matrix_permutations() {
    // Test all pass rates from 0/5 to 5/5
    for passed_count in 0..=5 {
        let failed_count = 5 - passed_count;
        let response = DrillResponse {
            id: format!("req-flake-{}", passed_count),
            ok: true,
            passed: passed_count == 5,
            iterations: 5,
            passed_iterations: passed_count,
            failed_iterations: failed_count,
            total_duration_ms: 2500, // 500ms avg <= 1000ms baseline (speed = 100)
            runs: vec![],
            error: if failed_count > 0 {
                Some("Test assertion failed".into())
            } else {
                None
            },
        };

        // Clean AST (no Thread.sleep)
        let ast_clean = StaticAnalysisReport {
            file_path: "Solution.java".to_string(),
            has_wait_for_timeout: false,
            locator_quality_score: 100.0,
            ..Default::default()
        };

        let card_clean = evaluate_feedback(
            &response,
            &ast_clean,
            "restassured-java",
            "1.0.0",
            85.0,
            1000,
        );
        let expected_raw_flake = (passed_count as f64 / 5.0) * 100.0;
        assert_eq!(card_clean.flakiness.score, expected_raw_flake);

        // A REST Assured drill has no locators, so Locator Quality does not
        // apply and its 15% is redistributed across the three dimensions that
        // do. Previously it paid out a flat 15 points for having nothing to
        // judge, which is why every API drill trended to 100.
        //   total = (0.35*correctness + 0.35*flakiness + 0.15*speed) / 0.85
        let expected_c = if passed_count == 5 {
            100.0
        } else {
            expected_raw_flake
        };
        let expected_total =
            ((0.35 * expected_c) + (0.35 * expected_raw_flake) + 15.0) / 0.85;
        assert!((card_clean.total_score - expected_total).abs() < 0.001);

        if passed_count == 5 {
            assert!(card_clean.passed, "5/5 clean runs must pass");
            assert_eq!(card_clean.total_score, 100.0);
        } else {
            assert!(!card_clean.passed, "{}/5 runs must fail", passed_count);
        }

        // Flaky AST (with Thread.sleep) -> flakiness capped at 40.0
        let ast_with_sleep = StaticAnalysisReport {
            file_path: "Exercise.java".to_string(),
            has_wait_for_timeout: true,
            locator_quality_score: 100.0,
            ..Default::default()
        };

        let card_with_sleep = evaluate_feedback(
            &response,
            &ast_with_sleep,
            "restassured-java",
            "1.0.0",
            85.0,
            1000,
        );
        let expected_capped_flake = expected_raw_flake.min(40.0);
        assert_eq!(card_with_sleep.flakiness.score, expected_capped_flake);
        assert!(
            !card_with_sleep.passed,
            "Code with Thread.sleep must never pass scorecard"
        );
    }
}

// =========================================================================
// 4. AST THREAD.SLEEP DETECTION ON VARIOUS SYNTAX STYLES
// =========================================================================

#[test]
fn test_ast_thread_sleep_syntax_styles() {
    let variants = vec![
        // Standard Thread.sleep
        ("Thread.sleep(100);", true, Some(100)),
        ("Thread.sleep(0);", true, Some(0)),
        ("Thread.sleep(5000);", true, Some(5000)),
        // Fully qualified java.lang.Thread.sleep
        ("java.lang.Thread.sleep(250);", true, Some(250)),
        ("java.lang.Thread.sleep(1000);", true, Some(1000)),
        // TimeUnit sleep styles
        ("TimeUnit.SECONDS.sleep(5);", true, Some(5)),
        ("TimeUnit.MILLISECONDS.sleep(500);", true, Some(500)),
        ("TimeUnit.MINUTES.sleep(1);", true, Some(1)),
        ("TimeUnit.NANOSECONDS.sleep(100000);", true, Some(100000)),
        ("TimeUnit.HOURS.sleep(2);", true, Some(2)),
        // Fully qualified package before TimeUnit
        (
            "java.util.concurrent.TimeUnit.SECONDS.sleep(3);",
            true,
            Some(3),
        ),
        // Unusual spacing around dots and parentheses
        ("Thread . sleep ( 300 );", true, Some(300)),
        ("java.lang.Thread . sleep ( 450 );", true, Some(450)),
        ("TimeUnit . SECONDS . sleep ( 10 );", true, Some(10)),
        // Variable or expression sleep argument (duration_ms is None)
        ("Thread.sleep(TIMEOUT_MS);", true, None),
        ("TimeUnit.SECONDS.sleep(delay);", true, None),
        // Commented out sleeps (MUST NOT trigger)
        ("// Thread.sleep(100);", false, None),
        ("// java.lang.Thread.sleep(200);", false, None),
        ("// TimeUnit.SECONDS.sleep(5);", false, None),
        ("/* Thread.sleep(100); */", false, None),
        ("/* \n * Thread.sleep(500);\n */", false, None),
    ];

    for (code, should_flag, expected_duration) in variants {
        let report = analyze_source(code, "Test.java");
        assert_eq!(
            report.has_wait_for_timeout, should_flag,
            "Failed AST detection for '{}': expected has_wait_for_timeout={}, got {}",
            code, should_flag, report.has_wait_for_timeout
        );

        if should_flag {
            let ap = report
                .anti_patterns
                .iter()
                .find(|a| matches!(a.kind, AntiPatternKind::ThreadSleep { .. }));
            assert!(
                ap.is_some(),
                "Expected ThreadSleep anti-pattern for '{}'",
                code
            );
            if let Some(AntiPatternKind::ThreadSleep { duration_ms }) = ap.map(|a| &a.kind) {
                assert_eq!(
                    *duration_ms, expected_duration,
                    "Mismatched duration for '{}': expected {:?}, got {:?}",
                    code, expected_duration, duration_ms
                );
            }
        } else {
            assert_eq!(
                report.anti_patterns.len(),
                0,
                "Expected 0 anti-patterns for non-triggering code: '{}'",
                code
            );
        }
    }
}

// =========================================================================
// 5. ALL 3 DRILLS EXERCISE VS SOLUTION COMPARISON
// =========================================================================

#[test]
fn test_all_3_drills_ast_and_scorecard_evaluations() {
    let drills = [
        (
            "Drill 01 Idempotency",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Solution.java",
            false, // Exercise 01 does not use Thread.sleep, but fails on 409
        ),
        (
            "Drill 02 JWT Auth",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth/Solution.java",
            false, // Exercise 02 does not use Thread.sleep, but fails on 401
        ),
        (
            "Drill 03 Kafka Lag",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java",
            true, // Exercise 03 HAS Thread.sleep(100)
        ),
        (
            "Drill 04 Pagination",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill04_pagination_boundary/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill04_pagination_boundary/Solution.java",
            false,
        ),
        (
            "Drill 05 JSON Schema",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill05_json_schema_validation/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill05_json_schema_validation/Solution.java",
            false,
        ),
        (
            "Drill 06 GraphQL",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill06_graphql_assertions/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill06_graphql_assertions/Solution.java",
            false,
        ),
        (
            "Drill 07 RequestSpecBuilder",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill07_request_spec_reuse/Exercise.java",
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill07_request_spec_reuse/Solution.java",
            false,
        ),
    ];

    for (name, ex_path, sol_path, ex_has_sleep) in drills {
        if !Path::new(ex_path).exists() || !Path::new(sol_path).exists() {
            continue;
        }

        let ex_ast = feedback::analyze_file(ex_path).expect("Analyze exercise");
        let sol_ast = feedback::analyze_file(sol_path).expect("Analyze solution");

        assert_eq!(
            ex_ast.has_wait_for_timeout, ex_has_sleep,
            "{}: Exercise has_wait_for_timeout expectation failed",
            name
        );
        assert!(
            !sol_ast.has_wait_for_timeout,
            "{}: Solution must never have has_wait_for_timeout",
            name
        );

        // Simulated Exercise failure response (e.g. 0/5 passed under chaos)
        let ex_resp = DrillResponse {
            id: format!("{}-ex", name),
            ok: true,
            passed: false,
            iterations: 5,
            passed_iterations: 0,
            failed_iterations: 5,
            total_duration_ms: 3000,
            runs: vec![],
            error: Some("Failure under chaos".into()),
        };
        let ex_card = evaluate_feedback(&ex_resp, &ex_ast, "restassured-java", "1.0.0", 85.0, 1000);
        assert!(!ex_card.passed, "{}: Exercise must fail evaluation", name);

        // Simulated Solution success response (5/5 passed under chaos)
        let sol_resp = DrillResponse {
            id: format!("{}-sol", name),
            ok: true,
            passed: true,
            iterations: 5,
            passed_iterations: 5,
            failed_iterations: 0,
            total_duration_ms: 2000, // 400ms avg
            runs: vec![],
            error: None,
        };
        let sol_card =
            evaluate_feedback(&sol_resp, &sol_ast, "restassured-java", "1.0.0", 85.0, 1000);
        assert!(sol_card.passed, "{}: Solution must pass evaluation", name);
        assert_eq!(
            sol_card.total_score, 100.0,
            "{}: Solution total score must be 100.0",
            name
        );
    }
}
