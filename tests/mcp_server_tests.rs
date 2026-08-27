//! The MCP stdio server had no test coverage at all: a JSON-RPC surface that
//! external agents drive, where a broken dispatch would only ever surface as a
//! confusing failure inside somebody else's tool.
//!
//! These drive the real binary over stdin/stdout, which is exactly how a client
//! uses it — a unit test of the handler would not catch a regression in the
//! NDJSON framing, and the framing is half the contract.

use std::io::Write;
use std::process::{Command, Stdio};

/// Feed newline-delimited requests to `cherenkov-lings mcp` and collect the
/// responses, keyed by request id.
fn rpc(requests: &[&str]) -> Vec<serde_json::Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cherenkov-lings"))
        .arg("mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the mcp subcommand must start");

    {
        let stdin = child.stdin.as_mut().expect("stdin is piped");
        for request in requests {
            writeln!(stdin, "{request}").expect("write request");
        }
        // Dropping stdin closes it, which is how the server is told to stop.
    }

    let output = child.wait_with_output().expect("mcp server must exit");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("each line must be one JSON object"))
        .collect()
}

#[test]
fn initialize_advertises_the_protocol_and_tool_capability() {
    let responses = rpc(&[r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#]);
    assert_eq!(responses.len(), 1, "one request, one response line");

    let result = &responses[0]["result"];
    assert_eq!(responses[0]["id"], 1);
    assert_eq!(result["protocolVersion"], "2024-11-05");
    assert_eq!(result["serverInfo"]["name"], "cherenkov-lings-mcp");
    assert!(
        result["capabilities"]["tools"].is_object(),
        "a client that sees no tools capability will never call tools/list"
    );
}

#[test]
fn tools_list_advertises_both_tools_with_their_required_arguments() {
    let responses = rpc(&[r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#]);
    let tools = responses[0]["result"]["tools"]
        .as_array()
        .expect("tools must be an array");

    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    assert_eq!(names, vec!["get_diagnostic_report", "get_hints"]);

    for tool in tools {
        assert!(
            !tool["description"].as_str().unwrap_or("").is_empty(),
            "{} has no description; a client picks tools by description",
            tool["name"]
        );
        assert!(
            tool["inputSchema"]["required"].is_array(),
            "{} must declare its required arguments",
            tool["name"]
        );
    }
}

#[test]
fn get_hints_returns_the_first_level_for_a_real_drill() {
    let responses = rpc(&[
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_hints","arguments":{"exercise_dir":"exercises/00_foundations/01_what_is_a_test","level":1}}}"#,
    ]);

    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("content must carry text");

    assert!(
        text.contains("Hint 1"),
        "level 1 should be the conceptual nudge, got: {text}"
    );
    assert!(
        !text.contains("Hint 3"),
        "one call must not leak later levels — the last one carries the solution: {text}"
    );
}

#[test]
fn get_diagnostic_report_finds_the_anti_pattern_in_a_drill_exercise() {
    let responses = rpc(&[
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"get_diagnostic_report","arguments":{"file_path":"exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts"}}}"#,
    ]);

    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("content must carry text");
    let report: serde_json::Value =
        serde_json::from_str(text).expect("the diagnostic report is JSON inside the text block");

    let anti_patterns = report["anti_patterns"]
        .as_array()
        .expect("anti_patterns must be an array");
    assert!(
        !anti_patterns.is_empty(),
        "the hydration drill ships a hardcoded sleep; finding none means the scan is not running"
    );
    assert!(
        report["score"].is_number(),
        "locator score must be reported"
    );
}

#[test]
fn an_unknown_method_is_an_error_rather_than_a_silent_empty_result() {
    let responses = rpc(&[r#"{"jsonrpc":"2.0","id":5,"method":"nope/unknown"}"#]);
    assert_eq!(responses[0]["error"]["code"], -32601);
}

#[test]
fn requests_are_answered_in_order_on_one_connection() {
    // The framing half of the contract: a client pipelines requests and matches
    // responses by id, so a dropped or reordered line breaks it.
    let responses = rpc(&[
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"nope/unknown"}"#,
    ]);

    let ids: Vec<i64> = responses.iter().filter_map(|r| r["id"].as_i64()).collect();
    assert_eq!(ids, vec![1, 2, 3]);
    for response in &responses {
        assert_eq!(response["jsonrpc"], "2.0");
    }
}
