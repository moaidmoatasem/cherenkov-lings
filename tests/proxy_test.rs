use cherenkov_lings::proxy::{ProxyConfig, ProxyServer};
use std::net::SocketAddr;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// Helper: starts a mock HTTP server on an OS-assigned ephemeral port (`127.0.0.1:0`).
/// Returns the bound `SocketAddr` and a receiver for inspected incoming request strings.
async fn spawn_mock_upstream() -> (SocketAddr, tokio::sync::mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock upstream");
    let local_addr = listener.local_addr().expect("local addr");
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = stream.read(&mut buf).await
                    && n > 0
                {
                    let req_str = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = tx.send(req_str).await;

                    let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 27\r\nConnection: close\r\n\r\n{\"status\":\"mock_ok\",\"id\":1}";
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                }
            });
        }
    });

    (local_addr, rx)
}

/// Helper: starts a mock HTTP server specifically returning 201 Created for POSTs.
async fn spawn_mock_checkout_upstream() -> (SocketAddr, tokio::sync::mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind mock checkout upstream");
    let local_addr = listener.local_addr().expect("local addr");
    let (tx, rx) = tokio::sync::mpsc::channel(100);

    tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                if let Ok(n) = stream.read(&mut buf).await
                    && n > 0
                {
                    let req_str = String::from_utf8_lossy(&buf[..n]).to_string();
                    let _ = tx.send(req_str).await;

                    let body = r#"{"order_id":"ord_1001","status":"confirmed"}"#;
                    let resp = format!(
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    let _ = stream.write_all(resp.as_bytes()).await;
                    let _ = stream.flush().await;
                }
            });
        }
    });

    (local_addr, rx)
}

#[tokio::test]
async fn test_proxy_transparent_routing_and_passthrough() {
    let (upstream_addr, mut req_rx) = spawn_mock_checkout_upstream().await;

    // Pick an ephemeral port for the proxy
    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Connect to proxy
    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");

    let post_body = r#"{"item":"fusion_cell","qty":5}"#;
    let req = format!(
        "POST /checkout HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Client-Trace: cl-999\r\n\r\n{}",
        proxy_addr.port(),
        post_body.len(),
        post_body
    );

    client
        .write_all(req.as_bytes())
        .await
        .expect("send request");
    client.flush().await.expect("flush request");

    // Read response from proxy
    let mut resp_buf = vec![0u8; 4096];
    let n = client.read(&mut resp_buf).await.expect("read response");
    assert!(n > 0, "Expected response from proxy");

    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(
        resp_str.contains("201 Created"),
        "Response should be 201 Created, got: {}",
        resp_str
    );
    assert!(resp_str.contains(r#"{"order_id":"ord_1001","status":"confirmed"}"#));

    // Verify upstream received the exact headers and body
    let received_req = req_rx
        .recv()
        .await
        .expect("upstream should receive request");
    assert!(received_req.starts_with("POST /checkout HTTP/1.1"));
    assert!(received_req.contains("X-Client-Trace: cl-999"));
    assert!(received_req.contains(post_body));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_l4_tcp_drop_on_x_chaos_header() {
    let (upstream_addr, _req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Case 1: X-Chaos: drop=true
    {
        let mut client = TcpStream::connect(proxy_addr)
            .await
            .expect("connect to proxy");
        let req = format!(
            "GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: drop=true\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut resp_buf = vec![0u8; 1024];
        let n = client.read(&mut resp_buf).await.unwrap_or(0);
        assert_eq!(
            n, 0,
            "L4 Drop should close TCP stream with 0 bytes returned"
        );
    }

    // Case 2: X-Chaos-Drop: true
    {
        let mut client = TcpStream::connect(proxy_addr)
            .await
            .expect("connect to proxy");
        let req = format!(
            "GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos-Drop: true\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut resp_buf = vec![0u8; 1024];
        let n = client.read(&mut resp_buf).await.unwrap_or(0);
        assert_eq!(
            n, 0,
            "L4 Drop should close TCP stream with 0 bytes returned"
        );
    }

    // Case 3: X-Chaos-Fault: drop
    {
        let mut client = TcpStream::connect(proxy_addr)
            .await
            .expect("connect to proxy");
        let req = format!(
            "GET /api/status HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos-Fault: drop\r\n\r\n",
            proxy_addr.port()
        );
        client.write_all(req.as_bytes()).await.unwrap();
        client.flush().await.unwrap();

        let mut resp_buf = vec![0u8; 1024];
        let n = client.read(&mut resp_buf).await.unwrap_or(0);
        assert_eq!(
            n, 0,
            "L4 Drop should close TCP stream with 0 bytes returned"
        );
    }

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_l4_tcp_drop_on_configured_drop_rate() {
    let (upstream_addr, _req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    // 100% drop rate
    let config = ProxyConfig::new(proxy_addr, upstream_addr).with_drop_rate(1.0);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "GET /api/balance HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 1024];
    let n = client.read(&mut resp_buf).await.unwrap_or(0);
    assert_eq!(
        n, 0,
        "Configured 100% drop rate should immediately close client connection"
    );

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_l7_synthetic_502_bad_gateway() {
    let (upstream_addr, _req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Send request with X-Chaos: fault=502
    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "GET /api/test HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: fault=502\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = client.read(&mut resp_buf).await.expect("read 502 response");
    assert!(n > 0);

    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(
        resp_str.contains("502 Bad Gateway"),
        "Expected 502 Bad Gateway, got: {}",
        resp_str
    );
    assert!(resp_str.contains("Content-Type: application/json"));
    assert!(resp_str.contains(r#""status":502"#) || resp_str.contains(r#""status": 502"#));
    assert!(resp_str.contains("Bad Gateway (Chaos Injected)"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_l7_synthetic_504_gateway_timeout() {
    let (upstream_addr, _req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Send request with X-Chaos-Fault: 504
    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "GET /api/long-poll HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos-Fault: 504\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = client.read(&mut resp_buf).await.expect("read 504 response");
    assert!(n > 0);

    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(
        resp_str.contains("504 Gateway Timeout"),
        "Expected 504 Gateway Timeout, got: {}",
        resp_str
    );
    assert!(resp_str.contains("Content-Type: application/json"));
    assert!(resp_str.contains(r#""status":504"#) || resp_str.contains(r#""status": 504"#));
    assert!(resp_str.contains("Gateway Timeout (Chaos Injected)"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_upstream_unreachable_returns_synthetic_502() {
    // Pick an unused port for upstream
    let unused_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead_upstream_addr = unused_listener.local_addr().unwrap();
    drop(unused_listener); // release so connection will be refused

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
        "GET /api/data HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = client.read(&mut resp_buf).await.expect("read response");
    assert!(n > 0);

    let resp_str = String::from_utf8_lossy(&resp_buf[..n]);
    assert!(
        resp_str.contains("502 Bad Gateway"),
        "Expected 502 Bad Gateway for unreachable upstream, got: {}",
        resp_str
    );
    assert!(resp_str.contains("Upstream Unreachable"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_l7_latency_delay_and_jitter() {
    let (upstream_addr, _req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "GET /api/slow HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: delay=120ms;jitter=10ms\r\n\r\n",
        proxy_addr.port()
    );

    let start = Instant::now();
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let n = client.read(&mut resp_buf).await.expect("read response");
    let elapsed = start.elapsed();

    assert!(n > 0);
    assert!(
        elapsed.as_millis() >= 100,
        "Expected elapsed time >= 100ms (120ms - 10ms jitter), got {}ms",
        elapsed.as_millis()
    );

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_application_chaos_header_passthrough() {
    let (upstream_addr, mut req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect to proxy");
    let req = format!(
        "POST /auth/refresh HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nX-Chaos: token_expire=immediate;kafka_lag=1500ms;idempotency_conflict=true\r\n\r\n",
        proxy_addr.port()
    );

    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 2048];
    let _ = client.read(&mut resp_buf).await.unwrap();

    let upstream_req = req_rx.recv().await.expect("upstream received request");
    assert!(
        upstream_req.contains("token_expire=immediate"),
        "X-Chaos header should pass through application directives"
    );
    assert!(upstream_req.contains("kafka_lag=1500ms"));
    assert!(upstream_req.contains("idempotency_conflict=true"));

    let _ = shutdown_tx.send(());
    let _ = handle.await;
}

#[tokio::test]
async fn test_proxy_lifecycle_shutdown_cleans_port() {
    let (upstream_addr, _req_rx) = spawn_mock_upstream().await;

    let proxy_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let proxy_addr = proxy_listener.local_addr().unwrap();
    drop(proxy_listener);

    let config = ProxyConfig::new(proxy_addr, upstream_addr);
    let (handle, shutdown_tx) = ProxyServer::spawn_background(config)
        .await
        .expect("spawn proxy");

    // Check that proxy is accepting connections
    let mut client = TcpStream::connect(proxy_addr)
        .await
        .expect("connect while running");
    let req = format!(
        "GET /ping HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        proxy_addr.port()
    );
    client.write_all(req.as_bytes()).await.unwrap();
    client.flush().await.unwrap();

    let mut resp_buf = vec![0u8; 512];
    let n = client.read(&mut resp_buf).await.unwrap();
    assert!(n > 0);

    // Shut down proxy
    shutdown_tx.send(()).expect("send shutdown signal");
    handle.await.expect("join proxy background task");

    // Verify port is freed / connection rejected
    let connect_res = TcpStream::connect(proxy_addr).await;
    assert!(
        connect_res.is_err(),
        "Proxy port should be closed after shutdown"
    );
}
