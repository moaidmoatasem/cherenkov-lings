//! Sprint 5 Phase 4 coverage: the real-time `lint_on_type` fast path and the
//! Playwright trace ingestion that feeds the AI mentor's prompt.

use cherenkov_lings::review::{AiMentorClient, RuleScanner, Severity};
use std::path::Path;

// ---------------------------------------------------------------------------
// lint_on_type — the as-you-type fast path
// ---------------------------------------------------------------------------

#[test]
fn lint_on_type_flags_a_hardcoded_sleep_in_a_typescript_snippet() {
    let violations = RuleScanner::lint_on_type("await page.waitForTimeout(3000);");

    assert!(
        violations
            .iter()
            .any(|v| v.rule_id.contains("HARDCODED_SLEEP")),
        "waitForTimeout is the canonical flake source and must be caught: {violations:?}"
    );
}

#[test]
fn lint_on_type_works_without_a_file_extension_to_infer_language_from() {
    // The editor buffer has no path yet, so the scanner runs in Unknown-language
    // mode. That must not silently disable the rules — the whole feature is
    // warning a learner *before* they save.
    let python = RuleScanner::lint_on_type("time.sleep(5)");
    let java = RuleScanner::lint_on_type("Thread.sleep(2000);");

    assert!(
        !python.is_empty(),
        "a Python sleep must be caught in an unsaved buffer"
    );
    assert!(
        !java.is_empty(),
        "a Java sleep must be caught in an unsaved buffer"
    );
}

#[test]
fn lint_on_type_agrees_with_scan_content_on_the_same_snippet() {
    // lint_on_type is a convenience wrapper. If it ever diverges from the
    // full scanner, the learner gets one verdict while typing and a different
    // one on save, which is worse than no hint at all.
    let snippet = "await page.waitForTimeout(1500);";

    let fast_path = RuleScanner::lint_on_type(snippet);
    let full_scan = RuleScanner::scan_content("virtual_buffer.tmp", snippet);

    assert_eq!(
        fast_path.len(),
        full_scan.len(),
        "fast path and full scan disagree on violation count"
    );
    assert_eq!(
        fast_path.iter().map(|v| &v.rule_id).collect::<Vec<_>>(),
        full_scan.iter().map(|v| &v.rule_id).collect::<Vec<_>>(),
    );
}

#[test]
fn lint_on_type_returns_nothing_for_clean_code() {
    let violations =
        RuleScanner::lint_on_type("await expect(page.getByRole('button')).toBeVisible();");

    assert!(
        violations.is_empty(),
        "a well-written assertion must not produce a hint: {violations:?}"
    );
}

#[test]
fn lint_on_type_tolerates_empty_and_partial_input() {
    // Every keystroke calls this, including the first one, and a half-typed
    // line is the normal case rather than an edge case.
    assert!(RuleScanner::lint_on_type("").is_empty());
    assert!(RuleScanner::lint_on_type("   \n\t\n").is_empty());

    for partial in [
        "await page.",
        "await page.waitFor",
        "await page.waitForTimeout(",
    ] {
        // The contract is "does not panic and does not fabricate", not a
        // specific verdict on an incomplete expression.
        let _ = RuleScanner::lint_on_type(partial);
    }
}

#[test]
fn lint_on_type_reports_line_numbers_relative_to_the_snippet() {
    let snippet = "const a = 1;\nconst b = 2;\nawait page.waitForTimeout(500);";

    let violations = RuleScanner::lint_on_type(snippet);
    let sleep = violations
        .iter()
        .find(|v| v.rule_id.contains("HARDCODED_SLEEP"))
        .expect("sleep on line 3");

    assert_eq!(
        sleep.line_number, 3,
        "editor gutter markers depend on snippet-relative line numbers"
    );
}

#[test]
fn lint_on_type_flags_hardcoded_credentials_at_error_severity() {
    let violations = RuleScanner::lint_on_type(r#"const password = "hunter2secret";"#);

    let secret = violations
        .iter()
        .find(|v| v.rule_id.contains("SECRET"))
        .unwrap_or_else(|| panic!("plaintext credential not flagged: {violations:?}"));

    assert_eq!(
        secret.severity,
        Severity::Error,
        "a leaked credential is never a warning"
    );
}

#[test]
fn lint_on_type_ignores_anti_patterns_inside_comments() {
    let violations = RuleScanner::lint_on_type("// await page.waitForTimeout(3000);");

    assert!(
        violations.is_empty(),
        "commented-out code must not raise a live hint: {violations:?}"
    );
}

// ---------------------------------------------------------------------------
// ingest_trace_file — Playwright trace telemetry for the mentor prompt
// ---------------------------------------------------------------------------

fn mentor() -> AiMentorClient {
    AiMentorClient::new(None, None, true)
}

#[test]
fn ingest_trace_file_names_the_trace_it_summarizes() {
    let summary = mentor().ingest_trace_file(Path::new("artifacts/checkout-trace.zip"));

    assert!(
        summary.contains("checkout-trace.zip"),
        "the mentor prompt must say which trace it is quoting: {summary}"
    );
}

#[test]
fn ingest_trace_file_emits_telemetry_the_prompt_can_reason_about() {
    let summary = mentor().ingest_trace_file(Path::new("trace.zip"));

    for expected in ["Network Idle", "DOMContentLoaded", "Network Errors"] {
        assert!(
            summary.contains(expected),
            "trace summary is missing '{expected}': {summary}"
        );
    }
}

#[test]
fn ingest_trace_file_handles_a_bare_filename_and_a_nested_path_alike() {
    let m = mentor();
    let bare = m.ingest_trace_file(Path::new("trace.zip"));
    let nested = m.ingest_trace_file(Path::new("a/b/c/trace.zip"));

    assert_eq!(
        bare, nested,
        "only the file name should influence the summary, not the directory"
    );
}

#[test]
fn ingest_trace_file_does_not_panic_on_a_pathological_path() {
    // The path comes from a CLI argument, so an empty or directory-like value
    // is reachable. unwrap_or_default() on the file name is what keeps this
    // from panicking; this test pins that behaviour.
    let m = mentor();
    for path in ["", "..", "/", "trace/"] {
        let summary = m.ingest_trace_file(Path::new(path));
        assert!(
            summary.contains("Trace Telemetry"),
            "expected a usable summary for {path:?}, got {summary}"
        );
    }
}

#[test]
fn ingest_trace_file_output_is_a_single_prompt_line() {
    // The summary is concatenated into an LLM prompt; a stray newline would
    // break the surrounding prompt structure.
    let summary = mentor().ingest_trace_file(Path::new("trace.zip"));

    assert!(!summary.contains('\n'), "trace summary must stay one line");
    assert!(
        summary.starts_with('['),
        "summary should be bracketed: {summary}"
    );
    assert!(
        summary.ends_with(']'),
        "summary should be bracketed: {summary}"
    );
}
