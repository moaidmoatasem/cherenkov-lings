use crate::feedback;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::Path;

/// Protocol version echoed back when a client does not name one in `initialize`.
const DEFAULT_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct RpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

/// Render a single progressive hint level for a drill directory.
///
/// Deliberately returns one level at a time rather than the whole `hints.md`:
/// the last level of every drill is a solution diff, so dumping the file hands
/// the learner the answer on the first request.
fn render_hint(exercise_dir: &Path, args: &Value) -> Option<String> {
    let mut hints = feedback::ProgressiveHints::load_from_dir(exercise_dir);

    let dir_str = exercise_dir.to_string_lossy();
    let topic = args.get("topic").and_then(Value::as_str).unwrap_or("");
    let is_telemetry = dir_str.contains("otel")
        || dir_str.contains("telemetry")
        || dir_str.contains("tracing")
        || topic == "telemetry"
        || topic == "otel"
        || topic == "span_id_correlation"
        || topic.contains("telemetry")
        || topic.contains("otel");

    if hints.is_none() && is_telemetry {
        hints = Some(feedback::ProgressiveHints::telemetry_hints());
    }

    let hints = hints?;

    let selected = match args.get("score").and_then(Value::as_f64) {
        Some(score) => hints.get_hint_for_score(score),
        None => {
            let level = args.get("level").and_then(Value::as_u64).unwrap_or(1) as usize;
            hints.get_hint_at_level(level)
        }
    };

    let (level, total, text) = selected?;
    let footer = if level < total {
        format!(
            "Hint {} of {}. Let the learner attempt a fix first; call get_hints again with \"level\": {} only if they are still stuck.",
            level,
            total,
            level + 1
        )
    } else {
        format!(
            "Hint {} of {} — final level, this reveals the solution.",
            level, total
        )
    };

    Some(format!("{}\n\n---\n{}", text, footer))
}

pub fn run_mcp_server() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = stdin.lock();

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                if let Ok(req) = serde_json::from_str::<RpcRequest>(line) {
                    let mut res = RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id.clone().unwrap_or(Value::Null),
                        result: None,
                        error: None,
                    };

                    match req.method.as_str() {
                        "initialize" => {
                            // `protocolVersion` is required by the MCP spec; clients
                            // that validate InitializeResult reject a handshake without it.
                            let requested = req
                                .params
                                .as_ref()
                                .and_then(|p| p.get("protocolVersion"))
                                .and_then(Value::as_str)
                                .unwrap_or(DEFAULT_PROTOCOL_VERSION);
                            res.result = Some(json!({
                                "protocolVersion": requested,
                                "serverInfo": {
                                    "name": "cherenkov-lings-mcp",
                                    "version": "1.0.0"
                                },
                                "capabilities": {
                                    "tools": {}
                                }
                            }));
                        }
                        "tools/list" => {
                            res.result = Some(json!({
                                "tools": [
                                    {
                                        "name": "get_diagnostic_report",
                                        "description": "Runs static source analysis (regex-based anti-pattern and locator scanning) on an exercise file and returns detected anti-patterns plus locator quality scoring.",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "file_path": { "type": "string", "description": "Path to the exercise file to analyze." }
                                            },
                                            "required": ["file_path"]
                                        }
                                    },
                                    {
                                        "name": "get_hints",
                                        "description": "Returns ONE progressive hint level for a drill. Defaults to level 1 (a conceptual nudge). Escalate one level at a time, and only while the learner is still stuck — the final level contains the solution diff.",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "exercise_dir": { "type": "string", "description": "Directory of the drill, containing its hints.md." },
                                                "level": { "type": "integer", "minimum": 1, "description": "1-based hint level, clamped to the levels available. Defaults to 1." },
                                                "score": { "type": "number", "description": "Optional 4D Matrix total score. When supplied, the level is derived from the score and `level` is ignored." },
                                                "topic": { "type": "string", "description": "Optional challenge topic (e.g. 'telemetry', 'otel', 'span_id_correlation')." }
                                            },
                                            "required": ["exercise_dir"]
                                        }
                                    }
                                ]
                            }));
                        }
                        "tools/call" => {
                            if let Some(params) = req.params {
                                let name = params["name"].as_str().unwrap_or("");
                                let args = &params["arguments"];

                                match name {
                                    "get_diagnostic_report" => {
                                        let path = args["file_path"].as_str().unwrap_or("");
                                        if let Ok(report) = feedback::analyze_file(Path::new(path))
                                        {
                                            // Ensure we implement or debug print
                                            let json_report = json!({
                                                "anti_patterns": report.anti_patterns.iter().map(|ap| {
                                                    json!({
                                                        "snippet": ap.snippet,
                                                        "explanation": ap.explanation,
                                                        "recommendation": ap.recommendation
                                                    })
                                                }).collect::<Vec<_>>(),
                                                "locators": report.locators.iter().map(|l| {
                                                    json!({ "selector": l.selector, "score": l.score })
                                                }).collect::<Vec<_>>(),
                                                "score": report.locator_quality_score
                                            });
                                            res.result = Some(json!({
                                                "content": [{ "type": "text", "text": json_report.to_string() }]
                                            }));
                                        } else {
                                            res.result = Some(json!({
                                                "content": [{ "type": "text", "text": "Error analyzing file" }]
                                            }));
                                        }
                                    }
                                    "get_hints" => {
                                        let dir = args["exercise_dir"].as_str().unwrap_or("");
                                        let text = render_hint(Path::new(dir), args)
                                            .unwrap_or_else(|| "No hints found.".to_string());
                                        res.result = Some(json!({
                                            "content": [{ "type": "text", "text": text }]
                                        }));
                                    }
                                    _ => {
                                        res.error = Some(
                                            json!({ "code": -32601, "message": "Method not found" }),
                                        );
                                    }
                                }
                            }
                        }
                        "notifications/initialized" => continue,
                        _ => {
                            res.error =
                                Some(json!({ "code": -32601, "message": "Method not found" }));
                        }
                    }

                    if let Ok(res_str) = serde_json::to_string(&res) {
                        println!("{}", res_str);
                        stdout.flush().unwrap();
                    }
                }
            }
            Err(_) => break,
        }
    }
}
