use cherenkov_lings::feedback::{
    self, AntiPatternKind, analyze_file, analyze_source, evaluate_feedback,
};
use cherenkov_lings::proxy::{ChaosDirectives, ProxyConfig, ProxyServer, parse_duration_ms};
use cherenkov_lings::runner::{DrillResponse, JvmRunner, RunResult};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Helper: Spawns a mock HTTP server on an OS-assigned ephemeral port (`127.0.0.1:0`).
/// Returns the bound `SocketAddr` and an MPSC receiver for inspected raw request strings.
async fn spawn_mock_upstream_server() -> (SocketAddr, tokio::sync::mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock upstream");
    let local_addr = listener.local_addr().expect("get local addr");
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                if let Ok(n) = stream.read(&mut buf).await
                    && n > 0
                {
                    let req_str = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = tx.send(req_str.clone()).await;

                    let resp_body = if req_str.starts_with("POST /checkout") {
                        r#"{"status":"success","order_id":"ORD-9021"}"#
                    } else if req_str.starts_with("POST /auth/login") {
                        r#"{"access_token":"fresh_token_123","expires_in":3600}"#
                    } else if req_str.starts_with("GET /balance") {
                        r#"{"balance":750.0,"pending_count":0}"#
                    } else {
                        r#"{"status":"ok","message":"mock upstream ready"}"#
                    };

                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        resp_body.len(),
                        resp_body
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                }
            });
        }
    });

    (local_addr, rx)
}

// =========================================================================
// TIER 1: FEATURE COVERAGE TESTS
// =========================================================================

#[tokio::test]
async fn test_tier1_proxy_routing_default_ports() {
    let (upstream_addr, mut req_rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    assert_eq!(config.listen_addr, proxy_addr);
    assert_eq!(config.upstream_addr, upstream_addr);

    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = client.read(&mut resp_buf).await.expect("read response");
    assert!(n > 0);

    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(resp_str.contains("200 OK"));
    assert!(resp_str.contains("mock upstream ready"));

    let received_req = req_rx.recv().await.expect("upstream received request");
    assert!(received_req.starts_with("GET /health HTTP/1.1"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_tier1_proxy_l4_tcp_connection_drops() {
    let (upstream_addr, _rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Case 1: X-Chaos: drop=true
    {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /drop-test HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: drop=true\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).await.unwrap_or(0);
        assert_eq!(n, 0, "L4 Drop must close TCP stream with 0 bytes");
    }

    // Case 2: X-Chaos-Fault: drop
    {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /drop-fault HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos-Fault: drop\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).await.unwrap_or(0);
        assert_eq!(n, 0, "L4 Drop via X-Chaos-Fault must return 0 bytes");
    }

    // Case 3: X-Chaos-Drop: true
    {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /drop-header HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos-Drop: true\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 1024];
        let n = client.read(&mut buf).await.unwrap_or(0);
        assert_eq!(n, 0, "L4 Drop via X-Chaos-Drop must return 0 bytes");
    }

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_tier1_proxy_l7_fault_injection_and_jitter() {
    let (upstream_addr, _rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // 1. Synthetic 502 Bad Gateway
    {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /fail HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: fault=502\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 2048];
        let n = client.read(&mut buf).await.unwrap();
        assert!(n > 0);
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("502 Bad Gateway"));
        assert!(resp.contains("Bad Gateway (Chaos Injected)"));
    }

    // 2. Synthetic 504 Gateway Timeout
    {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /timeout HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: fault=504\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 2048];
        let n = client.read(&mut buf).await.unwrap();
        assert!(n > 0);
        let resp = String::from_utf8_lossy(&buf[..n]);
        assert!(resp.contains("504 Gateway Timeout"));
        assert!(resp.contains("Gateway Timeout (Chaos Injected)"));
    }

    // 3. Latency Delay + Jitter
    {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /slow HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: delay=100ms;jitter=10ms\r\n\r\n",
            proxy_addr.port()
        );
        let start = Instant::now();
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut buf = vec![0u8; 2048];
        let n = client.read(&mut buf).await.unwrap();
        let elapsed = start.elapsed();
        assert!(n > 0);
        assert!(
            elapsed.as_millis() >= 80,
            "Elapsed latency was {}ms, expected >= 80ms",
            elapsed.as_millis()
        );
    }

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[test]
fn test_tier1_pom_xml_java_track_structure() {
    let pom_path = Path::new("exercises/02_api_restassured_java/pom.xml");
    assert!(
        pom_path.exists(),
        "pom.xml must exist at {}",
        pom_path.display()
    );

    let pom_content = fs::read_to_string(pom_path).expect("read pom.xml");
    assert!(pom_content.contains("restassured-java-drills"));
    assert!(pom_content.contains("io.rest-assured"));
    assert!(pom_content.contains("junit-jupiter"));
    assert!(pom_content.contains("awaitility"));
    assert!(pom_content.contains("jackson-databind"));
    assert!(pom_content.contains("maven-surefire-plugin"));
    assert!(pom_content.contains("**/Exercise.java"));
    assert!(pom_content.contains("**/Solution.java"));
}

#[test]
fn test_tier1_drills_file_structure_and_hints_contracts() {
    let drill_dirs = [
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency",
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill02_jwt_auth",
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag",
    ];

    for dir in drill_dirs {
        let p = Path::new(dir);
        assert!(p.exists(), "Drill directory {} must exist", dir);

        let exercise = p.join("Exercise.java");
        let solution = p.join("Solution.java");
        let hints = p.join("hints.md");

        assert!(exercise.exists(), "Exercise.java must exist in {}", dir);
        assert!(solution.exists(), "Solution.java must exist in {}", dir);
        assert!(hints.exists(), "hints.md must exist in {}", dir);

        // Validate 3 progressive hints
        let hint_content = fs::read_to_string(hints).expect("read hints.md");
        assert!(
            hint_content.contains("Hint 1:"),
            "hints.md in {} must have Hint 1",
            dir
        );
        assert!(
            hint_content.contains("Hint 2:"),
            "hints.md in {} must have Hint 2",
            dir
        );
        assert!(
            hint_content.contains("Hint 3:"),
            "hints.md in {} must have Hint 3",
            dir
        );
    }
}

#[test]
fn test_tier1_jvm_runner_class_extraction() {
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
        // Unix style path
        let extracted = JvmRunner::extract_class_name(file_path);
        assert_eq!(extracted.as_deref(), Some(expected_class));

        // Windows style path
        let win_path = file_path.replace('/', "\\");
        let win_extracted = JvmRunner::extract_class_name(&win_path);
        assert_eq!(win_extracted.as_deref(), Some(expected_class));
    }
}

#[test]
fn test_tier1_cli_proxy_and_diagnose_integration() {
    let d1_ex = Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java",
    );
    let d3_ex = Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
    );

    if d1_ex.exists() {
        let rep1 = analyze_file(d1_ex).expect("analyze drill 1");
        assert!(!rep1.file_path.is_empty());
    }

    if d3_ex.exists() {
        let rep3 = analyze_file(d3_ex).expect("analyze drill 3");
        assert!(
            rep3.has_wait_for_timeout,
            "Drill 3 Exercise must detect Thread.sleep"
        );
        let rendered = feedback::render_diagnostic(&rep3, "REST Assured Java", "1.0.0");
        assert!(rendered.contains("CHERENKOV-LINGS DIAGNOSTIC"));
        assert!(rendered.contains("Thread.sleep"));
    }
}

// =========================================================================
// TIER 2: BOUNDARY & CORNER CASES
// =========================================================================

#[tokio::test]
async fn test_tier2_proxy_port_conflict_error_handling() {
    // Bind a listener first to occupy the port
    let bound_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let occupied_addr = bound_listener.local_addr().unwrap();

    let dummy_upstream: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    let config = ProxyConfig::new(occupied_addr, dummy_upstream);

    let server = ProxyServer::new(config);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // Running server on already-bound port must return an Err
    let result = server.run(shutdown_rx).await;
    assert!(
        result.is_err(),
        "ProxyServer::run should error on port conflict"
    );
}

#[test]
fn test_tier2_proxy_zero_delay_and_negative_duration_handling() {
    // Zero delay
    assert_eq!(parse_duration_ms("0ms"), Some(0));
    assert_eq!(parse_duration_ms("0s"), Some(0));
    assert_eq!(parse_duration_ms("0"), Some(0));

    // Negative values (sanitized to 0)
    assert_eq!(parse_duration_ms("-50ms"), Some(0));
    assert_eq!(parse_duration_ms("-1000s"), Some(0));

    // Sub-millisecond floating values
    assert_eq!(parse_duration_ms("0.1ms"), Some(0));
    assert_eq!(parse_duration_ms("0.5s"), Some(500));

    // Extreme large durations
    let directives = ChaosDirectives::parse_x_chaos_value("delay=9999999999ms;jitter=100000ms");
    assert_eq!(directives.delay_ms, Some(9999999999));
    assert_eq!(directives.jitter_ms, Some(100000));
}

#[tokio::test]
async fn test_tier2_proxy_unreachable_upstream_returns_502_bad_gateway() {
    // Find an unused ephemeral port and close it immediately
    let dead_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_upstream_addr = dead_listener.local_addr().unwrap();
    drop(dead_listener);

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, dead_upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "GET /test-unreachable HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut buf = vec![0u8; 2048];
    let n = client.read(&mut buf).await.expect("read 502 response");
    assert!(n > 0);

    let resp_str = String::from_utf8_lossy(&buf[..n]);
    assert!(resp_str.contains("502 Bad Gateway"));
    assert!(resp_str.contains("Upstream Unreachable"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_tier2_proxy_malformed_packets_and_non_http_data() {
    let (upstream_addr, _rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Send binary non-HTTP garbage data and shutdown write
    let garbage = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF, 0x42];
    let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
    let _ = client.write_all(&garbage).await;
    let _ = client.shutdown().await;

    // Stream should close or handle without crashing the server
    let mut buf = vec![0u8; 512];
    let _ = client.read(&mut buf).await;
    drop(client);

    // Verify proxy is still alive after receiving binary garbage
    let mut healthy_client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect healthy client");
    let req = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        proxy_addr.port()
    );
    healthy_client.write_all(req.as_bytes()).await.unwrap();
    healthy_client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 1024];
    let n = healthy_client.read(&mut resp_buf).await.unwrap();
    assert!(n > 0);
    assert!(String::from_utf8_lossy(&resp_buf[..n]).contains("200 OK"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[test]
fn test_tier2_feedback_ast_boundary_and_corner_cases() {
    // 1. Empty source
    let rep_empty = analyze_source("", "empty.java");
    assert_eq!(rep_empty.total_lines, 0);
    assert!(!rep_empty.has_wait_for_timeout);
    assert_eq!(rep_empty.anti_patterns.len(), 0);

    // 2. Multiline comment with false positive keywords
    let rep_comments = analyze_source(
        "/*\n * Thread.sleep(5000);\n * page.waitForTimeout(1000);\n */\npublic class Test {}",
        "comments.java",
    );
    assert!(!rep_comments.has_wait_for_timeout);
    assert_eq!(rep_comments.anti_patterns.len(), 0);

    // 3. String literal containing Thread.sleep keyword
    let rep_string = analyze_source(
        "String msg = \"Avoid using Thread.sleep in tests\";",
        "strings.java",
    );
    assert!(!rep_string.has_wait_for_timeout);
    assert_eq!(rep_string.anti_patterns.len(), 0);
}

// =========================================================================
// TIER 3: CROSS-FEATURE INTERACTIONS
// =========================================================================

#[tokio::test]
async fn test_tier3_proxy_micro_crucible_chaos_end_to_end() {
    let (upstream_addr, mut req_rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Scenario: POST /checkout with application chaos passthrough headers
    let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
    let post_body = r#"{"item_id":"drill-01","qty":1}"#;
    let req = format!(
        "POST /checkout HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nIdempotency-Key: key-999\r\nX-Chaos: idempotency_conflict=true;kafka_lag=1000ms\r\nConnection: close\r\n\r\n{}",
        proxy_addr.port(),
        post_body.len(),
        post_body
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = client.read(&mut resp_buf).await.unwrap();
    assert!(n > 0);

    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(resp_str.contains("200 OK") || resp_str.contains("success"));
    assert!(resp_str.contains("ORD-9021"));

    // Verify upstream received the exact application chaos headers
    let upstream_req = req_rx.recv().await.expect("upstream received");
    assert!(upstream_req.contains("Idempotency-Key: key-999"));
    assert!(upstream_req.contains("idempotency_conflict=true"));
    assert!(upstream_req.contains("kafka_lag=1000ms"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_tier3_watch_lifecycle_and_background_proxy_management() {
    let (upstream_addr, _rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Confirm proxy is operational
    let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
    let req = format!(
        "GET /ping HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut buf = vec![0u8; 1024];
    let n = client.read(&mut buf).await.unwrap();
    assert!(n > 0);

    // Trigger graceful shutdown
    shutdown_tx.send(()).expect("send shutdown signal");
    handle.await.expect("join proxy background task");

    // Confirm socket port is released
    let connect_res = TcpStream::connect(proxy_addr).await;
    assert!(connect_res.is_err(), "Port must be freed after shutdown");
}

#[test]
fn test_tier3_jvm_runner_surefire_to_feedback_matrix_integration() {
    let reports_dir = Path::new("exercises/02_api_restassured_java/target/surefire-reports");
    if !reports_dir.exists() {
        eprintln!("Skipping: Surefire reports not present");
        return;
    }

    let d1_sol = reports_dir.join("TEST-com.cherenkov.drill01_idempotency.Solution.xml");
    if d1_sol.exists() {
        let surefire_report =
            JvmRunner::parse_surefire_report(&d1_sol).expect("parse surefire report");
        let passed = surefire_report.failures == 0 && surefire_report.errors == 0;
        assert!(passed, "Drill 1 Solution Surefire report must pass");

        let drill_response = DrillResponse {
            id: "d1-sol".to_string(),
            ok: true,
            passed,
            iterations: surefire_report.tests,
            passed_iterations: surefire_report.tests
                - surefire_report.failures
                - surefire_report.errors,
            failed_iterations: surefire_report.failures + surefire_report.errors,
            total_duration_ms: (surefire_report.time_sec * 1000.0) as u64,
            runs: vec![
                RunResult {
                    iteration: 1,
                    passed: true,
                    duration_ms: 200,
                    error: None,
                },
                RunResult {
                    iteration: 2,
                    passed: true,
                    duration_ms: 200,
                    error: None,
                },
            ],
            error: None,
        };

        let d1_sol_source = Path::new(
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Solution.java",
        );
        let ast_report = if d1_sol_source.exists() {
            analyze_file(d1_sol_source).unwrap()
        } else {
            analyze_source("public class Solution {}", "Solution.java")
        };

        let scorecard = evaluate_feedback(
            &drill_response,
            &ast_report,
            "API Resilience & Security (REST Assured Java)",
            "1.0.0",
            85.0,
            1000,
        );

        assert!(scorecard.passed, "Drill 1 Solution must pass 4D scorecard");
        assert_eq!(scorecard.correctness.score, 100.0);
        assert_eq!(scorecard.flakiness.score, 100.0);
        assert!(scorecard.total_score >= 85.0);
    }
}

// =========================================================================
// TIER 4: REAL-WORLD SCENARIOS
// =========================================================================

#[test]
fn test_tier4_all_3_drills_exercise_failure_vs_solution_pass_surefire() {
    let reports_dir = Path::new("exercises/02_api_restassured_java/target/surefire-reports");
    if !reports_dir.exists() {
        eprintln!("Skipping: Surefire reports not present");
        return;
    }

    // Drill 1: Idempotency Collisions
    let d1_ex = reports_dir.join("TEST-com.cherenkov.drill01_idempotency.Exercise.xml");
    let d1_sol = reports_dir.join("TEST-com.cherenkov.drill01_idempotency.Solution.xml");
    if d1_ex.exists() && d1_sol.exists() {
        let rep_ex = JvmRunner::parse_surefire_report(&d1_ex).expect("parse d1 exercise");
        let rep_sol = JvmRunner::parse_surefire_report(&d1_sol).expect("parse d1 solution");
        assert!(
            rep_ex.failures > 0,
            "Drill 1 Exercise must fail on 409 Conflict"
        );
        assert_eq!(rep_sol.failures, 0, "Drill 1 Solution must pass");
    }

    // Drill 2: JWT Authentication Refresh
    let d2_ex = reports_dir.join("TEST-com.cherenkov.drill02_jwt_auth.Exercise.xml");
    let d2_sol = reports_dir.join("TEST-com.cherenkov.drill02_jwt_auth.Solution.xml");
    if d2_ex.exists() && d2_sol.exists() {
        let rep_ex = JvmRunner::parse_surefire_report(&d2_ex).expect("parse d2 exercise");
        let rep_sol = JvmRunner::parse_surefire_report(&d2_sol).expect("parse d2 solution");
        assert!(
            rep_ex.failures > 0,
            "Drill 2 Exercise must fail on 401 Unauthorized"
        );
        assert_eq!(rep_sol.failures, 0, "Drill 2 Solution must pass");
    }

    // Drill 3: Kafka Lag Assertions
    let d3_ex = reports_dir.join("TEST-com.cherenkov.drill03_kafka_lag.Exercise.xml");
    let d3_sol = reports_dir.join("TEST-com.cherenkov.drill03_kafka_lag.Solution.xml");
    if d3_ex.exists() && d3_sol.exists() {
        let rep_ex = JvmRunner::parse_surefire_report(&d3_ex).expect("parse d3 exercise");
        let rep_sol = JvmRunner::parse_surefire_report(&d3_sol).expect("parse d3 solution");
        assert!(
            rep_ex.failures > 0,
            "Drill 3 Exercise must fail on stale balance"
        );
        assert_eq!(rep_sol.failures, 0, "Drill 3 Solution must pass");
    }
}

#[test]
fn test_tier4_feedback_matrix_4d_scoring_and_ast_sleep_penalty() {
    let d3_ex = Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java",
    );
    let d3_sol = Path::new(
        "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java",
    );

    if !d3_ex.exists() || !d3_sol.exists() {
        eprintln!("Skipping: Drill 3 source files not present");
        return;
    }

    let ast_ex = analyze_file(d3_ex).expect("analyze drill 3 exercise");
    let ast_sol = analyze_file(d3_sol).expect("analyze drill 3 solution");

    assert!(
        ast_ex.has_wait_for_timeout,
        "Exercise must flag Thread.sleep"
    );
    assert_eq!(ast_ex.anti_patterns.len(), 1);
    assert_eq!(
        ast_ex.anti_patterns[0].kind,
        AntiPatternKind::ThreadSleep {
            duration_ms: Some(100)
        }
    );

    assert!(
        !ast_sol.has_wait_for_timeout,
        "Solution must NOT flag Thread.sleep"
    );
    assert_eq!(ast_sol.anti_patterns.len(), 0);

    // Mock 5-run passing test execution
    let mock_response = DrillResponse {
        id: "d3-eval".to_string(),
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

    // Exercise scorecard with Thread.sleep penalty cap
    let card_ex = evaluate_feedback(
        &mock_response,
        &ast_ex,
        "REST Assured Java",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(
        card_ex.flakiness.score, 40.0,
        "Flakiness score must be capped at 40.0 on Thread.sleep"
    );
    assert!(
        card_ex.total_score < 85.0,
        "Total score must be below 85.0 threshold"
    );
    assert!(
        !card_ex.passed,
        "Scorecard must fail on Thread.sleep anti-pattern"
    );
    assert!(
        card_ex
            .diagnostics
            .iter()
            .any(|d| d.contains("Thread.sleep(100ms)"))
    );

    // Solution scorecard
    let card_sol = evaluate_feedback(
        &mock_response,
        &ast_sol,
        "REST Assured Java",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card_sol.flakiness.score, 100.0);
    assert_eq!(card_sol.total_score, 100.0);
    assert!(card_sol.passed, "Solution must pass with 100.0 score");
}

#[tokio::test]
async fn test_tier4_high_volume_chaos_proxy_stress_scenario() {
    let (upstream_addr, _rx) = spawn_mock_upstream_server().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let total_requests = 60;
    let normal_count = Arc::new(AtomicUsize::new(0));
    let drop_count = Arc::new(AtomicUsize::new(0));
    let fault_count = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::with_capacity(total_requests);

    for i in 0..total_requests {
        let normal = Arc::clone(&normal_count);
        let dropped = Arc::clone(&drop_count);
        let faulted = Arc::clone(&fault_count);

        let task = tokio::spawn(async move {
            let mut client = TcpStream::connect(proxy_addr).await.expect("connect");

            let header = match i % 3 {
                0 => "",
                1 => "X-Chaos: drop=true\r\n",
                _ => "X-Chaos: fault=502\r\n",
            };

            let req = format!(
                "GET /stress/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n{}Connection: close\r\n\r\n",
                i,
                proxy_addr.port(),
                header
            );

            client.write_all(req.as_bytes()).await.unwrap();
            client.flush().await.unwrap();

            let mut buf = vec![0u8; 1024];
            let n = client.read(&mut buf).await.unwrap_or(0);

            if i % 3 == 0 {
                if n > 0 && String::from_utf8_lossy(&buf[..n]).contains("200 OK") {
                    normal.fetch_add(1, Ordering::Relaxed);
                }
            } else if i % 3 == 1 {
                if n == 0 {
                    dropped.fetch_add(1, Ordering::Relaxed);
                }
            } else {
                if n > 0 && String::from_utf8_lossy(&buf[..n]).contains("502 Bad Gateway") {
                    faulted.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        tasks.push(task);
    }

    for t in tasks {
        t.await.expect("task join failed");
    }

    assert_eq!(normal_count.load(Ordering::Relaxed), 20);
    assert_eq!(drop_count.load(Ordering::Relaxed), 20);
    assert_eq!(fault_count.load(Ordering::Relaxed), 20);

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

// =========================================================================
// TIER 1-4 EXTENSIONS: 5 POLYGLOT TRACKS & GENAI QA INTEGRATION
// =========================================================================

#[test]
fn test_tier1_genai_qa_track_artifacts_and_playwright_config() {
    let genai_drills = [
        "exercises/06_genai_qa/01_rag_context_faithfulness",
        "exercises/06_genai_qa/02_llm_assertion_flakiness",
    ];

    for dir in genai_drills {
        let p = Path::new(dir);
        assert!(p.exists(), "GenAI QA drill directory {} must exist", dir);

        let ex = p.join("exercise.ts");
        let sol = p.join("solution.ts");
        let hints = p.join("hints.md");

        assert!(ex.exists(), "exercise.ts must exist in {}", dir);
        assert!(sol.exists(), "solution.ts must exist in {}", dir);
        assert!(hints.exists(), "hints.md must exist in {}", dir);

        let hints_text = fs::read_to_string(&hints).expect("read hints.md");
        assert!(
            hints_text.contains("Hint 1"),
            "Missing Hint 1 in {}",
            hints.display()
        );
        assert!(
            hints_text.contains("Hint 2"),
            "Missing Hint 2 in {}",
            hints.display()
        );
        assert!(
            hints_text.contains("Hint 3"),
            "Missing Hint 3 in {}",
            hints.display()
        );
    }

    let pw_cfg = Path::new("playwright.config.ts");
    assert!(pw_cfg.exists(), "playwright.config.ts must exist");
    let pw_content = fs::read_to_string(pw_cfg).expect("read playwright config");
    assert!(
        pw_content.contains("exercises"),
        "playwright config must target exercises"
    );
}

#[test]
fn test_tier1_polyglot_5_tracks_directory_layout_and_contracts() {
    let toml_str = fs::read_to_string("lings.toml").expect("read lings.toml");
    let track_ids = [
        "playwright-ts",
        "restassured-java",
        "k6-js",
        "maestro-mobile",
        "genai-qa",
        "devsecops-python",
        "foundations",
        "jmeter",
        "tool-decisions",
        "contract-pact",
        "a11y-axe",
        "ci-pipeline",
    ];

    for id in track_ids {
        assert!(
            toml_str.contains(&format!("id = \"{}\"", id)),
            "lings.toml must define track '{}'",
            id
        );
    }

    // Every drill declared in the manifest must exist on disk with its hints
    // file. Derived from lings.toml rather than a hardcoded list: the literal
    // list this replaced had gone stale and knew nothing about the ci-pipeline
    // track, so it asserted "all drills exist" while checking only some of them.
    let cfg = cherenkov_lings::config::load_config("lings.toml").expect("lings.toml must parse");
    let all_drills: Vec<String> = cfg
        .tracks
        .iter()
        .flat_map(|t| t.drills.iter().map(move |d| t.drill_path(&d.id)))
        .collect();

    assert_eq!(
        all_drills.len(),
        cfg.tracks.iter().map(|t| t.drills.len()).sum::<usize>(),
        "drill paths must be one-to-one with manifest drills"
    );

    for drill in &all_drills {
        let p = Path::new(drill);
        assert!(p.exists(), "Drill directory {} must exist", drill);
        assert!(
            p.join("hints.md").exists(),
            "hints.md must exist in {}",
            drill
        );
    }
}

#[test]
fn test_tier2_polyglot_ast_anti_pattern_matrix_across_all_languages() {
    // 1. TypeScript anti-pattern
    let ts_bad = "await page.waitForTimeout(5000);";
    let rep_ts_bad = analyze_source(ts_bad, "test.ts");
    assert!(rep_ts_bad.has_wait_for_timeout);
    assert_eq!(rep_ts_bad.anti_patterns.len(), 1);

    // 2. Java anti-pattern
    let java_bad = "Thread.sleep(1000);";
    let rep_java_bad = analyze_source(java_bad, "Test.java");
    assert!(rep_java_bad.has_wait_for_timeout);
    assert_eq!(rep_java_bad.anti_patterns.len(), 1);

    // 3. YAML anti-patterns
    let yaml_bad = "- launchApp:\n    appId: com.app\n- tapOn:\n    text: Login with Biometric\n- assertVisible:\n    text: Welcome\n";
    let rep_yaml_bad = analyze_source(yaml_bad, "flow.yaml");
    assert!(rep_yaml_bad.has_wait_for_timeout);
    assert_eq!(rep_yaml_bad.anti_patterns.len(), 1);

    // 4. Clean GenAI QA TypeScript solution
    let ts_clean = "const facts = body.source_facts; expect(facts).toContain('cherenkov');";
    let rep_ts_clean = analyze_source(ts_clean, "solution.ts");
    assert!(!rep_ts_clean.has_wait_for_timeout);
    assert_eq!(rep_ts_clean.anti_patterns.len(), 0);
}

#[test]
fn test_tier3_genai_qa_and_polyglot_feedback_matrix_flow() {
    let tracks = [
        "Modern Web Automation (Playwright TypeScript)",
        "API Resilience & Security (REST Assured Java)",
        "High-Concurrency Load Testing (k6 JS)",
        "Mobile UI Automation (Maestro YAML)",
        "GenAI QA Testing (Playwright TypeScript)",
    ];

    let mock_response = DrillResponse {
        id: "eval-polyglot".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 500,
        runs: vec![
            RunResult {
                iteration: 1,
                passed: true,
                duration_ms: 100,
                error: None,
            },
            RunResult {
                iteration: 2,
                passed: true,
                duration_ms: 100,
                error: None,
            },
            RunResult {
                iteration: 3,
                passed: true,
                duration_ms: 100,
                error: None,
            },
            RunResult {
                iteration: 4,
                passed: true,
                duration_ms: 100,
                error: None,
            },
            RunResult {
                iteration: 5,
                passed: true,
                duration_ms: 100,
                error: None,
            },
        ],
        error: None,
    };

    let clean_ast = analyze_source("const x = 1;", "solution.ts");

    for track in tracks {
        let scorecard = evaluate_feedback(&mock_response, &clean_ast, track, "1.0.0", 85.0, 1000);
        assert!(scorecard.passed, "Clean run on track '{}' must pass", track);
        assert_eq!(scorecard.correctness.score, 100.0);
        assert_eq!(scorecard.flakiness.score, 100.0);
        assert!(scorecard.total_score >= 85.0);
    }
}

#[test]
fn test_tier4_all_5_tracks_exercise_anti_patterns_vs_solutions() {
    // Check GenAI QA drill 01
    let d1_ex = fs::read_to_string("exercises/06_genai_qa/01_rag_context_faithfulness/exercise.ts")
        .expect("read d1 exercise");
    let d1_sol =
        fs::read_to_string("exercises/06_genai_qa/01_rag_context_faithfulness/solution.ts")
            .expect("read d1 solution");
    assert!(d1_ex.contains("body.answer") && d1_ex.contains(".toBe("));
    assert!(d1_sol.contains("body.grounded") && d1_sol.contains("body.source_facts"));

    // Check GenAI QA drill 02
    let d2_ex = fs::read_to_string("exercises/06_genai_qa/02_llm_assertion_flakiness/exercise.ts")
        .expect("read d2 exercise");
    let d2_sol = fs::read_to_string("exercises/06_genai_qa/02_llm_assertion_flakiness/solution.ts")
        .expect("read d2 solution");
    assert!(d2_ex.contains("body.raw_text") && d2_ex.contains(".toBe("));
    assert!(d2_sol.contains("body.intent") && d2_sol.contains("body.confidence"));

    // Check Mobile drill 01
    let m1_ex = analyze_file("exercises/03_mobile_maestro/01_biometric_fallback/exercise.yaml")
        .expect("analyze m1 ex");
    let m1_sol = analyze_file("exercises/03_mobile_maestro/01_biometric_fallback/solution.yaml")
        .expect("analyze m1 sol");
    assert!(
        m1_ex.has_wait_for_timeout,
        "Mobile drill 01 exercise must flag anti-pattern"
    );
    assert!(
        !m1_sol.has_wait_for_timeout,
        "Mobile drill 01 solution must have 0 anti-patterns"
    );

    // Check Java drill 03
    let j3_ex = analyze_file("exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java")
        .expect("analyze j3 ex");
    let j3_sol = analyze_file("exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill03_kafka_lag/Solution.java")
        .expect("analyze j3 sol");
    assert!(
        j3_ex.has_wait_for_timeout,
        "Java drill 03 exercise must flag Thread.sleep"
    );
    assert!(
        !j3_sol.has_wait_for_timeout,
        "Java drill 03 solution must have 0 Thread.sleep"
    );
}
