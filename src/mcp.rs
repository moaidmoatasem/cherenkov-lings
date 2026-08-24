use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use crate::feedback;
use std::path::Path;
use std::fs;

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
                if line.is_empty() { continue; }
                
                if let Ok(req) = serde_json::from_str::<RpcRequest>(line) {
                    let mut res = RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: req.id.clone().unwrap_or(Value::Null),
                        result: None,
                        error: None,
                    };

                    match req.method.as_str() {
                        "initialize" => {
                            res.result = Some(json!({
                                "serverInfo": {
                                    "name": "cherenkov-lings-mcp",
                                    "version": "1.0.0"
                                },
                                "capabilities": {
                                    "tools": {}
                                }
                            }));
                        },
                        "tools/list" => {
                            res.result = Some(json!({
                                "tools": [
                                    {
                                        "name": "get_diagnostic_report",
                                        "description": "Analyzes an exercise file and returns AST anti-patterns.",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "file_path": { "type": "string" }
                                            },
                                            "required": ["file_path"]
                                        }
                                    },
                                    {
                                        "name": "get_hints",
                                        "description": "Returns progressive hints for an exercise.",
                                        "inputSchema": {
                                            "type": "object",
                                            "properties": {
                                                "exercise_dir": { "type": "string" }
                                            },
                                            "required": ["exercise_dir"]
                                        }
                                    }
                                ]
                            }));
                        },
                        "tools/call" => {
                            if let Some(params) = req.params {
                                let name = params["name"].as_str().unwrap_or("");
                                let args = &params["arguments"];
                                
                                match name {
                                    "get_diagnostic_report" => {
                                        let path = args["file_path"].as_str().unwrap_or("");
                                        if let Ok(report) = feedback::analyze_file(Path::new(path)) {
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
                                    },
                                    "get_hints" => {
                                        let dir = args["exercise_dir"].as_str().unwrap_or("");
                                        let hints_path = Path::new(dir).join("hints.md");
                                        let content = fs::read_to_string(&hints_path)
                                            .unwrap_or_else(|_| "No hints found.".to_string());
                                        res.result = Some(json!({
                                            "content": [{ "type": "text", "text": content }]
                                        }));
                                    },
                                    _ => {
                                        res.error = Some(json!({ "code": -32601, "message": "Method not found" }));
                                    }
                                }
                            }
                        },
                        "notifications/initialized" => continue,
                        _ => {
                            res.error = Some(json!({ "code": -32601, "message": "Method not found" }));
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
