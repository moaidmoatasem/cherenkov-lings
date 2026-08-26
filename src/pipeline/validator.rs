use crate::pipeline::parser::{JobDefinition, WorkflowDefinition};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineValidation {
    pub valid: bool,
    pub errors: Vec<PipelineError>,
    pub warnings: Vec<PipelineWarning>,
    pub matrix_detected: bool,
    pub artifact_upload_detected: bool,
    pub summary: String,
    pub sdet_score: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineError {
    pub code: String,
    pub message: String,
    pub job: Option<String>,
    pub step: Option<String>,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineWarning {
    pub code: String,
    pub message: String,
    pub job: Option<String>,
    pub step: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidationConfig {
    pub require_matrix_for_tests: bool,
    pub require_artifact_upload: bool,
    pub enforce_secret_scanning: bool,
    pub enforce_timeout: bool,
    pub enforce_concurrency: bool,
}

impl ValidationConfig {
    pub fn strict() -> Self {
        Self {
            require_matrix_for_tests: true,
            require_artifact_upload: true,
            enforce_secret_scanning: true,
            enforce_timeout: true,
            enforce_concurrency: true,
        }
    }
}

// Regex patterns for secret detection
static RE_GH_TOKEN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"ghp_[A-Za-z0-9]{36}").unwrap());
static RE_GL_TOKEN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"glpat-[A-Za-z0-9\-]{20,}").unwrap());
static RE_AWS_KEY: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"AKIA[0-9A-Z]{16}").unwrap());
static RE_AWS_SECRET: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)AWS_SECRET_ACCESS_KEY\s*[:=]\s*["']?[A-Za-z0-9/+=]{30,}["']?"#).unwrap()
});
static RE_JWT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Bearer\s+ey[A-Za-z0-9_\-\.]{20,}").unwrap());
static RE_PRIVATE_KEY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"-----BEGIN\s+(?:RSA|OPENSSH|EC|DSA|PGP)?\s*PRIVATE KEY-----").unwrap()
});
static RE_HARDCODED_CREDENTIAL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)(?:password|passwd|api_key|secret_key|auth_token)\s*[:=]\s*["']([^"'\s]{8,})["']"#,
    )
    .unwrap()
});

/// Validates raw workflow YAML content against enterprise SDET policies.
pub fn validate_workflow(yaml_content: &str) -> PipelineValidation {
    match crate::pipeline::parser::parse_workflow_str(yaml_content) {
        Ok(workflow) => validate_definition(&workflow, &ValidationConfig::strict()),
        Err(err) => PipelineValidation {
            valid: false,
            errors: vec![PipelineError {
                code: "YAML_SYNTAX_ERROR".to_string(),
                message: format!("Failed to parse workflow YAML: {}", err),
                job: None,
                step: None,
                line: None,
                suggestion: Some(
                    "Ensure YAML indentation and syntax follow GitHub Actions specifications."
                        .to_string(),
                ),
            }],
            warnings: Vec::new(),
            matrix_detected: false,
            artifact_upload_detected: false,
            summary: format!("YAML Syntax Error: {}", err),
            sdet_score: 0,
        },
    }
}

/// Validates parsed WorkflowDefinition against SDET policies.
pub fn validate_definition(
    workflow: &WorkflowDefinition,
    config: &ValidationConfig,
) -> PipelineValidation {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut matrix_detected = false;
    let mut artifact_upload_detected = false;

    // 1. Concurrency Validation
    if config.enforce_concurrency {
        let triggers_pr_or_push =
            workflow.on.has_trigger("push") || workflow.on.has_trigger("pull_request");
        if triggers_pr_or_push {
            if let Some(ref conc) = workflow.concurrency {
                if !conc.cancels_in_progress() {
                    warnings.push(PipelineWarning {
                        code: "CONCURRENCY_CANCEL_DISABLED".to_string(),
                        message: format!(
                            "Concurrency group '{}' does not set 'cancel-in-progress: true'. Stale branch runs will not be cancelled on rapid commits.",
                            conc.group_name()
                        ),
                        job: None,
                        step: None,
                        suggestion: Some("Add 'cancel-in-progress: true' to the concurrency block.".to_string()),
                    });
                }
            } else {
                warnings.push(PipelineWarning {
                    code: "MISSING_CONCURRENCY".to_string(),
                    message: "Workflow is triggered on push/pull_request but lacks top-level 'concurrency' configuration with 'cancel-in-progress: true'.".to_string(),
                    job: None,
                    step: None,
                    suggestion: Some("Add `concurrency: group: ${{ github.workflow }}-${{ github.ref }}, cancel-in-progress: true` to prevent redundant CI runs.".to_string()),
                });
            }
        }
    }

    // 2. Scan workflow-level env for secrets
    if config.enforce_secret_scanning {
        for (k, v) in &workflow.env {
            if let Some(finding) = check_plaintext_secret(k, v) {
                errors.push(PipelineError {
                    code: "HARDCODED_SECRET".to_string(),
                    message: format!("Workflow-level env variable '{}' contains plaintext secret: {}", k, finding),
                    job: None,
                    step: None,
                    line: None,
                    suggestion: Some(format!("Store this secret in GitHub repository secrets and reference via `${{{{ secrets.{} }}}}`.", k.to_uppercase())),
                });
            }
        }
    }

    // 3. Validate each Job
    for (job_id, job) in &workflow.jobs {
        let is_test_job = is_testing_job(job_id, job);

        // A. Matrix Strategy Validation
        let has_matrix = job
            .strategy
            .as_ref()
            .and_then(|s| s.matrix.as_ref())
            .is_some();

        if has_matrix {
            matrix_detected = true;
            let matrix_def = job.strategy.as_ref().unwrap().matrix.as_ref().unwrap();
            let combos = matrix_def.expand_combinations();

            if combos.is_empty()
                || (combos.len() == 1 && matrix_def.variable_dimensions().is_empty())
            {
                if is_test_job && config.require_matrix_for_tests {
                    errors.push(PipelineError {
                        code: "INSUFFICIENT_MATRIX_DIMENSIONS".to_string(),
                        message: format!(
                            "Testing job '{}' defines an empty matrix strategy with 0 effective runner combinations.",
                            job_id
                        ),
                        job: Some(job_id.clone()),
                        step: None,
                        line: None,
                        suggestion: Some("Define multi-OS (e.g. os: [ubuntu-latest, macos-latest]) or multi-version (e.g. node-version: [18, 20]) matrix axes.".to_string()),
                    });
                }
            } else if combos.len() == 1 && is_test_job && config.require_matrix_for_tests {
                // If it only has 1 combination, warn/error that matrix lacks true parallelism
                warnings.push(PipelineWarning {
                    code: "SINGLE_MATRIX_COMBINATION".to_string(),
                    message: format!(
                        "Testing job '{}' matrix produces only 1 combination. Enterprise SDET standards recommend multi-version or multi-OS matrix validation.",
                        job_id
                    ),
                    job: Some(job_id.clone()),
                    step: None,
                    suggestion: Some("Add additional OS targets or runtime versions to test cross-platform compatibility.".to_string()),
                });
            }
        } else if is_test_job && config.require_matrix_for_tests {
            errors.push(PipelineError {
                code: "MISSING_MATRIX_STRATEGY".to_string(),
                message: format!(
                    "Testing job '{}' is missing a 'strategy.matrix' configuration. Enterprise SDET policy requires parallel multi-version or multi-OS matrix execution.",
                    job_id
                ),
                job: Some(job_id.clone()),
                step: None,
                line: None,
                suggestion: Some("Configure `strategy: matrix: { os: [ubuntu-latest, windows-latest], node-version: [18, 20] }` for parallel cross-platform execution.".to_string()),
            });
        }

        // B. Artifact Upload Validation
        let mut job_has_artifact_upload = false;
        for step in &job.steps {
            if step.is_artifact_upload() {
                job_has_artifact_upload = true;
                artifact_upload_detected = true;
                break;
            }
        }

        if is_test_job && config.require_artifact_upload && !job_has_artifact_upload {
            errors.push(PipelineError {
                code: "MISSING_ARTIFACT_UPLOAD".to_string(),
                message: format!(
                    "Testing job '{}' does not upload test reports, traces, or artifacts via 'actions/upload-artifact'. SDET policy requires archiving test telemetry for observability and failure triage.",
                    job_id
                ),
                job: Some(job_id.clone()),
                step: None,
                line: None,
                suggestion: Some("Add an `actions/upload-artifact@v4` step archiving test-results/, allure-results/, or junit reports with `if: always()`.".to_string()),
            });
        }

        // C. Timeout Validation
        if config.enforce_timeout {
            if let Some(timeout) = job.timeout_minutes {
                if timeout > 120 {
                    warnings.push(PipelineWarning {
                        code: "EXCESSIVE_TIMEOUT".to_string(),
                        message: format!(
                            "Job '{}' defines timeout-minutes: {} (exceeds recommended 120 min maximum).",
                            job_id, timeout
                        ),
                        job: Some(job_id.clone()),
                        step: None,
                        suggestion: Some("Reduce timeout-minutes to 15-30 minutes to fail fast on hung CI runners.".to_string()),
                    });
                }
            } else {
                warnings.push(PipelineWarning {
                    code: "MISSING_TIMEOUT".to_string(),
                    message: format!(
                        "Job '{}' does not specify 'timeout-minutes'. Default GitHub Actions timeout is 360 minutes (6 hours), risking runaway costs.",
                        job_id
                    ),
                    job: Some(job_id.clone()),
                    step: None,
                    suggestion: Some("Set `timeout-minutes: 30` to prevent hung test runners from burning CI quota.".to_string()),
                });
            }
        }

        // D. Secret Scanning in Job Env & Steps
        if config.enforce_secret_scanning {
            for (k, v) in &job.env {
                if let Some(finding) = check_plaintext_secret(k, v) {
                    errors.push(PipelineError {
                        code: "HARDCODED_SECRET".to_string(),
                        message: format!(
                            "Job '{}' env variable '{}' contains plaintext secret: {}",
                            job_id, k, finding
                        ),
                        job: Some(job_id.clone()),
                        step: None,
                        line: None,
                        suggestion: Some(format!(
                            "Reference via `${{{{ secrets.{} }}}}` instead of plaintext.",
                            k.to_uppercase()
                        )),
                    });
                }
            }

            for step in &job.steps {
                let step_name = step.display_name();

                // Check step env
                for (k, v) in &step.env {
                    if let Some(finding) = check_plaintext_secret(k, v) {
                        errors.push(PipelineError {
                            code: "HARDCODED_SECRET".to_string(),
                            message: format!(
                                "Step '{}' in job '{}' env '{}' contains secret: {}",
                                step_name, job_id, k, finding
                            ),
                            job: Some(job_id.clone()),
                            step: Some(step_name.clone()),
                            line: None,
                            suggestion: Some(
                                "Replace plaintext value with `${{ secrets.SECRET_NAME }}`."
                                    .to_string(),
                            ),
                        });
                    }
                }

                // Check step run script
                if let Some(ref script) = step.run
                    && let Some(finding) = scan_text_for_secrets(script)
                {
                    errors.push(PipelineError {
                            code: "HARDCODED_SECRET".to_string(),
                            message: format!("Step '{}' in job '{}' run command contains hardcoded secret: {}", step_name, job_id, finding),
                            job: Some(job_id.clone()),
                            step: Some(step_name.clone()),
                            line: None,
                            suggestion: Some("Pass secrets via environment variables: `env: MY_KEY: ${{ secrets.MY_KEY }}`.".to_string()),
                        });
                }

                // Check step `with` parameters
                for (k, v) in &step.with {
                    let val_str = format!("{:?}", v);
                    if let Some(finding) = check_plaintext_secret(k, &val_str) {
                        errors.push(PipelineError {
                            code: "HARDCODED_SECRET".to_string(),
                            message: format!(
                                "Step '{}' parameter '{}' contains plaintext secret: {}",
                                step_name, k, finding
                            ),
                            job: Some(job_id.clone()),
                            step: Some(step_name.clone()),
                            line: None,
                            suggestion: Some(
                                "Pass secrets securely using `${{ secrets.SECRET_NAME }}`."
                                    .to_string(),
                            ),
                        });
                    }
                }
            }
        }
    }

    // Compute SDET Quality Score (0 to 100)
    let penalty = (errors.len() as u32 * 25) + (warnings.len() as u32 * 10);
    let sdet_score = 100_u32.saturating_sub(penalty);
    let valid = errors.is_empty();

    let summary = if valid && warnings.is_empty() {
        "✓ 100% SDET Policy Compliance: Parallel matrix strategies, artifact uploads, secret security, and timeouts are properly configured.".to_string()
    } else if valid {
        format!(
            "⚠ Passed with {} warning(s): Workflow is executable, but enterprise SDET best practices can be improved.",
            warnings.len()
        )
    } else {
        format!(
            "✗ FAILED SDET Policy Validation: Found {} error(s) and {} warning(s). Strict enterprise testing policies violated.",
            errors.len(),
            warnings.len()
        )
    };

    PipelineValidation {
        valid,
        errors,
        warnings,
        matrix_detected,
        artifact_upload_detected,
        summary,
        sdet_score,
    }
}

/// Detects whether a job is performing test/QA/verification operations.
fn is_testing_job(job_id: &str, job: &JobDefinition) -> bool {
    let lower_id = job_id.to_lowercase();
    let lower_name = job.name.as_deref().unwrap_or("").to_lowercase();

    let test_keywords = [
        "test",
        "spec",
        "qa",
        "e2e",
        "integration",
        "unit",
        "chaos",
        "verify",
        "validation",
        "audit",
        "playwright",
        "cypress",
        "k6",
        "jmeter",
        "pytest",
        "cargo test",
        "mvn test",
        "npm test",
    ];

    if test_keywords
        .iter()
        .any(|k| lower_id.contains(k) || lower_name.contains(k))
    {
        return true;
    }

    for step in &job.steps {
        if let Some(ref r) = step.run {
            let lower_r = r.to_lowercase();
            if lower_r.contains("test")
                || lower_r.contains("pytest")
                || lower_r.contains("playwright")
                || lower_r.contains("cypress")
                || lower_r.contains("jest")
                || lower_r.contains("vitest")
                || lower_r.contains("mocha")
                || lower_r.contains("k6")
                || lower_r.contains("jmeter")
                || lower_r.contains("mvn verify")
                || lower_r.contains("cargo test")
            {
                return true;
            }
        }
        if let Some(ref u) = step.uses {
            let lower_u = u.to_lowercase();
            if lower_u.contains("playwright")
                || lower_u.contains("cypress")
                || lower_u.contains("codecov")
                || lower_u.contains("allure")
            {
                return true;
            }
        }
    }

    false
}

fn check_plaintext_secret(key: &str, val: &str) -> Option<String> {
    if val.contains("${{") || val.contains("$") {
        return None;
    }

    let lower_k = key.to_lowercase();
    if (lower_k.contains("password")
        || lower_k.contains("secret")
        || lower_k.contains("token")
        || lower_k.contains("api_key")
        || lower_k.contains("private_key"))
        && val.len() >= 8
        && !val.starts_with("test")
        && !val.starts_with("mock")
        && !val.starts_with("dummy")
    {
        return Some(format!(
            "Sensitive key '{}' has plaintext value '{}...'",
            key,
            &val[..val.len().min(4)]
        ));
    }

    scan_text_for_secrets(val)
}

fn scan_text_for_secrets(text: &str) -> Option<String> {
    if let Some(m) = RE_GH_TOKEN.find(text) {
        return Some(format!("GitHub token '{}...'", &m.as_str()[..8]));
    }
    if let Some(m) = RE_GL_TOKEN.find(text) {
        return Some(format!("GitLab token '{}...'", &m.as_str()[..10]));
    }
    if let Some(m) = RE_AWS_KEY.find(text) {
        return Some(format!("AWS Access Key ID '{}...'", &m.as_str()[..8]));
    }
    if RE_AWS_SECRET.find(text).is_some() {
        return Some("AWS Secret Access Key pattern".to_string());
    }
    if let Some(m) = RE_JWT.find(text) {
        return Some(format!("JWT Bearer token '{}...'", &m.as_str()[..15]));
    }
    if RE_PRIVATE_KEY.find(text).is_some() {
        return Some("Private RSA/OpenSSH Key block".to_string());
    }
    if let Some(caps) = RE_HARDCODED_CREDENTIAL.captures(text)
        && let Some(val) = caps.get(1)
    {
        let s = val.as_str();
        if !s.starts_with('$') && !s.starts_with('{') {
            return Some(format!(
                "Hardcoded credential value '{}...'",
                &s[..s.len().min(6)]
            ));
        }
    }
    None
}
