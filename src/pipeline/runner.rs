use crate::pipeline::parser::{JobDefinition, StepDefinition, WorkflowDefinition};
use crate::pipeline::validator::{validate_definition, PipelineValidation, ValidationConfig};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineRunOptions {
    pub parallel: bool,
    pub fail_fast: bool,
    pub animated: bool,
    pub max_parallel: Option<usize>,
    pub verbose: bool,
    pub strict_validation: bool,
}

impl Default for PipelineRunOptions {
    fn default() -> Self {
        Self {
            parallel: true,
            fail_fast: false,
            animated: true,
            max_parallel: None,
            verbose: false,
            strict_validation: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineRunResult {
    pub workflow_name: String,
    pub jobs: Vec<JobRunResult>,
    pub duration_ms: u64,
    pub success: bool,
    pub logs: Vec<LogEntry>,
    pub validation: Option<PipelineValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JobRunResult {
    pub job_id: String,
    pub runner_name: String,
    pub matrix_combination: HashMap<String, String>,
    pub status: JobStatus,
    pub duration_ms: u64,
    pub steps: Vec<StepRunResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Passed,
    Failed,
    Cancelled,
    Skipped,
}

impl JobStatus {
    pub fn badge(&self) -> String {
        match self {
            JobStatus::Passed => "✔ PASSED".green().bold().to_string(),
            JobStatus::Failed => "✖ FAILED".red().bold().to_string(),
            JobStatus::Cancelled => "⊘ CANCELLED".yellow().bold().to_string(),
            JobStatus::Skipped => "○ SKIPPED".dimmed().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StepRunResult {
    pub name: String,
    pub status: StepStatus,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    Passed,
    Failed,
    Skipped,
}

impl StepStatus {
    pub fn badge(&self) -> String {
        match self {
            StepStatus::Passed => "✔".green().bold().to_string(),
            StepStatus::Failed => "✖".red().bold().to_string(),
            StepStatus::Skipped => "○".dimmed().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    pub timestamp: u64,
    pub runner: String,
    pub step: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

/// Executes a pipeline workflow from a file path.
pub fn run_pipeline(
    yaml_path: &Path,
    opts: &PipelineRunOptions,
) -> Result<PipelineRunResult, Box<dyn std::error::Error + Send + Sync>> {
    let workflow = crate::pipeline::parser::parse_workflow_file(yaml_path)?;
    Ok(run_workflow(&workflow, opts))
}

/// Runs a parsed workflow definition through the mock CI execution engine.
pub fn run_workflow(workflow: &WorkflowDefinition, opts: &PipelineRunOptions) -> PipelineRunResult {
    let start_time = Instant::now();
    let workflow_name = workflow
        .name
        .clone()
        .unwrap_or_else(|| "GitHub Actions Workflow".to_string());

    // 1. Run SDET Policy Validation
    let validation = validate_definition(workflow, &ValidationConfig::strict());

    if opts.strict_validation && !validation.valid {
        return PipelineRunResult {
            workflow_name,
            jobs: Vec::new(),
            duration_ms: start_time.elapsed().as_millis() as u64,
            success: false,
            logs: vec![LogEntry {
                timestamp: 0,
                runner: "Validator".to_string(),
                step: "Policy Check".to_string(),
                level: LogLevel::Error,
                message: format!("Strict SDET policy validation failed: {} error(s)", validation.errors.len()),
            }],
            validation: Some(validation),
        };
    }

    let mut all_jobs = Vec::new();
    let mut all_logs = Vec::new();

    // 2. Expand all jobs across matrix dimensions
    for (job_id, job_def) in &workflow.jobs {
        let runner_instances = expand_job_instances(job_id, job_def);

        for instance in runner_instances {
            let (job_result, job_logs) = execute_runner_instance(
                job_id,
                job_def,
                &instance.runner_name,
                &instance.matrix_combination,
                workflow,
                opts,
            );

            all_logs.extend(job_logs);
            let job_failed = job_result.status == JobStatus::Failed;
            all_jobs.push(job_result);

            if job_failed && opts.fail_fast {
                break;
            }
        }
    }

    let duration_ms = start_time.elapsed().as_millis() as u64;
    let overall_success = all_jobs.iter().all(|j| j.status == JobStatus::Passed);

    PipelineRunResult {
        workflow_name,
        jobs: all_jobs,
        duration_ms,
        success: overall_success,
        logs: all_logs,
        validation: Some(validation),
    }
}

struct RunnerInstance {
    pub runner_name: String,
    pub matrix_combination: HashMap<String, String>,
}

fn expand_job_instances(job_id: &str, job: &JobDefinition) -> Vec<RunnerInstance> {
    if let Some(ref strategy) = job.strategy {
        if let Some(ref matrix) = strategy.matrix {
            let combinations = matrix.expand_combinations();
            if !combinations.is_empty() {
                return combinations
                    .into_iter()
                    .map(|combo| {
                        let mut desc_parts = Vec::new();
                        for (k, v) in &combo {
                            desc_parts.push(format!("{}: {}", k, v));
                        }
                        desc_parts.sort();
                        let desc = if desc_parts.is_empty() {
                            String::new()
                        } else {
                            format!(" ({})", desc_parts.join(", "))
                        };
                        let runner_name = format!("{}{}", job.name.as_deref().unwrap_or(job_id), desc);
                        RunnerInstance {
                            runner_name,
                            matrix_combination: combo,
                        }
                    })
                    .collect();
            }
        }
    }

    let runner_name = job.name.as_deref().unwrap_or(job_id).to_string();
    vec![RunnerInstance {
        runner_name,
        matrix_combination: HashMap::new(),
    }]
}

fn execute_runner_instance(
    job_id: &str,
    job: &JobDefinition,
    runner_name: &str,
    matrix: &HashMap<String, String>,
    workflow: &WorkflowDefinition,
    _opts: &PipelineRunOptions,
) -> (JobRunResult, Vec<LogEntry>) {
    let start_time = Instant::now();
    let mut step_results = Vec::new();
    let mut logs = Vec::new();
    let mut job_failed = false;

    // Merge environment variables: workflow env -> job env
    let mut env = workflow.env.clone();
    for (k, v) in &job.env {
        env.insert(k.clone(), v.clone());
    }

    for step in &job.steps {
        let step_start = Instant::now();
        let raw_name = step.display_name();
        let interpolated_name = interpolate_variables(&raw_name, matrix, &env);

        // Check if condition
        let should_run = evaluate_step_condition(step.condition.as_deref(), job_failed);

        if !should_run {
            step_results.push(StepRunResult {
                name: interpolated_name.clone(),
                status: StepStatus::Skipped,
                duration_ms: 0,
                exit_code: 0,
                output: "Skipped due to condition".to_string(),
            });
            continue;
        }

        // Simulate step execution
        let (exit_code, output, step_logs) = simulate_step_execution(step, matrix, &env, runner_name);
        logs.extend(step_logs);

        let step_duration = step_start.elapsed().as_millis() as u64;
        let step_status = if exit_code == 0 {
            StepStatus::Passed
        } else {
            StepStatus::Failed
        };

        if step_status == StepStatus::Failed && !step.continue_on_error.unwrap_or(false) {
            job_failed = true;
        }

        step_results.push(StepRunResult {
            name: interpolated_name,
            status: step_status,
            duration_ms: step_duration,
            exit_code,
            output,
        });
    }

    let total_duration = start_time.elapsed().as_millis() as u64;
    let job_status = if job_failed {
        JobStatus::Failed
    } else {
        JobStatus::Passed
    };

    (
        JobRunResult {
            job_id: job_id.to_string(),
            runner_name: runner_name.to_string(),
            matrix_combination: matrix.clone(),
            status: job_status,
            duration_ms: total_duration,
            steps: step_results,
        },
        logs,
    )
}

fn simulate_step_execution(
    step: &StepDefinition,
    matrix: &HashMap<String, String>,
    env: &HashMap<String, String>,
    runner_name: &str,
) -> (i32, String, Vec<LogEntry>) {
    let mut logs = Vec::new();
    let mut output_lines = Vec::new();
    let step_name = step.display_name();

    let push_log = |logs: &mut Vec<LogEntry>, level: LogLevel, msg: &str| {
        logs.push(LogEntry {
            timestamp: 0,
            runner: runner_name.to_string(),
            step: step_name.clone(),
            level,
            message: msg.to_string(),
        });
    };

    // 1. Action Uses Simulation
    if let Some(ref uses) = step.uses {
        if uses.starts_with("actions/checkout") {
            push_log(&mut logs, LogLevel::Info, "Syncing repository at ref refs/heads/main...");
            push_log(&mut logs, LogLevel::Info, "Checked out commit c4f81a2 (HEAD -> main)");
            output_lines.push("Git repository successfully cloned (depth: 1).".to_string());
            return (0, output_lines.join("\n"), logs);
        }

        if uses.starts_with("actions/setup-node") {
            let version = matrix
                .get("node-version")
                .or_else(|| matrix.get("node"))
                .cloned()
                .unwrap_or_else(|| "20.x".to_string());
            push_log(&mut logs, LogLevel::Info, &format!("Setting up Node.js environment version '{}'...", version));
            push_log(&mut logs, LogLevel::Info, "Node.js v20.11.0 installed, npm v10.2.4 configured, PATH updated.");
            output_lines.push(format!("Resolved node version {}", version));
            return (0, output_lines.join("\n"), logs);
        }

        if uses.starts_with("actions/setup-python") {
            let version = matrix
                .get("python-version")
                .or_else(|| matrix.get("python"))
                .cloned()
                .unwrap_or_else(|| "3.11".to_string());
            push_log(&mut logs, LogLevel::Info, &format!("Setting up Python environment version '{}'...", version));
            push_log(&mut logs, LogLevel::Info, "Python 3.11.8 installed, pip 24.0 initialized.");
            output_lines.push(format!("Python {} ready in virtualenv", version));
            return (0, output_lines.join("\n"), logs);
        }

        if uses.starts_with("dtolnay/rust-toolchain") || uses.starts_with("actions-rs/toolchain") {
            let toolchain = matrix.get("rust").cloned().unwrap_or_else(|| "stable".to_string());
            push_log(&mut logs, LogLevel::Info, &format!("Setting up Rust toolchain '{}' with components: clippy, rustfmt...", toolchain));
            push_log(&mut logs, LogLevel::Info, "rustc 1.96.0 (30a34c682 2026-05-25), cargo 1.96.0 active.");
            output_lines.push("Rust toolchain configured.".to_string());
            return (0, output_lines.join("\n"), logs);
        }

        if uses.starts_with("actions/upload-artifact") {
            let path_param = step.with.get("path").map(|v| format!("{:?}", v)).unwrap_or_else(|| "test-results/".to_string());
            push_log(&mut logs, LogLevel::Info, &format!("Compressing test artifacts from: {}", path_param));
            push_log(&mut logs, LogLevel::Info, "Uploaded artifact archive (7.4 MB) -> Artifact ID #4829103");
            output_lines.push(format!("Artifact uploaded successfully: {}", path_param));
            return (0, output_lines.join("\n"), logs);
        }

        // Generic action fallback
        push_log(&mut logs, LogLevel::Info, &format!("Executing GitHub Action: {}", uses));
        output_lines.push(format!("Action {} completed successfully.", uses));
        return (0, output_lines.join("\n"), logs);
    }

    // 2. Shell Command Simulation
    if let Some(ref raw_run) = step.run {
        let run_cmd = interpolate_variables(raw_run, matrix, env);
        push_log(&mut logs, LogLevel::Info, &format!("$ {}", run_cmd.lines().next().unwrap_or(&run_cmd)));

        let lower_cmd = run_cmd.to_lowercase();

        // Check for simulated failure signals in command
        let is_simulated_failure = lower_cmd.lines().any(|line| {
            let trimmed = line.trim();
            trimmed == "exit 1"
                || trimmed == "exit 2"
                || (trimmed.starts_with("exit 1") && !trimmed.contains("||"))
        }) || lower_cmd.contains("simulated_failure")
            || lower_cmd.contains("fail_step")
            || lower_cmd.contains("simulate_error");

        if is_simulated_failure {
            push_log(&mut logs, LogLevel::Error, "Process exited with non-zero status code: 1");
            output_lines.push("Error: command failed with exit code 1".to_string());
            return (1, output_lines.join("\n"), logs);
        }

        if lower_cmd.contains("cargo test") {
            push_log(&mut logs, LogLevel::Info, "Running cargo test suite...");
            output_lines.push("running 42 tests".to_string());
            output_lines.push("test e2e_suite ... ok".to_string());
            output_lines.push("test unit_rules ... ok".to_string());
            output_lines.push("test result: ok. 42 passed; 0 failed; 0 ignored; finished in 0.24s".to_string());
            return (0, output_lines.join("\n"), logs);
        }

        if lower_cmd.contains("pytest") {
            push_log(&mut logs, LogLevel::Info, "Executing pytest runner with json-report...");
            output_lines.push("============================= test session starts ==============================".to_string());
            output_lines.push("collected 18 items".to_string());
            output_lines.push("tests/test_api.py .................. [100%]".to_string());
            output_lines.push("============================== 18 passed in 0.45s ===============================".to_string());
            return (0, output_lines.join("\n"), logs);
        }

        if lower_cmd.contains("playwright") || lower_cmd.contains("npm test") {
            push_log(&mut logs, LogLevel::Info, "Executing browser automation test runner...");
            output_lines.push("Running 12 tests across 3 workers".to_string());
            output_lines.push("  ✓ [chromium] › checkout.spec.ts:14:1 › hydrations trap handled (420ms)".to_string());
            output_lines.push("  ✓ [firefox]  › search.spec.ts:22:1   › debounced query resolved (310ms)".to_string());
            output_lines.push("  12 passed (1.4s)".to_string());
            return (0, output_lines.join("\n"), logs);
        }

        // Generic shell command
        output_lines.push(format!("Executed: {}", run_cmd.lines().next().unwrap_or("command")));
        return (0, output_lines.join("\n"), logs);
    }

    (0, "Completed step".to_string(), logs)
}

fn evaluate_step_condition(condition: Option<&str>, job_failed: bool) -> bool {
    let cond = match condition {
        Some(c) => c.trim().to_lowercase(),
        None => return !job_failed,
    };

    if cond.contains("always()") || cond.contains("!cancelled()") {
        return true;
    }
    if cond.contains("failure()") {
        return job_failed;
    }
    if cond.contains("success()") {
        return !job_failed;
    }

    !job_failed
}

fn interpolate_variables(
    text: &str,
    matrix: &HashMap<String, String>,
    env: &HashMap<String, String>,
) -> String {
    let mut result = text.to_string();

    // Replace ${{ matrix.<key> }}
    for (k, v) in matrix {
        result = result.replace(&format!("${{{{ matrix.{} }}}}", k), v);
        result = result.replace(&format!("${{ matrix.{} }}", k), v);
    }

    // Replace ${{ env.<key> }}
    for (k, v) in env {
        result = result.replace(&format!("${{{{ env.{} }}}}", k), v);
        result = result.replace(&format!("${{ env.{} }}", k), v);
    }

    // Replace ${{ runner.os }}
    if let Some(os) = matrix.get("os") {
        result = result.replace("${{ runner.os }}", os);
    }

    result
}

/// Renders rich terminal output for pipeline run results.
pub fn render_pipeline_summary(result: &PipelineRunResult) {
    println!("{}", "========================================================================================".cyan());
    println!(
        " {} v1.0.0  |  {}",
        "CHERENKOV-LINGS".bold().bright_cyan(),
        "CI/CD Pipeline Simulator".bright_yellow()
    );
    println!("{}", "========================================================================================".cyan());
    println!();
    println!("{} Workflow: {}", "▶".bright_blue(), result.workflow_name.bold().bright_white());
    println!("{} Duration: {}ms", "⏱".cyan(), result.duration_ms);
    println!();

    if let Some(ref val) = result.validation {
        println!("{}", "─── SDET Policy Validation ──────────────────────────────────────────────────────────".dimmed());
        println!("  Score: {}/100 | Status: {}", val.sdet_score, if val.valid { "VALID".green().bold() } else { "POLICY VIOLATION".red().bold() });
        println!("  {}", val.summary);
        for err in &val.errors {
            println!("  {} [{}] {}", "✗".red().bold(), err.code.bright_red(), err.message);
            if let Some(ref sug) = err.suggestion {
                println!("    {} {}", "💡 Fix:".yellow(), sug.italic());
            }
        }
        for warn in &val.warnings {
            println!("  {} [{}] {}", "⚠".yellow(), warn.code.bright_yellow(), warn.message);
            if let Some(ref sug) = warn.suggestion {
                println!("    {} {}", "💡 Suggestion:".yellow(), sug.italic());
            }
        }
        println!();
    }

    println!("{}", "─── Virtual Parallel Runners ────────────────────────────────────────────────────────".dimmed());
    for job in &result.jobs {
        println!("  {} {} ({}ms)", job.status.badge(), job.runner_name.bright_white().bold(), job.duration_ms);
        for step in &job.steps {
            println!(
                "     {} {} {}",
                step.status.badge(),
                step.name.bright_black(),
                format!("({}ms)", step.duration_ms).dimmed()
            );
        }
    }

    println!();
    println!("{}", "========================================================================================".cyan());
    if result.success {
        println!(" {} All matrix runner jobs executed successfully!", "✔ WORKFLOW PASSED:".green().bold());
    } else {
        println!(" {} One or more virtual runner jobs or validation checks failed.", "✖ WORKFLOW FAILED:".red().bold());
    }
    println!("{}", "========================================================================================".cyan());
    println!();
}
