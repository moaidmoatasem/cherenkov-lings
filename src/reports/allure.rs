use crate::reports::chaos_dataset::{
    generate_chaos_dataset, ChaosTestResult, FailureCategory, TestStatus,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// High-level summary metrics of an Allure & Chaos report run
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllureReportSummary {
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub broken: usize,
    pub flaky: usize,
    pub skipped: usize,
    pub real_bugs: usize,
    pub flaky_infra: usize,
    pub anti_patterns: usize,
    pub duration_ms: u64,
    pub pass_percentage: f64,
    pub results_dir: String,
    pub report_html_path: String,
    pub generated_at: String,
}

/// Standard Allure JSON Test Result Schema for compatibility with Allure CLI
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllureTestResultJson {
    pub uuid: String,
    pub history_id: String,
    pub full_name: String,
    pub name: String,
    pub status: String,
    pub status_details: AllureStatusDetails,
    pub stage: String,
    pub start: u64,
    pub stop: u64,
    pub description: String,
    pub parameters: Vec<AllureParameter>,
    pub labels: Vec<AllureLabel>,
    pub steps: Vec<AllureStep>,
    pub attachments: Vec<AllureAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AllureStatusDetails {
    pub message: Option<String>,
    pub trace: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllureParameter {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllureLabel {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllureStep {
    pub name: String,
    pub status: String,
    pub stage: String,
    pub start: u64,
    pub stop: u64,
    pub status_details: AllureStatusDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AllureAttachment {
    pub name: String,
    pub source: String,
    pub r#type: String,
}

/// Category definition for Allure categories.json
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AllureCategoryDef {
    pub name: String,
    pub matched_statuses: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message_regex: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_regex: Option<String>,
}

/// Compute summary statistics from a chaotic test dataset
pub fn summarize_dataset(
    dataset: &[ChaosTestResult],
    results_dir: &str,
    report_html_path: &str,
) -> AllureReportSummary {
    let total_tests = dataset.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut broken = 0;
    let mut flaky = 0;
    let mut skipped = 0;
    let mut real_bugs = 0;
    let mut flaky_infra = 0;
    let mut anti_patterns = 0;
    let mut duration_ms = 0;

    for test in dataset {
        duration_ms += test.duration_ms;
        match test.status {
            TestStatus::Passed => passed += 1,
            TestStatus::Failed => failed += 1,
            TestStatus::Broken => broken += 1,
            TestStatus::Flaky => flaky += 1,
            TestStatus::Skipped => skipped += 1,
        }

        match test.category {
            FailureCategory::RealBug => real_bugs += 1,
            FailureCategory::FlakyInfra => flaky_infra += 1,
            FailureCategory::AntiPattern => anti_patterns += 1,
            FailureCategory::None => {}
        }
    }

    let pass_percentage = if total_tests > 0 {
        (passed as f64 / total_tests as f64) * 100.0
    } else {
        0.0
    };

    AllureReportSummary {
        total_tests,
        passed,
        failed,
        broken,
        flaky,
        skipped,
        real_bugs,
        flaky_infra,
        anti_patterns,
        duration_ms,
        pass_percentage,
        results_dir: results_dir.to_string(),
        report_html_path: report_html_path.to_string(),
        generated_at: crate::gamification::current_utc_iso_timestamp(),
    }
}

/// Primary Entrypoint: Generate standard Allure-compatible JSON results and interactive HTML report
pub fn generate_chaos_allure_report(output_dir: &Path) -> std::io::Result<AllureReportSummary> {
    let dataset = generate_chaos_dataset();
    generate_allure_report_for_dataset(&dataset, output_dir)
}

/// Generate Allure report for a specific custom test dataset
pub fn generate_allure_report_for_dataset(
    dataset: &[ChaosTestResult],
    output_dir: &Path,
) -> std::io::Result<AllureReportSummary> {
    // 1. Create output directories
    let results_dir = output_dir.join("allure-results");
    let report_dir = output_dir.join("allure-report");
    fs::create_dir_all(&results_dir)?;
    fs::create_dir_all(&report_dir)?;

    // Also ensure target/allure-results exists for standard Allure CLI workflows
    let default_target_results = Path::new("target/allure-results");
    if !default_target_results.exists() {
        let _ = fs::create_dir_all(default_target_results);
    }

    // 2. Write raw Allure JSON files to results directory
    generate_allure_results(dataset, &results_dir)?;
    if default_target_results.exists() && default_target_results != results_dir {
        let _ = generate_allure_results(dataset, default_target_results);
    }

    // 3. Write categories.json, environment.properties, executor.json
    write_allure_metadata_files(&results_dir)?;
    if default_target_results.exists() && default_target_results != results_dir {
        let _ = write_allure_metadata_files(default_target_results);
    }

    // 4. Generate self-contained interactive HTML report
    let primary_html_path = output_dir.join("index.html");
    let secondary_html_path = report_dir.join("index.html");

    let summary = summarize_dataset(
        dataset,
        &results_dir.to_string_lossy(),
        &primary_html_path.to_string_lossy(),
    );

    let html_content = render_html_report_string(dataset, &summary);
    fs::write(&primary_html_path, &html_content)?;
    fs::write(&secondary_html_path, &html_content)?;

    Ok(summary)
}

/// Generate individual `<uuid>-result.json` files conforming to the Allure 2 test result schema
pub fn generate_allure_results(
    dataset: &[ChaosTestResult],
    results_dir: &Path,
) -> std::io::Result<usize> {
    fs::create_dir_all(results_dir)?;
    let base_start_time = 1756054800000u64; // Deterministic base timestamp in ms

    let mut count = 0;
    for (idx, test) in dataset.iter().enumerate() {
        let safe_id: String = test
            .test_id
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        let test_uuid = format!("cherenkov-{:04x}-{}", idx + 1, safe_id.to_lowercase());
        let start_time = base_start_time + (idx as u64 * 500);
        let stop_time = start_time + test.duration_ms;

        // Construct labels
        let mut labels = vec![
            AllureLabel {
                name: "suite".to_string(),
                value: test.suite.clone(),
            },
            AllureLabel {
                name: "track".to_string(),
                value: test.track_id.clone(),
            },
            AllureLabel {
                name: "testId".to_string(),
                value: test.test_id.clone(),
            },
            AllureLabel {
                name: "category".to_string(),
                value: test.category.to_string(),
            },
            AllureLabel {
                name: "framework".to_string(),
                value: "cherenkov-matrix".to_string(),
            },
            AllureLabel {
                name: "language".to_string(),
                value: match test.track_id.as_str() {
                    "playwright-ts" => "typescript".to_string(),
                    "restassured-java" => "java".to_string(),
                    "k6-js" => "javascript".to_string(),
                    "devsecops-python" | "foundations" => "python".to_string(),
                    "jmeter" => "jmeter".to_string(),
                    "maestro-mobile" => "yaml".to_string(),
                    _ => "rust".to_string(),
                },
            },
        ];

        for (k, v) in &test.labels {
            if k != "suite" && k != "track" && k != "framework" {
                labels.push(AllureLabel {
                    name: k.clone(),
                    value: v.clone(),
                });
            }
        }

        // Construct parameters
        let mut parameters = vec![
            AllureParameter {
                name: "Failure Category".to_string(),
                value: test.category.display_name().to_string(),
            },
            AllureParameter {
                name: "Execution Duration".to_string(),
                value: format!("{}ms", test.duration_ms),
            },
        ];

        if let Some(ref chaos) = test.chaos_event {
            parameters.push(AllureParameter {
                name: "Chaos Layer".to_string(),
                value: chaos.layer.clone(),
            });
            parameters.push(AllureParameter {
                name: "Chaos Event Type".to_string(),
                value: chaos.event_type.clone(),
            });
            if chaos.latency_ms > 0 {
                parameters.push(AllureParameter {
                    name: "Injected Latency".to_string(),
                    value: format!("{}ms (±{}ms)", chaos.latency_ms, chaos.jitter_ms),
                });
            }
            if chaos.packet_loss_rate > 0.0 {
                parameters.push(AllureParameter {
                    name: "Packet Loss Rate".to_string(),
                    value: format!("{:.1}%", chaos.packet_loss_rate * 100.0),
                });
            }
        }

        // Construct steps
        let mut steps = Vec::new();
        let mut current_step_start = start_time;
        for s in &test.steps {
            let step_stop = current_step_start + s.duration_ms;
            steps.push(AllureStep {
                name: s.name.clone(),
                status: s.status.to_string(),
                stage: "finished".to_string(),
                start: current_step_start,
                stop: step_stop,
                status_details: AllureStatusDetails {
                    message: s.error.clone(),
                    trace: None,
                },
            });
            current_step_start = step_stop;
        }

        // Construct attachments
        let mut attachments = Vec::new();
        if let Some(ref chaos) = test.chaos_event {
            if let Some(ref proxy_log) = chaos.proxy_log {
                let attach_filename = format!("{}-chaos-log.txt", test_uuid);
                let attach_path = results_dir.join(&attach_filename);
                let _ = fs::write(&attach_path, proxy_log);
                attachments.push(AllureAttachment {
                    name: "Chaos Telemetry Log".to_string(),
                    source: attach_filename,
                    r#type: "text/plain".to_string(),
                });
            }
        }

        let full_name = format!("com.cherenkov.{}.{}", test.track_id, test.name);
        let status_str = match test.status {
            TestStatus::Passed => "passed",
            TestStatus::Failed => "failed",
            TestStatus::Broken => "broken",
            TestStatus::Flaky => "failed", // Allure records flaky runs as failed with flaky label
            TestStatus::Skipped => "skipped",
        };

        let result_json = AllureTestResultJson {
            uuid: test_uuid.clone(),
            history_id: format!("hist-{}", test.test_id.to_lowercase()),
            full_name,
            name: test.name.clone(),
            status: status_str.to_string(),
            status_details: AllureStatusDetails {
                message: test.error_message.clone(),
                trace: test.stack_trace.clone(),
            },
            stage: "finished".to_string(),
            start: start_time,
            stop: stop_time,
            description: format!(
                "Track: {} | Suite: {} | Category: {}",
                test.track_id,
                test.suite,
                test.category.display_name()
            ),
            parameters,
            labels,
            steps,
            attachments,
        };

        let json_filename = format!("{}-result.json", test_uuid);
        let json_path = results_dir.join(json_filename);
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = serde_json::to_string_pretty(&result_json)?;
        fs::write(json_path, serialized)?;
        count += 1;
    }

    Ok(count)
}

/// Write standard Allure metadata files (categories.json, environment.properties, executor.json)
pub fn write_allure_metadata_files(results_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(results_dir)?;

    // 1. categories.json
    let categories = vec![
        AllureCategoryDef {
            name: "Genuine Product Defects".to_string(),
            matched_statuses: vec!["failed".to_string(), "broken".to_string()],
            message_regex: Some(".*(500|403|Forbidden|NullPointer|Deadlock|ConstraintViolation|Overflow|Bypass|IDOR|AxeViolation).*".to_string()),
            trace_regex: None,
        },
        AllureCategoryDef {
            name: "Flaky Infrastructure Failures".to_string(),
            matched_statuses: vec!["failed".to_string(), "broken".to_string()],
            message_regex: Some(".*(ChaosProxy|502|504|SocketException|Connection reset|TimeoutError|TIME_WAIT|gaierror|DNS).*".to_string()),
            trace_regex: None,
        },
        AllureCategoryDef {
            name: "Test Automation Anti-Patterns".to_string(),
            matched_statuses: vec!["failed".to_string(), "broken".to_string()],
            message_regex: Some(".*(waitForTimeout|StaleElementReference|NoAssertionsExecuted|KeyError|XPath|ThresholdError|UnhandledPromiseRejection|AmbiguityError|Gitleaks).*".to_string()),
            trace_regex: None,
        },
    ];
    let categories_path = results_dir.join("categories.json");
    let cat_json = serde_json::to_string_pretty(&categories)?;
    fs::write(categories_path, cat_json)?;

    // 2. environment.properties
    let env_props = "Platform=Cherenkov-Lings Enterprise SDET Simulator\n\
                     Version=1.0.0\n\
                     Engine=Cherenkov Chaos Matrix 4D\n\
                     ChaosProxy=Active (Port 8086)\n\
                     ChaosL4DropRate=25%\n\
                     ChaosL7Latency=0-3500ms\n\
                     OS=Cross-Platform (Windows/Linux/macOS)\n\
                     ZeroCloud=Enabled (100% Local First)\n";
    fs::write(results_dir.join("environment.properties"), env_props)?;

    // 3. executor.json
    let executor_json = serde_json::json!({
        "name": "Cherenkov-Lings Local CI Runner",
        "type": "cherenkov-matrix",
        "url": "http://localhost:8080",
        "buildOrder": 1,
        "buildName": "Enterprise SDET Simulator Run #1",
        "buildUrl": "http://localhost:8080/dashboard",
        "reportUrl": "allure-report/index.html"
    });
    fs::write(
        results_dir.join("executor.json"),
        serde_json::to_string_pretty(&executor_json)?,
    )?;

    Ok(())
}

/// Generate a complete, standalone, self-contained interactive HTML report with embedded styles and JS
pub fn generate_interactive_html_report(
    dataset: &[ChaosTestResult],
    report_path: &Path,
) -> std::io::Result<()> {
    if let Some(parent) = report_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let summary = summarize_dataset(
        dataset,
        "target/allure-results",
        &report_path.to_string_lossy(),
    );
    let html = render_html_report_string(dataset, &summary);
    fs::write(report_path, html)?;
    Ok(())
}

/// Render the complete standalone HTML string for the interactive Allure Chaos Report
pub fn render_html_report_string(
    dataset: &[ChaosTestResult],
    summary: &AllureReportSummary,
) -> String {
    let mut tests_json = Vec::new();
    for t in dataset {
        tests_json.push(serde_json::json!({
            "id": t.test_id,
            "name": t.name,
            "suite": t.suite,
            "track": t.track_id,
            "status": t.status.to_string(),
            "category": t.category.to_string(),
            "categoryName": t.category.display_name(),
            "duration": t.duration_ms,
            "error": t.error_message,
            "stackTrace": t.stack_trace,
            "chaos": t.chaos_event,
            "flakiness": t.flakiness_metrics,
            "steps": t.steps,
            "labels": t.labels,
            "rootCauseHint": t.root_cause_hint
        }));
    }
    let tests_payload_json = serde_json::to_string(&tests_json).unwrap_or_else(|_| "[]".to_string());

    format!(r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Cherenkov-Lings — Enterprise Allure & Chaos Test Report</title>
  <style>
    :root {{
      --bg-main: #0b0f19;
      --bg-card: #111827;
      --bg-card-hover: #1e293b;
      --border-color: #1f293d;
      --text-main: #f3f4f6;
      --text-muted: #9ca3af;
      --primary: #38bdf8;
      --primary-hover: #0ea5e9;
      --status-passed: #10b981;
      --status-failed: #ef4444;
      --status-broken: #f59e0b;
      --status-flaky: #a855f7;
      --cat-bug: #f43f5e;
      --cat-infra: #3b82f6;
      --cat-antipattern: #ec4899;
      --font-mono: 'Consolas', 'Fira Code', 'Courier New', monospace;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; }}
    body {{
      background-color: var(--bg-main);
      color: var(--text-main);
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      line-height: 1.5;
      padding-bottom: 60px;
    }}
    header {{
      background: linear-gradient(180deg, #131d33 0%, var(--bg-main) 100%);
      border-bottom: 1px solid var(--border-color);
      padding: 24px 32px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 16px;
    }}
    .brand {{
      display: flex;
      align-items: center;
      gap: 12px;
    }}
    .brand-icon {{
      width: 36px;
      height: 36px;
      background: linear-gradient(135deg, #0284c7 0%, #38bdf8 100%);
      border-radius: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      font-weight: 900;
      color: #0b0f19;
      font-size: 20px;
    }}
    .brand-title h1 {{
      font-size: 20px;
      font-weight: 700;
      letter-spacing: -0.5px;
      color: #ffffff;
    }}
    .brand-title p {{
      font-size: 12px;
      color: var(--text-muted);
    }}
    .meta-badge-group {{
      display: flex;
      gap: 8px;
      flex-wrap: wrap;
    }}
    .meta-badge {{
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      padding: 6px 12px;
      border-radius: 6px;
      font-size: 12px;
      color: var(--text-muted);
    }}
    .meta-badge strong {{ color: var(--text-main); }}

    .container {{
      max-width: 1400px;
      margin: 0 auto;
      padding: 24px 32px;
    }}

    /* Metrics Grid */
    .kpi-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
      gap: 16px;
      margin-bottom: 24px;
    }}
    .kpi-card {{
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      border-radius: 10px;
      padding: 16px 20px;
      position: relative;
      overflow: hidden;
    }}
    .kpi-card .label {{
      font-size: 12px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: var(--text-muted);
      margin-bottom: 6px;
    }}
    .kpi-card .value {{
      font-size: 26px;
      font-weight: 800;
      color: #ffffff;
    }}
    .kpi-card .sub {{
      font-size: 11px;
      color: var(--text-muted);
      margin-top: 4px;
    }}
    .kpi-card.passed {{ border-top: 3px solid var(--status-passed); }}
    .kpi-card.failed {{ border-top: 3px solid var(--status-failed); }}
    .kpi-card.broken {{ border-top: 3px solid var(--status-broken); }}
    .kpi-card.flaky {{ border-top: 3px solid var(--status-flaky); }}
    .kpi-card.bugs {{ border-top: 3px solid var(--cat-bug); }}
    .kpi-card.infra {{ border-top: 3px solid var(--cat-infra); }}
    .kpi-card.anti {{ border-top: 3px solid var(--cat-antipattern); }}

    /* Charts Row */
    .charts-row {{
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 20px;
      margin-bottom: 28px;
    }}
    @media (max-width: 900px) {{
      .charts-row {{ grid-template-columns: 1fr; }}
    }}
    .chart-panel {{
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      border-radius: 10px;
      padding: 20px;
    }}
    .chart-panel h3 {{
      font-size: 14px;
      font-weight: 600;
      margin-bottom: 16px;
      color: var(--text-main);
      display: flex;
      align-items: center;
      gap: 8px;
    }}
    .bar-chart-container {{
      display: flex;
      flex-direction: column;
      gap: 10px;
    }}
    .bar-row {{
      display: flex;
      align-items: center;
      gap: 12px;
      font-size: 12px;
    }}
    .bar-label {{
      width: 140px;
      color: var(--text-muted);
    }}
    .bar-track {{
      flex: 1;
      height: 18px;
      background: #1e293b;
      border-radius: 4px;
      overflow: hidden;
      display: flex;
    }}
    .bar-fill {{
      height: 100%;
      transition: width 0.3s ease;
    }}
    .bar-val {{
      width: 50px;
      text-align: right;
      font-weight: 600;
      color: var(--text-main);
    }}

    /* Filters Toolbar */
    .toolbar {{
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      border-radius: 10px;
      padding: 14px 20px;
      margin-bottom: 20px;
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      align-items: center;
      justify-content: space-between;
    }}
    .filter-buttons {{
      display: flex;
      gap: 6px;
      flex-wrap: wrap;
    }}
    .btn-filter {{
      background: #1e293b;
      border: 1px solid var(--border-color);
      color: var(--text-muted);
      padding: 6px 14px;
      border-radius: 6px;
      font-size: 12px;
      cursor: pointer;
      font-weight: 500;
      transition: all 0.2s;
    }}
    .btn-filter:hover {{
      background: #334155;
      color: #ffffff;
    }}
    .btn-filter.active {{
      background: var(--primary);
      border-color: var(--primary);
      color: #0b0f19;
      font-weight: 700;
    }}
    .search-box input {{
      background: #0f172a;
      border: 1px solid var(--border-color);
      color: #ffffff;
      padding: 8px 14px;
      border-radius: 6px;
      font-size: 13px;
      min-width: 260px;
      outline: none;
    }}
    .search-box input:focus {{
      border-color: var(--primary);
    }}

    /* Test Table */
    .table-container {{
      background: var(--bg-card);
      border: 1px solid var(--border-color);
      border-radius: 10px;
      overflow: hidden;
    }}
    table.test-table {{
      width: 100%;
      border-collapse: collapse;
      text-align: left;
      font-size: 13px;
    }}
    table.test-table th {{
      background: #162032;
      padding: 12px 16px;
      color: var(--text-muted);
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      border-bottom: 1px solid var(--border-color);
    }}
    table.test-table td {{
      padding: 12px 16px;
      border-bottom: 1px solid var(--border-color);
      vertical-align: middle;
    }}
    table.test-table tr.test-row {{
      cursor: pointer;
      transition: background-color 0.15s;
    }}
    table.test-table tr.test-row:hover {{
      background-color: var(--bg-card-hover);
    }}
    table.test-table tr.test-row.active {{
      background-color: #1e293b;
    }}

    /* Badges */
    .status-badge {{
      display: inline-block;
      padding: 3px 8px;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 700;
      text-transform: uppercase;
    }}
    .status-badge.passed {{ background: rgba(16, 185, 129, 0.15); color: var(--status-passed); border: 1px solid var(--status-passed); }}
    .status-badge.failed {{ background: rgba(239, 68, 68, 0.15); color: var(--status-failed); border: 1px solid var(--status-failed); }}
    .status-badge.broken {{ background: rgba(245, 158, 11, 0.15); color: var(--status-broken); border: 1px solid var(--status-broken); }}
    .status-badge.flaky {{ background: rgba(168, 85, 247, 0.15); color: var(--status-flaky); border: 1px solid var(--status-flaky); }}

    .cat-badge {{
      display: inline-block;
      padding: 3px 8px;
      border-radius: 4px;
      font-size: 11px;
      font-weight: 600;
    }}
    .cat-badge.real_bug {{ background: rgba(244, 63, 94, 0.15); color: var(--cat-bug); border: 1px solid var(--cat-bug); }}
    .cat-badge.flaky_infra {{ background: rgba(59, 130, 246, 0.15); color: var(--cat-infra); border: 1px solid var(--cat-infra); }}
    .cat-badge.anti_pattern {{ background: rgba(236, 72, 153, 0.15); color: var(--cat-antipattern); border: 1px solid var(--cat-antipattern); }}
    .cat-badge.none {{ background: rgba(16, 185, 129, 0.1); color: var(--status-passed); }}

    .id-tag {{
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--primary);
      font-weight: 700;
    }}

    /* Detail Drawer */
    .detail-drawer {{
      display: none;
      background: #0d1322;
      border-bottom: 1px solid var(--border-color);
      padding: 20px 24px;
    }}
    .detail-drawer.open {{
      display: table-row;
    }}
    .drawer-content {{
      padding: 10px 0;
    }}
    .drawer-tabs {{
      display: flex;
      gap: 8px;
      border-bottom: 1px solid var(--border-color);
      margin-bottom: 16px;
    }}
    .tab-btn {{
      background: none;
      border: none;
      border-bottom: 2px solid transparent;
      color: var(--text-muted);
      padding: 8px 16px;
      font-size: 13px;
      font-weight: 600;
      cursor: pointer;
    }}
    .tab-btn.active {{
      color: var(--primary);
      border-bottom-color: var(--primary);
    }}
    .tab-pane {{
      display: none;
    }}
    .tab-pane.active {{
      display: block;
    }}

    .code-block {{
      background: #050811;
      border: 1px solid #1e293b;
      border-radius: 6px;
      padding: 14px;
      font-family: var(--font-mono);
      font-size: 12px;
      color: #e2e8f0;
      overflow-x: auto;
      white-space: pre-wrap;
      line-height: 1.5;
    }}
    .telemetry-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
      gap: 12px;
      margin-bottom: 16px;
    }}
    .telemetry-item {{
      background: #131c2e;
      border: 1px solid #1e293b;
      padding: 10px 14px;
      border-radius: 6px;
    }}
    .telemetry-item .t-label {{
      font-size: 11px;
      color: var(--text-muted);
      text-transform: uppercase;
    }}
    .telemetry-item .t-val {{
      font-size: 14px;
      font-weight: 700;
      color: #ffffff;
      margin-top: 2px;
    }}

    .triage-action-box {{
      background: linear-gradient(135deg, #1e1b4b 0%, #172554 100%);
      border: 1px solid #3b82f6;
      border-radius: 8px;
      padding: 16px 20px;
      margin-top: 16px;
      display: flex;
      justify-content: space-between;
      align-items: center;
      flex-wrap: wrap;
      gap: 12px;
    }}
    .triage-action-box .desc h4 {{
      font-size: 14px;
      color: #ffffff;
      margin-bottom: 4px;
    }}
    .triage-action-box .desc p {{
      font-size: 12px;
      color: #93c5fd;
    }}
    .btn-triage-cli {{
      background: var(--primary);
      color: #0b0f19;
      padding: 8px 16px;
      border-radius: 6px;
      font-weight: 700;
      font-size: 12px;
      border: none;
      cursor: pointer;
      display: flex;
      align-items: center;
      gap: 6px;
    }}
    .btn-triage-cli:hover {{
      background: var(--primary-hover);
    }}
  </style>
</head>
<body>

<header>
  <div class="brand">
    <div class="brand-icon">⚡</div>
    <div class="brand-title">
      <h1>CHERENKOV-LINGS ALLURE REPORT</h1>
      <p>Enterprise SDET Simulator & Chaos Matrix Telemetry</p>
    </div>
  </div>
  <div class="meta-badge-group">
    <div class="meta-badge">Generated: <strong>{generated_at}</strong></div>
    <div class="meta-badge">Framework: <strong>Cherenkov 4D Matrix</strong></div>
    <div class="meta-badge">Chaos Proxy: <strong>Active (L4/L7)</strong></div>
    <div class="meta-badge">Mode: <strong>Local-First (Zero Cloud)</strong></div>
  </div>
</header>

<div class="container">

  <!-- KPI Cards -->
  <div class="kpi-grid">
    <div class="kpi-card">
      <div class="label">Total Tests</div>
      <div class="value">{total_tests}</div>
      <div class="sub">100% Deterministic</div>
    </div>
    <div class="kpi-card passed">
      <div class="label">Passed Rate</div>
      <div class="value">{pass_percentage:.1}%</div>
      <div class="sub">{passed} Resilient Tests</div>
    </div>
    <div class="kpi-card failed">
      <div class="label">Failed</div>
      <div class="value">{failed}</div>
      <div class="sub">Assertion Failures</div>
    </div>
    <div class="kpi-card broken">
      <div class="label">Broken</div>
      <div class="value">{broken}</div>
      <div class="sub">Runtime / Config Errors</div>
    </div>
    <div class="kpi-card flaky">
      <div class="label">Flaky Iterations</div>
      <div class="value">{flaky}</div>
      <div class="sub">Chaos Sensitive</div>
    </div>
    <div class="kpi-card bugs">
      <div class="label">Product Bugs</div>
      <div class="value">{real_bugs}</div>
      <div class="sub">Genuine Defects</div>
    </div>
    <div class="kpi-card infra">
      <div class="label">Flaky Infra</div>
      <div class="value">{flaky_infra}</div>
      <div class="sub">Proxy / Network Jitter</div>
    </div>
    <div class="kpi-card anti">
      <div class="label">Anti-Patterns</div>
      <div class="value">{anti_patterns}</div>
      <div class="sub">Sleeps / Bad Locators</div>
    </div>
  </div>

  <!-- Charts Row -->
  <div class="charts-row">
    <div class="chart-panel">
      <h3>📊 Execution Status Breakdown</h3>
      <div class="bar-chart-container">
        <div class="bar-row">
          <div class="bar-label">Passed</div>
          <div class="bar-track"><div class="bar-fill" style="width: {p_pct:.1}%; background: var(--status-passed);"></div></div>
          <div class="bar-val">{passed}</div>
        </div>
        <div class="bar-row">
          <div class="bar-label">Failed</div>
          <div class="bar-track"><div class="bar-fill" style="width: {f_pct:.1}%; background: var(--status-failed);"></div></div>
          <div class="bar-val">{failed}</div>
        </div>
        <div class="bar-row">
          <div class="bar-label">Broken</div>
          <div class="bar-track"><div class="bar-fill" style="width: {b_pct:.1}%; background: var(--status-broken);"></div></div>
          <div class="bar-val">{broken}</div>
        </div>
        <div class="bar-row">
          <div class="bar-label">Flaky (Chaos)</div>
          <div class="bar-track"><div class="bar-fill" style="width: {fl_pct:.1}%; background: var(--status-flaky);"></div></div>
          <div class="bar-val">{flaky}</div>
        </div>
      </div>
    </div>

    <div class="chart-panel">
      <h3>🔬 Root-Cause Taxonomy Breakdown</h3>
      <div class="bar-chart-container">
        <div class="bar-row">
          <div class="bar-label">Genuine Product Defects</div>
          <div class="bar-track"><div class="bar-fill" style="width: {bug_pct:.1}%; background: var(--cat-bug);"></div></div>
          <div class="bar-val">{real_bugs}</div>
        </div>
        <div class="bar-row">
          <div class="bar-label">Flaky Infrastructure</div>
          <div class="bar-track"><div class="bar-fill" style="width: {inf_pct:.1}%; background: var(--cat-infra);"></div></div>
          <div class="bar-val">{flaky_infra}</div>
        </div>
        <div class="bar-row">
          <div class="bar-label">Test Anti-Patterns</div>
          <div class="bar-track"><div class="bar-fill" style="width: {anti_pct:.1}%; background: var(--cat-antipattern);"></div></div>
          <div class="bar-val">{anti_patterns}</div>
        </div>
        <div class="bar-row">
          <div class="bar-label">Healthy Tests</div>
          <div class="bar-track"><div class="bar-fill" style="width: {p_pct:.1}%; background: var(--status-passed);"></div></div>
          <div class="bar-val">{passed}</div>
        </div>
      </div>
    </div>
  </div>

  <!-- Filters & Search Toolbar -->
  <div class="toolbar">
    <div class="filter-buttons">
      <button class="btn-filter active" onclick="filterByStatus('all', this)">All ({total_tests})</button>
      <button class="btn-filter" onclick="filterByStatus('passed', this)">Passed ({passed})</button>
      <button class="btn-filter" onclick="filterByStatus('failed', this)">Failed ({failed})</button>
      <button class="btn-filter" onclick="filterByStatus('broken', this)">Broken ({broken})</button>
      <button class="btn-filter" onclick="filterByStatus('flaky', this)">Flaky ({flaky})</button>
      <span style="border-left: 1px solid var(--border-color); margin: 0 4px;"></span>
      <button class="btn-filter" onclick="filterByCategory('real_bug', this)">Bugs ({real_bugs})</button>
      <button class="btn-filter" onclick="filterByCategory('flaky_infra', this)">Infra ({flaky_infra})</button>
      <button class="btn-filter" onclick="filterByCategory('anti_pattern', this)">Anti-Patterns ({anti_patterns})</button>
    </div>
    <div class="search-box">
      <input type="text" id="searchInput" placeholder="Search by test ID, name, error..." oninput="handleSearch()">
    </div>
  </div>

  <!-- Test Table -->
  <div class="table-container">
    <table class="test-table" id="testTable">
      <thead>
        <tr>
          <th style="width: 100px;">ID</th>
          <th>Test Name & Suite</th>
          <th style="width: 140px;">Track</th>
          <th style="width: 110px;">Status</th>
          <th style="width: 180px;">Category</th>
          <th style="width: 90px; text-align: right;">Duration</th>
        </tr>
      </thead>
      <tbody id="testTableBody">
        <!-- Rendered by JS -->
      </tbody>
    </table>
  </div>

</div>

<script>
  const ALL_TESTS = {tests_payload_json};
  let currentStatusFilter = 'all';
  let currentCategoryFilter = 'all';
  let currentSearchQuery = '';

  function renderTable() {{
    const tbody = document.getElementById('testTableBody');
    tbody.innerHTML = '';

    const filtered = ALL_TESTS.filter(t => {{
      if (currentStatusFilter !== 'all' && t.status !== currentStatusFilter) return false;
      if (currentCategoryFilter !== 'all' && t.category !== currentCategoryFilter) return false;
      if (currentSearchQuery) {{
        const q = currentSearchQuery.toLowerCase();
        const text = (t.id + ' ' + t.name + ' ' + t.suite + ' ' + t.track + ' ' + (t.error || '')).toLowerCase();
        if (!text.includes(q)) return false;
      }}
      return true;
    }});

    filtered.forEach((t, idx) => {{
      const tr = document.createElement('tr');
      tr.className = 'test-row';
      tr.id = 'row-' + t.id;
      tr.onclick = () => toggleDrawer(t.id);

      tr.innerHTML = `
        <td><span class="id-tag">${{t.id}}</span></td>
        <td>
          <div style="font-weight: 600; color: #ffffff;">${{escapeHtml(t.name)}}</div>
          <div style="font-size: 11px; color: var(--text-muted);">${{escapeHtml(t.suite)}}</div>
        </td>
        <td><span style="font-size: 11px; color: #94a3b8; font-family: var(--font-mono);">${{t.track}}</span></td>
        <td><span class="status-badge ${{t.status}}">${{t.status}}</span></td>
        <td><span class="cat-badge ${{t.category}}">${{escapeHtml(t.categoryName)}}</span></td>
        <td style="text-align: right; font-family: var(--font-mono);">${{t.duration}}ms</td>
      `;
      tbody.appendChild(tr);

      // Drawer row
      const drawerTr = document.createElement('tr');
      drawerTr.className = 'detail-drawer';
      drawerTr.id = 'drawer-' + t.id;

      let chaosHtml = '<p style="color: var(--text-muted);">No correlated chaos event recorded for this test.</p>';
      if (t.chaos) {{
        chaosHtml = `
          <div class="telemetry-grid">
            <div class="telemetry-item"><div class="t-label">Injected Layer</div><div class="t-val">${{t.chaos.layer}}</div></div>
            <div class="telemetry-item"><div class="t-label">Event Type</div><div class="t-val">${{t.chaos.event_type}}</div></div>
            <div class="telemetry-item"><div class="t-label">Latency / Jitter</div><div class="t-val">${{t.chaos.latency_ms}}ms (±${{t.chaos.jitter_ms}}ms)</div></div>
            <div class="telemetry-item"><div class="t-label">Packet Loss</div><div class="t-val">${{(t.chaos.packet_loss_rate * 100).toFixed(1)}}%</div></div>
          </div>
          ${{t.chaos.proxy_log ? `
            <div style="font-size: 12px; font-weight: 600; color: var(--text-muted); margin-bottom: 6px;">Correlated Proxy L4/L7 Log:</div>
            <div class="code-block" style="color: #67e8f9;">${{escapeHtml(t.chaos.proxy_log)}}</div>
          ` : ''}}
        `;
      }}

      let stepsHtml = '';
      if (t.steps && t.steps.length > 0) {{
        stepsHtml = `
          <table style="width: 100%; border-collapse: collapse; font-size: 12px; margin-top: 8px;">
            ${{t.steps.map(s => `
              <tr style="border-bottom: 1px solid #1e293b;">
                <td style="padding: 6px 10px; color: #ffffff;">${{escapeHtml(s.name)}}</td>
                <td style="padding: 6px 10px; width: 90px;"><span class="status-badge ${{s.status}}">${{s.status}}</span></td>
                <td style="padding: 6px 10px; width: 70px; text-align: right; font-family: var(--font-mono); color: var(--text-muted);">${{s.duration_ms}}ms</td>
              </tr>
            `).join('')}}
          </table>
        `;
      }}

      drawerTr.innerHTML = `
        <td colspan="6" style="padding: 0;">
          <div class="detail-drawer-content" style="padding: 20px 24px;">
            <div class="drawer-tabs">
              <button class="tab-btn active" onclick="switchTab(event, '${{t.id}}', 'tab-error')">Stack Trace & Error</button>
              <button class="tab-btn" onclick="switchTab(event, '${{t.id}}', 'tab-chaos')">Correlated L4/L7 Chaos Telemetry</button>
              <button class="tab-btn" onclick="switchTab(event, '${{t.id}}', 'tab-steps')">Execution Steps (${{t.steps ? t.steps.length : 0}})</button>
              <button class="tab-btn" onclick="switchTab(event, '${{t.id}}', 'tab-triage')">💡 Triage Solver</button>
            </div>

            <div id="${{t.id}}-tab-error" class="tab-pane active">
              ${{t.error ? `<div style="color: #f87171; font-weight: 600; margin-bottom: 8px; font-size: 13px;">${{escapeHtml(t.error)}}</div>` : '<p style="color: var(--status-passed);">Test passed cleanly with zero errors.</p>'}}
              ${{t.stackTrace ? `<div class="code-block">${{escapeHtml(t.stackTrace)}}</div>` : ''}}
            </div>

            <div id="${{t.id}}-tab-chaos" class="tab-pane">
              ${{chaosHtml}}
            </div>

            <div id="${{t.id}}-tab-steps" class="tab-pane">
              ${{stepsHtml || '<p style="color: var(--text-muted);">No steps recorded.</p>'}}
            </div>

            <div id="${{t.id}}-tab-triage" class="tab-pane">
              <div class="triage-action-box">
                <div class="desc">
                  <h4>🎯 Investigate Root-Cause in Terminal Triage</h4>
                  <p>Run the interactive hypothesis wizard to score root cause and earn SDET XP.</p>
                </div>
                <button class="btn-triage-cli" onclick="copyCliCommand('${{t.id}}')">
                  📋 Copy: cherenkov-lings triage --test-id ${{t.id}}
                </button>
              </div>
              ${{t.rootCauseHint ? `
                <div style="margin-top: 12px; font-size: 12px; color: #a5b4fc; background: rgba(99, 102, 241, 0.1); border: 1px solid rgba(99, 102, 241, 0.3); border-radius: 6px; padding: 10px 14px;">
                  <strong>Senior SDET Analysis Note:</strong> ${{escapeHtml(t.rootCauseHint)}}
                </div>
              ` : ''}}
            </div>
          </div>
        </td>
      `;
      tbody.appendChild(drawerTr);
    }});
  }}

  function toggleDrawer(testId) {{
    const drawer = document.getElementById('drawer-' + testId);
    const row = document.getElementById('row-' + testId);
    if (!drawer) return;
    const isOpen = drawer.classList.contains('open');
    document.querySelectorAll('.detail-drawer').forEach(d => d.classList.remove('open'));
    document.querySelectorAll('.test-row').forEach(r => r.classList.remove('active'));
    if (!isOpen) {{
      drawer.classList.add('open');
      row.classList.add('active');
    }}
  }}

  function switchTab(evt, testId, tabId) {{
    evt.stopPropagation();
    const parent = evt.target.closest('.detail-drawer-content');
    parent.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
    parent.querySelectorAll('.tab-pane').forEach(p => p.classList.remove('active'));
    evt.target.classList.add('active');
    const targetPane = document.getElementById(testId + '-' + tabId);
    if (targetPane) targetPane.classList.add('active');
  }}

  function filterByStatus(status, btn) {{
    currentStatusFilter = status;
    document.querySelectorAll('.filter-buttons .btn-filter').forEach(b => b.classList.remove('active'));
    if (btn) btn.classList.add('active');
    renderTable();
  }}

  function filterByCategory(cat, btn) {{
    currentCategoryFilter = (currentCategoryFilter === cat) ? 'all' : cat;
    document.querySelectorAll('.filter-buttons .btn-filter').forEach(b => b.classList.remove('active'));
    if (currentCategoryFilter !== 'all' && btn) btn.classList.add('active');
    renderTable();
  }}

  function handleSearch() {{
    currentSearchQuery = document.getElementById('searchInput').value.trim();
    renderTable();
  }}

  function copyCliCommand(testId) {{
    const cmd = `cherenkov-lings triage --test-id ${{testId}}`;
    navigator.clipboard.writeText(cmd).then(() => {{
      alert('Copied to clipboard:\n' + cmd);
    }}).catch(() => {{
      prompt('Copy command:', cmd);
    }});
  }}

  function escapeHtml(str) {{
    if (!str) return '';
    return String(str)
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }}

  // Initial render
  renderTable();
</script>

</body>
</html>
"##,
        generated_at = summary.generated_at,
        total_tests = summary.total_tests,
        passed = summary.passed,
        failed = summary.failed,
        broken = summary.broken,
        flaky = summary.flaky,
        real_bugs = summary.real_bugs,
        flaky_infra = summary.flaky_infra,
        anti_patterns = summary.anti_patterns,
        pass_percentage = summary.pass_percentage,
        p_pct = (summary.passed as f64 / summary.total_tests.max(1) as f64) * 100.0,
        f_pct = (summary.failed as f64 / summary.total_tests.max(1) as f64) * 100.0,
        b_pct = (summary.broken as f64 / summary.total_tests.max(1) as f64) * 100.0,
        fl_pct = (summary.flaky as f64 / summary.total_tests.max(1) as f64) * 100.0,
        bug_pct = (summary.real_bugs as f64 / summary.total_tests.max(1) as f64) * 100.0,
        inf_pct = (summary.flaky_infra as f64 / summary.total_tests.max(1) as f64) * 100.0,
        anti_pct = (summary.anti_patterns as f64 / summary.total_tests.max(1) as f64) * 100.0,
        tests_payload_json = tests_payload_json,
    )
}
