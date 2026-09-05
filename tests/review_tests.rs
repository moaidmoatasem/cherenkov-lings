use cherenkov_lings::review::{
    AiMentorClient, AstViolation, ReviewConfig, RuleScanner, Severity, SupportedLanguage,
    apply_all_fixes, apply_automated_fixes, apply_fix, calculate_score, generate_unified_diff,
    run_review, run_review_on_content,
};
use std::fs;
use std::path::Path;

#[test]
fn test_language_detection_from_file_extensions() {
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.ts")),
        SupportedLanguage::TypeScript
    );
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.tsx")),
        SupportedLanguage::TypeScript
    );
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.js")),
        SupportedLanguage::JavaScript
    );
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.py")),
        SupportedLanguage::Python
    );
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.java")),
        SupportedLanguage::Java
    );
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.rs")),
        SupportedLanguage::Rust
    );
    assert_eq!(
        SupportedLanguage::from_path(Path::new("test.txt")),
        SupportedLanguage::Unknown
    );
}

#[test]
fn test_ast_hardcoded_sleep_detection_polyglot() {
    // TypeScript / JS
    let ts_code = r#"
        test('flaky sleep test', async ({ page }) => {
            await page.goto('/login');
            await page.waitForTimeout(5000);
            expect(page).toBeDefined();
        });
    "#;
    let ts_violations = RuleScanner::scan_content("test.ts", ts_code);
    assert!(
        ts_violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP" && v.line_number == 4)
    );
    assert_eq!(ts_violations[0].severity, Severity::Error);

    // Python
    let py_code = r#"
import time

def test_slow_search(page):
    page.goto("/search")
    time.sleep(3.5)
    assert page.title() == "Results"
    "#;
    let py_violations = RuleScanner::scan_content("test.py", py_code);
    assert!(
        py_violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP" && v.line_number == 6)
    );

    // Java
    let java_code = r#"
        @Test
        public void testCheckoutFlow() throws InterruptedException {
            driver.get("http://shop.local");
            Thread.sleep(3000);
            assertTrue(driver.getTitle().contains("Shop"));
        }
    "#;
    let java_violations = RuleScanner::scan_content("CheckoutTest.java", java_code);
    assert!(
        java_violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP" && v.line_number == 5)
    );

    // Rust
    let rs_code = r#"
        #[tokio::test]
        async fn test_async_worker() {
            std::thread::sleep(std::time::Duration::from_millis(500));
            assert_eq!(2 + 2, 4);
        }
    "#;
    let rs_violations = RuleScanner::scan_content("test.rs", rs_code);
    assert!(
        rs_violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP")
    );
}

#[test]
fn test_ast_fragile_locator_detection() {
    let bad_locators_code = r#"
        test('locator fragility', async ({ page }) => {
            // Absolute xpath
            const btn1 = page.locator('/html/body/div[1]/div/div[2]/button');
            // Deep CSS
            const btn2 = page.locator('div > div > div > button');
            // Auto generated ID
            const input = page.locator('#input-a1b2c3d4e5');
            expect(btn1).toBeDefined();
        });
    "#;
    let violations = RuleScanner::scan_content("test.ts", bad_locators_code);

    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_FRAGILE_LOCATOR_XPATH")
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_FRAGILE_LOCATOR_CSS")
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_FRAGILE_LOCATOR_AUTO_ID")
    );
}

#[test]
fn test_ast_floating_unawaited_promise_detection() {
    let unawaited_code = r#"
        test('floating promise hazard', async ({ page }) => {
            page.goto('http://localhost:8080');
            page.click('button#submit');
            expect(page).toBeDefined();
        });
    "#;
    let violations = RuleScanner::scan_content("test.ts", unawaited_code);
    let floating_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "ANTI_PATTERN_FLOATING_PROMISE")
        .collect();

    assert_eq!(floating_violations.len(), 2);
    assert!(floating_violations.iter().any(|v| v.line_number == 3));
    assert!(floating_violations.iter().any(|v| v.line_number == 4));
    assert!(
        floating_violations[0]
            .suggested_fix
            .as_ref()
            .unwrap()
            .starts_with("await ")
    );
}

#[test]
fn test_ast_hardcoded_secrets_detection() {
    let secrets_code = r#"
        test('insecure test', async ({ request }) => {
            const password = "super_secret_password_123!";
            const apiKey = "sk_live_998877665544332211";
            expect(password).toBeDefined();
        });
    "#;
    let violations = RuleScanner::scan_content("test.ts", secrets_code);
    let secret_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SECRET")
        .collect();

    assert_eq!(secret_violations.len(), 2);
    assert_eq!(secret_violations[0].severity, Severity::Error);
    assert!(
        secret_violations[0]
            .message
            .contains("Plaintext credential")
    );
}

#[test]
fn test_ast_vacuous_and_missing_assertions() {
    // Tautological assertion
    let tautology_code = r#"
        test('fake test', async ({ page }) => {
            await page.goto('/');
            expect(true).toBe(true);
        });
    "#;
    let violations = RuleScanner::scan_content("test.ts", tautology_code);
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_VACUOUS_ASSERTION")
    );

    // Missing assertions entirely in test file
    let no_assert_code = r#"
        test('no assertion test', async ({ page }) => {
            await page.goto('/');
            await page.click('button');
        });
    "#;
    let violations_missing = RuleScanner::scan_content("test.ts", no_assert_code);
    assert!(
        violations_missing
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_MISSING_ASSERTION")
    );
}

#[test]
fn test_ast_unsafe_unwraps_and_type_bypass() {
    // Rust raw unwrap
    let rust_code = r#"
        #[test]
        fn test_risky_operation() {
            let res = std::fs::read_to_string("missing.txt").unwrap();
            assert_eq!(res, "hello");
        }
    "#;
    let violations_rs = RuleScanner::scan_content("test.rs", rust_code);
    assert!(
        violations_rs
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_UNSAFE_UNWRAP")
    );

    // TypeScript as any
    let ts_code = r#"
        test('unsafe typing', async ({ page }) => {
            const data: any = await page.evaluate(() => window.payload);
            expect(data).toBeDefined();
        });
    "#;
    let violations_ts = RuleScanner::scan_content("test.ts", ts_code);
    assert!(
        violations_ts
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_UNSAFE_TYPE_BYPASS")
    );
}

#[test]
fn test_score_calculation_and_thresholds() {
    let clean_violations: Vec<AstViolation> = vec![];
    assert_eq!(calculate_score(&clean_violations), 100);

    let error_violations = vec![AstViolation {
        rule_id: "ANTI_PATTERN_HARDCODED_SLEEP".to_string(),
        severity: Severity::Error,
        file_path: "test.ts".to_string(),
        line_number: 5,
        message: "Sleep".to_string(),
        code_snippet: "await page.waitForTimeout(1000);".to_string(),
        suggested_fix: None,
    }];
    assert_eq!(calculate_score(&error_violations), 75);

    let multiple_violations = vec![
        AstViolation {
            rule_id: "RULE1".to_string(),
            severity: Severity::Error,
            file_path: "test.ts".to_string(),
            line_number: 1,
            message: "Error 1".to_string(),
            code_snippet: "".to_string(),
            suggested_fix: None,
        },
        AstViolation {
            rule_id: "RULE2".to_string(),
            severity: Severity::Warning,
            file_path: "test.ts".to_string(),
            line_number: 2,
            message: "Warning 1".to_string(),
            code_snippet: "".to_string(),
            suggested_fix: None,
        },
        AstViolation {
            rule_id: "RULE3".to_string(),
            severity: Severity::Info,
            file_path: "test.ts".to_string(),
            line_number: 3,
            message: "Info 1".to_string(),
            code_snippet: "".to_string(),
            suggested_fix: None,
        },
    ];
    // 100 - 25 - 10 - 5 = 60
    assert_eq!(calculate_score(&multiple_violations), 60);
}

#[test]
fn test_run_review_on_content_end_to_end() {
    let bad_code = r#"
        import { test, expect } from '@playwright/test';

        test('unstable test', async ({ page }) => {
            page.goto('http://localhost:8080');
            await page.waitForTimeout(5000);
            const password = "my_plain_text_password";
            expect(page).toBeDefined();
        });
    "#;

    let config = ReviewConfig::default();
    let report = run_review_on_content("test.ts", bad_code, &config).expect("Review succeeds");

    assert!(!report.passed);
    assert!(report.score < 80);
    assert!(!report.violations.is_empty());
    assert!(!report.mentor_critique.is_empty());
    assert!(!report.socratic_questions.is_empty());
    assert!(report.suggested_diff.is_some());
}

#[test]
fn test_automated_fixes_application_and_diff() {
    let original = "test('floating', async ({ page }) => {\n    page.goto('http://localhost:8080');\n    expect(page).toBeDefined();\n});\n";
    let violations = RuleScanner::scan_content("test.ts", original);
    assert!(!violations.is_empty());

    let fixed = apply_automated_fixes(original, &violations);
    assert!(fixed.contains("await page.goto('http://localhost:8080');"));

    let diff = generate_unified_diff(original, &fixed, "test.ts");
    assert!(diff.contains("--- a/test.ts"));
    assert!(diff.contains("+++ b/test.ts"));
    assert!(diff.contains("-    page.goto('http://localhost:8080');"));
    assert!(diff.contains("+    await page.goto('http://localhost:8080');"));
}

#[test]
fn test_apply_fix_on_disk_temp_file() {
    let temp_dir = std::env::temp_dir();
    let file_path = temp_dir.join(format!(
        "test_review_fix_{}.ts",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    let code = "test('demo', async ({ page }) => {\n    page.click('button');\n    expect(page).toBeDefined();\n});\n";
    fs::write(&file_path, code).unwrap();

    let res = apply_fix(&file_path, "ANTI_PATTERN_FLOATING_PROMISE");
    assert!(res.is_ok());

    let modified_content = fs::read_to_string(&file_path).unwrap();
    assert!(modified_content.contains("await page.click('button');"));

    // Test apply_all_fixes
    let res_all = apply_all_fixes(&file_path);
    assert!(res_all.is_ok());

    let _ = fs::remove_file(file_path);
}

#[test]
fn test_ai_mentor_offline_socratic_feedback() {
    let client = AiMentorClient::default();
    let violations = vec![
        AstViolation {
            rule_id: "ANTI_PATTERN_HARDCODED_SLEEP".to_string(),
            severity: Severity::Error,
            file_path: "test.ts".to_string(),
            line_number: 10,
            message: "Hardcoded sleep".to_string(),
            code_snippet: "page.waitForTimeout(3000)".to_string(),
            suggested_fix: None,
        },
        AstViolation {
            rule_id: "ANTI_PATTERN_FRAGILE_LOCATOR_XPATH".to_string(),
            severity: Severity::Error,
            file_path: "test.ts".to_string(),
            line_number: 12,
            message: "Absolute XPath".to_string(),
            code_snippet: "page.locator('//div/div/button')".to_string(),
            suggested_fix: None,
        },
    ];

    let review = client.generate_offline_mentor_review("flaky_test.ts", "", &violations);
    assert!(review.critique.contains("Hardcoded Sleep Anti-Pattern"));
    assert!(review.critique.contains("Fragile Structural Locators"));
    assert!(review.socratic_questions.len() >= 2);
    assert!(!review.architectural_advice.is_empty());
}

#[test]
fn test_clean_test_review_full_score() {
    let clean_code = r#"
        import { test, expect } from '@playwright/test';

        test('resilient login flow', async ({ page }) => {
            await page.goto('/login');
            await page.getByRole('textbox', { name: 'Username' }).fill('alice');
            await page.getByRole('textbox', { name: 'Password' }).fill(process.env.TEST_PASSWORD || '');
            await page.getByRole('button', { name: 'Sign in' }).click();
            await expect(page.getByRole('heading', { name: 'Dashboard' })).toBeVisible();
        });
    "#;

    let config = ReviewConfig::default();
    let report =
        run_review_on_content("clean_test.ts", clean_code, &config).expect("Review passes");

    assert_eq!(report.score, 100);
    assert!(report.passed);
    assert!(report.violations.is_empty());
    assert!(report.mentor_critique.contains("Exemplary test design"));
    assert!(report.suggested_diff.is_none());
}

#[test]
fn test_run_review_on_actual_file_path() {
    let drill_path =
        Path::new("exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts");
    if drill_path.exists() {
        let config = ReviewConfig::default();
        let report = run_review(drill_path, &config).expect("Review on file path succeeds");
        assert!(!report.exercise_name.is_empty());
    }
}

#[test]
fn test_ast_java_performance_traps_client_churn_and_timeouts() {
    let java_code = r#"
        package com.cherenkov.api;

        import io.restassured.RestAssured;
        import org.junit.jupiter.api.Test;
        import static io.restassured.RestAssured.given;
        import static io.restassured.module.jsv.JsonSchemaValidator.matchesJsonSchemaInClasspath;

        public class UserApiTest {
            @Test
            public void testGetUserProfile() {
                // Client churn trap
                RestAssured.reset();

                // Missing timeout trap and inline schema reload trap
                given()
                    .when()
                    .get("/users/42")
                    .then()
                    .statusCode(200)
                    .body(matchesJsonSchemaInClasspath("schemas/user.json"));
            }
        }
    "#;

    let violations = RuleScanner::scan_content("UserApiTest.java", java_code);
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_CLIENT_CHURN"),
        "Should detect RestAssured.reset() client churn"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_MISSING_TIMEOUT"),
        "Should detect missing socket/connection timeouts on given() call"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_REPEATED_SCHEMA_RELOAD"),
        "Should detect inline matchesJsonSchemaInClasspath schema reload"
    );
}

#[test]
fn test_ast_java_resilient_avoids_performance_traps() {
    let resilient_java = r#"
        package com.cherenkov.api;

        import io.restassured.RestAssured;
        import io.restassured.config.RestAssuredConfig;
        import org.hamcrest.Matcher;
        import org.junit.jupiter.api.BeforeAll;
        import org.junit.jupiter.api.Test;
        import static io.restassured.RestAssured.given;
        import static io.restassured.config.HttpClientConfig.httpClientConfig;
        import static io.restassured.module.jsv.JsonSchemaValidator.matchesJsonSchemaInClasspath;

        public class ResilientUserApiTest {
            private static final Matcher<String> USER_SCHEMA = matchesJsonSchemaInClasspath("schemas/user.json");

            @BeforeAll
            public static void setup() {
                RestAssured.config = RestAssuredConfig.config()
                    .httpClient(httpClientConfig().setParam("http.connection.timeout", 5000).setParam("http.socket.timeout", 5000));
            }

            @Test
            public void testGetUserProfile() {
                given()
                    .when()
                    .get("/users/42")
                    .then()
                    .statusCode(200)
                    .body(USER_SCHEMA);
            }
        }
    "#;

    let violations = RuleScanner::scan_content("ResilientUserApiTest.java", resilient_java);
    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_CLIENT_CHURN"),
        "No client churn should be reported"
    );
    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_MISSING_TIMEOUT"),
        "No missing timeout should be reported when timeout configured"
    );
    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_REPEATED_SCHEMA_RELOAD"),
        "Static final matcher should not be flagged as schema reload"
    );
}

#[test]
fn test_ast_python_performance_traps_async_blocking_and_fixture_scope() {
    let py_code = r#"
import pytest
import time
import requests
from sqlalchemy import create_engine

@pytest.fixture
def db_conn():
    engine = create_engine("sqlite:///test.db")
    return engine

@pytest.fixture(scope="function")
def api_client():
    return requests.Session()

def test_sync_leak():
    client = requests.Session()
    resp = client.get("http://localhost:8080")
    assert resp.status_code == 200

async def test_async_blocking():
    time.sleep(2.0)
    response = requests.get("http://localhost:8080")
    assert response.status_code == 200
    "#;

    let violations = RuleScanner::scan_content("test_perf_traps.py", py_code);
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_INEFFICIENT_FIXTURE_SCOPE"),
        "Should detect inefficient function scope on db/client fixtures"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_UNCLOSED_SESSION"),
        "Should detect unclosed requests.Session() without context manager"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_BLOCKING_CALL_IN_ASYNC"),
        "Should detect blocking time.sleep/requests.get inside async def"
    );
    assert!(
        violations
            .iter()
            .any(|v| v.rule_id == "ANTI_PATTERN_HARDCODED_SLEEP"),
        "Should also flag hardcoded sleep"
    );
}

#[test]
fn test_ast_python_resilient_avoids_performance_traps() {
    let clean_py = r#"
import pytest
import asyncio
import httpx
from sqlalchemy import create_engine

@pytest.fixture(scope="session")
def db_conn():
    engine = create_engine("sqlite:///test.db")
    yield engine
    engine.dispose()

@pytest.fixture(scope="session")
def client():
    with httpx.Client() as client:
        yield client

def test_sync_with_context():
    with httpx.Client() as client:
        resp = client.get("http://localhost:8080")
        assert resp.status_code == 200

async def test_async_clean():
    await asyncio.sleep(0.01)
    async with httpx.AsyncClient() as client:
        resp = await client.get("http://localhost:8080")
        assert resp.status_code == 200
    "#;

    let violations = RuleScanner::scan_content("test_clean.py", clean_py);
    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_INEFFICIENT_FIXTURE_SCOPE"),
        "Session scope fixture should not be flagged"
    );
    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_UNCLOSED_SESSION"),
        "Context manager client should not be flagged"
    );
    assert!(
        !violations
            .iter()
            .any(|v| v.rule_id == "PERF_TRAP_BLOCKING_CALL_IN_ASYNC"),
        "Async sleep and async client should not be flagged"
    );
}
