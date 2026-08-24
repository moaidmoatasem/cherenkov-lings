//! Layer 4 / Layer 7 Programmable Chaos Proxy for Cherenkov-Lings
//!
//! Provides reverse proxy capabilities with programmable fault injection:
//! - **Layer 4**: Abrupt raw TCP connection drops (socket closed immediately without HTTP response)
//! - **Layer 7**: Synthetic 502 Bad Gateway and 504 Gateway Timeout JSON responses
//! - **Layer 7**: Latency delays with configurable jitter (asynchronous sleep via `tokio::time::sleep`)
//! - **Passthrough**: Forwarding application-level chaos directives (`token_expire`, `kafka_lag`, `idempotency_conflict`) to upstream
//! - **Lifecycle**: Background supervisor with clean oneshot shutdown channels

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep, timeout};

/// Thread-safe, high-speed XorShift64 pseudo-random number generator.
pub struct FastRng {
    state: AtomicU64,
}

impl Default for FastRng {
    fn default() -> Self {
        Self::new()
    }
}

impl FastRng {
    /// Creates a new `FastRng` seeded from the current high-resolution system timestamp.
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x853c49e6748fea9b)
            ^ 0x9e3779b97f4a7c15;
        Self {
            state: AtomicU64::new(if seed == 0 { 0x123456789abcdef } else { seed }),
        }
    }

    /// Generates a pseudo-random `u64`.
    pub fn next_u64(&self) -> u64 {
        let mut current = self.state.load(Ordering::Relaxed);
        loop {
            let mut x = current;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let next = if x == 0 { 0x543210987654321 } else { x };
            match self.state.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return next,
                Err(actual) => current = actual,
            }
        }
    }

    /// Generates a pseudo-random floating point number in `[0.0, 1.0)`.
    pub fn next_f64(&self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Generates a random `i64` in the inclusive range `[low, high]`.
    pub fn gen_range_i64(&self, low: i64, high: i64) -> i64 {
        if low >= high {
            return low;
        }
        let span = (high - low + 1) as u64;
        low + (self.next_u64() % span) as i64
    }
}

/// Configuration settings for the Chaos Proxy server.
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Address and port the proxy binds and listens on (e.g. `127.0.0.1:8086`).
    pub listen_addr: SocketAddr,
    /// Upstream target server address (e.g. `127.0.0.1:8081`).
    pub upstream_addr: SocketAddr,
    /// Default baseline artificial latency in milliseconds.
    pub default_latency_ms: u64,
    /// Default latency variance / jitter in milliseconds (± jitter).
    pub default_jitter_ms: u64,
    /// Default probability of dropping TCP connections (0.0 = never, 1.0 = always).
    pub default_drop_rate: f64,
    /// Default probability of injecting synthetic 502/504 gateway errors.
    pub default_fault_rate: f64,
    /// Upstream connection timeout in milliseconds.
    pub upstream_timeout_ms: u64,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            listen_addr: "127.0.0.1:8086".parse().expect("valid listen addr"),
            upstream_addr: "127.0.0.1:8081".parse().expect("valid upstream addr"),
            default_latency_ms: 0,
            default_jitter_ms: 0,
            default_drop_rate: 0.0,
            default_fault_rate: 0.0,
            upstream_timeout_ms: 5000,
        }
    }
}

impl ProxyConfig {
    /// Creates a new `ProxyConfig` with specified listen and upstream socket addresses.
    pub fn new(listen_addr: SocketAddr, upstream_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            upstream_addr,
            ..Default::default()
        }
    }

    /// Sets default latency and jitter in milliseconds.
    pub fn with_latency(mut self, latency_ms: u64, jitter_ms: u64) -> Self {
        self.default_latency_ms = latency_ms;
        self.default_jitter_ms = jitter_ms;
        self
    }

    /// Sets default TCP connection drop probability.
    pub fn with_drop_rate(mut self, drop_rate: f64) -> Self {
        self.default_drop_rate = drop_rate.clamp(0.0, 1.0);
        self
    }

    /// Sets default synthetic fault probability.
    pub fn with_fault_rate(mut self, fault_rate: f64) -> Self {
        self.default_fault_rate = fault_rate.clamp(0.0, 1.0);
        self
    }
}

/// Parsed chaos directives extracted from HTTP request headers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChaosDirectives {
    /// Layer 4 raw TCP connection drop flag.
    pub drop: bool,
    /// Layer 7 synthetic HTTP status code (e.g. 502 or 504).
    pub fault_status: Option<u16>,
    /// Layer 7 artificial latency in milliseconds.
    pub delay_ms: Option<u64>,
    /// Layer 7 latency jitter in milliseconds.
    pub jitter_ms: Option<u64>,
    /// Application-level chaos directives to pass through upstream (e.g. `token_expire=immediate`).
    pub passthrough_directives: Vec<String>,
}

/// Parses duration strings such as `"200ms"`, `"1.5s"`, `"1s"`, or `"200"` into milliseconds.
pub fn parse_duration_ms(s: &str) -> Option<u64> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return None;
    }
    if let Some(num_str) = s.strip_suffix("ms") {
        num_str
            .trim()
            .parse::<f64>()
            .ok()
            .map(|v| v.max(0.0) as u64)
    } else if let Some(num_str) = s.strip_suffix('s') {
        num_str
            .trim()
            .parse::<f64>()
            .ok()
            .map(|v| (v.max(0.0) * 1000.0) as u64)
    } else {
        s.parse::<f64>().ok().map(|v| v.max(0.0) as u64)
    }
}

impl ChaosDirectives {
    /// Parses chaos directives from a list of header (name, value) tuples.
    pub fn parse_from_headers(headers: &[(String, String)]) -> Self {
        let mut directives = ChaosDirectives::default();

        for (name, val) in headers {
            let lower_name = name.to_lowercase();
            let val_trimmed = val.trim();

            match lower_name.as_str() {
                "x-chaos" => {
                    let parsed = Self::parse_x_chaos_value(val_trimmed);
                    if parsed.drop {
                        directives.drop = true;
                    }
                    if parsed.fault_status.is_some() {
                        directives.fault_status = parsed.fault_status;
                    }
                    if parsed.delay_ms.is_some() {
                        directives.delay_ms = parsed.delay_ms;
                    }
                    if parsed.jitter_ms.is_some() {
                        directives.jitter_ms = parsed.jitter_ms;
                    }
                    directives
                        .passthrough_directives
                        .extend(parsed.passthrough_directives);
                }
                "x-chaos-fault" => {
                    if val_trimmed.eq_ignore_ascii_case("drop") {
                        directives.drop = true;
                    } else if let Ok(code) = val_trimmed.parse::<u16>() {
                        directives.fault_status = Some(code);
                    }
                }
                "x-chaos-drop" => {
                    if val_trimmed.eq_ignore_ascii_case("true")
                        || val_trimmed == "1"
                        || val_trimmed.eq_ignore_ascii_case("yes")
                    {
                        directives.drop = true;
                    }
                }
                "x-chaos-status" => {
                    if let Ok(code) = val_trimmed.parse::<u16>() {
                        directives.fault_status = Some(code);
                    }
                }
                "x-chaos-delay" => {
                    if let Some(ms) = parse_duration_ms(val_trimmed) {
                        directives.delay_ms = Some(ms);
                    }
                }
                "x-chaos-jitter" => {
                    if let Some(ms) = parse_duration_ms(val_trimmed) {
                        directives.jitter_ms = Some(ms);
                    }
                }
                _ => {}
            }
        }

        directives
    }

    /// Parses compound `X-Chaos` header value (e.g. `delay=200ms;jitter=75ms;token_expire=immediate`).
    pub fn parse_x_chaos_value(val: &str) -> Self {
        let mut directives = ChaosDirectives::default();
        if val.is_empty() {
            return directives;
        }

        let tokens = val.split([';', ',']);
        for raw_token in tokens {
            let token = raw_token.trim();
            if token.is_empty() {
                continue;
            }

            if let Some((k, v)) = token.split_once('=') {
                let k_lower = k.trim().to_lowercase();
                let v_trimmed = v.trim();

                match k_lower.as_str() {
                    "drop" => {
                        if v_trimmed.eq_ignore_ascii_case("true")
                            || v_trimmed == "1"
                            || v_trimmed.eq_ignore_ascii_case("yes")
                        {
                            directives.drop = true;
                        }
                    }
                    "fault" => {
                        if v_trimmed.eq_ignore_ascii_case("drop") {
                            directives.drop = true;
                        } else if let Ok(code) = v_trimmed.parse::<u16>() {
                            directives.fault_status = Some(code);
                        }
                    }
                    "status" => {
                        if let Ok(code) = v_trimmed.parse::<u16>() {
                            directives.fault_status = Some(code);
                        }
                    }
                    "delay" => {
                        directives.delay_ms = parse_duration_ms(v_trimmed);
                    }
                    "jitter" => {
                        directives.jitter_ms = parse_duration_ms(v_trimmed);
                    }
                    _ => {
                        directives.passthrough_directives.push(token.to_string());
                    }
                }
            } else {
                let token_lower = token.to_lowercase();
                match token_lower.as_str() {
                    "drop" => directives.drop = true,
                    "502" => directives.fault_status = Some(502),
                    "504" => directives.fault_status = Some(504),
                    _ => directives.passthrough_directives.push(token.to_string()),
                }
            }
        }

        directives
    }
}

/// The Chaos Proxy server engine.
pub struct ProxyServer {
    config: Arc<ProxyConfig>,
    rng: Arc<FastRng>,
}

impl ProxyServer {
    /// Creates a new `ProxyServer` instance.
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            config: Arc::new(config),
            rng: Arc::new(FastRng::new()),
        }
    }

    /// Runs the proxy server loop until a message is received on `shutdown_rx`.
    pub async fn run(
        &self,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(self.config.listen_addr).await?;
        let config = Arc::clone(&self.config);
        let rng = Arc::clone(&self.rng);

        loop {
            tokio::select! {
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((client_stream, peer_addr)) => {
                            let cfg = Arc::clone(&config);
                            let r = Arc::clone(&rng);
                            tokio::spawn(async move {
                                if let Err(_err) = handle_connection(client_stream, peer_addr, cfg, r).await {
                                    // Connection closed or dropped
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Proxy accept error: {}", e);
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }

        Ok(())
    }

    /// Spawns the proxy server as a background Tokio task with a shutdown channel.
    pub async fn spawn_background(
        config: ProxyConfig,
    ) -> Result<(JoinHandle<()>, oneshot::Sender<()>), Box<dyn std::error::Error + Send + Sync>>
    {
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let server = Self::new(config);

        let handle = tokio::spawn(async move {
            let _ = server.run(shutdown_rx).await;
        });

        Ok((handle, shutdown_tx))
    }
}

/// Handles an incoming client TCP connection.
async fn handle_connection(
    mut client_stream: TcpStream,
    _peer_addr: SocketAddr,
    config: Arc<ProxyConfig>,
    rng: Arc<FastRng>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Read incoming request headers (up to 64KB)
    let mut header_buf = Vec::with_capacity(4096);
    let mut temp_buf = [0u8; 2048];
    let mut header_end_idx = None;

    while header_buf.len() < 65536 {
        let n = client_stream.read(&mut temp_buf).await?;
        if n == 0 {
            // Client closed stream prematurely
            return Ok(());
        }
        header_buf.extend_from_slice(&temp_buf[..n]);

        // Look for \r\n\r\n or \n\n
        if let Some(pos) = find_header_end(&header_buf) {
            header_end_idx = Some(pos);
            break;
        }
    }

    let end_idx = match header_end_idx {
        Some(idx) => idx,
        None => {
            // Headers too large or malformed
            return Ok(());
        }
    };

    // 2. Parse HTTP headers
    let header_str = String::from_utf8_lossy(&header_buf[..end_idx]);
    let parsed_headers = extract_headers(&header_str);
    let directives = ChaosDirectives::parse_from_headers(&parsed_headers);

    // 3. Layer 4 Fault Injection: Raw TCP drop
    let should_drop = directives.drop
        || (config.default_drop_rate > 0.0 && rng.next_f64() < config.default_drop_rate);

    if should_drop {
        // Abruptly close TCP socket without writing HTTP response
        let _ = client_stream.shutdown().await;
        drop(client_stream);
        return Ok(());
    }

    // 4. Layer 7 Latency Jitter
    let delay_ms = directives.delay_ms.unwrap_or(config.default_latency_ms);
    let jitter_ms = directives.jitter_ms.unwrap_or(config.default_jitter_ms);

    if delay_ms > 0 || jitter_ms > 0 {
        let jitter_offset = if jitter_ms > 0 {
            rng.gen_range_i64(-(jitter_ms as i64), jitter_ms as i64)
        } else {
            0
        };
        let actual_delay = (delay_ms as i64 + jitter_offset).max(0) as u64;
        if actual_delay > 0 {
            sleep(Duration::from_millis(actual_delay)).await;
        }
    }

    // 5. Layer 7 Fault Injection: Synthetic 502 / 504 Gateway Error
    let mut fault_status = directives.fault_status;
    if fault_status.is_none()
        && config.default_fault_rate > 0.0
        && rng.next_f64() < config.default_fault_rate
    {
        fault_status = Some(502);
    }

    if let Some(status) = fault_status {
        let (status_line, body) = match status {
            504 => (
                "504 Gateway Timeout",
                r#"{"error":"Gateway Timeout (Chaos Injected)","status":504,"chaos":true}"#,
            ),
            _ => (
                "502 Bad Gateway",
                r#"{"error":"Bad Gateway (Chaos Injected)","status":502,"chaos":true}"#,
            ),
        };
        let resp = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nX-Chaos-Injected: true\r\n\r\n{}",
            status_line,
            body.len(),
            body
        );
        let _ = client_stream.write_all(resp.as_bytes()).await;
        let _ = client_stream.flush().await;
        return Ok(());
    }

    // 6. Upstream Forwarding
    let connect_timeout = Duration::from_millis(config.upstream_timeout_ms.max(1000));
    let upstream_res = timeout(connect_timeout, TcpStream::connect(config.upstream_addr)).await;

    let mut upstream_stream = match upstream_res {
        Ok(Ok(stream)) => stream,
        _ => {
            // Upstream unreachable or connection timed out -> return synthetic 502
            let body = r#"{"error":"Bad Gateway (Chaos Injected - Upstream Unreachable)","status":502,"chaos":true}"#;
            let resp = format!(
                "HTTP/1.1 502 Bad Gateway\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nX-Chaos-Injected: true\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = client_stream.write_all(resp.as_bytes()).await;
            let _ = client_stream.flush().await;
            return Ok(());
        }
    };

    // Forward the initial bytes read from the client (headers + any initial body bytes)
    upstream_stream.write_all(&header_buf).await?;
    upstream_stream.flush().await?;

    // Bidirectional streaming of remaining request/response data
    let _ = tokio::io::copy_bidirectional(&mut client_stream, &mut upstream_stream).await;

    Ok(())
}

/// Finds the end of the HTTP header block (`\r\n\r\n` or `\n\n`) and returns the index past it.
fn find_header_end(buf: &[u8]) -> Option<usize> {
    for i in 0..buf.len() {
        if i + 4 <= buf.len() && &buf[i..i + 4] == b"\r\n\r\n" {
            return Some(i + 4);
        }
        if i + 2 <= buf.len() && &buf[i..i + 2] == b"\n\n" {
            return Some(i + 2);
        }
    }
    None
}

/// Extracts header key-value pairs from the HTTP header section.
fn extract_headers(header_str: &str) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    let mut lines = header_str.lines();

    // Skip the request line (e.g. GET /path HTTP/1.1)
    let _ = lines.next();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration_ms() {
        assert_eq!(parse_duration_ms("200ms"), Some(200));
        assert_eq!(parse_duration_ms("500 MS"), Some(500));
        assert_eq!(parse_duration_ms("1.5s"), Some(1500));
        assert_eq!(parse_duration_ms("2s"), Some(2000));
        assert_eq!(parse_duration_ms("350"), Some(350));
        assert_eq!(parse_duration_ms("0"), Some(0));
        assert_eq!(parse_duration_ms(""), None);
        assert_eq!(parse_duration_ms("abc"), None);
    }

    #[test]
    fn test_parse_chaos_directives_drop_variations() {
        let headers = vec![("X-Chaos".to_string(), "drop=true".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert!(d.drop);

        let headers = vec![("X-Chaos".to_string(), "drop=1".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert!(d.drop);

        let headers = vec![("X-Chaos".to_string(), "fault=drop".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert!(d.drop);

        let headers = vec![("X-Chaos-Drop".to_string(), "true".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert!(d.drop);

        let headers = vec![("X-Chaos-Fault".to_string(), "drop".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert!(d.drop);
    }

    #[test]
    fn test_parse_chaos_directives_synthetic_faults() {
        let headers = vec![("X-Chaos".to_string(), "fault=502".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert_eq!(d.fault_status, Some(502));

        let headers = vec![("X-Chaos".to_string(), "fault=504".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert_eq!(d.fault_status, Some(504));

        let headers = vec![("X-Chaos-Fault".to_string(), "502".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert_eq!(d.fault_status, Some(502));

        let headers = vec![("X-Chaos-Status".to_string(), "504".to_string())];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert_eq!(d.fault_status, Some(504));
    }

    #[test]
    fn test_parse_chaos_directives_latency_and_jitter() {
        let headers = vec![(
            "X-Chaos".to_string(),
            "delay=200ms;jitter=75ms;token_expire=immediate;kafka_lag=1500ms".to_string(),
        )];
        let d = ChaosDirectives::parse_from_headers(&headers);
        assert_eq!(d.delay_ms, Some(200));
        assert_eq!(d.jitter_ms, Some(75));
        assert_eq!(
            d.passthrough_directives,
            vec!["token_expire=immediate", "kafka_lag=1500ms"]
        );
    }

    #[test]
    fn test_proxy_config_builder() {
        let addr1: SocketAddr = "127.0.0.1:8086".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8081".parse().unwrap();
        let cfg = ProxyConfig::new(addr1, addr2)
            .with_latency(150, 50)
            .with_drop_rate(0.25)
            .with_fault_rate(0.1);

        assert_eq!(cfg.listen_addr, addr1);
        assert_eq!(cfg.upstream_addr, addr2);
        assert_eq!(cfg.default_latency_ms, 150);
        assert_eq!(cfg.default_jitter_ms, 50);
        assert_eq!(cfg.default_drop_rate, 0.25);
        assert_eq!(cfg.default_fault_rate, 0.1);
    }

    #[test]
    fn test_fast_rng_distribution() {
        let rng = FastRng::new();
        for _ in 0..100 {
            let f = rng.next_f64();
            assert!((0.0..1.0).contains(&f));
            let r = rng.gen_range_i64(-50, 50);
            assert!((-50..=50).contains(&r));
        }
    }
}
