use cherenkov_lings::feedback::{self, StaticAnalysisReport};
use cherenkov_lings::runner::{
    AnyRunner, DrillResponse, JMeterRunner, RunResult, parse_jmeter_jtl_csv,
};
use std::sync::Arc;

#[test]
fn test_jmeter_runner_initialization_and_options() {
    let runner = JMeterRunner::new();
    assert_eq!(runner.jmeter_cmd(), "jmeter");

    let default_runner = JMeterRunner::default();
    assert_eq!(default_runner.jmeter_cmd(), "jmeter");

    let custom = JMeterRunner::with_jmeter_cmd("custom-jmeter-binary");
    assert_eq!(custom.jmeter_cmd(), "custom-jmeter-binary");
}

#[test]
fn test_any_runner_jmeter_wrapping() {
    let runner = JMeterRunner::new();
    let any_runner = AnyRunner::Jmeter(Arc::new(runner));

    match any_runner {
        AnyRunner::Jmeter(r) => {
            assert_eq!(r.jmeter_cmd(), "jmeter");
        }
        _ => panic!("Expected AnyRunner::Jmeter"),
    }
}

#[test]
fn test_parse_jmeter_jtl_csv_success_metrics() {
    let csv_data = r#"timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage,bytes,sentBytes,grpThreads,allThreads,URL,Latency,IdleTime,Connect
1700000000000,25,GET /api/v1/health,200,OK,Thread Group 1-1,text,true,,128,50,1,1,http://localhost:8081/api/v1/health,20,0,2
1700000000100,50,GET /api/v1/products,200,OK,Thread Group 1-1,text,true,,2048,50,1,1,http://localhost:8081/api/v1/products,45,0,5
1700000000200,75,POST /api/v1/orders,201,Created,Thread Group 1-1,text,true,,512,200,1,1,http://localhost:8081/api/v1/orders,70,0,5
1700000000300,150,GET /api/v1/orders/1,200,OK,Thread Group 1-1,text,true,,400,50,1,1,http://localhost:8081/api/v1/orders/1,140,0,10
"#;
    let metrics = parse_jmeter_jtl_csv(csv_data).expect("Parse JTL CSV success");
    assert_eq!(metrics.total_samples, 4);
    assert_eq!(metrics.passed_samples, 4);
    assert_eq!(metrics.failed_samples, 0);
    assert_eq!(metrics.error_rate, 0.0);
    assert_eq!(metrics.min_elapsed_ms, 25);
    assert_eq!(metrics.max_elapsed_ms, 150);
    assert_eq!(metrics.avg_elapsed_ms, 75); // (25+50+75+150)/4 = 75
    assert_eq!(metrics.p90_elapsed_ms, 150);
    assert_eq!(metrics.p95_elapsed_ms, 150);
    assert_eq!(metrics.p99_elapsed_ms, 150);
    assert!(metrics.first_failure_reason.is_none());
    assert_eq!(metrics.samples.len(), 4);
    assert_eq!(metrics.samples[0].label, "GET /api/v1/health");
    assert_eq!(metrics.samples[0].response_code, "200");
}

#[test]
fn test_parse_jmeter_jtl_csv_error_rate_and_reasons() {
    let csv_data = r#"timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage
1700000000000,45,GET /api/v1/catalog,200,OK,Thread Group 1-1,text,true,
1700000000100,600,POST /api/v1/checkout,500,Internal Server Error,Thread Group 1-1,text,false,Assertion failed: expected HTTP 200
1700000000200,35,GET /api/v1/cart,200,OK,Thread Group 1-1,text,true,
1700000000300,750,POST /api/v1/payment,503,Service Unavailable,Thread Group 1-1,text,false,Gateway timeout occurred
1700000000400,20,GET /api/v1/user,200,OK,Thread Group 1-1,text,true,
"#;
    let metrics = parse_jmeter_jtl_csv(csv_data).expect("Parse JTL CSV errors");
    assert_eq!(metrics.total_samples, 5);
    assert_eq!(metrics.passed_samples, 3);
    assert_eq!(metrics.failed_samples, 2);
    assert!((metrics.error_rate - 0.4).abs() < 0.001);
    assert_eq!(metrics.min_elapsed_ms, 20);
    assert_eq!(metrics.max_elapsed_ms, 750);
    assert_eq!(metrics.avg_elapsed_ms, 290); // (20+35+45+600+750)/5 = 290
    assert!(metrics.first_failure_reason.is_some());
    let reason = metrics.first_failure_reason.unwrap();
    assert!(reason.contains("POST /api/v1/checkout"));
    assert!(reason.contains("500"));
    assert!(reason.contains("Assertion failed: expected HTTP 200"));
}

#[test]
fn test_parse_jmeter_jtl_csv_percentile_precision() {
    let mut csv_data = String::from(
        "timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage\n",
    );
    for i in 1..=100 {
        csv_data.push_str(&format!(
            "1700000000000,{},Sample {},200,OK,Thread 1,text,true,\n",
            i, i
        ));
    }

    let metrics = parse_jmeter_jtl_csv(&csv_data).expect("Parse JTL CSV 100 samples");
    assert_eq!(metrics.total_samples, 100);
    assert_eq!(metrics.passed_samples, 100);
    assert_eq!(metrics.failed_samples, 0);
    assert_eq!(metrics.min_elapsed_ms, 1);
    assert_eq!(metrics.max_elapsed_ms, 100);
    assert_eq!(metrics.avg_elapsed_ms, 51);
    assert_eq!(metrics.p90_elapsed_ms, 90);
    assert_eq!(metrics.p95_elapsed_ms, 95);
    assert_eq!(metrics.p99_elapsed_ms, 99);
}

#[test]
fn test_parse_jmeter_jtl_csv_quoted_and_escaped_fields() {
    let csv_data = r#"timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage
1700000000000,18,"GET /api/v1/items?filter=a,b,c",200,"OK, processed successfully",Thread Group 1-1,text,true,""
1700000000100,320,"POST /api/v1/items,batch",422,"Unprocessable Entity",Thread Group 1-1,text,false,"Validation error, ""field"" is required"
"#;
    let metrics = parse_jmeter_jtl_csv(csv_data).expect("Parse JTL CSV quotes");
    assert_eq!(metrics.total_samples, 2);
    assert_eq!(metrics.passed_samples, 1);
    assert_eq!(metrics.failed_samples, 1);
    assert_eq!(metrics.samples[0].label, "GET /api/v1/items?filter=a,b,c");
    assert_eq!(
        metrics.samples[0].response_message,
        "OK, processed successfully"
    );
    assert_eq!(metrics.samples[1].label, "POST /api/v1/items,batch");
    assert!(
        metrics.samples[1]
            .failure_message
            .contains("Validation error, \"field\" is required")
    );
}

#[test]
fn test_jmeter_feedback_matrix_evaluation() {
    let mock_failed_response = DrillResponse {
        id: "jmeter-req-1".to_string(),
        ok: true,
        passed: false,
        iterations: 1,
        passed_iterations: 0,
        failed_iterations: 1,
        total_duration_ms: 3200,
        runs: vec![RunResult {
            iteration: 1,
            passed: false,
            duration_ms: 3200,
            error: Some("Sample 'POST /orders' failed (HTTP 500): Connection refused".to_string()),
        }],
        error: Some("Sample 'POST /orders' failed (HTTP 500): Connection refused".to_string()),
    };

    let ast = StaticAnalysisReport {
        file_path: "exercises/05_perf_jmeter/01_thread_group_concurrency/exercise.jmx".to_string(),
        locator_quality_score: 100.0,
        ..Default::default()
    };

    let scorecard = feedback::evaluate_feedback(
        &mock_failed_response,
        &ast,
        "Enterprise Performance Testing (JMeter)",
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
            .any(|d| d.contains("Connection refused"))
    );

    let mock_passed_response = DrillResponse {
        id: "jmeter-req-2".to_string(),
        ok: true,
        passed: true,
        iterations: 1,
        passed_iterations: 1,
        failed_iterations: 0,
        total_duration_ms: 450,
        runs: vec![RunResult {
            iteration: 1,
            passed: true,
            duration_ms: 450,
            error: None,
        }],
        error: None,
    };

    let scorecard_pass = feedback::evaluate_feedback(
        &mock_passed_response,
        &ast,
        "Enterprise Performance Testing (JMeter)",
        "1.0.0",
        85.0,
        1000,
    );

    assert!(scorecard_pass.passed);
    assert_eq!(scorecard_pass.correctness.score, 100.0);
    assert_eq!(scorecard_pass.total_score, 100.0);
}

#[tokio::test]
async fn test_jmeter_runner_missing_binary_graceful_handling() {
    let runner = JMeterRunner::with_jmeter_cmd("nonexistent-jmeter-bin-999999");
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("test_plan_dummy.jmx");
    std::fs::write(&test_file, "<jmeterTestPlan></jmeterTestPlan>").expect("Write dummy jmx");

    let response = runner
        .run_drill(test_file.to_str().unwrap(), "", 2, 5000)
        .await
        .expect("Missing binary should return Ok(DrillResponse) without panic");

    let _ = std::fs::remove_file(&test_file);

    assert!(!response.ok);
    assert!(!response.passed);
    assert_eq!(response.passed_iterations, 0);
    assert_eq!(response.failed_iterations, 2);
    assert!(response.error.is_some());
    let err = response.error.unwrap();
    assert!(err.contains("not found on PATH"));
    assert!(err.contains("Apache JMeter"));
}

#[tokio::test]
async fn test_jmeter_runner_nonexistent_file() {
    let runner = JMeterRunner::new();
    let response = runner
        .run_drill("non_existent_exercise_plan.jmx", "", 1, 5000)
        .await
        .expect("Nonexistent file should return Ok(DrillResponse)");

    assert!(!response.ok);
    assert!(!response.passed);
    assert!(response.error.is_some());
    assert!(response.error.unwrap().contains("does not exist"));
}
