use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::Mutex;
use tokio::time::timeout;

#[derive(Debug)]
pub enum RunnerError {
    Io(std::io::Error),
    Json(serde_json::Error),
    ProcessExited(String),
    WorkerError(String),
    Timeout(Duration),
    ProtocolError(String),
    ScriptNotFound(PathBuf),
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Json(e) => write!(f, "JSON serialization error: {}", e),
            Self::ProcessExited(msg) => write!(f, "Worker process exited: {}", msg),
            Self::WorkerError(msg) => write!(f, "Worker error: {}", msg),
            Self::Timeout(dur) => write!(f, "Runner operation timed out after {:?}", dur),
            Self::ProtocolError(msg) => write!(f, "IPC Protocol error: {}", msg),
            Self::ScriptNotFound(p) => write!(f, "Worker script not found at {:?}", p),
        }
    }
}

impl std::error::Error for RunnerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for RunnerError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for RunnerError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrillRequest {
    pub id: String,
    pub action: String,
    pub file: String,
    #[serde(default)]
    pub chaos: String,
    #[serde(default = "default_iterations")]
    pub iterations: u32,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

fn default_iterations() -> u32 {
    1
}

fn default_timeout_ms() -> u64 {
    30000
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunResult {
    pub iteration: u32,
    pub passed: bool,
    pub duration_ms: u64,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DrillResponse {
    pub id: String,
    pub ok: bool,
    #[serde(default)]
    pub passed: bool,
    #[serde(default)]
    pub iterations: u32,
    #[serde(default)]
    pub passed_iterations: u32,
    #[serde(default)]
    pub failed_iterations: u32,
    #[serde(default)]
    pub total_duration_ms: u64,
    #[serde(default)]
    pub runs: Vec<RunResult>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Generic IPC message wrapper for ping/pong and control messages
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ControlMessage {
    pub id: String,
    pub action: String,
    #[serde(default)]
    pub ok: Option<bool>,
}

/// NodeRunner coordinates the persistent background Node.js worker process
/// over Stdio Line-Delimited JSON (NDJSON) IPC.
pub struct NodeRunner {
    child: Mutex<Child>,
    stdin: Mutex<BufWriter<ChildStdin>>,
    lines: Mutex<Lines<BufReader<ChildStdout>>>,
    request_counter: AtomicU64,
    worker_script: PathBuf,
}

impl NodeRunner {
    /// Spawn the Node.js IPC worker process and perform a ping-pong handshake
    pub async fn start<P: AsRef<Path>>(worker_script: P) -> Result<Self, RunnerError> {
        let script_path = worker_script.as_ref().to_path_buf();
        if !script_path.exists() {
            return Err(RunnerError::ScriptNotFound(script_path));
        }

        let mut child = Command::new("node")
            .arg(&script_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| RunnerError::ProcessExited("Failed to capture worker stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| RunnerError::ProcessExited("Failed to capture worker stdout".into()))?;

        let mut stdin_writer = BufWriter::new(stdin);
        let mut stdout_lines = BufReader::new(stdout).lines();

        // Perform initial handshake ping
        let ping_msg = serde_json::json!({
            "id": "handshake-ping",
            "action": "ping"
        });
        let ping_line = format!("{}\n", ping_msg);
        stdin_writer.write_all(ping_line.as_bytes()).await?;
        stdin_writer.flush().await?;

        let handshake_timeout = Duration::from_secs(5);
        let ping_resp = timeout(handshake_timeout, stdout_lines.next_line())
            .await
            .map_err(|_| RunnerError::Timeout(handshake_timeout))?
            .map_err(RunnerError::Io)?
            .ok_or_else(|| {
                RunnerError::ProcessExited("Worker process terminated during handshake".into())
            })?;

        let parsed: ControlMessage = serde_json::from_str(&ping_resp)?;
        if parsed.action != "pong" && parsed.ok != Some(true) {
            return Err(RunnerError::ProtocolError(format!(
                "Unexpected handshake response: {}",
                ping_resp
            )));
        }

        Ok(Self {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin_writer),
            lines: Mutex::new(stdout_lines),
            request_counter: AtomicU64::new(1),
            worker_script: script_path,
        })
    }

    /// Dispatch a drill test execution request to the worker and wait for the structured JSON response
    pub async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        let req_id = format!(
            "req-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );
        let request = DrillRequest {
            id: req_id.clone(),
            action: "run_drill".to_string(),
            file: file.to_string(),
            chaos: chaos.to_string(),
            iterations,
            timeout_ms,
        };

        let req_json = serde_json::to_string(&request)?;
        let req_line = format!("{}\n", req_json);

        // Lock stdin and stdout sequentially to ensure atomic request/response pairs
        let mut stdin = self.stdin.lock().await;
        let mut lines = self.lines.lock().await;

        stdin.write_all(req_line.as_bytes()).await?;
        stdin.flush().await?;

        // Give a grace margin on top of requested timeout for IPC serialization overhead
        let total_timeout = Duration::from_millis(timeout_ms + 5000);
        let resp_line = timeout(total_timeout, lines.next_line())
            .await
            .map_err(|_| RunnerError::Timeout(total_timeout))?
            .map_err(RunnerError::Io)?
            .ok_or_else(|| {
                RunnerError::ProcessExited("Worker closed stdout stream unexpectedly".into())
            })?;

        let response: DrillResponse = serde_json::from_str(&resp_line).map_err(|e| {
            RunnerError::ProtocolError(format!("Failed to parse response '{}': {}", resp_line, e))
        })?;

        if response.id != req_id {
            return Err(RunnerError::ProtocolError(format!(
                "Response ID mismatch: expected {}, got {}",
                req_id, response.id
            )));
        }

        Ok(response)
    }

    /// Send a ping request to verify worker responsiveness
    pub async fn ping(&self) -> Result<bool, RunnerError> {
        let req_id = format!(
            "ping-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );
        let ping_msg = serde_json::json!({
            "id": req_id,
            "action": "ping"
        });
        let ping_line = format!("{}\n", ping_msg);

        let mut stdin = self.stdin.lock().await;
        let mut lines = self.lines.lock().await;

        stdin.write_all(ping_line.as_bytes()).await?;
        stdin.flush().await?;

        let ping_timeout = Duration::from_secs(3);
        let resp_line = timeout(ping_timeout, lines.next_line())
            .await
            .map_err(|_| RunnerError::Timeout(ping_timeout))?
            .map_err(RunnerError::Io)?
            .ok_or_else(|| {
                RunnerError::ProcessExited("Worker closed stdout stream during ping".into())
            })?;

        let parsed: ControlMessage = serde_json::from_str(&resp_line)?;
        Ok(parsed.action == "pong" || parsed.ok == Some(true))
    }

    /// Gracefully stop the worker process
    pub async fn stop(&mut self) -> Result<(), RunnerError> {
        let shutdown_msg = serde_json::json!({
            "id": "shutdown",
            "action": "shutdown"
        });
        let shutdown_line = format!("{}\n", shutdown_msg);

        let mut stdin = self.stdin.lock().await;
        let _ = stdin.write_all(shutdown_line.as_bytes()).await;
        let _ = stdin.flush().await;

        let mut child = self.child.lock().await;
        let _ = timeout(Duration::from_millis(500), child.wait()).await;
        let _ = child.kill().await;

        Ok(())
    }

    /// Path to the worker script
    pub fn worker_script_path(&self) -> &Path {
        &self.worker_script
    }
}

impl Drop for NodeRunner {
    fn drop(&mut self) {
        // Attempt to kill child process if still running when runner is dropped
        if let Ok(mut child) = self.child.try_lock() {
            let _ = child.start_kill();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurefireFailure {
    pub message: String,
    pub failure_type: String,
    pub stack_trace: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurefireTestCase {
    pub name: String,
    pub classname: String,
    pub time_sec: f64,
    pub failure: Option<SurefireFailure>,
    pub error: Option<SurefireFailure>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SurefireReport {
    pub name: String,
    pub time_sec: f64,
    pub tests: u32,
    pub errors: u32,
    pub skipped: u32,
    pub failures: u32,
    pub test_cases: Vec<SurefireTestCase>,
}

fn unescape_xml(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#10;", "\n")
        .replace("&#13;", "\r")
        .replace("&#9;", "\t")
        .replace("&amp;", "&")
}

fn extract_xml_attr(tag: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}\s*=\s*(?:"([^"]*)"|'([^']*)')"#, regex::escape(attr));
    let re = regex::Regex::new(&pattern).ok()?;
    let caps = re.captures(tag)?;
    caps.get(1)
        .or_else(|| caps.get(2))
        .map(|m| unescape_xml(m.as_str()))
}

pub fn parse_surefire_xml(xml: &str) -> Result<SurefireReport, RunnerError> {
    let re_suite = regex::Regex::new(r#"<testsuite\b([^>]*)>"#)
        .map_err(|e| RunnerError::WorkerError(e.to_string()))?;
    let suite_caps = re_suite
        .captures(xml)
        .ok_or_else(|| RunnerError::WorkerError("No <testsuite> tag found in XML".into()))?;
    let suite_attrs = suite_caps.get(1).map(|m| m.as_str()).unwrap_or("");

    let name = extract_xml_attr(suite_attrs, "name").unwrap_or_default();
    let time_sec: f64 = extract_xml_attr(suite_attrs, "time")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let tests: u32 = extract_xml_attr(suite_attrs, "tests")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let errors: u32 = extract_xml_attr(suite_attrs, "errors")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let skipped: u32 = extract_xml_attr(suite_attrs, "skipped")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let failures: u32 = extract_xml_attr(suite_attrs, "failures")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let re_case =
        regex::Regex::new(r#"(?s)<testcase\b([^>]*?)>(.*?)</testcase>|<testcase\b([^/>]*?)/>"#)
            .map_err(|e| RunnerError::WorkerError(e.to_string()))?;
    let re_failure =
        regex::Regex::new(r#"(?s)<failure\b([^>]*?)>(.*?)</failure>|<failure\b([^/>]*?)/>"#)
            .map_err(|e| RunnerError::WorkerError(e.to_string()))?;
    let re_error = regex::Regex::new(r#"(?s)<error\b([^>]*?)>(.*?)</error>|<error\b([^/>]*?)/>"#)
        .map_err(|e| RunnerError::WorkerError(e.to_string()))?;
    let re_cdata = regex::Regex::new(r#"<!\[CDATA\[(.*?)\]\]>"#)
        .map_err(|e| RunnerError::WorkerError(e.to_string()))?;

    let mut test_cases = Vec::new();

    for cap in re_case.captures_iter(xml) {
        let (case_attrs, case_body) = if let Some(attrs) = cap.get(1) {
            (attrs.as_str(), cap.get(2).map(|m| m.as_str()).unwrap_or(""))
        } else if let Some(attrs) = cap.get(3) {
            (attrs.as_str(), "")
        } else {
            continue;
        };

        let tc_name = extract_xml_attr(case_attrs, "name").unwrap_or_default();
        let tc_classname = extract_xml_attr(case_attrs, "classname").unwrap_or_default();
        let tc_time: f64 = extract_xml_attr(case_attrs, "time")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let mut failure = None;
        if let Some(f_cap) = re_failure.captures(case_body) {
            let (f_attrs, f_body) = if let Some(attrs) = f_cap.get(1) {
                (
                    attrs.as_str(),
                    f_cap.get(2).map(|m| m.as_str()).unwrap_or(""),
                )
            } else if let Some(attrs) = f_cap.get(3) {
                (attrs.as_str(), "")
            } else {
                ("", "")
            };
            let msg = extract_xml_attr(f_attrs, "message").unwrap_or_default();
            let f_type = extract_xml_attr(f_attrs, "type").unwrap_or_default();
            let stack = if let Some(cdata) = re_cdata.captures(f_body) {
                cdata
                    .get(1)
                    .map(|m| m.as_str().trim())
                    .unwrap_or(f_body.trim())
                    .to_string()
            } else {
                f_body.trim().to_string()
            };
            failure = Some(SurefireFailure {
                message: msg,
                failure_type: f_type,
                stack_trace: stack,
            });
        }

        let mut error = None;
        if let Some(e_cap) = re_error.captures(case_body) {
            let (e_attrs, e_body) = if let Some(attrs) = e_cap.get(1) {
                (
                    attrs.as_str(),
                    e_cap.get(2).map(|m| m.as_str()).unwrap_or(""),
                )
            } else if let Some(attrs) = e_cap.get(3) {
                (attrs.as_str(), "")
            } else {
                ("", "")
            };
            let msg = extract_xml_attr(e_attrs, "message").unwrap_or_default();
            let e_type = extract_xml_attr(e_attrs, "type").unwrap_or_default();
            let stack = if let Some(cdata) = re_cdata.captures(e_body) {
                cdata
                    .get(1)
                    .map(|m| m.as_str().trim())
                    .unwrap_or(e_body.trim())
                    .to_string()
            } else {
                e_body.trim().to_string()
            };
            error = Some(SurefireFailure {
                message: msg,
                failure_type: e_type,
                stack_trace: stack,
            });
        }

        test_cases.push(SurefireTestCase {
            name: tc_name,
            classname: tc_classname,
            time_sec: tc_time,
            failure,
            error,
        });
    }

    Ok(SurefireReport {
        name,
        time_sec,
        tests,
        errors,
        skipped,
        failures,
        test_cases,
    })
}

/// JvmRunner executes Java Maven test suites (`mvn test -B -Dtest={className}`)
/// and parses Surefire XML test reports to evaluate correctness and flakiness.
pub struct JvmRunner {
    exercise_dir: PathBuf,
    maven_cmd: String,
    request_counter: AtomicU64,
}

impl JvmRunner {
    /// Create a new JvmRunner targeting the specified exercise directory
    pub fn new<P: AsRef<Path>>(exercise_dir: P) -> Self {
        Self {
            exercise_dir: exercise_dir.as_ref().to_path_buf(),
            maven_cmd: Self::default_maven_cmd(),
            request_counter: AtomicU64::new(1),
        }
    }

    /// Create a JvmRunner with a custom maven command path
    pub fn with_maven_cmd<P: AsRef<Path>>(exercise_dir: P, maven_cmd: &str) -> Self {
        Self {
            exercise_dir: exercise_dir.as_ref().to_path_buf(),
            maven_cmd: maven_cmd.to_string(),
            request_counter: AtomicU64::new(1),
        }
    }

    /// Default maven command based on operating system
    pub fn default_maven_cmd() -> String {
        if cfg!(windows) {
            "mvn.cmd".to_string()
        } else {
            "mvn".to_string()
        }
    }

    pub fn exercise_dir(&self) -> &Path {
        &self.exercise_dir
    }

    pub fn maven_cmd(&self) -> &str {
        &self.maven_cmd
    }

    /// Extract fully-qualified Java class name from relative or absolute file paths
    pub fn extract_class_name<P: AsRef<Path>>(file_path: P) -> Option<String> {
        let p = file_path.as_ref();
        let path_str = p.to_string_lossy();
        let normalized = path_str.replace('\\', "/");

        let markers = [
            "src/test/java/",
            "src/main/java/",
            "test/java/",
            "main/java/",
        ];

        for marker in &markers {
            if let Some(pos) = normalized.find(marker) {
                let rel = &normalized[pos + marker.len()..];
                let class_rel = if let Some(stripped) = rel.strip_suffix(".java") {
                    stripped
                } else {
                    rel
                };
                let class_name = class_rel.replace('/', ".").trim_matches('.').to_string();
                if !class_name.is_empty() {
                    return Some(class_name);
                }
            }
        }

        // Fallback: Check if file exists on disk and extract package name + file stem
        if p.exists()
            && p.is_file()
            && let Ok(content) = std::fs::read_to_string(p)
        {
            let file_stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("package ") && trimmed.ends_with(';') {
                    let pkg = trimmed
                        .trim_start_matches("package")
                        .trim_end_matches(';')
                        .trim();
                    if !pkg.is_empty() && !file_stem.is_empty() {
                        return Some(format!("{}.{}", pkg, file_stem));
                    }
                }
            }
        }

        let candidate = if let Some(stripped) = normalized.strip_suffix(".java") {
            stripped
        } else {
            &normalized
        };

        if candidate.contains('.') && !candidate.contains('/') {
            return Some(candidate.to_string());
        }

        if candidate.contains('/') {
            let dotted = candidate.replace('/', ".").trim_matches('.').to_string();
            return Some(dotted);
        }

        if !candidate.is_empty() {
            Some(candidate.to_string())
        } else {
            None
        }
    }

    /// Parse a Surefire XML report from file
    pub fn parse_surefire_report<P: AsRef<Path>>(
        report_path: P,
    ) -> Result<SurefireReport, RunnerError> {
        let content = std::fs::read_to_string(report_path)?;
        parse_surefire_xml(&content)
    }

    /// Execute a single test iteration via Maven and parse results
    pub async fn run_single_iteration(
        &self,
        class_name: &str,
        chaos: &str,
        timeout_ms: u64,
        iteration: u32,
    ) -> Result<RunResult, RunnerError> {
        let start_time = std::time::Instant::now();
        let surefire_report_path = self
            .exercise_dir
            .join("target")
            .join("surefire-reports")
            .join(format!("TEST-{}.xml", class_name));

        // Remove previous surefire report before running to prevent stale reads
        if surefire_report_path.exists() {
            let _ = std::fs::remove_file(&surefire_report_path);
        }

        let mut cmd = Command::new(&self.maven_cmd);
        cmd.current_dir(&self.exercise_dir);
        cmd.arg("test");
        cmd.arg("-B");
        cmd.arg(format!("-Dtest={}", class_name));

        if !chaos.is_empty() {
            cmd.env("CHAOS_DIRECTIVES", chaos);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let timeout_duration = Duration::from_millis(timeout_ms);
        let child = cmd.spawn().map_err(RunnerError::Io)?;

        let output = match timeout(timeout_duration, child.wait_with_output()).await {
            Ok(res) => res.map_err(RunnerError::Io)?,
            Err(_) => {
                return Ok(RunResult {
                    iteration,
                    passed: false,
                    duration_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(format!("Test execution timed out after {}ms", timeout_ms)),
                });
            }
        };

        let elapsed_ms = start_time.elapsed().as_millis() as u64;

        if surefire_report_path.exists()
            && let Ok(report) = Self::parse_surefire_report(&surefire_report_path)
        {
            let passed = report.failures == 0 && report.errors == 0 && report.tests > 0;
            let duration_ms = if report.time_sec > 0.0 {
                (report.time_sec * 1000.0).round() as u64
            } else {
                elapsed_ms
            };

            let error = if !passed {
                let mut err_msgs = Vec::new();
                for tc in &report.test_cases {
                    if let Some(ref f) = tc.failure {
                        let msg = f.message.trim();
                        if !msg.is_empty() {
                            err_msgs.push(format!("{}: {}", tc.name, msg));
                        } else {
                            err_msgs.push(format!("{}: {}", tc.name, f.failure_type));
                        }
                    }
                    if let Some(ref e) = tc.error {
                        let msg = e.message.trim();
                        if !msg.is_empty() {
                            err_msgs.push(format!("{}: {}", tc.name, msg));
                        } else {
                            err_msgs.push(format!("{}: {}", tc.name, e.failure_type));
                        }
                    }
                }
                if err_msgs.is_empty() {
                    Some(format!(
                        "Test suite failed with {} failure(s) and {} error(s)",
                        report.failures, report.errors
                    ))
                } else {
                    Some(err_msgs.join("; "))
                }
            } else {
                None
            };

            return Ok(RunResult {
                iteration,
                passed,
                duration_ms,
                error,
            });
        }

        // Fallback to stdout/stderr inspection if surefire report wasn't generated
        let stdout_str = String::from_utf8_lossy(&output.stdout);
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let success = output.status.success();

        if success && (stdout_str.contains("BUILD SUCCESS") || stdout_str.contains("Tests run:")) {
            Ok(RunResult {
                iteration,
                passed: true,
                duration_ms: elapsed_ms,
                error: None,
            })
        } else {
            let mut err_lines = Vec::new();
            for line in stdout_str.lines().chain(stderr_str.lines()) {
                let t = line.trim();
                if t.starts_with("[ERROR]")
                    && !t.contains("[ERROR] -> [Help 1]")
                    && !t.contains("[ERROR] To see the full stack trace")
                {
                    err_lines.push(t.to_string());
                }
            }
            let error = if !err_lines.is_empty() {
                Some(err_lines.join("\n"))
            } else {
                Some(format!(
                    "Maven command failed with exit code: {:?}",
                    output.status.code()
                ))
            };

            Ok(RunResult {
                iteration,
                passed: false,
                duration_ms: elapsed_ms,
                error,
            })
        }
    }

    /// Dispatch multi-iteration drill test execution and aggregate into DrillResponse
    pub async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        let req_id = format!(
            "jvm-req-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );
        let class_name = Self::extract_class_name(file).ok_or_else(|| {
            RunnerError::WorkerError(format!(
                "Could not resolve Java class name from path: {}",
                file
            ))
        })?;

        let iterations = iterations.max(1);
        let mut runs = Vec::with_capacity(iterations as usize);
        let mut passed_iterations = 0;
        let mut failed_iterations = 0;
        let mut total_duration_ms = 0;
        let mut first_error = None;

        let timeout_per_iter = (timeout_ms / (iterations as u64)).max(5000);

        for i in 1..=iterations {
            let result = self
                .run_single_iteration(&class_name, chaos, timeout_per_iter, i)
                .await?;
            if result.passed {
                passed_iterations += 1;
            } else {
                failed_iterations += 1;
                if first_error.is_none() {
                    first_error = result.error.clone();
                }
            }
            total_duration_ms += result.duration_ms;
            runs.push(result);
        }

        let overall_passed = failed_iterations == 0 && passed_iterations > 0;

        Ok(DrillResponse {
            id: req_id,
            ok: true,
            passed: overall_passed,
            iterations,
            passed_iterations,
            failed_iterations,
            total_duration_ms,
            runs,
            error: first_error,
        })
    }
}

/// k6 Summary Report threshold evaluation and metric values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct K6ThresholdResult {
    #[serde(default)]
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct K6MetricValues {
    #[serde(default)]
    pub avg: Option<f64>,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub med: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(rename = "p(90)", default)]
    pub p90: Option<f64>,
    #[serde(rename = "p(95)", default)]
    pub p95: Option<f64>,
    #[serde(rename = "p(99)", default)]
    pub p99: Option<f64>,
    #[serde(default)]
    pub rate: Option<f64>,
    #[serde(default)]
    pub passes: Option<u64>,
    #[serde(default)]
    pub fails: Option<u64>,
    #[serde(default)]
    pub count: Option<u64>,
    #[serde(default)]
    pub value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct K6Metric {
    #[serde(rename = "type", default)]
    pub metric_type: Option<String>,
    #[serde(default)]
    pub contains: Option<String>,
    #[serde(default)]
    pub values: Option<K6MetricValues>,
    #[serde(default)]
    pub thresholds: Option<std::collections::HashMap<String, K6ThresholdResult>>,
    #[serde(default)]
    pub threshold: Option<std::collections::HashMap<String, K6ThresholdResult>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct K6SummaryRaw {
    #[serde(default)]
    pub metrics: std::collections::HashMap<String, K6Metric>,
    #[serde(default)]
    pub root_group: Option<serde_json::Value>,
    #[serde(default)]
    pub state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct K6SummaryReport {
    pub all_thresholds_passed: bool,
    pub failed_thresholds: Vec<String>,
    pub avg_duration_ms: u64,
    pub p95_duration_ms: Option<f64>,
    pub p99_duration_ms: Option<f64>,
    pub http_req_failed_rate: Option<f64>,
    pub metrics: std::collections::HashMap<String, K6Metric>,
}

/// Parse k6 summary-export JSON into structured report and evaluate threshold assertions
pub fn parse_k6_summary_json(json_str: &str) -> Result<K6SummaryReport, RunnerError> {
    let raw: K6SummaryRaw = serde_json::from_str(json_str).map_err(RunnerError::Json)?;

    let mut all_thresholds_passed = true;
    let mut failed_thresholds = Vec::new();

    for (metric_name, metric_data) in &raw.metrics {
        if let Some(ref thresh_map) = metric_data.thresholds {
            for (thresh_expr, thresh_res) in thresh_map {
                if !thresh_res.ok {
                    all_thresholds_passed = false;
                    failed_thresholds.push(format!(
                        "{}: threshold '{}' failed",
                        metric_name, thresh_expr
                    ));
                }
            }
        }
        if let Some(ref thresh_map) = metric_data.threshold {
            for (thresh_expr, thresh_res) in thresh_map {
                if !thresh_res.ok {
                    all_thresholds_passed = false;
                    failed_thresholds.push(format!(
                        "{}: threshold '{}' failed",
                        metric_name, thresh_expr
                    ));
                }
            }
        }
    }

    let mut avg_duration_ms = 0;
    let mut p95_duration_ms = None;
    let mut p99_duration_ms = None;
    let mut http_req_failed_rate = None;

    if let Some(dur_metric) = raw.metrics.get("http_req_duration")
        && let Some(ref v) = dur_metric.values
    {
        if let Some(avg) = v.avg {
            avg_duration_ms = avg.round() as u64;
        }
        p95_duration_ms = v.p95;
        p99_duration_ms = v.p99;
    }

    // Inspect custom trend / duration metrics if standard http_req_duration fields are absent
    for (name, metric) in &raw.metrics {
        if name != "http_req_duration"
            && let Some(ref v) = metric.values
        {
            if p99_duration_ms.is_none() && v.p99.is_some() {
                p99_duration_ms = v.p99;
            }
            if p95_duration_ms.is_none() && v.p95.is_some() {
                p95_duration_ms = v.p95;
            }
            if avg_duration_ms == 0
                && let Some(avg) = v.avg
            {
                avg_duration_ms = avg.round() as u64;
            }
        }
    }

    if let Some(failed_metric) = raw.metrics.get("http_req_failed") {
        if let Some(ref v) = failed_metric.values {
            http_req_failed_rate = v.rate;
        }
    } else {
        // Check for custom rate metrics like chaos_errors or search_errors
        for (name, metric) in &raw.metrics {
            if (name.contains("error") || name.contains("fail"))
                && let Some(ref v) = metric.values
                && v.rate.is_some()
                && http_req_failed_rate.is_none()
            {
                http_req_failed_rate = v.rate;
            }
        }
    }

    Ok(K6SummaryReport {
        all_thresholds_passed,
        failed_thresholds,
        avg_duration_ms,
        p95_duration_ms,
        p99_duration_ms,
        http_req_failed_rate,
        metrics: raw.metrics,
    })
}

/// K6Runner executes k6 load testing scripts and parses the summary-export JSON
pub struct K6Runner {
    k6_cmd: String,
    request_counter: AtomicU64,
}

impl K6Runner {
    pub fn new() -> Self {
        Self {
            k6_cmd: "k6".to_string(),
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn with_k6_cmd<S: Into<String>>(cmd: S) -> Self {
        Self {
            k6_cmd: cmd.into(),
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn k6_cmd(&self) -> &str {
        &self.k6_cmd
    }

    pub fn parse_summary_report<P: AsRef<Path>>(
        report_path: P,
    ) -> Result<K6SummaryReport, RunnerError> {
        let content = std::fs::read_to_string(report_path)?;
        parse_k6_summary_json(&content)
    }

    pub async fn run_single_iteration(
        &self,
        file: &str,
        chaos: &str,
        timeout_ms: u64,
        iteration: u32,
    ) -> Result<RunResult, RunnerError> {
        let start = std::time::Instant::now();
        let exercise_path = Path::new(file);
        if !exercise_path.exists() {
            return Err(RunnerError::WorkerError(format!(
                "Exercise file does not exist: {}",
                file
            )));
        }

        let req_id = self.request_counter.fetch_add(1, Ordering::SeqCst);
        let summary_file = std::env::temp_dir().join(format!(
            "k6-summary-{}-{}-{}.json",
            std::process::id(),
            req_id,
            iteration
        ));

        let summary_arg = format!("--summary-export={}", summary_file.display());
        let mut cmd = Command::new(&self.k6_cmd);
        cmd.arg("run")
            .arg(&summary_arg)
            .arg(file)
            .env("X_CHAOS", chaos)
            .env("PW_CHAOS_HEADER", chaos)
            .env("CHAOS_DIRECTIVES", chaos)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let result_future = async {
            let child = cmd.spawn()?;
            let output = child.wait_with_output().await?;
            Ok::<std::process::Output, std::io::Error>(output)
        };

        let dur_limit = Duration::from_millis(timeout_ms.max(1000));
        let exec_result = match timeout(dur_limit, result_future).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                let _ = std::fs::remove_file(&summary_file);
                return Err(RunnerError::Io(e));
            }
            Err(_) => {
                let _ = std::fs::remove_file(&summary_file);
                return Err(RunnerError::Timeout(dur_limit));
            }
        };

        let elapsed = start.elapsed().as_millis() as u64;

        if summary_file.exists() {
            let content = std::fs::read_to_string(&summary_file).map_err(RunnerError::Io)?;
            let _ = std::fs::remove_file(&summary_file);

            let report = parse_k6_summary_json(&content)?;
            let passed = exec_result.status.success() && report.all_thresholds_passed;
            let error = if !passed {
                if !report.failed_thresholds.is_empty() {
                    Some(report.failed_thresholds.join("; "))
                } else if !exec_result.status.success() {
                    let stderr = String::from_utf8_lossy(&exec_result.stderr);
                    Some(format!("k6 exited with non-zero status: {}", stderr.trim()))
                } else {
                    Some("k6 threshold assertions failed".to_string())
                }
            } else {
                None
            };

            let duration = if report.avg_duration_ms > 0 {
                report.avg_duration_ms
            } else {
                elapsed
            };

            Ok(RunResult {
                iteration,
                passed,
                duration_ms: duration,
                error,
            })
        } else {
            let passed = exec_result.status.success();
            let error = if !passed {
                let stderr = String::from_utf8_lossy(&exec_result.stderr);
                Some(format!(
                    "k6 failed without summary export: {}",
                    stderr.trim()
                ))
            } else {
                None
            };

            Ok(RunResult {
                iteration,
                passed,
                duration_ms: elapsed,
                error,
            })
        }
    }

    pub async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        let req_id = format!(
            "k6-req-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );
        let exercise_path = Path::new(file);
        if !exercise_path.exists() {
            return Ok(DrillResponse {
                id: req_id,
                ok: false,
                passed: false,
                iterations,
                passed_iterations: 0,
                failed_iterations: iterations,
                total_duration_ms: 0,
                runs: Vec::new(),
                error: Some(format!("Exercise file does not exist: {}", file)),
            });
        }

        let iterations = iterations.max(1);
        let mut runs = Vec::with_capacity(iterations as usize);
        let mut passed_iterations = 0;
        let mut failed_iterations = 0;
        let mut total_duration_ms = 0;
        let mut first_error = None;

        let timeout_per_iter = (timeout_ms / (iterations as u64)).max(5000);

        for i in 1..=iterations {
            let result = self
                .run_single_iteration(file, chaos, timeout_per_iter, i)
                .await?;
            if result.passed {
                passed_iterations += 1;
            } else {
                failed_iterations += 1;
                if first_error.is_none() {
                    first_error = result.error.clone();
                }
            }
            total_duration_ms += result.duration_ms;
            runs.push(result);
        }

        let overall_passed = failed_iterations == 0 && passed_iterations > 0;

        Ok(DrillResponse {
            id: req_id,
            ok: true,
            passed: overall_passed,
            iterations,
            passed_iterations,
            failed_iterations,
            total_duration_ms,
            runs,
            error: first_error,
        })
    }
}

impl Default for K6Runner {
    fn default() -> Self {
        Self::new()
    }
}

/// MaestroRunner performs definition validation and syntax verification for Maestro YAML flows
pub struct MaestroRunner {
    maestro_cmd: String,
    request_counter: AtomicU64,
}

impl MaestroRunner {
    pub fn new() -> Self {
        Self {
            maestro_cmd: "maestro".to_string(),
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn with_maestro_cmd<S: Into<String>>(cmd: S) -> Self {
        Self {
            maestro_cmd: cmd.into(),
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn maestro_cmd(&self) -> &str {
        &self.maestro_cmd
    }

    /// Validate Maestro YAML flow syntax and basic definition structure
    pub fn validate_flow_definition(yaml_content: &str) -> Result<(), String> {
        if yaml_content.trim().is_empty() {
            return Err("Maestro flow definition file is empty".to_string());
        }

        let mut has_commands = false;
        let mut in_list_item = false;

        for (idx, raw_line) in yaml_content.lines().enumerate() {
            let line_num = idx + 1;
            let trimmed = raw_line.trim();

            // Ignore comments and document markers
            if trimmed.is_empty()
                || trimmed.starts_with('#')
                || trimmed == "---"
                || trimmed == "..."
            {
                continue;
            }

            // Check for tab characters in indentation (invalid in standard YAML)
            if raw_line.contains('\t') {
                return Err(format!(
                    "YAML syntax error on line {}: tabs are not allowed for indentation",
                    line_num
                ));
            }

            // Check list item starter
            if trimmed.starts_with('-') {
                has_commands = true;
                in_list_item = true;
            } else if in_list_item {
                // Must be a key-value or indented block under list item
                if !trimmed.contains(':') && !trimmed.starts_with('-') {
                    return Err(format!(
                        "YAML syntax error on line {}: expected key-value mapping or list item",
                        line_num
                    ));
                }
            } else if trimmed.contains(':') {
                has_commands = true;
            } else {
                return Err(format!(
                    "YAML syntax error on line {}: unrecognized YAML structure '{}'",
                    line_num, trimmed
                ));
            }
        }

        if !has_commands {
            return Err(
                "No valid Maestro commands or flow steps found in YAML definition".to_string(),
            );
        }

        Ok(())
    }

    pub async fn run_single_iteration(
        &self,
        file: &str,
        _chaos: &str,
        _timeout_ms: u64,
        iteration: u32,
    ) -> Result<RunResult, RunnerError> {
        let start = std::time::Instant::now();
        let exercise_path = Path::new(file);
        if !exercise_path.exists() {
            return Err(RunnerError::WorkerError(format!(
                "Exercise file does not exist: {}",
                file
            )));
        }

        let content = std::fs::read_to_string(exercise_path).map_err(RunnerError::Io)?;

        let validation_res = Self::validate_flow_definition(&content);
        let elapsed = start.elapsed().as_millis() as u64;

        match validation_res {
            Ok(()) => Ok(RunResult {
                iteration,
                passed: true,
                duration_ms: elapsed.max(1),
                error: None,
            }),
            Err(err_msg) => Ok(RunResult {
                iteration,
                passed: false,
                duration_ms: elapsed.max(1),
                error: Some(err_msg),
            }),
        }
    }

    pub async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        let req_id = format!(
            "maestro-req-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );
        let exercise_path = Path::new(file);
        if !exercise_path.exists() {
            return Ok(DrillResponse {
                id: req_id,
                ok: false,
                passed: false,
                iterations,
                passed_iterations: 0,
                failed_iterations: iterations,
                total_duration_ms: 0,
                runs: Vec::new(),
                error: Some(format!("Exercise file does not exist: {}", file)),
            });
        }

        let iterations = iterations.max(1);
        let mut runs = Vec::with_capacity(iterations as usize);
        let mut passed_iterations = 0;
        let mut failed_iterations = 0;
        let mut total_duration_ms = 0;
        let mut first_error = None;

        let timeout_per_iter = (timeout_ms / (iterations as u64)).max(5000);

        for i in 1..=iterations {
            let result = self
                .run_single_iteration(file, chaos, timeout_per_iter, i)
                .await?;
            if result.passed {
                passed_iterations += 1;
            } else {
                failed_iterations += 1;
                if first_error.is_none() {
                    first_error = result.error.clone();
                }
            }
            total_duration_ms += result.duration_ms;
            runs.push(result);
        }

        let overall_passed = failed_iterations == 0 && passed_iterations > 0;

        Ok(DrillResponse {
            id: req_id,
            ok: true,
            passed: overall_passed,
            iterations,
            passed_iterations,
            failed_iterations,
            total_duration_ms,
            runs,
            error: first_error,
        })
    }
}

impl Default for MaestroRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JtlSample {
    pub elapsed: u64,
    pub label: String,
    pub response_code: String,
    pub response_message: String,
    pub success: bool,
    pub failure_message: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct JtlMetrics {
    pub total_samples: usize,
    pub passed_samples: usize,
    pub failed_samples: usize,
    pub error_rate: f64,
    pub avg_elapsed_ms: u64,
    pub min_elapsed_ms: u64,
    pub max_elapsed_ms: u64,
    pub p90_elapsed_ms: u64,
    pub p95_elapsed_ms: u64,
    pub p99_elapsed_ms: u64,
    pub samples: Vec<JtlSample>,
    pub first_failure_reason: Option<String>,
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Parse JMeter JTL CSV format for elapsed, success, label, responseCode to compute latency and error metrics
pub fn parse_jmeter_jtl_csv(csv_content: &str) -> Result<JtlMetrics, RunnerError> {
    let mut lines = csv_content.lines().map(str::trim).filter(|l| !l.is_empty());
    let header_line = match lines.next() {
        Some(h) => h,
        None => return Ok(JtlMetrics::default()),
    };

    let header_fields = split_csv_line(header_line);
    let mut elapsed_idx = None;
    let mut label_idx = None;
    let mut response_code_idx = None;
    let mut response_message_idx = None;
    let mut success_idx = None;
    let mut failure_message_idx = None;

    for (idx, field) in header_fields.iter().enumerate() {
        let lower = field.to_ascii_lowercase();
        match lower.as_str() {
            "elapsed" => elapsed_idx = Some(idx),
            "label" => label_idx = Some(idx),
            "responsecode" | "response_code" | "rc" => response_code_idx = Some(idx),
            "responsemessage" | "response_message" | "rm" => response_message_idx = Some(idx),
            "success" => success_idx = Some(idx),
            "failuremessage" | "failure_message" => failure_message_idx = Some(idx),
            _ => {}
        }
    }

    // Default column indices based on standard JMeter CSV order if not explicitly matched
    let elapsed_idx = elapsed_idx.unwrap_or(1);
    let label_idx = label_idx.unwrap_or(2);
    let response_code_idx = response_code_idx.unwrap_or(3);
    let response_message_idx = response_message_idx.unwrap_or(4);
    let success_idx = success_idx.unwrap_or(7);
    let failure_message_idx = failure_message_idx.unwrap_or(8);

    let mut samples = Vec::new();
    let mut first_failure_reason = None;

    for line in lines {
        if line.starts_with('#') {
            continue;
        }
        let fields = split_csv_line(line);
        if fields.is_empty() {
            continue;
        }

        let elapsed = fields
            .get(elapsed_idx)
            .and_then(|s| s.parse::<f64>().ok())
            .map(|f| f.round() as u64)
            .unwrap_or(0);

        let label = fields.get(label_idx).cloned().unwrap_or_default();
        let response_code = fields.get(response_code_idx).cloned().unwrap_or_default();
        let response_message = fields
            .get(response_message_idx)
            .cloned()
            .unwrap_or_default();
        let success = fields
            .get(success_idx)
            .map(|s| s.eq_ignore_ascii_case("true") || s == "1" || s.eq_ignore_ascii_case("ok"))
            .unwrap_or(false);
        let failure_message = fields.get(failure_message_idx).cloned().unwrap_or_default();

        if !success && first_failure_reason.is_none() {
            let reason = if !failure_message.is_empty() {
                format!(
                    "Sample '{}' failed (HTTP {}): {}",
                    label, response_code, failure_message
                )
            } else if !response_message.is_empty() {
                format!(
                    "Sample '{}' failed (HTTP {}): {}",
                    label, response_code, response_message
                )
            } else {
                format!(
                    "Sample '{}' failed with response code '{}'",
                    label, response_code
                )
            };
            first_failure_reason = Some(reason);
        }

        samples.push(JtlSample {
            elapsed,
            label,
            response_code,
            response_message,
            success,
            failure_message,
        });
    }

    let total_samples = samples.len();
    if total_samples == 0 {
        return Ok(JtlMetrics::default());
    }

    let passed_samples = samples.iter().filter(|s| s.success).count();
    let failed_samples = total_samples - passed_samples;
    let error_rate = failed_samples as f64 / total_samples as f64;

    let mut elapsed_list: Vec<u64> = samples.iter().map(|s| s.elapsed).collect();
    elapsed_list.sort_unstable();

    let min_elapsed_ms = *elapsed_list.first().unwrap_or(&0);
    let max_elapsed_ms = *elapsed_list.last().unwrap_or(&0);
    let sum_elapsed: u64 = elapsed_list.iter().sum();
    let avg_elapsed_ms = (sum_elapsed as f64 / total_samples as f64).round() as u64;

    let p90_idx = ((total_samples as f64 * 0.90).ceil() as usize)
        .saturating_sub(1)
        .min(total_samples - 1);
    let p95_idx = ((total_samples as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(total_samples - 1);
    let p99_idx = ((total_samples as f64 * 0.99).ceil() as usize)
        .saturating_sub(1)
        .min(total_samples - 1);

    let p90_elapsed_ms = elapsed_list[p90_idx];
    let p95_elapsed_ms = elapsed_list[p95_idx];
    let p99_elapsed_ms = elapsed_list[p99_idx];

    Ok(JtlMetrics {
        total_samples,
        passed_samples,
        failed_samples,
        error_rate,
        avg_elapsed_ms,
        min_elapsed_ms,
        max_elapsed_ms,
        p90_elapsed_ms,
        p95_elapsed_ms,
        p99_elapsed_ms,
        samples,
        first_failure_reason,
    })
}

/// JMeterRunner executes Apache JMeter non-GUI test plans and parses the resulting JTL CSV output
pub struct JMeterRunner {
    jmeter_cmd: String,
    request_counter: AtomicU64,
}

impl JMeterRunner {
    pub fn new() -> Self {
        Self {
            jmeter_cmd: "jmeter".to_string(),
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn with_jmeter_cmd<S: Into<String>>(cmd: S) -> Self {
        Self {
            jmeter_cmd: cmd.into(),
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn jmeter_cmd(&self) -> &str {
        &self.jmeter_cmd
    }

    pub fn parse_jtl_report<P: AsRef<Path>>(report_path: P) -> Result<JtlMetrics, RunnerError> {
        let content = std::fs::read_to_string(report_path)?;
        parse_jmeter_jtl_csv(&content)
    }

    pub async fn run_single_iteration(
        &self,
        file: &str,
        chaos: &str,
        timeout_ms: u64,
        iteration: u32,
    ) -> Result<RunResult, RunnerError> {
        let start = std::time::Instant::now();
        let exercise_path = Path::new(file);
        if !exercise_path.exists() {
            return Err(RunnerError::WorkerError(format!(
                "Exercise file does not exist: {}",
                file
            )));
        }

        let req_id = self.request_counter.fetch_add(1, Ordering::SeqCst);
        let jtl_file = std::env::temp_dir().join(format!(
            "jmeter-results-{}-{}-{}.jtl",
            std::process::id(),
            req_id,
            iteration
        ));

        let _ = std::fs::remove_file(&jtl_file);

        let mut cmd = Command::new(&self.jmeter_cmd);
        cmd.arg("-n")
            .arg("-t")
            .arg(file)
            .arg("-l")
            .arg(&jtl_file)
            .env("X_CHAOS", chaos)
            .env("PW_CHAOS_HEADER", chaos)
            .env("CHAOS_DIRECTIVES", chaos)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let result_future = async {
            let child = cmd.spawn()?;
            let output = child.wait_with_output().await?;
            Ok::<std::process::Output, std::io::Error>(output)
        };

        let dur_limit = Duration::from_millis(timeout_ms.max(1000));
        let exec_result = match timeout(dur_limit, result_future).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                let _ = std::fs::remove_file(&jtl_file);
                return Err(RunnerError::Io(e));
            }
            Err(_) => {
                let _ = std::fs::remove_file(&jtl_file);
                return Err(RunnerError::Timeout(dur_limit));
            }
        };

        let elapsed = start.elapsed().as_millis() as u64;

        if jtl_file.exists() {
            let content = std::fs::read_to_string(&jtl_file).map_err(RunnerError::Io)?;
            let _ = std::fs::remove_file(&jtl_file);

            let metrics = parse_jmeter_jtl_csv(&content)?;
            let passed = exec_result.status.success()
                && metrics.total_samples > 0
                && metrics.failed_samples == 0;
            let error = if !passed {
                if let Some(reason) = metrics.first_failure_reason {
                    Some(reason)
                } else if !exec_result.status.success() {
                    let stderr = String::from_utf8_lossy(&exec_result.stderr);
                    let stdout = String::from_utf8_lossy(&exec_result.stdout);
                    let err_msg = if !stderr.trim().is_empty() {
                        stderr.trim()
                    } else {
                        stdout.trim()
                    };
                    Some(format!("JMeter exited with non-zero status: {}", err_msg))
                } else if metrics.total_samples == 0 {
                    Some("JMeter produced no sample results in JTL report".to_string())
                } else {
                    Some("JMeter performance assertions failed".to_string())
                }
            } else {
                None
            };

            let duration = if metrics.avg_elapsed_ms > 0 {
                metrics.avg_elapsed_ms
            } else {
                elapsed
            };

            Ok(RunResult {
                iteration,
                passed,
                duration_ms: duration,
                error,
            })
        } else {
            let passed = exec_result.status.success();
            let error = if !passed {
                let stderr = String::from_utf8_lossy(&exec_result.stderr);
                let stdout = String::from_utf8_lossy(&exec_result.stdout);
                let err_msg = if !stderr.trim().is_empty() {
                    stderr.trim()
                } else {
                    stdout.trim()
                };
                Some(format!("JMeter failed without JTL output: {}", err_msg))
            } else {
                None
            };

            Ok(RunResult {
                iteration,
                passed,
                duration_ms: elapsed,
                error,
            })
        }
    }

    pub async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        let req_id = format!(
            "jmeter-req-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );
        let exercise_path = Path::new(file);
        if !exercise_path.exists() {
            return Ok(DrillResponse {
                id: req_id,
                ok: false,
                passed: false,
                iterations,
                passed_iterations: 0,
                failed_iterations: iterations,
                total_duration_ms: 0,
                runs: Vec::new(),
                error: Some(format!("Exercise file does not exist: {}", file)),
            });
        }

        let iterations = iterations.max(1);
        let mut runs = Vec::with_capacity(iterations as usize);
        let mut passed_iterations = 0;
        let mut failed_iterations = 0;
        let mut total_duration_ms = 0;
        let mut first_error = None;

        let timeout_per_iter = (timeout_ms / (iterations as u64)).max(5000);

        for i in 1..=iterations {
            match self
                .run_single_iteration(file, chaos, timeout_per_iter, i)
                .await
            {
                Ok(result) => {
                    if result.passed {
                        passed_iterations += 1;
                    } else {
                        failed_iterations += 1;
                        if first_error.is_none() {
                            first_error = result.error.clone();
                        }
                    }
                    total_duration_ms += result.duration_ms;
                    runs.push(result);
                }
                Err(RunnerError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(DrillResponse {
                        id: req_id,
                        ok: false,
                        passed: false,
                        iterations,
                        passed_iterations: 0,
                        failed_iterations: iterations,
                        total_duration_ms: 0,
                        runs: Vec::new(),
                        error: Some(format!(
                            "JMeter binary ('{}') was not found on PATH. Please install Apache JMeter (https://jmeter.apache.org/download_jmeter.cgi) and add its 'bin' directory to your system PATH (e.g. 'winget install Apache.JMeter' on Windows or 'brew install jmeter' on macOS).",
                            self.jmeter_cmd
                        )),
                    });
                }
                Err(e) => return Err(e),
            }
        }

        let overall_passed = failed_iterations == 0 && passed_iterations > 0;

        Ok(DrillResponse {
            id: req_id,
            ok: true,
            passed: overall_passed,
            iterations,
            passed_iterations,
            failed_iterations,
            total_duration_ms,
            runs,
            error: first_error,
        })
    }
}

impl Default for JMeterRunner {
    fn default() -> Self {
        Self::new()
    }
}

pub struct PytestRunner {
    python_cmd: String,
    worker_script: PathBuf,
    request_counter: AtomicU64,
}

impl Default for PytestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl PytestRunner {
    pub fn new() -> Self {
        Self::with_config("python", PathBuf::from("workers/pytest_worker.py"))
    }

    pub fn with_config(python_cmd: &str, worker_script: PathBuf) -> Self {
        Self {
            python_cmd: python_cmd.to_string(),
            worker_script,
            request_counter: AtomicU64::new(1),
        }
    }

    pub fn python_cmd(&self) -> &str {
        &self.python_cmd
    }

    pub fn worker_script(&self) -> &Path {
        &self.worker_script
    }
}

impl Runner for PytestRunner {
    fn run_drill(
        &self,
        path: &str,
        chaos_header: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> impl std::future::Future<Output = Result<DrillResponse, RunnerError>> + Send {
        let path = path.to_string();
        let chaos = chaos_header.to_string();
        let python_cmd = self.python_cmd.clone();
        let worker_script = self.worker_script.clone();
        let req_id = format!(
            "pytest-req-{}",
            self.request_counter.fetch_add(1, Ordering::SeqCst)
        );

        async move {
            let iterations = iterations.max(1);
            let start_time = std::time::Instant::now();

            let mut cmd = Command::new(&python_cmd);
            cmd.arg(&worker_script)
                .arg(&path)
                .arg("--iterations")
                .arg(iterations.to_string())
                .arg("--timeout")
                .arg(timeout_ms.to_string());

            if !chaos.is_empty() {
                cmd.arg("--chaos").arg(&chaos);
                cmd.env("CHAOS_DIRECTIVES", &chaos);
                cmd.env("PW_CHAOS_HEADER", &chaos);
            }

            cmd.stdout(Stdio::piped());
            cmd.stderr(Stdio::piped());

            let timeout_duration = Duration::from_millis(timeout_ms.max(5000));
            let child = cmd.spawn().map_err(RunnerError::Io)?;

            let output = match timeout(timeout_duration, child.wait_with_output()).await {
                Ok(res) => res.map_err(RunnerError::Io)?,
                Err(_) => {
                    return Ok(DrillResponse {
                        id: req_id,
                        ok: false,
                        passed: false,
                        iterations,
                        passed_iterations: 0,
                        failed_iterations: iterations,
                        total_duration_ms: start_time.elapsed().as_millis() as u64,
                        runs: vec![],
                        error: Some(format!(
                            "Pytest execution timed out after {}ms",
                            timeout_ms
                        )),
                    });
                }
            };

            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let elapsed_ms = start_time.elapsed().as_millis() as u64;

            // Attempt to parse structured DrillResponse JSON from worker stdout
            for line in stdout.lines().rev() {
                let trimmed = line.trim();
                if trimmed.starts_with('{') && trimmed.ends_with('}') {
                    if let Ok(mut drill_resp) = serde_json::from_str::<DrillResponse>(trimmed) {
                        drill_resp.id = req_id;
                        return Ok(drill_resp);
                    }
                }
            }

            // Fallback parsing if JSON wasn't returned
            let passed = output.status.success() && stdout.contains("\"passed\": true");
            let error_msg = if !passed {
                let err = stderr.trim();
                if !err.is_empty() {
                    Some(err.to_string())
                } else {
                    let out = stdout.trim();
                    if !out.is_empty() {
                        Some(out.to_string())
                    } else {
                        Some(format!("Worker process exited with status {:?}", output.status))
                    }
                }
            } else {
                None
            };

            let runs = (1..=iterations)
                .map(|i| RunResult {
                    iteration: i,
                    passed,
                    duration_ms: elapsed_ms / (iterations as u64).max(1),
                    error: error_msg.clone(),
                })
                .collect();

            Ok(DrillResponse {
                id: req_id,
                ok: output.status.success(),
                passed,
                iterations,
                passed_iterations: if passed { iterations } else { 0 },
                failed_iterations: if passed { 0 } else { iterations },
                total_duration_ms: elapsed_ms,
                runs,
                error: error_msg,
            })
        }
    }
}

/// SDET policy score a workflow must reach for the drill to count as solved.
///
/// 100 means zero policy errors *and* zero warnings: the validator deducts 25
/// per error and 10 per warning, so anything less leaves a real finding on the
/// table.
pub const PIPELINE_PASS_SCORE: u32 = 100;

/// Runs CI/CD workflow drills against the in-process pipeline simulator.
///
/// Unlike the other runners this one spawns no subprocess — `src/pipeline`
/// parses the workflow, applies the enterprise SDET policy set, and simulates
/// matrix execution entirely in memory. That keeps the track fully offline: no
/// Docker-in-Docker, no `act`, no network.
pub struct PipelineRunner {
    pass_score: u32,
}

impl Default for PipelineRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineRunner {
    pub fn new() -> Self {
        Self {
            pass_score: PIPELINE_PASS_SCORE,
        }
    }

    pub fn with_pass_score(pass_score: u32) -> Self {
        Self { pass_score }
    }

    pub fn pass_score(&self) -> u32 {
        self.pass_score
    }

    /// Validates and simulates a workflow file once, returning the policy score
    /// and the first blocking finding (if any).
    fn evaluate(&self, file: &str) -> Result<(u32, bool, Option<String>), RunnerError> {
        let path = Path::new(file);
        let result = crate::pipeline::run_pipeline(
            path,
            &crate::pipeline::PipelineRunOptions {
                parallel: false,
                fail_fast: false,
                animated: false,
                max_parallel: None,
                verbose: false,
                // Simulate even when policy fails so the learner still sees the
                // job graph execute; the score, not the simulation, gates pass.
                strict_validation: false,
            },
        )
        .map_err(|e| RunnerError::WorkerError(format!("{}: {}", path.display(), e)))?;

        let validation = result.validation.ok_or_else(|| {
            RunnerError::ProtocolError("pipeline run returned no validation".into())
        })?;

        // Surface the highest-severity finding first: errors before warnings.
        let finding = validation
            .errors
            .first()
            .map(|e| format!("[{}] {}", e.code, e.message))
            .or_else(|| {
                validation
                    .warnings
                    .first()
                    .map(|w| format!("[{}] {}", w.code, w.message))
            });

        Ok((validation.sdet_score, result.success, finding))
    }
}

impl PipelineRunner {
    pub async fn run_drill(
        &self,
        file: &str,
        _chaos: &str,
        iterations: u32,
        _timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        let iterations = iterations.max(1);
        let mut runs = Vec::with_capacity(iterations as usize);
        let mut total_duration_ms = 0u64;
        let mut passed_iterations = 0u32;

        for iteration in 1..=iterations {
            let started = std::time::Instant::now();
            let outcome = self.evaluate(file);
            let duration_ms = started.elapsed().as_millis() as u64;
            total_duration_ms += duration_ms;

            let (passed, error) = match outcome {
                Ok((score, simulated_ok, finding)) => {
                    let passed = score >= self.pass_score && simulated_ok;
                    if passed {
                        passed_iterations += 1;
                        (true, None)
                    } else {
                        let detail = finding.unwrap_or_else(|| {
                            "workflow simulation reported a failing job".to_string()
                        });
                        (
                            false,
                            Some(format!(
                                "SDET policy score {}/{} — {}",
                                score, self.pass_score, detail
                            )),
                        )
                    }
                }
                Err(e) => (false, Some(e.to_string())),
            };

            runs.push(RunResult {
                iteration,
                passed,
                duration_ms,
                error,
            });
        }

        let first_error = runs.iter().find_map(|r| r.error.clone());

        Ok(DrillResponse {
            id: format!("pipeline-{}", file),
            ok: true,
            passed: passed_iterations == iterations,
            iterations,
            passed_iterations,
            failed_iterations: iterations - passed_iterations,
            total_duration_ms,
            runs,
            error: first_error,
        })
    }
}

/// Unified Runner enum supporting NodeRunner, JvmRunner, K6Runner, MaestroRunner, PytestRunner, JMeterRunner, and PipelineRunner
pub enum AnyRunner {
    Node(Arc<NodeRunner>),
    Jvm(Arc<JvmRunner>),
    K6(Arc<K6Runner>),
    Maestro(Arc<MaestroRunner>),
    Pytest(Arc<PytestRunner>),
    Jmeter(Arc<JMeterRunner>),
    Pipeline(Arc<PipelineRunner>),
}

impl AnyRunner {
    pub async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        match self {
            Self::Node(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
            Self::Jvm(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
            Self::K6(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
            Self::Maestro(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
            Self::Pytest(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
            Self::Jmeter(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
            Self::Pipeline(runner) => runner.run_drill(file, chaos, iterations, timeout_ms).await,
        }
    }
}

pub trait Runner: Send + Sync {
    fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> impl std::future::Future<Output = Result<DrillResponse, RunnerError>> + Send;
}

impl Runner for NodeRunner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

impl Runner for JvmRunner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

impl Runner for K6Runner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

impl Runner for MaestroRunner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

impl Runner for JMeterRunner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

impl Runner for PipelineRunner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

impl Runner for AnyRunner {
    async fn run_drill(
        &self,
        file: &str,
        chaos: &str,
        iterations: u32,
        timeout_ms: u64,
    ) -> Result<DrillResponse, RunnerError> {
        self.run_drill(file, chaos, iterations, timeout_ms).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drill_request_serialization() {
        let req = DrillRequest {
            id: "req-1".to_string(),
            action: "run_drill".to_string(),
            file: "exercises/01/exercise.ts".to_string(),
            chaos: "delay=200ms;jitter=75ms".to_string(),
            iterations: 5,
            timeout_ms: 15000,
        };

        let json = serde_json::to_string(&req).expect("Failed to serialize");
        assert!(json.contains("\"action\":\"run_drill\""));
        assert!(json.contains("\"file\":\"exercises/01/exercise.ts\""));
        assert!(json.contains("\"iterations\":5"));

        let deserialized: DrillRequest =
            serde_json::from_str(&json).expect("Failed to deserialize");
        assert_eq!(deserialized, req);
    }

    #[test]
    fn test_drill_response_deserialization() {
        let raw = r#"{
            "id": "req-101",
            "ok": true,
            "passed": false,
            "iterations": 5,
            "passed_iterations": 1,
            "failed_iterations": 4,
            "total_duration_ms": 2450,
            "runs": [
                {
                    "iteration": 1,
                    "passed": false,
                    "duration_ms": 510,
                    "error": "Error: locator.click failed"
                },
                {
                    "iteration": 2,
                    "passed": true,
                    "duration_ms": 480,
                    "error": null
                }
            ],
            "error": null
        }"#;

        let resp: DrillResponse = serde_json::from_str(raw).expect("Failed to parse response");
        assert_eq!(resp.id, "req-101");
        assert!(resp.ok);
        assert!(!resp.passed);
        assert_eq!(resp.iterations, 5);
        assert_eq!(resp.passed_iterations, 1);
        assert_eq!(resp.failed_iterations, 4);
        assert_eq!(resp.total_duration_ms, 2450);
        assert_eq!(resp.runs.len(), 2);
        assert_eq!(resp.runs[0].iteration, 1);
        assert!(!resp.runs[0].passed);
        assert_eq!(
            resp.runs[0].error.as_deref(),
            Some("Error: locator.click failed")
        );
        assert!(resp.runs[1].passed);
        assert_eq!(resp.runs[1].error, None);
    }

    #[tokio::test]
    async fn test_node_runner_lifecycle_and_ping() {
        let worker_path = Path::new("workers/node_worker.js");
        if !worker_path.exists() {
            eprintln!("Skipping test: workers/node_worker.js not found in current directory");
            return;
        }

        let runner = NodeRunner::start(worker_path)
            .await
            .expect("Failed to start NodeRunner");

        let ping_result = runner.ping().await.expect("Ping failed");
        assert!(ping_result, "Expected ping to return true");
    }

    #[tokio::test]
    async fn test_node_runner_nonexistent_file() {
        let worker_path = Path::new("workers/node_worker.js");
        if !worker_path.exists() {
            eprintln!("Skipping test: workers/node_worker.js not found in current directory");
            return;
        }

        let runner = NodeRunner::start(worker_path)
            .await
            .expect("Failed to start NodeRunner");

        let response = runner
            .run_drill("non_existent_exercise.ts", "", 1, 5000)
            .await
            .expect("Runner communication failed");

        assert!(!response.ok);
        assert!(!response.passed);
        assert!(response.error.is_some());
        assert!(
            response
                .error
                .unwrap()
                .contains("Exercise file does not exist")
        );
    }

    #[test]
    fn test_extract_class_name_variations() {
        // Standard Maven directory layout
        assert_eq!(
            JvmRunner::extract_class_name(
                "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java"
            ),
            Some("com.cherenkov.drill01_idempotency.Exercise".to_string())
        );

        // Windows backslash path
        assert_eq!(
            JvmRunner::extract_class_name(
                r"exercises\02_api_restassured_java\src\test\java\com\cherenkov\drill02_jwt_auth\Solution.java"
            ),
            Some("com.cherenkov.drill02_jwt_auth.Solution".to_string())
        );

        // Subpath starting at src/test/java
        assert_eq!(
            JvmRunner::extract_class_name(
                "src/test/java/com/cherenkov/drill03_kafka_lag/Exercise.java"
            ),
            Some("com.cherenkov.drill03_kafka_lag.Exercise".to_string())
        );

        // Fully-qualified class name
        assert_eq!(
            JvmRunner::extract_class_name("com.cherenkov.drill01_idempotency.Exercise"),
            Some("com.cherenkov.drill01_idempotency.Exercise".to_string())
        );

        // Relative slash class path
        assert_eq!(
            JvmRunner::extract_class_name("com/cherenkov/drill01_idempotency/Solution.java"),
            Some("com.cherenkov.drill01_idempotency.Solution".to_string())
        );

        // Existing file on disk in exercises/02_api_restassured_java
        let drill1_ex = Path::new(
            "exercises/02_api_restassured_java/src/test/java/com/cherenkov/drill01_idempotency/Exercise.java",
        );
        if drill1_ex.exists() {
            assert_eq!(
                JvmRunner::extract_class_name(drill1_ex),
                Some("com.cherenkov.drill01_idempotency.Exercise".to_string())
            );
        }
    }

    #[test]
    fn test_parse_surefire_xml_success() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.cherenkov.drill01_idempotency.Solution" time="2.707" tests="2" errors="0" skipped="0" failures="0">
  <testcase name="testFreshCheckoutWithDynamicKey" classname="com.cherenkov.drill01_idempotency.Solution" time="2.07"/>
  <testcase name="testIdempotencyCollisionHandling" classname="com.cherenkov.drill01_idempotency.Solution" time="0.148"/>
</testsuite>"#;

        let report = parse_surefire_xml(xml).expect("Failed to parse surefire XML");
        assert_eq!(report.name, "com.cherenkov.drill01_idempotency.Solution");
        assert_eq!(report.tests, 2);
        assert_eq!(report.failures, 0);
        assert_eq!(report.errors, 0);
        assert_eq!(report.skipped, 0);
        assert_eq!(report.time_sec, 2.707);
        assert_eq!(report.test_cases.len(), 2);
        assert_eq!(report.test_cases[0].name, "testFreshCheckoutWithDynamicKey");
        assert_eq!(report.test_cases[0].time_sec, 2.07);
        assert!(report.test_cases[0].failure.is_none());
        assert!(report.test_cases[1].failure.is_none());
    }

    #[test]
    fn test_parse_surefire_xml_failure() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.cherenkov.drill01_idempotency.Exercise" time="2.344" tests="1" errors="0" skipped="0" failures="1">
  <testcase name="testCheckoutWithStaticKey" classname="com.cherenkov.drill01_idempotency.Exercise" time="1.713">
    <failure message="1 expectation failed.&#10;Expected status code &lt;200&gt; but was &lt;409&gt;.&#10;" type="java.lang.AssertionError"><![CDATA[java.lang.AssertionError: 
1 expectation failed.
Expected status code <200> but was <409>.
	at com.cherenkov.drill01_idempotency.Exercise.testCheckoutWithStaticKey(Exercise.java:51)
]]></failure>
  </testcase>
</testsuite>"#;

        let report = parse_surefire_xml(xml).expect("Failed to parse surefire XML");
        assert_eq!(report.name, "com.cherenkov.drill01_idempotency.Exercise");
        assert_eq!(report.tests, 1);
        assert_eq!(report.failures, 1);
        assert_eq!(report.errors, 0);
        assert_eq!(report.test_cases.len(), 1);

        let tc = &report.test_cases[0];
        assert_eq!(tc.name, "testCheckoutWithStaticKey");
        let failure = tc.failure.as_ref().expect("Expected failure");
        assert!(
            failure
                .message
                .contains("Expected status code <200> but was <409>")
        );
        assert_eq!(failure.failure_type, "java.lang.AssertionError");
        assert!(failure.stack_trace.contains("Exercise.java:51"));
    }

    #[test]
    fn test_parse_surefire_xml_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuite name="com.cherenkov.drill02_jwt_auth.Exercise" time="1.120" tests="1" errors="1" skipped="0" failures="0">
  <testcase name="testAuthMeWithExpiredToken" classname="com.cherenkov.drill02_jwt_auth.Exercise" time="0.950">
    <error message="Connection refused" type="java.net.ConnectException"><![CDATA[java.net.ConnectException: Connection refused
	at java.base/sun.nio.ch.Net.connect0(Native Method)
]]></error>
  </testcase>
</testsuite>"#;

        let report = parse_surefire_xml(xml).expect("Failed to parse surefire XML");
        assert_eq!(report.tests, 1);
        assert_eq!(report.failures, 0);
        assert_eq!(report.errors, 1);
        assert_eq!(report.test_cases.len(), 1);

        let tc = &report.test_cases[0];
        let err = tc.error.as_ref().expect("Expected error");
        assert_eq!(err.message, "Connection refused");
        assert_eq!(err.failure_type, "java.net.ConnectException");
    }

    #[test]
    fn test_jvm_runner_configuration_and_paths() {
        let runner = JvmRunner::new("exercises/02_api_restassured_java");
        assert_eq!(
            runner.exercise_dir(),
            Path::new("exercises/02_api_restassured_java")
        );
        if cfg!(windows) {
            assert_eq!(runner.maven_cmd(), "mvn.cmd");
        } else {
            assert_eq!(runner.maven_cmd(), "mvn");
        }

        let custom_runner =
            JvmRunner::with_maven_cmd("exercises/02_api_restassured_java", "custom-mvn");
        assert_eq!(custom_runner.maven_cmd(), "custom-mvn");
    }

    #[test]
    fn test_parse_k6_summary_json_success() {
        let json = r#"{
            "metrics": {
                "http_req_duration": {
                    "type": "trend",
                    "contains": "time",
                    "values": {
                        "avg": 145.2,
                        "min": 23.1,
                        "med": 110.5,
                        "max": 650.0,
                        "p(90)": 280.0,
                        "p(95)": 390.0,
                        "p(99)": 520.0
                    },
                    "thresholds": {
                        "p(95)<2000": { "ok": true }
                    }
                },
                "http_req_failed": {
                    "type": "rate",
                    "contains": "default",
                    "values": {
                        "rate": 0.005,
                        "passes": 1,
                        "fails": 199
                    },
                    "thresholds": {
                        "rate<0.01": { "ok": true }
                    }
                }
            }
        }"#;

        let report = parse_k6_summary_json(json).expect("Parse k6 summary JSON");
        assert!(report.all_thresholds_passed);
        assert!(report.failed_thresholds.is_empty());
        assert_eq!(report.avg_duration_ms, 145);
        assert_eq!(report.p95_duration_ms, Some(390.0));
        assert_eq!(report.p99_duration_ms, Some(520.0));
        assert_eq!(report.http_req_failed_rate, Some(0.005));
    }

    #[test]
    fn test_parse_k6_summary_json_failure_thresholds() {
        let json = r#"{
            "metrics": {
                "http_req_duration": {
                    "type": "trend",
                    "values": {
                        "avg": 2500.0,
                        "p(95)": 3200.0
                    },
                    "thresholds": {
                        "p(95)<2000": { "ok": false }
                    }
                },
                "http_req_failed": {
                    "type": "rate",
                    "values": {
                        "rate": 0.15
                    },
                    "thresholds": {
                        "rate<0.01": { "ok": false }
                    }
                }
            }
        }"#;

        let report = parse_k6_summary_json(json).expect("Parse k6 summary JSON");
        assert!(!report.all_thresholds_passed);
        assert_eq!(report.failed_thresholds.len(), 2);
        assert!(
            report
                .failed_thresholds
                .iter()
                .any(|f| f.contains("http_req_duration") && f.contains("p(95)<2000"))
        );
        assert!(
            report
                .failed_thresholds
                .iter()
                .any(|f| f.contains("http_req_failed") && f.contains("rate<0.01"))
        );
    }

    #[test]
    fn test_parse_k6_summary_json_custom_metrics() {
        let json = r#"{
            "metrics": {
                "search_response_time": {
                    "type": "trend",
                    "values": {
                        "avg": 820.0,
                        "p(99)": 4200.0
                    },
                    "thresholds": {
                        "p(99)<5000": { "ok": true }
                    }
                },
                "chaos_errors": {
                    "type": "rate",
                    "values": {
                        "rate": 0.02
                    },
                    "thresholds": {
                        "rate<0.05": { "ok": true }
                    }
                }
            }
        }"#;

        let report = parse_k6_summary_json(json).expect("Parse custom k6 metrics");
        assert!(report.all_thresholds_passed);
        assert_eq!(report.p99_duration_ms, Some(4200.0));
        assert_eq!(report.http_req_failed_rate, Some(0.02));
    }

    #[test]
    fn test_parse_k6_summary_json_empty_and_malformed() {
        let empty_json = r#"{"metrics": {}}"#;
        let report = parse_k6_summary_json(empty_json).expect("Parse empty metrics");
        assert!(report.all_thresholds_passed);
        assert_eq!(report.avg_duration_ms, 0);

        let malformed = "not valid json";
        assert!(parse_k6_summary_json(malformed).is_err());
    }

    #[test]
    fn test_k6_runner_configuration() {
        let runner = K6Runner::new();
        assert_eq!(runner.k6_cmd(), "k6");

        let custom = K6Runner::with_k6_cmd("custom-k6");
        assert_eq!(custom.k6_cmd(), "custom-k6");
    }

    #[test]
    fn test_maestro_runner_validate_flow_definition() {
        let valid_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: Login with Biometric
- assertVisible:
    text: Welcome, SDET Engineer
"#;
        assert!(MaestroRunner::validate_flow_definition(valid_yaml).is_ok());

        let valid_quoted_yaml = r#"
---
- launchApp:
    appId: com.cherenkov.bankapp
- tapOn:
    text: View Balance
- assertVisible:
    text: "Account Balance: USD 1000"
- setOrientation:
    orientation: landscape
- assertVisible:
    text: "Account Balance: USD 1000"
"#;
        assert!(MaestroRunner::validate_flow_definition(valid_quoted_yaml).is_ok());

        let invalid_yaml_tabs = "-\tlaunchApp:\n\tappId: com.cherenkov.bankapp\n";
        assert!(MaestroRunner::validate_flow_definition(invalid_yaml_tabs).is_err());

        let empty_yaml = "";
        assert!(MaestroRunner::validate_flow_definition(empty_yaml).is_err());
    }

    #[tokio::test]
    async fn test_maestro_runner_run_drill_on_actual_drill_file() {
        let runner = MaestroRunner::new();
        let file = "exercises/03_mobile_maestro/01_biometric_fallback/solution.yaml";
        if Path::new(file).exists() {
            let response = runner
                .run_drill(file, "", 2, 5000)
                .await
                .expect("Run drill");
            assert!(response.ok);
            assert!(response.passed);
            assert_eq!(response.iterations, 2);
            assert_eq!(response.passed_iterations, 2);
        }
    }

    #[tokio::test]
    async fn test_maestro_runner_nonexistent_file() {
        let runner = MaestroRunner::new();
        let response = runner
            .run_drill("non_existent_flow.yaml", "", 1, 5000)
            .await
            .expect("Runner execution");
        assert!(!response.ok);
        assert!(!response.passed);
        assert!(response.error.is_some());
    }

    #[test]
    fn test_parse_jmeter_jtl_csv_all_passed() {
        let csv_data = r#"timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage,bytes,sentBytes,grpThreads,allThreads,URL,Latency,IdleTime,Connect
1700000000000,20,GET /api/v1/health,200,OK,Thread Group 1-1,text,true,,120,50,1,1,http://localhost:8081/api/v1/health,18,0,2
1700000000100,40,GET /api/v1/products,200,OK,Thread Group 1-1,text,true,,2048,50,1,1,http://localhost:8081/api/v1/products,35,0,5
1700000000200,80,POST /api/v1/orders,201,Created,Thread Group 1-1,text,true,,512,200,1,1,http://localhost:8081/api/v1/orders,75,0,5
1700000000300,120,GET /api/v1/orders/1,200,OK,Thread Group 1-1,text,true,,400,50,1,1,http://localhost:8081/api/v1/orders/1,110,0,10
"#;
        let metrics = parse_jmeter_jtl_csv(csv_data).expect("Parse JTL CSV");
        assert_eq!(metrics.total_samples, 4);
        assert_eq!(metrics.passed_samples, 4);
        assert_eq!(metrics.failed_samples, 0);
        assert_eq!(metrics.error_rate, 0.0);
        assert_eq!(metrics.min_elapsed_ms, 20);
        assert_eq!(metrics.max_elapsed_ms, 120);
        assert_eq!(metrics.avg_elapsed_ms, 65); // (20+40+80+120)/4 = 65
        assert_eq!(metrics.p90_elapsed_ms, 120);
        assert_eq!(metrics.p95_elapsed_ms, 120);
        assert_eq!(metrics.p99_elapsed_ms, 120);
        assert!(metrics.first_failure_reason.is_none());
        assert_eq!(metrics.samples.len(), 4);
        assert_eq!(metrics.samples[0].label, "GET /api/v1/health");
        assert_eq!(metrics.samples[0].response_code, "200");
    }

    #[test]
    fn test_parse_jmeter_jtl_csv_with_failures() {
        let csv_data = r#"timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage
1700000000000,50,GET /api/v1/catalog,200,OK,Thread Group 1-1,text,true,
1700000000100,500,POST /api/v1/checkout,500,Internal Server Error,Thread Group 1-1,text,false,Test failed: code expected to match /200/
1700000000200,30,GET /api/v1/cart,200,OK,Thread Group 1-1,text,true,
1700000000300,600,POST /api/v1/payment,503,Service Unavailable,Thread Group 1-1,text,false,Gateway timeout
1700000000400,20,GET /api/v1/user,200,OK,Thread Group 1-1,text,true,
"#;
        let metrics = parse_jmeter_jtl_csv(csv_data).expect("Parse JTL CSV");
        assert_eq!(metrics.total_samples, 5);
        assert_eq!(metrics.passed_samples, 3);
        assert_eq!(metrics.failed_samples, 2);
        assert!((metrics.error_rate - 0.4).abs() < 0.001);
        assert_eq!(metrics.min_elapsed_ms, 20);
        assert_eq!(metrics.max_elapsed_ms, 600);
        assert_eq!(metrics.avg_elapsed_ms, 240); // (20+30+50+500+600)/5 = 240
        assert!(metrics.first_failure_reason.is_some());
        let reason = metrics.first_failure_reason.unwrap();
        assert!(reason.contains("POST /api/v1/checkout"));
        assert!(reason.contains("500"));
        assert!(reason.contains("Test failed: code expected to match /200/"));
    }

    #[test]
    fn test_parse_jmeter_jtl_csv_percentiles_100_samples() {
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
        assert_eq!(metrics.avg_elapsed_ms, 51); // (1+100)*100/2 / 100 = 50.5 -> 51
        assert_eq!(metrics.p90_elapsed_ms, 90);
        assert_eq!(metrics.p95_elapsed_ms, 95);
        assert_eq!(metrics.p99_elapsed_ms, 99);
    }

    #[test]
    fn test_parse_jmeter_jtl_csv_quoted_fields_and_commas() {
        let csv_data = r#"timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage
1700000000000,15,"GET /api/v1/items?filter=a,b,c",200,"OK, processed successfully",Thread Group 1-1,text,true,""
1700000000100,250,"POST /api/v1/items,batch",400,"Bad Request, invalid JSON",Thread Group 1-1,text,false,"Validation failed, field 'name' is required"
"#;
        let metrics = parse_jmeter_jtl_csv(csv_data).expect("Parse JTL CSV with quotes");
        assert_eq!(metrics.total_samples, 2);
        assert_eq!(metrics.passed_samples, 1);
        assert_eq!(metrics.failed_samples, 1);
        assert_eq!(metrics.samples[0].label, "GET /api/v1/items?filter=a,b,c");
        assert_eq!(
            metrics.samples[0].response_message,
            "OK, processed successfully"
        );
        assert_eq!(metrics.samples[1].label, "POST /api/v1/items,batch");
        assert_eq!(
            metrics.samples[1].failure_message,
            "Validation failed, field 'name' is required"
        );
    }

    #[test]
    fn test_parse_jmeter_jtl_csv_empty_and_corrupt() {
        let empty = "";
        let metrics_empty = parse_jmeter_jtl_csv(empty).expect("Parse empty");
        assert_eq!(metrics_empty.total_samples, 0);

        let header_only = "timeStamp,elapsed,label,responseCode,responseMessage,threadName,dataType,success,failureMessage\n";
        let metrics_header = parse_jmeter_jtl_csv(header_only).expect("Parse header only");
        assert_eq!(metrics_header.total_samples, 0);
    }

    #[test]
    fn test_jmeter_runner_options_and_default() {
        let runner = JMeterRunner::new();
        assert_eq!(runner.jmeter_cmd(), "jmeter");

        let default_runner = JMeterRunner::default();
        assert_eq!(default_runner.jmeter_cmd(), "jmeter");

        let custom = JMeterRunner::with_jmeter_cmd("custom-jmeter-path");
        assert_eq!(custom.jmeter_cmd(), "custom-jmeter-path");
    }

    #[tokio::test]
    async fn test_jmeter_runner_missing_binary_graceful_handling() {
        let runner = JMeterRunner::with_jmeter_cmd("nonexistent-jmeter-bin-xyz-99999");
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_plan_sample.jmx");
        std::fs::write(&test_file, "<jmeterTestPlan></jmeterTestPlan>").expect("Write test plan");

        let response = runner
            .run_drill(test_file.to_str().unwrap(), "", 2, 5000)
            .await
            .expect("Runner execution should not panic or return Err");

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
    async fn test_jmeter_runner_nonexistent_exercise_file() {
        let runner = JMeterRunner::new();
        let response = runner
            .run_drill("non_existent_plan.jmx", "", 1, 5000)
            .await
            .expect("Runner execution");
        assert!(!response.ok);
        assert!(!response.passed);
        assert!(response.error.is_some());
        assert!(response.error.unwrap().contains("does not exist"));
    }

    #[test]
    fn test_any_runner_jmeter_wrapping() {
        let runner = Arc::new(JMeterRunner::new());
        let any_runner = AnyRunner::Jmeter(runner);
        match any_runner {
            AnyRunner::Jmeter(r) => {
                assert_eq!(r.jmeter_cmd(), "jmeter");
            }
            _ => panic!("Expected AnyRunner::Jmeter"),
        }
    }

    #[test]
    fn test_pytest_runner_options_and_default() {
        let runner = PytestRunner::new();
        assert_eq!(runner.python_cmd(), "python");
        assert_eq!(runner.worker_script(), Path::new("workers/pytest_worker.py"));

        let default_runner = PytestRunner::default();
        assert_eq!(default_runner.python_cmd(), "python");

        let custom = PytestRunner::with_config("python3", PathBuf::from("custom/worker.py"));
        assert_eq!(custom.python_cmd(), "python3");
        assert_eq!(custom.worker_script(), Path::new("custom/worker.py"));
    }

    #[tokio::test]
    async fn test_pytest_runner_nonexistent_file() {
        let runner = PytestRunner::new();
        let response = runner
            .run_drill("non_existent_exercise_file_9999.py", "", 1, 5000)
            .await
            .expect("Runner execution");
        assert!(!response.ok);
        assert!(!response.passed);
        assert!(response.error.is_some());
        assert!(response.error.unwrap().contains("does not exist"));
    }

    #[test]
    fn test_any_runner_pytest_wrapping() {
        let runner = Arc::new(PytestRunner::new());
        let any_runner = AnyRunner::Pytest(runner);
        match any_runner {
            AnyRunner::Pytest(r) => {
                assert_eq!(r.python_cmd(), "python");
            }
            _ => panic!("Expected AnyRunner::Pytest"),
        }
    }
}
