use cherenkov_lings::proxy::{ChaosDirectives, ProxyConfig, ProxyServer, parse_duration_ms};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// How long any single socket read in these tests may block.
///
/// libtest has no per-test timeout, so an async test waiting on a socket that
/// never answers waits forever: the job burns its whole allowance and reports
/// nothing at all. A ceiling turns "the proxy went quiet" into a failure with a
/// message attached.
const IO_TIMEOUT: Duration = Duration::from_secs(10);

/// `AsyncReadExt::read` with [`IO_TIMEOUT`] over it. Read errors still surface as
/// `Err` so call sites keep the handling they already had; only a block that
/// never resolves is turned into a panic.
async fn read_bounded(stream: &mut TcpStream, buf: &mut [u8]) -> std::io::Result<usize> {
    match tokio::time::timeout(IO_TIMEOUT, stream.read(buf)).await {
        Ok(result) => result,
        Err(_) => panic!("read blocked for {IO_TIMEOUT:?}: the peer never answered"),
    }
}

/// Helper: starts a mock HTTP echo/response server on an OS-assigned ephemeral port.
async fn spawn_echo_upstream() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind echo upstream");
    let local_addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = vec![0u8; 8192];
                if let Ok(n) = stream.read(&mut buf).await
                    && n > 0
                {
                    let body = r#"{"status":"echo_ok","received":true}"#;
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                }
            });
        }
    });

    local_addr
}

/// Helper: starts an upstream that abruptly closes connection at different phases.
async fn spawn_flaky_terminating_upstream() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind flaky upstream");
    let local_addr = listener.local_addr().expect("local addr");
    let counter = Arc::new(AtomicUsize::new(0));

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let c = counter.fetch_add(1, Ordering::Relaxed);
            tokio::spawn(async move {
                let mut buf = vec![0u8; 1024];
                let _ = stream.read(&mut buf).await;

                match c % 3 {
                    0 => {
                        // Abrupt drop: close immediately without writing anything
                        let _ = stream.shutdown().await;
                        drop(stream);
                    }
                    1 => {
                        // Partial header then drop
                        let partial = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n";
                        let _ = stream.write_all(partial.as_bytes()).await;
                        let _ = stream.flush().await;
                        let _ = stream.shutdown().await;
                        drop(stream);
                    }
                    _ => {
                        // Header with promised 500 bytes body, but send only 10 bytes then drop
                        let truncated = "HTTP/1.1 200 OK\r\nContent-Length: 500\r\n\r\n0123456789";
                        let _ = stream.write_all(truncated.as_bytes()).await;
                        let _ = stream.flush().await;
                        let _ = stream.shutdown().await;
                        drop(stream);
                    }
                }
            });
        }
    });

    local_addr
}

// =========================================================================
// 1. HIGH CONCURRENCY / RAPID CONNECTION BURST STRESS TESTS
// =========================================================================

#[tokio::test]
async fn test_stress_high_concurrency_burst_100_requests() {
    let upstream_addr = spawn_echo_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let total_clients = 100;
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut tasks = Vec::with_capacity(total_clients);

    let start_time = Instant::now();

    for i in 0..total_clients {
        let counter = Arc::clone(&success_count);
        let task = tokio::spawn(async move {
            let mut client = match TcpStream::connect(proxy_addr).await {
                Ok(c) => c,
                Err(e) => panic!("Client {} failed to connect: {}", i, e),
            };

            let req = format!(
                "GET /api/stress/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
                i,
                proxy_addr.port()
            );

            client.write_all(req.as_bytes()).await.unwrap();
            client.flush().await.unwrap();

            let mut resp_buf = vec![0u8; 2048];
            let n = read_bounded(&mut client, &mut resp_buf).await.unwrap_or(0);
            if n > 0 {
                let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
                if resp_str.contains("200 OK") && resp_str.contains("echo_ok") {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            }
        });
        tasks.push(task);
    }

    for t in tasks {
        t.await.expect("task join failed");
    }

    let elapsed = start_time.elapsed();
    let total_succeeded = success_count.load(Ordering::Relaxed);

    assert_eq!(
        total_succeeded, total_clients,
        "All {} concurrent requests must complete successfully, got {}",
        total_clients, total_succeeded
    );
    assert!(
        elapsed < Duration::from_secs(5),
        "100 burst connections took too long: {:?}",
        elapsed
    );

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_stress_concurrent_mixed_workload() {
    let upstream_addr = spawn_echo_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let total_requests = 120;
    let normal_count = Arc::new(AtomicUsize::new(0));
    let drop_count = Arc::new(AtomicUsize::new(0));
    let fault_502_count = Arc::new(AtomicUsize::new(0));
    let fault_504_count = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::new();

    for i in 0..total_requests {
        let normal = Arc::clone(&normal_count);
        let dropped = Arc::clone(&drop_count);
        let f502 = Arc::clone(&fault_502_count);
        let f504 = Arc::clone(&fault_504_count);

        let task = tokio::spawn(async move {
            let mut client = TcpStream::connect(proxy_addr).await.expect("connect");

            let chaos_header = match i % 4 {
                0 => "",                       // Normal
                1 => "X-Chaos: drop=true\r\n", // L4 Drop
                2 => "X-Chaos: fault=502\r\n", // Synthetic 502
                _ => "X-Chaos: fault=504\r\n", // Synthetic 504
            };

            let req = format!(
                "GET /mixed/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n{}Connection: close\r\n\r\n",
                i,
                proxy_addr.port(),
                chaos_header
            );

            client.write_all(req.as_bytes()).await.unwrap();
            client.flush().await.unwrap();

            let mut resp_buf = vec![0u8; 2048];
            let n = read_bounded(&mut client, &mut resp_buf).await.unwrap_or(0);

            match i % 4 {
                0 => {
                    if n > 0 && String::from_utf8_lossy(&resp_buf[..n]).contains("200 OK") {
                        normal.fetch_add(1, Ordering::Relaxed);
                    }
                }
                1 => {
                    if n == 0 {
                        dropped.fetch_add(1, Ordering::Relaxed);
                    }
                }
                2 => {
                    if n > 0 && String::from_utf8_lossy(&resp_buf[..n]).contains("502 Bad Gateway")
                    {
                        f502.fetch_add(1, Ordering::Relaxed);
                    }
                }
                _ => {
                    if n > 0
                        && String::from_utf8_lossy(&resp_buf[..n]).contains("504 Gateway Timeout")
                    {
                        f504.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        });
        tasks.push(task);
    }

    for t in tasks {
        t.await.expect("task join failed");
    }

    assert_eq!(normal_count.load(Ordering::Relaxed), 30);
    assert_eq!(drop_count.load(Ordering::Relaxed), 30);
    assert_eq!(fault_502_count.load(Ordering::Relaxed), 30);
    assert_eq!(fault_504_count.load(Ordering::Relaxed), 30);

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

// =========================================================================
// 2. ADVERSARIAL & MALFORMED X-CHAOS HEADERS TESTS
// =========================================================================

#[test]
fn test_parse_duration_ms_adversarial_inputs() {
    // Extreme negative numbers -> should clamp to 0 or None
    assert_eq!(parse_duration_ms("-100ms"), Some(0));
    assert_eq!(parse_duration_ms("-500s"), Some(0));
    assert_eq!(parse_duration_ms("-999999999"), Some(0));

    // Floating point string values
    assert_eq!(parse_duration_ms("0.5s"), Some(500));
    assert_eq!(parse_duration_ms("0.001s"), Some(1));
    assert_eq!(parse_duration_ms("150.75ms"), Some(150));

    // Garbage / Special chars
    assert_eq!(parse_duration_ms("???"), None);
    assert_eq!(parse_duration_ms("ms"), None);
    assert_eq!(parse_duration_ms("s"), None);
    assert_eq!(parse_duration_ms("   "), None);
    assert_eq!(parse_duration_ms("\t\r\n"), None);
    assert_eq!(parse_duration_ms("delay=50ms"), None);
    assert_eq!(parse_duration_ms("NaN"), Some(0));
}

#[test]
fn test_chaos_directives_parser_malformed_and_adversarial() {
    // 1. Empty header
    let d1 = ChaosDirectives::parse_x_chaos_value("");
    assert_eq!(d1, ChaosDirectives::default());

    // 2. Garbage text with punctuation and symbols
    let d2 = ChaosDirectives::parse_x_chaos_value("!@#$%^&*()_+={}:\"<>?|`~");
    assert!(!d2.drop);
    assert_eq!(d2.fault_status, None);

    // 3. Repeated empty tokens and semicolons
    let d3 = ChaosDirectives::parse_x_chaos_value(";;;,;,;;;,;");
    assert_eq!(d3, ChaosDirectives::default());

    // 4. Invalid status code string (non-numeric)
    let d4 = ChaosDirectives::parse_x_chaos_value("fault=banana;status=corrupted");
    assert_eq!(d4.fault_status, None);

    // 5. Huge number overflow test
    let d5 = ChaosDirectives::parse_x_chaos_value(
        "delay=999999999999999999999999999999999999ms;fault=999999",
    );
    assert_eq!(d5.fault_status, None); // status > u16::MAX fails parse::<u16>()

    // 6. Conflicting drop tokens in same value
    let d6 = ChaosDirectives::parse_x_chaos_value("drop=false;drop=true;fault=502");
    assert!(d6.drop);
    assert_eq!(d6.fault_status, Some(502));

    // 7. Case insensitivity and whitespace fuzzing
    let d7 =
        ChaosDirectives::parse_x_chaos_value("  dRoP = TrUe ;  FaUlT =  504  ;  DeLaY = 50Ms ");
    assert!(d7.drop);
    assert_eq!(d7.fault_status, Some(504));
    assert_eq!(d7.delay_ms, Some(50));

    // 8. Thousand-token stress string
    let mut large_header = String::with_capacity(100_000);
    for i in 0..1000 {
        large_header.push_str(&format!("passthrough_key_{}=val_{};", i, i));
    }
    large_header.push_str("drop=true;fault=502");
    let d8 = ChaosDirectives::parse_x_chaos_value(&large_header);
    assert!(d8.drop);
    assert_eq!(d8.fault_status, Some(502));
    assert_eq!(d8.passthrough_directives.len(), 1000);
}

#[tokio::test]
async fn test_proxy_handles_malformed_header_requests_without_panic() {
    let upstream_addr = spawn_echo_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let malformed_headers = vec![
        "X-Chaos: \r\n",
        "X-Chaos: ;;;,;,;;\r\n",
        "X-Chaos: fault=NaN;delay=-500ms;jitter=-100s\r\n",
        "X-Chaos: fault=999999999999999999999\r\n",
        "X-Chaos: drop=invalid_boolean_string\r\n",
        "X-Chaos: !@#$%^&*()_+\r\n",
        "X-Chaos-Fault: non_numeric_code\r\n",
        "X-Chaos-Drop: not_a_boolean\r\n",
        "X-Chaos-Delay: abcdef\r\n",
        "X-Chaos-Jitter: @#$%\r\n",
    ];

    for (idx, header) in malformed_headers.iter().enumerate() {
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let req = format!(
            "GET /malformed/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n{}Connection: close\r\n\r\n",
            idx,
            proxy_addr.port(),
            header
        );

        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut resp_buf = vec![0u8; 2048];
        let n = read_bounded(&mut client, &mut resp_buf).await.unwrap_or(0);
        assert!(
            n > 0,
            "Proxy must not crash on malformed header: {}",
            header
        );
        let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
        assert!(
            resp_str.contains("200 OK"),
            "Expected fallback to normal forward on malformed header"
        );
    }

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

// =========================================================================
// 3. UPSTREAM SUDDEN TERMINATION & CONNECTION DROP HANDLING
// =========================================================================

#[tokio::test]
async fn test_proxy_survives_upstream_sudden_termination() {
    let flaky_upstream_addr = spawn_flaky_terminating_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, flaky_upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Send 30 requests to the flaky terminating upstream
    for i in 0..30 {
        let mut client = TcpStream::connect(proxy_addr)
            .await
            .expect("connect to proxy");
        let req = format!(
            "GET /flaky/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            i,
            proxy_addr.port()
        );

        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut resp_buf = vec![0u8; 1024];
        let _ = read_bounded(&mut client, &mut resp_buf).await;
        // The connection terminates abruptly from upstream, proxy should cleanly handle it without panicking
    }

    // Now verify the proxy is still healthy by redirecting a healthy upstream or checking response
    let mut check_client = TcpStream::connect(proxy_addr)
        .await
        .expect("proxy must remain responsive");
    let req = format!(
        "GET /check HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        proxy_addr.port()
    );
    check_client.write_all(req.as_bytes()).await.unwrap();
    check_client.flush().await.unwrap();

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_handles_client_premature_disconnect() {
    let upstream_addr = spawn_echo_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    for _ in 0..20 {
        // Connect, send 5 bytes (incomplete request), then immediately drop client stream
        let mut client = TcpStream::connect(proxy_addr).await.expect("connect");
        let _ = client.write_all(b"GET /").await;
        let _ = client.shutdown().await;
        drop(client);
    }

    // Proxy must still accept valid connections
    let mut healthy_client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect after drops");
    let req = format!(
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        proxy_addr.port()
    );
    healthy_client.write_all(req.as_bytes()).await.unwrap();
    healthy_client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 1024];
    let n = read_bounded(&mut healthy_client, &mut resp_buf)
        .await
        .unwrap();
    assert!(n > 0);
    assert!(String::from_utf8_lossy(&resp_buf[..n]).contains("200 OK"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

// =========================================================================
// 4. RAPID START / STOP SUPERVISOR LIFECYCLE CYCLES
// =========================================================================

#[tokio::test]
async fn test_rapid_proxy_start_stop_supervisor_cycles() {
    let upstream_addr = spawn_echo_upstream().await;

    // Ephemeral port selection
    let probe_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let target_port = probe_listener.local_addr().unwrap().port();
    drop(probe_listener);

    let proxy_addr: SocketAddr = format!("127.0.0.1:{}", target_port).parse().unwrap();

    let cycles = 25;
    for i in 0..cycles {
        let config = ProxyConfig::new(proxy_addr, upstream_addr);
        let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
            .await
            .unwrap_or_else(|_| panic!("spawn proxy iteration {}", i));

        // Connect and send a request
        let mut client = TcpStream::connect(proxy_addr)
            .await
            .expect("connect to proxy cycle");
        let req = format!(
            "GET /cycle/{} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            i, target_port
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut resp_buf = vec![0u8; 1024];
        let n = read_bounded(&mut client, &mut resp_buf).await.unwrap();
        assert!(n > 0, "Iteration {} must receive response", i);

        // Immediate shutdown
        shutdown_tx.send(()).expect("send shutdown");
        handle.await.expect("join handle");

        // Small yield to let OS TCP socket state clean up if needed
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

#[tokio::test]
async fn test_proxy_streaming_large_payload_5mb() {
    // Upstream server that echoes full body length
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind upstream");
    let upstream_addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                // Read request header first
                let mut header_buf = Vec::new();
                let mut byte = [0u8; 1];
                while stream.read_exact(&mut byte).await.is_ok() {
                    header_buf.push(byte[0]);
                    if header_buf.ends_with(b"\r\n\r\n") {
                        break;
                    }
                }

                // Now read the 5MB body
                let expected_bytes = 5 * 1024 * 1024;
                let mut body_read = 0;
                let mut buf = vec![0u8; 64 * 1024];
                while body_read < expected_bytes {
                    match stream.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => body_read += n,
                        Err(_) => break,
                    }
                }

                let resp_body = format!(r#"{{"received_bytes":{}}}"#, body_read);
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    resp_body.len(),
                    resp_body
                );
                let _ = stream.write_all(resp.as_bytes()).await;
                let _ = stream.flush().await;
            });
        }
    });

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr).await.expect("connect");

    let payload_size = 5 * 1024 * 1024; // 5MB
    let req_header = format!(
        "POST /upload HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\n\r\n",
        proxy_addr.port(),
        payload_size
    );

    client.write_all(req_header.as_bytes()).await.unwrap();

    // Stream 5MB payload in 64KB chunks
    let chunk = vec![0x42u8; 64 * 1024];
    let mut sent = 0;
    while sent < payload_size {
        let to_send = (payload_size - sent).min(chunk.len());
        client.write_all(&chunk[..to_send]).await.unwrap();
        sent += to_send;
    }
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = read_bounded(&mut client, &mut resp_buf).await.unwrap();
    assert!(n > 0);
    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(resp_str.contains("200 OK"));
    assert!(resp_str.contains(&format!(r#""received_bytes":{}"#, payload_size)));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_drops_oversized_headers_exceeding_64kb() {
    let upstream_addr = spawn_echo_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr).await.expect("connect");

    // Send 70KB of headers without \r\n\r\n
    let junk_header = vec![b'A'; 70 * 1024];
    let _ = client.write_all(&junk_header).await;
    let _ = client.flush().await;

    // Proxy must cut off stream and close connection without hanging
    let mut resp_buf = vec![0u8; 1024];
    let n = read_bounded(&mut client, &mut resp_buf).await.unwrap_or(0);
    assert_eq!(
        n, 0,
        "Oversized headers > 64KB should result in closed connection"
    );

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}
