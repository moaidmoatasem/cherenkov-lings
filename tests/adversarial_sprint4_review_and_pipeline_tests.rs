use cherenkov_lings::pipeline::{
    parse_workflow_str, run_workflow, validate_definition, validate_workflow, JobStatus,
    MatrixDefinition, PipelineRunOptions, StepStatus, ValidationConfig,
};
use cherenkov_lings::review::{
    apply_all_fixes, apply_automated_fixes, apply_fix, calculate_score, generate_unified_diff,
    run_review, run_review_on_content, AiMentorClient, AstViolation, ReviewConfig, RuleScanner,
    Severity, SupportedLanguage,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

// =============================================================================
// SECTION 1: AST CODE REVIEW ENGINE ADVERSARIAL STRESS TESTS
// =============================================================================

#[test]
fn test_adversarial_polyglot_sleep_detection_all_flavors() {
    // 1. TypeScript / JavaScript sleep variations
    let ts_samples = vec![
        ("await page.waitForTimeout(5000);", 1),
        ("await frame.waitForTimeout(1000);", 1),
        ("locator.waitForTimeout(250);", 1),
        ("setTimeout(() => { doSomething(); }, 3000);", 1),
        ("window.setTimeout(fn, 2000);", 1),
        ("await new Promise(resolve => setTimeout(resolve, 4000));", 1),
        ("await new Promise(r => setTimeout(r, 1500));", 1),
        ("// await page.waitForTimeout(5000);", 0), // Comment should be ignored
        ("/* page.waitForTimeout(5000); */", 0),
        ("const timeoutName = 'waitForTimeout';", 0),
    ];

    for (code, expected_violations) in ts_samples {
        let violations = RuleScanner::scan_content("test_spec.ts", code);
        let sleep_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP")
            .collect();
        assert_eq!(
            sleep_violations.len(),
            expected_violations,
            "Failed on TS sample: {}",
            code
        );
        if expected_violations > 0 {
            assert_eq!(sleep_violations[0].severity, Severity::Error);
            assert!(sleep_violations[0].suggested_fix.as_ref().unwrap().contains("expect"));
        }
    }

    // 2. Python sleep variations
    let py_samples = vec![
        ("time.sleep(5)", 1),
        ("time.sleep(0.5)", 1),
        ("asyncio.sleep(2.0)", 1),
        ("await asyncio.sleep(1)", 1),
        ("# time.sleep(10)", 0),
        ("search_term = 'time.sleep(5)'", 0),
    ];

    for (code, expected_violations) in py_samples {
        let violations = RuleScanner::scan_content("test_feature.py", code);
        let sleep_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP")
            .collect();
        assert_eq!(
            sleep_violations.len(),
            expected_violations,
            "Failed on Python sample: {}",
            code
        );
        if expected_violations > 0 {
            assert_eq!(sleep_violations[0].severity, Severity::Error);
            assert!(sleep_violations[0].suggested_fix.as_ref().unwrap().contains("get_by_role"));
        }
    }

    // 3. Java sleep variations
    let java_samples = vec![
        ("Thread.sleep(3000);", 1),
        ("java.lang.Thread.sleep(1500);", 1),
        ("TimeUnit.SECONDS.sleep(5);", 1),
        ("TimeUnit.MILLISECONDS.sleep(500);", 1),
        ("// Thread.sleep(2000);", 0),
    ];

    for (code, expected_violations) in java_samples {
        let violations = RuleScanner::scan_content("OrderFlowTest.java", code);
        let sleep_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP")
            .collect();
        assert_eq!(
            sleep_violations.len(),
            expected_violations,
            "Failed on Java sample: {}",
            code
        );
        if expected_violations > 0 {
            assert_eq!(sleep_violations[0].severity, Severity::Error);
            assert!(sleep_violations[0].suggested_fix.as_ref().unwrap().contains("Awaitility"));
        }
    }

    // 4. Rust sleep variations
    let rust_samples = vec![
        ("std::thread::sleep(std::time::Duration::from_millis(500));", 1),
        ("thread::sleep(Duration::from_secs(2));", 1),
        ("tokio::time::sleep(Duration::from_millis(100)).await;", 1),
        ("// thread::sleep(Duration::from_secs(1));", 0),
    ];

    for (code, expected_violations) in rust_samples {
        let violations = RuleScanner::scan_content("worker_test.rs", code);
        let sleep_violations: Vec<_> = violations
            .iter()
            .filter(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP")
            .collect();
        assert_eq!(
            sleep_violations.len(),
            expected_violations,
            "Failed on Rust sample: {}",
            code
        );
        if expected_violations > 0 {
            assert_eq!(sleep_violations[0].severity, Severity::Error);
            assert!(sleep_violations[0].suggested_fix.as_ref().unwrap().contains("tokio::time::timeout"));
        }
    }
}

#[test]
fn test_adversarial_fragile_locator_detection_and_false_positive_resistance() {
    let code_with_fragile_locators = r#"
        test('fragile locators stress', async ({ page }) => {
            // Fragile XPath variations
            const x1 = page.locator('/html/body/div[1]/main/div[2]/table/tbody/tr[3]/td[2]/button');
            const x2 = page.locator("//div/div/span/button[1]");
            
            // Fragile Deep CSS chains
            const c1 = page.locator('div > div > div > button');
            const c2 = page.locator('#container > :nth-child(2) > .card-body > .btn');
            
            // Fragile dynamic/auto-generated IDs
            const id1 = page.locator('#input-a1b2c3d4e5f6');
            const id2 = page.locator('#ember10492');
            const id3 = page.locator('#react-998877_btn');
            const id4 = page.locator('[id^="auto_submit_button_xyz"]');
            const id5 = page.locator('[id*="random_order_id_123"]');
            
            // Resilient semantic locators (MUST NOT FLAG)
            const r1 = page.getByRole('button', { name: 'Submit Order' });
            const r2 = page.getByTestId('order-submit-btn');
            const r3 = page.getByLabel('Credit Card Number');
            const r4 = page.getByText('Payment Succeeded');
            const r5 = page.getByPlaceholder('Enter email');
            
            expect(r1).toBeDefined();
        });
    "#;

    let violations = RuleScanner::scan_content("checkout.spec.ts", code_with_fragile_locators);

    let xpath_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "ANTI_PATTERN_FRAGILE_LOCATOR_XPATH")
        .collect();
    let css_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "ANTI_PATTERN_FRAGILE_LOCATOR_CSS")
        .collect();
    let dynamic_id_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "ANTI_PATTERN_FRAGILE_LOCATOR_AUTO_ID")
        .collect();

    assert_eq!(xpath_violations.len(), 2, "Expected 2 absolute XPath violations");
    assert_eq!(css_violations.len(), 2, "Expected 2 deep CSS violations");
    assert_eq!(dynamic_id_violations.len(), 5, "Expected 5 auto-generated ID violations");

    // Verify resilient locators were not flagged
    for v in &violations {
        assert!(!v.code_snippet.contains("getByRole"));
        assert!(!v.code_snippet.contains("getByTestId"));
        assert!(!v.code_snippet.contains("getByLabel"));
        assert!(!v.code_snippet.contains("getByText"));
        assert!(!v.code_snippet.contains("getByPlaceholder"));
    }
}

#[test]
fn test_adversarial_floating_promises_and_deep_block_actions() {
    let code = r#"
        test('deeply nested async actions', async ({ page }) => {
            if (true) {
                while (condition) {
                    try {
                        page.click('button#submit'); // MISSING AWAIT
                        page.fill('input#name', 'Alice'); // MISSING AWAIT
                        page.goto('https://example.com'); // MISSING AWAIT
                        expect(page.locator('#status')).toBeVisible(); // MISSING AWAIT ON MATCHER
                        
                        await page.click('button#ok'); // CORRECT (AWAITED)
                        const result = await page.textContent('#status'); // CORRECT
                        return page.click('button#done'); // CORRECT (RETURNED)
                    } catch (e) {
                        console.error(e);
                    }
                }
            }
        });
    "#;

    let violations = RuleScanner::scan_content("nested_test.ts", code);
    let floating_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "ANTI_PATTERN_FLOATING_PROMISE")
        .collect();

    assert_eq!(floating_violations.len(), 4, "Should catch 4 unawaited promises in nested blocks");
    for v in &floating_violations {
        assert_eq!(v.severity, Severity::Error);
        assert!(v.suggested_fix.as_ref().unwrap().starts_with("await "));
    }
}

#[test]
fn test_adversarial_vacuous_and_missing_assertions_polyglot() {
    // 1. Polyglot vacuous assertions
    let ts_tautology = "expect(true).toBe(true);\nexpect(1).toBe(1);";
    let ts_v = RuleScanner::scan_content("tautology.ts", ts_tautology);
    assert_eq!(ts_v.iter().filter(|v| v.rule_id == "ANTI_PATTERN_VACUOUS_ASSERTION").count(), 2);

    let py_tautology = "assert True\nassert 1 == 1";
    let py_v = RuleScanner::scan_content("tautology_test.py", py_tautology);
    assert_eq!(py_v.iter().filter(|v| v.rule_id == "ANTI_PATTERN_VACUOUS_ASSERTION").count(), 2);

    let java_tautology = "assertTrue(true);";
    let java_v = RuleScanner::scan_content("TautologyTest.java", java_tautology);
    assert_eq!(java_v.iter().filter(|v| v.rule_id == "ANTI_PATTERN_VACUOUS_ASSERTION").count(), 1);

    let rs_tautology = "assert!(true);\nassert_eq!(true, true);\nassert_eq!(1, 1);";
    let rs_v = RuleScanner::scan_content("tautology_test.rs", rs_tautology);
    assert_eq!(rs_v.iter().filter(|v| v.rule_id == "ANTI_PATTERN_VACUOUS_ASSERTION").count(), 3);

    // 2. Test file with zero assertions
    let test_without_assertions = r#"
        test('test without assertions', async ({ page }) => {
            await page.goto('/login');
            await page.fill('#username', 'user');
            await page.click('#submit');
        });
    "#;
    let missing_v = RuleScanner::scan_content("test_checkout_spec.ts", test_without_assertions);
    assert!(missing_v.iter().any(|v| v.rule_id == "ANTI_PATTERN_MISSING_ASSERTION"));
}

#[test]
fn test_adversarial_fix_it_together_patching_edge_cases() {
    // Edge case 1: Multiple violations out of order, preserving indentation
    let source_with_multiple_issues = r#"import { test, expect } from '@playwright/test';

test('complex checkout flow', async ({ page }) => {
    await page.goto('http://localhost:8080');
    page.click('#login-btn');
    await page.waitForTimeout(5000);
    expect(true).toBe(true);
});
"#;

    let violations = RuleScanner::scan_content("test_complex.ts", source_with_multiple_issues);
    assert_eq!(violations.len(), 3);

    let patched = apply_automated_fixes(source_with_multiple_issues, &violations);
    assert!(!patched.contains("waitForTimeout"));
    assert!(!patched.contains("expect(true).toBe(true)"));
    assert!(patched.contains("await page.click('#login-btn')"));

    // Verify indentation is preserved properly
    for line in patched.lines() {
        if line.contains("await page.click") || line.contains("expect(actualValue)") {
            assert!(line.starts_with("    "), "Indentation of 4 spaces must be preserved");
        }
    }

    // Verify unified diff generation produces standard git diff header
    let diff = generate_unified_diff(source_with_multiple_issues, &patched, "test_complex.ts");
    assert!(diff.starts_with("--- a/test_complex.ts"));
    assert!(diff.contains("+++ b/test_complex.ts"));
    assert!(diff.contains("-    await page.waitForTimeout(5000);"));

    // Edge case 2: Idempotent patching on already clean source
    let clean_source = r#"import { test, expect } from '@playwright/test';

test('clean test', async ({ page }) => {
    await page.goto('http://localhost:8080');
    await page.getByRole('button', { name: 'Submit' }).click();
    await expect(page.getByTestId('status')).toBeVisible();
});
"#;
    let clean_violations = RuleScanner::scan_content("clean.ts", clean_source);
    let patched_clean = apply_automated_fixes(clean_source, &clean_violations);
    assert_eq!(clean_source, patched_clean);

    // Edge case 3: Single fix application by rule_id on disk
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("temp_fix_test.ts");
    fs::write(&temp_file, source_with_multiple_issues).expect("Write temp file");

    let result = apply_fix(&temp_file, "ANTI_PATTERN_HARDCODED_SLEEP");
    assert!(result.is_ok());
    let modified_content = fs::read_to_string(&temp_file).expect("Read modified file");
    assert!(!modified_content.contains("waitForTimeout"));

    // Cleanup
    let _ = fs::remove_file(temp_file);
}

// =============================================================================
// SECTION 2: CI/CD PIPELINE SIMULATOR ADVERSARIAL STRESS TESTS
// =============================================================================

#[test]
fn test_adversarial_matrix_expansion_complex_3x3x2_and_includes_excludes() {
    let raw_yaml = r#"
name: Complex Matrix Pipeline
on: [push, pull_request]

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  test-matrix:
    name: Test Matrix
    runs-on: ${{ matrix.os }}
    timeout-minutes: 30
    strategy:
      fail-fast: false
      max-parallel: 8
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        node-version: [18, 20, 22]
        browser: [chromium, firefox]
        exclude:
          - os: macos-latest
            browser: firefox
          - os: windows-latest
            node-version: 18
        include:
          - os: ubuntu-latest
            node-version: 22
            browser: webkit
            experimental: true
          - os: linux-arm64
            node-version: 20
            browser: chromium
            runner-label: self-hosted-arm
    steps:
      - uses: actions/checkout@v4
      - name: Run Playwright E2E Tests
        run: npx playwright test --project=${{ matrix.browser }}
      - name: Archive Test Results & Allure
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report-${{ matrix.os }}-${{ matrix.node-version }}-${{ matrix.browser }}
          path: allure-results/
"#;

    let workflow = parse_workflow_str(raw_yaml).expect("Valid YAML syntax");
    let job = workflow.jobs.get("test-matrix").expect("job exists");
    let matrix = job.strategy.as_ref().unwrap().matrix.as_ref().unwrap();

    let combinations = matrix.expand_combinations();

    // Base Cartesian: 3 os * 3 node * 2 browser = 18 combinations.
    // Exclude 1: macos-latest + firefox (matches 3 combinations: node 18, 20, 22) -> -3
    // Exclude 2: windows-latest + node 18 (matches 2 combinations: chromium, firefox) -> -2
    // Base after excludes: 18 - 3 - 2 = 13 combinations.
    // Include 1: ubuntu-latest + node 22 + browser webkit (doesn't match existing tuple since browser was chromium/firefox) -> +1
    // Include 2: linux-arm64 + node 20 + browser chromium (new OS) -> +1
    // Total expected combinations = 13 + 2 = 15.
    assert_eq!(combinations.len(), 15, "Matrix expansion should yield 15 combinations");

    // Verify exclusions worked
    for combo in &combinations {
        if combo.get("os").map(|s| s.as_str()) == Some("macos-latest") {
            assert_ne!(combo.get("browser").map(|s| s.as_str()), Some("firefox"));
        }
        if combo.get("os").map(|s| s.as_str()) == Some("windows-latest") {
            assert_ne!(combo.get("node-version").map(|s| s.as_str()), Some("18"));
        }
    }

    // Verify includes worked
    let arm_combo = combinations.iter().find(|c| c.get("os").map(|s| s.as_str()) == Some("linux-arm64"));
    assert!(arm_combo.is_some());
    assert_eq!(arm_combo.unwrap().get("runner-label").map(|s| s.as_str()), Some("self-hosted-arm"));

    // Verify pipeline validation passes with score 100
    let validation = validate_definition(&workflow, &ValidationConfig::strict());
    assert!(validation.valid);
    assert_eq!(validation.sdet_score, 100);
    assert!(validation.matrix_detected);
    assert!(validation.artifact_upload_detected);
}

#[test]
fn test_adversarial_strict_sdet_validation_failures_and_actionable_errors() {
    // 1. Workflow missing strategy.matrix on test job
    let missing_matrix_yaml = r#"
name: Missing Matrix CI
on: push
jobs:
  e2e-test:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - run: npm test
      - uses: actions/upload-artifact@v4
        with:
          path: test-results/
"#;
    let val1 = validate_workflow(missing_matrix_yaml);
    assert!(!val1.valid, "Must fail strict validation when matrix is missing");
    let err = val1.errors.iter().find(|e| e.code == "MISSING_MATRIX_STRATEGY");
    assert!(err.is_some());
    assert!(err.unwrap().suggestion.as_ref().unwrap().contains("strategy: matrix:"));

    // 2. Workflow missing actions/upload-artifact on test job
    let missing_artifacts_yaml = r#"
name: Missing Artifacts CI
on: push
jobs:
  e2e-test:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - run: pytest tests/
"#;
    let val2 = validate_workflow(missing_artifacts_yaml);
    assert!(!val2.valid, "Must fail strict validation when artifact upload is missing");
    let err = val2.errors.iter().find(|e| e.code == "MISSING_ARTIFACT_UPLOAD");
    assert!(err.is_some());
    assert!(err.unwrap().suggestion.as_ref().unwrap().contains("actions/upload-artifact"));

    // 3. Workflow with plaintext credentials and tokens
    let insecure_yaml = r#"
name: Insecure CI
on: push
env:
  GITHUB_TOKEN: ghp_11223344556677889900aabbccddeeff0011
jobs:
  test:
    runs-on: ubuntu-latest
    timeout-minutes: 15
    strategy:
      matrix:
        node: [18, 20]
    steps:
      - run: pytest
        env:
          AWS_KEY: AKIA1234567890ABCDEF
      - uses: actions/upload-artifact@v4
        with:
          path: results/
"#;
    let val3 = validate_workflow(insecure_yaml);
    assert!(!val3.valid);
    let secret_errors: Vec<_> = val3.errors.iter().filter(|e| e.code == "HARDCODED_SECRET").collect();
    assert_eq!(secret_errors.len(), 2, "Should catch 2 hardcoded secrets");
}

#[test]
fn test_adversarial_malformed_yaml_and_extreme_timeout_handling() {
    // Malformed YAML
    let malformed_yaml = "name: Bad YAML\njobs:\n  test:\n    runs-on: [unterminated list";
    let val = validate_workflow(malformed_yaml);
    assert!(!val.valid);
    assert_eq!(val.sdet_score, 0);
    assert!(val.errors.iter().any(|e| e.code == "YAML_SYNTAX_ERROR"));

    // Extreme timeout (>120 mins)
    let extreme_timeout_yaml = r#"
name: Extreme Timeout CI
on: push
jobs:
  test-job:
    runs-on: ubuntu-latest
    timeout-minutes: 480
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    steps:
      - run: cargo test
      - uses: actions/upload-artifact@v4
        with:
          path: target/
"#;
    let val_timeout = validate_workflow(extreme_timeout_yaml);
    assert!(val_timeout.warnings.iter().any(|w| w.code == "EXCESSIVE_TIMEOUT"));
}

#[test]
fn test_adversarial_pipeline_runner_mock_parallel_execution_and_fail_fast() {
    let raw_yaml = r#"
name: Runner Test Pipeline
on: push
jobs:
  unit-test:
    runs-on: ubuntu-latest
    timeout-minutes: 10
    strategy:
      fail-fast: true
      matrix:
        shard: ["1/2", "2/2"]
    steps:
      - name: Setup
        run: echo "setting up"
      - name: Run Tests
        run: npm test
      - name: Publish Artifacts
        uses: actions/upload-artifact@v4
        with:
          path: test-results/
"#;

    let workflow = parse_workflow_str(raw_yaml).expect("Valid YAML");
    let opts = PipelineRunOptions {
        parallel: true,
        fail_fast: true,
        animated: false,
        max_parallel: Some(4),
        verbose: false,
        strict_validation: false,
    };

    let result = run_workflow(&workflow, &opts);
    assert_eq!(result.workflow_name, "Runner Test Pipeline");
    assert_eq!(result.jobs.len(), 2, "Should spawn 2 runner instances for matrix shards");
    assert!(result.duration_ms < 5000, "Simulation should complete rapidly");
    assert!(result.jobs.iter().all(|j| j.status == JobStatus::Passed));
    assert!(result.logs.len() > 5, "Should generate execution logs");
}
