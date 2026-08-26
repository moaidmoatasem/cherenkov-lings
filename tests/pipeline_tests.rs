use cherenkov_lings::pipeline::{
    JobStatus, MatrixDefinition, PipelineRunOptions, StepStatus, parse_workflow_str, run_pipeline,
    run_workflow, validate_workflow,
};
use std::collections::HashMap;
use std::path::Path;

const VALID_ENTERPRISE_WORKFLOW: &str = r#"
name: Enterprise SDET Test Pipeline

on:
  push:
    branches: [ "main" ]
  pull_request:
    branches: [ "main" ]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

jobs:
  parallel_e2e_tests:
    name: Cross-Platform E2E Tests (${{ matrix.os }} - Node ${{ matrix.node-version }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        node-version: [18, 20]
        include:
          - os: ubuntu-latest
            node-version: 22
            experimental: true
        exclude:
          - os: windows-latest
            node-version: 18

    steps:
      - name: Checkout Source Code
        uses: actions/checkout@v4

      - name: Setup Node Runtime
        uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node-version }}

      - name: Install & Run Playwright Test Suite
        run: npm ci && npx playwright test --reporter=html

      - name: Upload Test Results & Trace Artifacts
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report-${{ matrix.os }}-${{ matrix.node-version }}
          path: playwright-report/
          retention-days: 14
"#;

const MISSING_MATRIX_WORKFLOW: &str = r#"
name: Flaky Single-Runner Pipeline

on:
  push:
    branches: [ "main" ]

jobs:
  test:
    name: Single Linux Test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Pytest
        run: pytest tests/ -v
      - uses: actions/upload-artifact@v4
        with:
          path: test-results/
"#;

const MISSING_ARTIFACT_WORKFLOW: &str = r#"
name: Unobservable Test Pipeline

on:
  push:
    branches: [ "main" ]

concurrency:
  group: test-ci
  cancel-in-progress: true

jobs:
  test:
    name: Matrix Test Without Reports
    runs-on: ubuntu-latest
    timeout-minutes: 20
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Run Cargo Test
        run: cargo test --all
"#;

const HARDCODED_SECRET_WORKFLOW: &str = r#"
name: Insecure CI Pipeline

on:
  push:
    branches: [ "main" ]

concurrency:
  group: insecure-ci
  cancel-in-progress: true

jobs:
  test:
    name: Test with Leaked Secrets
    runs-on: ubuntu-latest
    timeout-minutes: 15
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    env:
      AWS_SECRET_ACCESS_KEY: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
    steps:
      - uses: actions/checkout@v4
      - name: Run Tests with Auth Token
        run: |
          export GITHUB_TOKEN=ghp_1234567890abcdefghijklmnopqrstuvwxyz
          cargo test
      - uses: actions/upload-artifact@v4
        with:
          path: allure-results/
"#;

#[test]
fn test_parse_valid_workflow_structure() {
    let workflow = parse_workflow_str(VALID_ENTERPRISE_WORKFLOW).expect("Failed to parse workflow");
    assert_eq!(
        workflow.name.as_deref(),
        Some("Enterprise SDET Test Pipeline")
    );
    assert!(workflow.on.has_trigger("push"));
    assert!(workflow.on.has_trigger("pull_request"));
    assert!(workflow.concurrency.is_some());
    assert!(workflow.concurrency.unwrap().cancels_in_progress());
    assert_eq!(workflow.jobs.len(), 1);

    let job = workflow.jobs.get("parallel_e2e_tests").unwrap();
    assert_eq!(job.timeout_minutes, Some(30));
    assert!(job.strategy.is_some());
    assert_eq!(job.steps.len(), 4);
    assert!(job.steps[3].is_artifact_upload());
}

#[test]
fn test_matrix_expansion_cartesian_product() {
    let mut dimensions = HashMap::new();
    dimensions.insert(
        "os".to_string(),
        serde_yaml::to_value(vec!["ubuntu-latest", "macos-latest"]).unwrap(),
    );
    dimensions.insert(
        "node".to_string(),
        serde_yaml::to_value(vec!["18", "20", "22"]).unwrap(),
    );

    let matrix_def = MatrixDefinition {
        dimensions,
        include: None,
        exclude: None,
    };

    let combinations = matrix_def.expand_combinations();
    // 2 OS * 3 Node = 6 combinations
    assert_eq!(combinations.len(), 6);

    for combo in &combinations {
        assert!(combo.contains_key("os"));
        assert!(combo.contains_key("node"));
    }
}

#[test]
fn test_matrix_expansion_with_includes_and_excludes() {
    let workflow = parse_workflow_str(VALID_ENTERPRISE_WORKFLOW).unwrap();
    let job = workflow.jobs.get("parallel_e2e_tests").unwrap();
    let matrix_def = job.strategy.as_ref().unwrap().matrix.as_ref().unwrap();

    let combinations = matrix_def.expand_combinations();
    // Base: 3 OS * 2 Node = 6
    // Exclude: windows-latest & 18 (-1) -> 5
    // Include: ubuntu-latest & 22 (+1) -> 6
    assert_eq!(combinations.len(), 6);

    // Verify windows-latest & 18 is NOT present
    let has_excluded = combinations.iter().any(|c| {
        c.get("os").map(|s| s.as_str()) == Some("windows-latest")
            && c.get("node-version").map(|s| s.as_str()) == Some("18")
    });
    assert!(!has_excluded, "Excluded combination was found in matrix");

    // Verify included ubuntu-latest & 22 IS present
    let has_included = combinations.iter().any(|c| {
        c.get("os").map(|s| s.as_str()) == Some("ubuntu-latest")
            && c.get("node-version").map(|s| s.as_str()) == Some("22")
            && c.get("experimental").map(|s| s.as_str()) == Some("true")
    });
    assert!(has_included, "Included combination was not added to matrix");
}

#[test]
fn test_parse_malformed_yaml_returns_error() {
    let bad_yaml = "name: Bad YAML\njobs:\n  test:\n    - unaligned:\n   [invalid";
    let res = parse_workflow_str(bad_yaml);
    assert!(res.is_err());
}

#[test]
fn test_validator_passes_valid_enterprise_workflow() {
    let validation = validate_workflow(VALID_ENTERPRISE_WORKFLOW);
    assert!(
        validation.valid,
        "Expected validation to pass, errors: {:?}",
        validation.errors
    );
    assert_eq!(validation.sdet_score, 100);
    assert!(validation.matrix_detected);
    assert!(validation.artifact_upload_detected);
    assert!(validation.errors.is_empty());
}

#[test]
fn test_validator_fails_missing_matrix_strategy() {
    let validation = validate_workflow(MISSING_MATRIX_WORKFLOW);
    assert!(!validation.valid);
    assert!(
        validation
            .errors
            .iter()
            .any(|e| e.code == "MISSING_MATRIX_STRATEGY")
    );
    assert!(validation.sdet_score < 100);
}

#[test]
fn test_validator_fails_missing_artifact_upload() {
    let validation = validate_workflow(MISSING_ARTIFACT_WORKFLOW);
    assert!(!validation.valid);
    assert!(
        validation
            .errors
            .iter()
            .any(|e| e.code == "MISSING_ARTIFACT_UPLOAD")
    );
}

#[test]
fn test_validator_detects_hardcoded_secrets() {
    let validation = validate_workflow(HARDCODED_SECRET_WORKFLOW);
    assert!(!validation.valid);
    let secret_errors: Vec<_> = validation
        .errors
        .iter()
        .filter(|e| e.code == "HARDCODED_SECRET")
        .collect();
    assert!(
        secret_errors.len() >= 2,
        "Expected at least 2 secret detection errors, found {}",
        secret_errors.len()
    );
}

#[test]
fn test_validator_detects_missing_concurrency_and_timeout() {
    let workflow = r#"
name: Minimal Pipeline
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - name: Run Tests
        run: cargo test
      - uses: actions/upload-artifact@v4
        with:
          path: target/
"#;
    let validation = validate_workflow(workflow);
    assert!(
        validation
            .warnings
            .iter()
            .any(|w| w.code == "MISSING_CONCURRENCY")
    );
    assert!(
        validation
            .warnings
            .iter()
            .any(|w| w.code == "MISSING_TIMEOUT")
    );
}

#[test]
fn test_runner_simulates_parallel_matrix_jobs() {
    let workflow = parse_workflow_str(VALID_ENTERPRISE_WORKFLOW).unwrap();
    let opts = PipelineRunOptions {
        parallel: true,
        fail_fast: false,
        animated: false,
        max_parallel: None,
        verbose: true,
        strict_validation: false,
    };

    let result = run_workflow(&workflow, &opts);
    assert!(result.success);
    // 6 expanded matrix runners
    assert_eq!(result.jobs.len(), 6);

    for job in &result.jobs {
        assert_eq!(job.status, JobStatus::Passed);
        assert_eq!(job.steps.len(), 4);
        assert_eq!(job.steps[0].status, StepStatus::Passed);
        assert_eq!(job.steps[1].status, StepStatus::Passed);
        assert_eq!(job.steps[2].status, StepStatus::Passed);
        assert_eq!(job.steps[3].status, StepStatus::Passed);
    }

    assert!(!result.logs.is_empty());
    assert!(
        result
            .logs
            .iter()
            .any(|l| l.message.contains("Playwright") || l.message.contains("Node.js"))
    );
}

#[test]
fn test_runner_simulates_step_failure_and_skipping() {
    let failing_workflow = r#"
name: Failing Pipeline
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Step 1 Setup
        run: echo "setup"
      - name: Step 2 Failure
        run: exit 1
      - name: Step 3 Dependent (Should Skip)
        run: echo "should not run"
      - name: Step 4 Always Cleanup
        if: always()
        uses: actions/upload-artifact@v4
        with:
          path: logs/
"#;
    let workflow = parse_workflow_str(failing_workflow).unwrap();
    let opts = PipelineRunOptions::default();
    let result = run_workflow(&workflow, &opts);

    assert!(!result.success);
    assert_eq!(result.jobs.len(), 1);
    let job = &result.jobs[0];
    assert_eq!(job.status, JobStatus::Failed);

    assert_eq!(job.steps[0].status, StepStatus::Passed);
    assert_eq!(job.steps[1].status, StepStatus::Failed);
    assert_eq!(job.steps[2].status, StepStatus::Skipped);
    assert_eq!(
        job.steps[3].status,
        StepStatus::Passed,
        "if: always() step should execute even after failure"
    );
}

#[test]
fn test_runner_continue_on_error() {
    let continue_workflow = r#"
name: Continue On Error Pipeline
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - name: Flaky Step
        run: exit 1
        continue-on-error: true
      - name: Next Step
        run: echo "running next"
"#;
    let workflow = parse_workflow_str(continue_workflow).unwrap();
    let result = run_workflow(&workflow, &PipelineRunOptions::default());

    assert!(result.success);
    let job = &result.jobs[0];
    assert_eq!(job.status, JobStatus::Passed);
    assert_eq!(job.steps[0].status, StepStatus::Failed);
    assert_eq!(job.steps[1].status, StepStatus::Passed);
}

#[test]
fn test_runner_strict_validation_abort() {
    let workflow = parse_workflow_str(MISSING_MATRIX_WORKFLOW).unwrap();
    let opts = PipelineRunOptions {
        strict_validation: true,
        ..Default::default()
    };
    let result = run_workflow(&workflow, &opts);

    assert!(!result.success);
    assert!(
        result.jobs.is_empty(),
        "Strict validation should abort before running jobs"
    );
    assert!(result.validation.is_some());
    assert!(!result.validation.unwrap().valid);
}

#[test]
fn test_runner_executes_actual_repo_ci_workflow() {
    let ci_path = Path::new(".github/workflows/ci.yml");
    if ci_path.exists() {
        let opts = PipelineRunOptions {
            parallel: true,
            fail_fast: false,
            animated: false,
            max_parallel: None,
            verbose: false,
            strict_validation: false,
        };

        let result = run_pipeline(ci_path, &opts).expect("Failed to execute actual ci.yml");
        assert_eq!(result.workflow_name, "Cherenkov-lings CI");
        assert!(result.jobs.len() >= 4);
        assert!(result.success);
    }
}
