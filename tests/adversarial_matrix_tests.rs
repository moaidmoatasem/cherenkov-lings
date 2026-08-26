#![allow(dead_code, unused_imports)]

// src/runner.rs references crate::pipeline (the in-process CI simulator
// backing the ci-pipeline track), so the module must exist in this test
// crate too for the #[path] include below to resolve.
#[path = "../src/pipeline/mod.rs"]
mod pipeline;

#[path = "../src/runner.rs"]
mod runner;

#[path = "../src/feedback.rs"]
mod feedback;

use feedback::*;
use runner::{DrillResponse, RunResult};
use std::time::Instant;

// =========================================================================
// 1. AST COMMENT STRIPPER & TOKENIZER ADVERSARIAL TESTS
// =========================================================================

#[test]
fn test_comment_stripper_single_line_wait_for_timeout_ignored() {
    let source = r#"
import { test, expect } from '@playwright/test';

test('commented out anti-pattern', async ({ page }) => {
    await page.goto('/checkout');
    // await page.waitForTimeout(5000);
    // page.waitForTimeout(2000);
    // waitForTimeout(1000);
    const btn = page.getByRole('button', { name: 'Submit' });
    await expect(btn).toBeEnabled();
    await btn.click();
});
"#;
    let report = analyze_source(source, "test_comments.ts");
    assert!(
        !report.has_wait_for_timeout,
        "Commented out waitForTimeout should NOT flag has_wait_for_timeout"
    );
    assert_eq!(
        report.anti_patterns.len(),
        0,
        "No anti-patterns should be detected in commented lines"
    );
    assert_eq!(report.locators.len(), 1);
    assert_eq!(report.locators[0].kind, LocatorKind::GetByRole);
    assert_eq!(report.locator_quality_score, 100.0);
}

#[test]
fn test_comment_stripper_multi_line_block_comment_ignored() {
    let source = r#"
import { test, expect } from '@playwright/test';

test('block commented anti-patterns', async ({ page }) => {
    /*
     * Anti-pattern documentation:
     * await page.waitForTimeout(3000);
     * window.setTimeout(() => {}, 1000);
     * await page.locator('/html/body/div/button').click();
     * await page.locator('.flaky-btn').click();
     */
    const submitBtn = page.getByRole('button', { name: 'Submit' });
    await expect(submitBtn).toBeVisible();
});
"#;
    let report = analyze_source(source, "test_block_comments.ts");
    assert!(!report.has_wait_for_timeout);
    assert_eq!(report.anti_patterns.len(), 0);
    assert_eq!(report.locators.len(), 1);
    assert_eq!(report.locators[0].kind, LocatorKind::GetByRole);
    assert_eq!(report.locator_quality_score, 100.0);
}

#[test]
fn test_comment_stripper_inline_trailing_comment_ignored() {
    let source = r#"
await page.getByRole('button', { name: 'Submit' }).click(); // await page.waitForTimeout(5000);
"#;
    let report = analyze_source(source, "test_inline.ts");
    assert!(!report.has_wait_for_timeout);
    assert_eq!(report.anti_patterns.len(), 0);
    assert_eq!(report.locators.len(), 1);
    assert_eq!(report.locators[0].kind, LocatorKind::GetByRole);
}

#[test]
fn test_comment_stripper_string_literals_and_urls_preserved() {
    let source = r#"
const url1 = "https://api.example.com/v1/waitForTimeout/status";
const url2 = 'http://localhost:8080/checkout//double-slash';
const template = `http://domain.com/path//${id}/*not-a-comment*/`;
const commentInString = "This is not /* a block comment */ and not // a line comment";
const escapedQuotes = "He said \"Don't use waitForTimeout\" in test";
const escapedBackslashes = "C:\\\\path\\\\to\\\\test";
"#;
    let stripped = strip_comments(source);
    // Line count must be preserved
    assert_eq!(source.lines().count(), stripped.lines().count());

    // Strings and URLs with slashes and asterisks must be preserved verbatim
    assert!(stripped.contains("https://api.example.com/v1/waitForTimeout/status"));
    assert!(stripped.contains("http://localhost:8080/checkout//double-slash"));
    assert!(stripped.contains("/*not-a-comment*/"));
    assert!(stripped.contains("This is not /* a block comment */"));
    assert!(stripped.contains("He said \\\"Don't use waitForTimeout\\\""));
    assert!(stripped.contains(r#"C:\\\\path\\\\to\\\\test"#));
}

#[test]
fn test_quote_styles_for_all_locator_types() {
    let source = r#"
// Single quotes
const b1 = page.locator('button.single');
const t1 = page.getByTestId('test-single');
const r1 = page.getByRole('button', { name: 'Role Single' });
const x1 = page.getByText('Text Single');
const l1 = page.getByLabel('Label Single');

// Double quotes
const b2 = page.locator("button.double");
const t2 = page.getByTestId("test-double");
const r2 = page.getByRole("checkbox", { name: "Role Double" });
const x2 = page.getByText("Text Double");
const l2 = page.getByLabel("Label Double");

// Backticks / Template literals
const b3 = page.locator(`button.template`);
const t3 = page.getByTestId(`test-template`);
const r3 = page.getByRole(`link`, { name: `Role Template` });
const x3 = page.getByText(`Text Template`);
const l3 = page.getByLabel(`Label Template`);
"#;
    let report = analyze_source(source, "test_quotes.ts");

    assert_eq!(report.locators.len(), 15);

    let get_by_role_count = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::GetByRole)
        .count();
    let get_by_testid_count = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::GetByTestId)
        .count();
    let get_by_text_label_count = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::GetByTextOrLabel)
        .count();
    let css_count = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::CssSelector)
        .count();

    assert_eq!(get_by_role_count, 3, "3 getByRole with ', \", ` quotes");
    assert_eq!(get_by_testid_count, 3, "3 getByTestId with ', \", ` quotes");
    assert_eq!(
        get_by_text_label_count, 6,
        "6 getByText/Label with ', \", ` quotes"
    );
    assert_eq!(css_count, 3, "3 page.locator with ', \", ` quotes");

    let expected_sum = (3.0 * 100.0) + (3.0 * 85.0) + (6.0 * 90.0) + (3.0 * 40.0);
    let expected_avg = expected_sum / 15.0; // 81.0
    assert_eq!(report.locator_quality_score, expected_avg);
}

#[test]
fn test_mixed_and_edge_case_xpath_and_css_selectors() {
    let source = r#"
// Absolute XPath variants (0 pts)
const x1 = page.locator('/html/body/div[1]/form/input');
const x2 = page.locator('//div[@class="container"]//button');
const x3 = page.locator('xpath=/html/body/div/span');
const x4 = page.locator("xpath=//button[text()='Submit']");
const x5 = page.locator(`xpath=/html/body/chaos-vault`);

// TestID attribute selectors in locator() (85 pts)
const t1 = page.locator('[data-testid="checkout-btn"]');
const t2 = page.locator('[data-testid=\'checkout-btn\']');
const t3 = page.locator(`[data-testid="checkout-btn"]`);
const t4 = page.locator('[data-test="order-id"]');

// CSS Selectors (40 pts)
const c1 = page.locator('#header-title');
const c2 = page.locator('.btn-primary');
const c3 = page.locator('div > ul > li');
const c4 = page.locator('div.item span');
const c5 = page.locator('button:disabled');
const c6 = page.locator('[class*="btn-active"]');
const c7 = page.locator('button');
const c8 = page.$('.jquery-style-selector');
const c9 = page.$$('div.items');
"#;
    let report = analyze_source(source, "test_xpath_css.ts");

    let xpath_locators: Vec<_> = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::AbsoluteXPath)
        .collect();
    let testid_locators: Vec<_> = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::GetByTestId)
        .collect();
    let css_locators: Vec<_> = report
        .locators
        .iter()
        .filter(|l| l.kind == LocatorKind::CssSelector)
        .collect();

    assert_eq!(
        xpath_locators.len(),
        5,
        "5 absolute XPath selectors detected"
    );
    assert_eq!(
        testid_locators.len(),
        4,
        "4 data-testid/data-test attribute selectors detected"
    );
    assert_eq!(
        css_locators.len(),
        9,
        "9 CSS selectors detected (including $, $$)"
    );

    assert_eq!(report.anti_patterns.len(), 13);
}

#[test]
fn test_ast_behavior_on_string_containing_wait_for_timeout_identifier() {
    let source = r#"
const apiUrl = "http://localhost:8080/api/waitForTimeout/metric";
const logMsg = "The server reported waitForTimeout was prevented";
await page.getByRole('button', { name: 'Submit' }).click();
"#;
    let report = analyze_source(source, "test_no_parens.ts");
    assert!(
        !report.has_wait_for_timeout,
        "waitForTimeout without parentheses in string should not trigger anti-pattern"
    );
    assert_eq!(report.anti_patterns.len(), 0);
    assert_eq!(report.locators.len(), 1);
}

#[test]
fn test_stress_ast_analysis_on_10000_lines() {
    let mut big_source = String::with_capacity(1_000_000);
    for i in 0..5000 {
        big_source.push_str(&format!(
            "// Line {} comment with waitForTimeout(1000)\n",
            i
        ));
        big_source.push_str("await page.getByRole('button', { name: 'Submit' }).click();\n");
    }

    let start = Instant::now();
    let report = analyze_source(&big_source, "benchmark_10k.ts");
    let elapsed = start.elapsed();

    assert_eq!(report.locators.len(), 5000);
    assert_eq!(report.anti_patterns.len(), 0);
    assert_eq!(report.locator_quality_score, 100.0);
    assert!(!report.has_wait_for_timeout);
    assert!(
        elapsed.as_millis() < 500,
        "10,000 lines AST analysis must execute under 500ms (took {}ms)",
        elapsed.as_millis()
    );
}

#[test]
fn test_unclosed_comments_and_strings_graceful_handling() {
    // 1. Unclosed block comment at EOF
    let unclosed_comment =
        "const a = 1; /* unclosed block comment with await page.waitForTimeout(500);";
    let r1 = analyze_source(unclosed_comment, "unclosed_comment.ts");
    assert!(
        !r1.has_wait_for_timeout,
        "Unclosed block comment should not crash or trigger anti-pattern"
    );

    // 2. Unclosed single quote string at EOF
    let unclosed_quote = "const s = 'unclosed string; await page.getByRole('button');";
    let r2 = analyze_source(unclosed_quote, "unclosed_quote.ts");
    assert_eq!(r2.total_lines, 1);

    // 3. Empty source string
    let empty_report = analyze_source("", "empty.ts");
    assert_eq!(empty_report.total_lines, 0);
    assert_eq!(empty_report.locators.len(), 0);
    assert_eq!(empty_report.anti_patterns.len(), 0);
    assert_eq!(empty_report.locator_quality_score, 100.0);
}

// =========================================================================
// 2. SCORING MECHANICS, WEIGHTS & FLAKINESS CAP VERIFICATION
// =========================================================================

#[test]
fn test_weights_sum_to_one() {
    let total_weight =
        WEIGHT_CORRECTNESS + WEIGHT_FLAKINESS + WEIGHT_LOCATOR_QUALITY + WEIGHT_SPEED;
    assert!(
        (total_weight - 1.0).abs() < f64::EPSILON,
        "Weights must sum to exactly 1.0"
    );
    assert_eq!(WEIGHT_CORRECTNESS, 0.35);
    assert_eq!(WEIGHT_FLAKINESS, 0.35);
    assert_eq!(WEIGHT_LOCATOR_QUALITY, 0.15);
    assert_eq!(WEIGHT_SPEED, 0.15);
}

#[test]
fn test_flakiness_penalty_cap_at_40_on_wait_for_timeout() {
    let drill_response = DrillResponse {
        id: "req-cap".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 2500, // 500ms avg <= 1000ms baseline
        runs: vec![
            RunResult {
                iteration: 1,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 2,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 3,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 4,
                passed: true,
                duration_ms: 500,
                error: None,
            },
            RunResult {
                iteration: 5,
                passed: true,
                duration_ms: 500,
                error: None,
            },
        ],
        error: None,
    };

    let ast_with_wait = StaticAnalysisReport {
        file_path: "flaky.ts".to_string(),
        has_wait_for_timeout: true,
        locator_quality_score: 100.0,
        ..Default::default()
    };

    let scorecard = evaluate_feedback(
        &drill_response,
        &ast_with_wait,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );

    // Dimension breakdown:
    // Correctness: 100.0 * 0.35 = 35.0
    // Flakiness: CAPPED AT 40.0 * 0.35 = 14.0
    // Locator: 100.0 * 0.15 = 15.0
    // Speed: 100.0 * 0.15 = 15.0
    // Total Score: 35.0 + 14.0 + 15.0 + 15.0 = 79.0
    assert_eq!(scorecard.correctness.score, 100.0);
    assert_eq!(scorecard.flakiness.score, 40.0);
    assert_eq!(scorecard.locator_quality.score, 100.0);
    assert_eq!(scorecard.speed.score, 100.0);

    assert_eq!(scorecard.total_score, 79.0);
    assert!(
        scorecard.total_score < 85.0,
        "Total score (79.0) must be below the 85.0 passing threshold"
    );
    assert!(
        !scorecard.passed,
        "Scorecard MUST NOT pass when waitForTimeout anti-pattern is present"
    );
}

#[test]
fn test_flakiness_penalty_when_raw_flakiness_is_below_cap() {
    let drill_response = DrillResponse {
        id: "req-low".to_string(),
        ok: true,
        passed: false,
        iterations: 5,
        passed_iterations: 1,
        failed_iterations: 4,
        total_duration_ms: 5000,
        runs: vec![],
        error: None,
    };

    let ast_with_wait = StaticAnalysisReport {
        file_path: "flaky.ts".to_string(),
        has_wait_for_timeout: true,
        locator_quality_score: 40.0,
        ..Default::default()
    };

    let scorecard = evaluate_feedback(
        &drill_response,
        &ast_with_wait,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );

    // Flakiness score: min(20.0, 40.0) = 20.0
    assert_eq!(scorecard.flakiness.score, 20.0);
    assert_eq!(scorecard.flakiness.weighted_score, 7.0);
    assert!(!scorecard.passed);
}

#[test]
fn test_speed_penalty_gradations() {
    let baseline = 1000;
    // 1. Faster than baseline (400ms avg) -> 100.0
    let (s1, avg1) = calculate_speed_score(2000, 5, baseline);
    assert_eq!(avg1, 400);
    assert_eq!(s1, 100.0);

    // 2. Exactly baseline (1000ms avg) -> 100.0
    let (s2, avg2) = calculate_speed_score(5000, 5, baseline);
    assert_eq!(avg2, 1000);
    assert_eq!(s2, 100.0);

    // 3. 1500ms avg (500ms over -> -10 pts) -> 90.0
    let (s3, avg3) = calculate_speed_score(7500, 5, baseline);
    assert_eq!(avg3, 1500);
    assert_eq!(s3, 90.0);

    // 4. 2000ms avg (1000ms over -> -20 pts) -> 80.0
    let (s4, avg4) = calculate_speed_score(10000, 5, baseline);
    assert_eq!(avg4, 2000);
    assert_eq!(s4, 80.0);

    // 5. 3000ms avg (2000ms over -> -40 pts) -> 60.0
    let (s5, avg5) = calculate_speed_score(15000, 5, baseline);
    assert_eq!(avg5, 3000);
    assert_eq!(s5, 60.0);

    // 6. 6000ms avg (5000ms over -> -100 pts) -> 0.0
    let (s6, avg6) = calculate_speed_score(30000, 5, baseline);
    assert_eq!(avg6, 6000);
    assert_eq!(s6, 0.0);

    // 7. 10000ms avg (9000ms over -> clamped to 0.0) -> 0.0
    let (s7, avg7) = calculate_speed_score(50000, 5, baseline);
    assert_eq!(avg7, 10000);
    assert_eq!(s7, 0.0);
}

#[test]
fn test_zero_and_overflow_safeguards_in_evaluator() {
    // 0 iterations in response (defensive division by zero handling)
    let zero_iter_resp = DrillResponse {
        id: "req-zero".to_string(),
        ok: true,
        passed: false,
        iterations: 0,
        passed_iterations: 0,
        failed_iterations: 0,
        total_duration_ms: 0,
        runs: vec![],
        error: None,
    };
    let empty_ast = StaticAnalysisReport::default();
    let card = evaluate_feedback(
        &zero_iter_resp,
        &empty_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    assert_eq!(card.correctness.score, 0.0);
    assert_eq!(card.flakiness.score, 0.0);
    assert_eq!(card.speed.score, 100.0);
    assert!(!card.passed);

    // Huge duration (no overflow)
    let (s_huge, avg_huge) = calculate_speed_score(u64::MAX / 4, 1, 1000);
    assert_eq!(s_huge, 0.0);
    assert!(avg_huge > 1000);
}

// =========================================================================
// 3. EXERCISE VS SOLUTION SCORING VALIDATION ON ACTUAL DRILL FILES
// =========================================================================

#[test]
fn test_drill_01_hydration_timing_exercise_vs_solution() {
    let exercise_path = "exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts";
    let solution_path = "exercises/01_web_playwright_ts/01_hydration_timing/solution.ts";

    let exercise_ast = feedback::analyze_file(exercise_path).expect("Read exercise.ts");
    let solution_ast = feedback::analyze_file(solution_path).expect("Read solution.ts");

    // Exercise AST analysis verification
    assert!(
        exercise_ast.has_wait_for_timeout,
        "Exercise 01 must have waitForTimeout"
    );
    assert_eq!(
        exercise_ast.anti_patterns.len(),
        2,
        "Exercise 01 has waitForTimeout and CSS selector"
    );
    assert_eq!(exercise_ast.locator_quality_score, 62.5); // CSS (40) + TestID (85) / 2 = 62.5

    // Solution AST analysis verification
    assert!(
        !solution_ast.has_wait_for_timeout,
        "Solution 01 must NOT have waitForTimeout"
    );
    assert_eq!(
        solution_ast.anti_patterns.len(),
        0,
        "Solution 01 must have 0 anti-patterns"
    );
    assert_eq!(solution_ast.locator_quality_score, 92.5); // Role (100) + TestID (85) / 2 = 92.5

    // Simulated Exercise Execution under Chaos (fails 4/5 iterations due to dropped clicks, avg 2450ms)
    let exercise_response = DrillResponse {
        id: "drill-01-ex".to_string(),
        ok: true,
        passed: false,
        iterations: 5,
        passed_iterations: 1,
        failed_iterations: 4,
        total_duration_ms: 12250, // 2450ms avg
        runs: vec![],
        error: None,
    };

    let exercise_card = evaluate_feedback(
        &exercise_response,
        &exercise_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    assert!(
        exercise_card.total_score < 85.0,
        "Exercise 01 total score ({}) must be < 85.0",
        exercise_card.total_score
    );
    assert!(!exercise_card.passed, "Exercise 01 must fail");

    // Simulated Solution Execution under Chaos (passes 5/5 iterations, avg 450ms)
    let solution_response = DrillResponse {
        id: "drill-01-sol".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 2250, // 450ms avg <= 1000ms
        runs: vec![],
        error: None,
    };

    let solution_card = evaluate_feedback(
        &solution_response,
        &solution_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    // Correctness: 100.0 * 0.35 = 35.0
    // Flakiness: 100.0 * 0.35 = 35.0
    // Locator: 92.5 * 0.15 = 13.875
    // Speed: 100.0 * 0.15 = 15.0
    // Total: 35 + 35 + 13.875 + 15 = 98.875 >= 85.0
    assert!(
        solution_card.total_score >= 85.0,
        "Solution 01 total score ({}) must be >= 85.0",
        solution_card.total_score
    );
    assert!(solution_card.passed, "Solution 01 must pass");
}

#[test]
fn test_drill_02_shadow_dom_v2_exercise_vs_solution() {
    let exercise_path = "exercises/01_web_playwright_ts/02_shadow_dom_v2/exercise.ts";
    let solution_path = "exercises/01_web_playwright_ts/02_shadow_dom_v2/solution.ts";

    let exercise_ast = feedback::analyze_file(exercise_path).expect("Read exercise.ts");
    let solution_ast = feedback::analyze_file(solution_path).expect("Read solution.ts");

    // Exercise 02: 3 absolute XPath locators
    assert_eq!(exercise_ast.locators.len(), 3);
    assert_eq!(exercise_ast.locator_quality_score, 0.0);
    assert_eq!(exercise_ast.anti_patterns.len(), 3);

    // Solution 02: 4 locators: CSS host (40) + TestID (85) + Role (100) + TestID (85) = 310 / 4 = 77.5
    assert_eq!(solution_ast.locators.len(), 4);
    assert_eq!(solution_ast.locator_quality_score, 77.5);
    assert_eq!(solution_ast.anti_patterns.len(), 0);

    // Exercise fails under Shadow DOM (0/5 passed)
    let exercise_response = DrillResponse {
        id: "drill-02-ex".to_string(),
        ok: true,
        passed: false,
        iterations: 5,
        passed_iterations: 0,
        failed_iterations: 5,
        total_duration_ms: 10000,
        runs: vec![],
        error: Some("Element not found across shadow boundary".to_string()),
    };
    let exercise_card = evaluate_feedback(
        &exercise_response,
        &exercise_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    assert!(exercise_card.total_score < 85.0);
    assert!(!exercise_card.passed);

    // Solution passes 5/5 (avg 380ms)
    let solution_response = DrillResponse {
        id: "drill-02-sol".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 1900,
        runs: vec![],
        error: None,
    };
    let solution_card = evaluate_feedback(
        &solution_response,
        &solution_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    // Correctness: 100 * 0.35 = 35
    // Flakiness: 100 * 0.35 = 35
    // Locator: 77.5 * 0.15 = 11.625
    // Speed: 100 * 0.15 = 15
    // Total: 35 + 35 + 11.625 + 15 = 96.625 >= 85.0
    assert!(solution_card.total_score >= 85.0);
    assert!(solution_card.passed);
}

#[test]
fn test_drill_03_debounce_race_exercise_vs_solution() {
    let exercise_path = "exercises/01_web_playwright_ts/03_debounce_race_condition/exercise.ts";
    let solution_path = "exercises/01_web_playwright_ts/03_debounce_race_condition/solution.ts";

    let exercise_ast = feedback::analyze_file(exercise_path).expect("Read exercise.ts");
    let solution_ast = feedback::analyze_file(solution_path).expect("Read solution.ts");

    // Exercise 03 has waitForTimeout(600)
    assert!(exercise_ast.has_wait_for_timeout);
    assert_eq!(exercise_ast.anti_patterns.len(), 1);

    // Solution 03 has 0 anti-patterns, Role (100) + 3 TestID (85, 85, 85) = 355 / 4 = 88.75
    assert!(!solution_ast.has_wait_for_timeout);
    assert_eq!(solution_ast.anti_patterns.len(), 0);
    assert_eq!(solution_ast.locator_quality_score, 88.75);

    // Exercise fails out-of-order search race (1/5 passed)
    let exercise_response = DrillResponse {
        id: "drill-03-ex".to_string(),
        ok: true,
        passed: false,
        iterations: 5,
        passed_iterations: 1,
        failed_iterations: 4,
        total_duration_ms: 6000,
        runs: vec![],
        error: None,
    };
    let exercise_card = evaluate_feedback(
        &exercise_response,
        &exercise_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    assert!(exercise_card.total_score < 85.0);
    assert!(!exercise_card.passed);

    // Solution passes 5/5 (avg 420ms)
    let solution_response = DrillResponse {
        id: "drill-03-sol".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 2100,
        runs: vec![],
        error: None,
    };
    let solution_card = evaluate_feedback(
        &solution_response,
        &solution_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    // Correctness: 100 * 0.35 = 35
    // Flakiness: 100 * 0.35 = 35
    // Locator: 88.75 * 0.15 = 13.3125
    // Speed: 100 * 0.15 = 15
    // Total: 35 + 35 + 13.3125 + 15 = 98.3125 >= 85.0
    assert!(solution_card.total_score >= 85.0);
    assert!(solution_card.passed);
}

// =========================================================================
// 4. ANSI PROGRESS BAR & SCORECARD RENDERING STRESS TESTS
// =========================================================================

#[test]
fn test_render_progress_bar_clamping_and_coloring() {
    let p_neg = render_progress_bar(-50.0, 10);
    assert!(p_neg.contains("░░░░░░░░░░"));

    let p_zero = render_progress_bar(0.0, 10);
    assert!(p_zero.contains("░░░░░░░░░░"));

    let p_mid = render_progress_bar(50.0, 10);
    assert!(p_mid.contains("█████░░░░░"));

    let p_high = render_progress_bar(85.0, 10);
    assert!(p_high.contains("█████████░"));

    let p_perfect = render_progress_bar(100.0, 10);
    assert!(p_perfect.contains("██████████"));

    let p_over = render_progress_bar(150.0, 10);
    assert!(p_over.contains("██████████"));
}

#[test]
fn test_scorecard_and_diagnostic_formatting_safety() {
    let dummy_resp = DrillResponse {
        id: "safe-test".to_string(),
        ok: true,
        passed: true,
        iterations: 5,
        passed_iterations: 5,
        failed_iterations: 0,
        total_duration_ms: 2500,
        runs: vec![],
        error: None,
    };
    let dummy_ast = StaticAnalysisReport {
        file_path: "exercises/01_web_playwright_ts/01_hydration_timing/solution.ts".to_string(),
        locator_quality_score: 92.5,
        ..Default::default()
    };
    let card = evaluate_feedback(
        &dummy_resp,
        &dummy_ast,
        "playwright-ts",
        "1.0.0",
        85.0,
        1000,
    );
    let rendered_scorecard = render_scorecard(&card);
    assert!(!rendered_scorecard.is_empty());
    assert!(rendered_scorecard.contains("CHERENKOV-LINGS"));

    let rendered_diag = render_diagnostic(&dummy_ast, "playwright-ts", "1.0.0");
    assert!(!rendered_diag.is_empty());
    assert!(rendered_diag.contains("CHERENKOV-LINGS DIAGNOSTIC"));
}
