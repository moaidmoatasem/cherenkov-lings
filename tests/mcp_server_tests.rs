//! The MCP stdio server had no test coverage at all: a JSON-RPC surface that
//! external agents drive, where a broken dispatch would only ever surface as a
//! confusing failure inside somebody else's tool.
//!
//! These drive the real binary over stdin/stdout, which is exactly how a client
//! uses it — a unit test of the handler would not catch a regression in the
//! NDJSON framing, and the framing is half the contract.

use std::fs;
use std::io::Write;
use std::path::Path;
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

#[test]
fn get_diagnostic_report_detects_java_performance_traps() {
    let test_dir = Path::new("target/test_samples");
    fs::create_dir_all(test_dir).expect("create test_samples dir");
    let sample_path = test_dir.join("SampleJavaTraps.java");

    let sample_java = r#"
package com.cherenkov.api;

import io.restassured.RestAssured;
import org.junit.jupiter.api.Test;
import static io.restassured.RestAssured.given;
import static io.restassured.module.jsv.JsonSchemaValidator.matchesJsonSchemaInClasspath;

public class SampleJavaTraps {
    @Test
    public void testUsers() {
        RestAssured.reset();
        given()
            .when()
            .get("/users")
            .then()
            .body(matchesJsonSchemaInClasspath("schemas/users.json"));
    }
}
"#;
    fs::write(&sample_path, sample_java).expect("write sample java file");
    let normalized_path = sample_path.to_string_lossy().replace('\\', "/");

    let req = format!(
        r#"{{"jsonrpc":"2.0","id":10,"method":"tools/call","params":{{"name":"get_diagnostic_report","arguments":{{"file_path":"{}"}}}}}}"#,
        normalized_path
    );
    let responses = rpc(&[&req]);

    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let report: serde_json::Value = serde_json::from_str(text).expect("diagnostic report json");

    let anti_patterns = report["anti_patterns"]
        .as_array()
        .expect("anti_patterns array");

    assert!(
        anti_patterns.len() >= 3,
        "Expected at least 3 Java performance traps, found {}: {:?}",
        anti_patterns.len(),
        anti_patterns
    );

    let texts = anti_patterns
        .iter()
        .map(|ap| {
            format!(
                "{} {} {}",
                ap["snippet"], ap["explanation"], ap["recommendation"]
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        texts.contains("RestAssured.reset")
            || texts.contains("client churn")
            || texts.contains("connection pools"),
        "Should detect RestAssured.reset() client churn in: {texts}"
    );
    assert!(
        texts.contains("timeout") || texts.contains("socket"),
        "Should detect missing socket/connection timeouts in: {texts}"
    );
    assert!(
        texts.contains("matchesJsonSchema") || texts.contains("schema"),
        "Should detect repeated schema reloads in: {texts}"
    );

    let _ = fs::remove_file(sample_path);
}

#[test]
fn get_diagnostic_report_detects_python_performance_traps() {
    let test_dir = Path::new("target/test_samples");
    fs::create_dir_all(test_dir).expect("create test_samples dir");
    let sample_path = test_dir.join("sample_python_traps.py");

    let sample_py = r#"
import pytest
import time
import requests
from sqlalchemy import create_engine

@pytest.fixture
def db():
    return create_engine("sqlite:///test.db")

def test_sync():
    session = requests.Session()
    resp = session.get("http://localhost:8080")
    assert resp.status_code == 200

async def test_async():
    time.sleep(1)
    response = requests.get("http://localhost:8080")
    assert response.status_code == 200
"#;
    fs::write(&sample_path, sample_py).expect("write sample python file");
    let normalized_path = sample_path.to_string_lossy().replace('\\', "/");

    let req = format!(
        r#"{{"jsonrpc":"2.0","id":11,"method":"tools/call","params":{{"name":"get_diagnostic_report","arguments":{{"file_path":"{}"}}}}}}"#,
        normalized_path
    );
    let responses = rpc(&[&req]);

    let text = responses[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("content text");
    let report: serde_json::Value = serde_json::from_str(text).expect("diagnostic report json");

    let anti_patterns = report["anti_patterns"]
        .as_array()
        .expect("anti_patterns array");

    assert!(
        anti_patterns.len() >= 3,
        "Expected at least 3 Python performance traps, found {}: {:?}",
        anti_patterns.len(),
        anti_patterns
    );

    let texts = anti_patterns
        .iter()
        .map(|ap| {
            format!(
                "{} {} {}",
                ap["snippet"], ap["explanation"], ap["recommendation"]
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        texts.contains("time.sleep") || texts.contains("sleep"),
        "Should detect time.sleep in: {texts}"
    );
    assert!(
        texts.contains("async") || texts.contains("blocking"),
        "Should detect blocking calls in async function in: {texts}"
    );
    assert!(
        texts.contains("Session")
            || texts.contains("unclosed")
            || texts.contains("context manager"),
        "Should detect unclosed client session in: {texts}"
    );
    assert!(
        texts.contains("fixture") || texts.contains("scope"),
        "Should detect inefficient fixture scope in: {texts}"
    );

    let _ = fs::remove_file(sample_path);
}

#[test]
fn get_hints_returns_progressive_3_tier_otel_guidance() {
    // 1. Test calling with exercise_dir pointing to exercises/02_api_pytest/02_otel_distributed_trace
    let drill_dir = "exercises/02_api_pytest/02_otel_distributed_trace";

    // Level 1: Architectural Nudge
    let req_l1 = format!(
        r#"{{"jsonrpc":"2.0","id":21,"method":"tools/call","params":{{"name":"get_hints","arguments":{{"exercise_dir":"{}","level":1}}}}}}"#,
        drill_dir
    );
    let res_l1 = rpc(&[&req_l1]);
    let text_l1 = res_l1[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(
        text_l1.contains("Hint 1") && text_l1.contains("Architectural Nudge"),
        "Tier 1 must return Architectural Nudge, got: {text_l1}"
    );
    assert!(
        text_l1.contains("traceparent") && text_l1.contains("trace_id"),
        "Tier 1 must explain distributed trace context propagation and traceparent"
    );
    assert!(
        text_l1.contains("200") && (text_l1.contains("async") || text_l1.contains("out-of-band")),
        "Tier 1 must explain why HTTP 200 checks alone miss async backend side-effects"
    );
    assert!(
        !text_l1.contains("Hint 3") && !text_l1.contains("Code Diff"),
        "Tier 1 must not leak Tier 3 code diff"
    );

    // Level 2: API Pattern with Span ID correlation details
    let req_l2 = format!(
        r#"{{"jsonrpc":"2.0","id":22,"method":"tools/call","params":{{"name":"get_hints","arguments":{{"exercise_dir":"{}","level":2}}}}}}"#,
        drill_dir
    );
    let res_l2 = rpc(&[&req_l2]);
    let text_l2 = res_l2[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(
        text_l2.contains("Hint 2") && text_l2.contains("API Pattern"),
        "Tier 2 must return API Pattern, got: {text_l2}"
    );
    assert!(
        text_l2.contains("00-")
            || text_l2.contains("00-{trace_id}-{parent_id}-01")
            || text_l2.contains("version-trace_id"),
        "Tier 2 must detail W3C traceparent formatting"
    );
    assert!(
        text_l2.contains("Span ID")
            || text_l2.contains("client_span_id")
            || text_l2.contains("parent_span_id"),
        "Tier 2 must explain Span ID correlation"
    );
    assert!(
        text_l2.contains("spans") || text_l2.contains("telemetry"),
        "Tier 2 must explain querying OTel traces/spans for parent-child tree assertions"
    );
    assert!(
        !text_l2.contains("Hint 3") && !text_l2.contains("Code Diff"),
        "Tier 2 must not leak Tier 3 code diff"
    );

    // Level 3: Code Diff
    let req_l3 = format!(
        r#"{{"jsonrpc":"2.0","id":23,"method":"tools/call","params":{{"name":"get_hints","arguments":{{"exercise_dir":"{}","level":3}}}}}}"#,
        drill_dir
    );
    let res_l3 = rpc(&[&req_l3]);
    let text_l3 = res_l3[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(
        text_l3.contains("Hint 3") && text_l3.contains("Code Diff"),
        "Tier 3 must return Code Diff, got: {text_l3}"
    );
    assert!(
        text_l3.contains("traceparent") && text_l3.contains("diff"),
        "Tier 3 must show diff with W3C traceparent injection"
    );
    assert!(
        text_l3.contains("spans") || text_l3.contains("wait_for_spans"),
        "Tier 3 must show asserting distributed span status and parent-child correlation"
    );

    // 2. Test calling with topic: "telemetry" with generic/fallback dir
    let req_topic_l1 = r#"{"jsonrpc":"2.0","id":24,"method":"tools/call","params":{"name":"get_hints","arguments":{"exercise_dir":"generic_drill","topic":"telemetry","level":1}}}"#;
    let res_topic_l1 = rpc(&[req_topic_l1]);
    let text_topic_l1 = res_topic_l1[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(
        text_topic_l1.contains("Architectural Nudge") && text_topic_l1.contains("traceparent"),
        "topic: 'telemetry' should trigger ProgressiveHints::telemetry_hints() fallback for Tier 1: {text_topic_l1}"
    );

    let req_topic_l2 = r#"{"jsonrpc":"2.0","id":25,"method":"tools/call","params":{"name":"get_hints","arguments":{"exercise_dir":"generic_drill","topic":"telemetry","level":2}}}"#;
    let res_topic_l2 = rpc(&[req_topic_l2]);
    let text_topic_l2 = res_topic_l2[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(
        text_topic_l2.contains("API Pattern")
            && (text_topic_l2.contains("00-") || text_topic_l2.contains("traceparent")),
        "topic: 'telemetry' should trigger ProgressiveHints::telemetry_hints() fallback for Tier 2: {text_topic_l2}"
    );

    let req_topic_l3 = r#"{"jsonrpc":"2.0","id":26,"method":"tools/call","params":{"name":"get_hints","arguments":{"exercise_dir":"generic_drill","topic":"telemetry","level":3}}}"#;
    let res_topic_l3 = rpc(&[req_topic_l3]);
    let text_topic_l3 = res_topic_l3[0]["result"]["content"][0]["text"]
        .as_str()
        .expect("text");
    assert!(
        text_topic_l3.contains("Code Diff") && text_topic_l3.contains("traceparent"),
        "topic: 'telemetry' should trigger ProgressiveHints::telemetry_hints() fallback for Tier 3: {text_topic_l3}"
    );
}

#[test]
fn tools_list_advertises_topic_property_for_get_hints() {
    let responses = rpc(&[r#"{"jsonrpc":"2.0","id":27,"method":"tools/list"}"#]);
    let tools = responses[0]["result"]["tools"]
        .as_array()
        .expect("tools must be an array");

    let hints_tool = tools
        .iter()
        .find(|t| t["name"] == "get_hints")
        .expect("get_hints tool must exist");

    let properties = &hints_tool["inputSchema"]["properties"];
    assert!(
        properties.get("topic").is_some(),
        "get_hints schema must advertise the optional 'topic' property"
    );
    assert_eq!(
        properties["topic"]["type"], "string",
        "topic property must be of type string"
    );
}
