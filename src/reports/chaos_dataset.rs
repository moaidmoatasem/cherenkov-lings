use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a test execution in the Allure & Chaos engine
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Passed,
    Failed,
    Broken,
    Flaky,
    Skipped,
}

impl std::fmt::Display for TestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestStatus::Passed => write!(f, "passed"),
            TestStatus::Failed => write!(f, "failed"),
            TestStatus::Broken => write!(f, "broken"),
            TestStatus::Flaky => write!(f, "flaky"),
            TestStatus::Skipped => write!(f, "skipped"),
        }
    }
}

/// Root-cause triage taxonomy for test failures
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum FailureCategory {
    /// Genuine Product Defect (HTTP 500, RBAC bypass, DB deadlock, foreign key violation, data corruption)
    RealBug,
    /// Flaky Infrastructure Failure (Proxy latency spike, TCP RST, 502/504 gateway error, DNS drop, port exhaustion)
    FlakyInfra,
    /// Test Automation Anti-Pattern (Hardcoded sleep race, stale DOM element, fragile locator, missing assertion)
    AntiPattern,
    /// Healthy passing test (no failure category)
    #[serde(rename = "none")]
    None,
}

impl std::fmt::Display for FailureCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FailureCategory::RealBug => write!(f, "real_bug"),
            FailureCategory::FlakyInfra => write!(f, "flaky_infra"),
            FailureCategory::AntiPattern => write!(f, "anti_pattern"),
            FailureCategory::None => write!(f, "none"),
        }
    }
}

impl FailureCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            FailureCategory::RealBug => "Genuine Product Defect",
            FailureCategory::FlakyInfra => "Flaky Infrastructure",
            FailureCategory::AntiPattern => "Test Automation Anti-Pattern",
            FailureCategory::None => "Healthy (Passed)",
        }
    }
}

/// Correlated L4/L7 Network and Chaos Proxy Telemetry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChaosEventTelemetry {
    pub layer: String,
    pub event_type: String,
    pub latency_ms: u64,
    pub jitter_ms: u64,
    pub packet_loss_rate: f64,
    pub proxy_log: Option<String>,
    pub correlated_timestamp: String,
    pub retry_attempts: u32,
    pub injection_target: String,
}

impl Default for ChaosEventTelemetry {
    fn default() -> Self {
        Self {
            layer: "L7".to_string(),
            event_type: "none".to_string(),
            latency_ms: 0,
            jitter_ms: 0,
            packet_loss_rate: 0.0,
            proxy_log: None,
            correlated_timestamp: "2026-08-24T18:00:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "127.0.0.1:8086".to_string(),
        }
    }
}

/// Flakiness metrics across multi-iteration stress evaluations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FlakinessMetrics {
    pub iterations: u32,
    pub passed_iterations: u32,
    pub failed_iterations: u32,
    pub flakiness_rate: f64,
    pub avg_duration_ms: u64,
    pub duration_stddev_ms: f64,
    pub historical_flake_score: f64,
}

/// Individual test execution step telemetry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestStepTelemetry {
    pub name: String,
    pub status: TestStatus,
    pub duration_ms: u64,
    pub error: Option<String>,
}

/// Complete chaotic test execution result with rich telemetry
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChaosTestResult {
    pub test_id: String,
    pub name: String,
    pub suite: String,
    pub track_id: String,
    pub status: TestStatus,
    pub category: FailureCategory,
    pub duration_ms: u64,
    pub error_message: Option<String>,
    pub stack_trace: Option<String>,
    pub chaos_event: Option<ChaosEventTelemetry>,
    pub flakiness_metrics: Option<FlakinessMetrics>,
    pub steps: Vec<TestStepTelemetry>,
    pub labels: HashMap<String, String>,
    pub root_cause_hint: Option<String>,
}

/// Generate the realistic deterministic dataset of 70 chaotic test executions
pub fn generate_chaos_dataset() -> Vec<ChaosTestResult> {
    let mut tests = Vec::with_capacity(70);

    // =========================================================================
    // 1. GENUINE PRODUCT DEFECTS (19 Tests)
    // =========================================================================

    tests.push(ChaosTestResult {
        test_id: "BUG-101".to_string(),
        name: "test_auth_role_privilege_escalation".to_string(),
        suite: "RBAC Security Suite".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 145,
        error_message: Some("AssertionError: Expected HTTP 403 Forbidden for non-admin role, received HTTP 200 OK with admin access token".to_string()),
        stack_trace: Some("File \"exercises/07_cloud_devsecops/01_rbac_security/exercise.py\", line 58, in test_auth_role_privilege_escalation\n    assert response.status_code == 403, f\"Privilege escalation vulnerability: {response.json()}\"\nAssertionError: Expected HTTP 403 Forbidden for non-admin role, received HTTP 200 OK with admin access token".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "rbac_bypass".to_string(),
            latency_ms: 22,
            jitter_ms: 4,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [L7 Proxy] POST /api/v1/auth/elevate -> 200 OK (Missing RBAC middleware check on role parameter)".to_string()),
            correlated_timestamp: "2026-08-24T18:01:10Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/auth/elevate".to_string(),
        }),
        flakiness_metrics: Some(FlakinessMetrics {
            iterations: 5,
            passed_iterations: 0,
            failed_iterations: 5,
            flakiness_rate: 0.0,
            avg_duration_ms: 145,
            duration_stddev_ms: 3.2,
            historical_flake_score: 0.0,
        }),
        steps: vec![
            TestStepTelemetry { name: "Authenticate as standard user".to_string(), status: TestStatus::Passed, duration_ms: 40, error: None },
            TestStepTelemetry { name: "Send role elevation request to /api/v1/auth/elevate".to_string(), status: TestStatus::Passed, duration_ms: 45, error: None },
            TestStepTelemetry { name: "Assert HTTP status code is 403 Forbidden".to_string(), status: TestStatus::Failed, duration_ms: 60, error: Some("Expected status 403 but got 200".to_string()) },
        ],
        labels: create_labels("critical", "devsecops-python", "Security", "01_rbac_security"),
        root_cause_hint: Some("Missing RBAC authorization check in authorization middleware allowing unprivileged users to claim admin role.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-102".to_string(),
        name: "test_concurrent_account_balance_transfer_deadlock".to_string(),
        suite: "Account Banking Service".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 820,
        error_message: Some("java.sql.SQLException: Deadlock detected when trying to get lock; try restarting transaction: UPDATE accounts SET balance = balance - 100 WHERE id = 1042".to_string()),
        stack_trace: Some("com.cherenkov.api.TransferServiceTest.testConcurrentTransfer(TransferServiceTest.java:114)\n    at org.springframework.dao.CannotAcquireLockException: Deadlock detected in PostgreSQL transaction isolation level READ COMMITTED\n    at com.cherenkov.service.AccountService.executeTransfer(AccountService.java:88)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Database".to_string(),
            event_type: "deadlock".to_string(),
            latency_ms: 110,
            jitter_ms: 15,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [PostgreSQL] Process 4192 waiting for ExclusiveLock on transaction 88192; blocked by process 4193".to_string()),
            correlated_timestamp: "2026-08-24T18:01:25Z".to_string(),
            retry_attempts: 1,
            injection_target: "tcp://127.0.0.1:5432".to_string(),
        }),
        flakiness_metrics: Some(FlakinessMetrics {
            iterations: 5,
            passed_iterations: 1,
            failed_iterations: 4,
            flakiness_rate: 0.8,
            avg_duration_ms: 820,
            duration_stddev_ms: 45.0,
            historical_flake_score: 0.4,
        }),
        steps: vec![
            TestStepTelemetry { name: "Setup account balance A=1000, B=1000".to_string(), status: TestStatus::Passed, duration_ms: 120, error: None },
            TestStepTelemetry { name: "Spawn parallel transfer thread 1: A -> B ($100)".to_string(), status: TestStatus::Passed, duration_ms: 300, error: None },
            TestStepTelemetry { name: "Spawn parallel transfer thread 2: B -> A ($200)".to_string(), status: TestStatus::Failed, duration_ms: 400, error: Some("Deadlock detected on resource accounts".to_string()) },
        ],
        labels: create_labels("blocker", "restassured-java", "Banking", "02_account_transfer"),
        root_cause_hint: Some("Unordered resource lock acquisition across concurrent transfer threads leading to database deadlocks.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-103".to_string(),
        name: "test_order_checkout_foreign_key_constraint_violation".to_string(),
        suite: "Order Checkout API".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::RealBug,
        duration_ms: 340,
        error_message: Some("java.lang.RuntimeException: HTTP 500 Internal Server Error: insert or update on table \"order_items\" violates foreign key constraint \"fk_product_id\"".to_string()),
        stack_trace: Some("com.cherenkov.api.OrderCheckoutTest.testCheckoutNonExistentSku(OrderCheckoutTest.java:78)\n    at io.restassured.internal.ResponseSpecificationImpl.validateResponseIfRequired(ResponseSpecificationImpl.groovy:696)\n    at io.restassured.internal.ResponseSpecificationImpl.statusCode(ResponseSpecificationImpl.groovy:139)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Database".to_string(),
            event_type: "constraint_violation".to_string(),
            latency_ms: 45,
            jitter_ms: 5,
            packet_loss_rate: 0.0,
            proxy_log: Some("ERROR [Database] ForeignKeyViolation: key (product_id)=(99999) is not present in table 'products'".to_string()),
            correlated_timestamp: "2026-08-24T18:01:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/orders".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Create cart with deleted product SKU 99999".to_string(), status: TestStatus::Passed, duration_ms: 90, error: None },
            TestStepTelemetry { name: "Submit checkout payload".to_string(), status: TestStatus::Broken, duration_ms: 250, error: Some("HTTP 500 Unhandled DB constraint violation".to_string()) },
        ],
        labels: create_labels("critical", "restassured-java", "Orders", "03_checkout_flow"),
        root_cause_hint: Some("Checkout service fails to validate SKU existence before executing SQL insert, triggering raw 500 constraint exception.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-104".to_string(),
        name: "test_user_profile_null_pointer_on_missing_avatar".to_string(),
        suite: "User Profile Web Flow".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 410,
        error_message: Some("TypeError: Cannot read properties of undefined (reading 'avatarUrl') at UserService.renderProfile".to_string()),
        stack_trace: Some("TypeError: Cannot read properties of undefined (reading 'avatarUrl')\n    at UserService.renderProfile (src/services/UserService.ts:92:28)\n    at async UserController.getProfile (src/controllers/UserController.ts:45:12)\n    at async Page.goto (exercises/01_web_playwright_ts/02_user_profile/exercise.ts:34:5)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "null_pointer".to_string(),
            latency_ms: 30,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("ERROR [Node Server] Unhandled Rejection: TypeError: Cannot read properties of undefined (reading 'avatarUrl')".to_string()),
            correlated_timestamp: "2026-08-24T18:01:55Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8080/profile/user_402".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Navigate to profile page for user without avatar".to_string(), status: TestStatus::Failed, duration_ms: 410, error: Some("Server returned 500 Uncaught TypeError".to_string()) },
        ],
        labels: create_labels("major", "playwright-ts", "UserProfile", "02_user_profile"),
        root_cause_hint: Some("Backend template crashes with 500 when avatar object is missing rather than rendering default fallback icon.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-105".to_string(),
        name: "test_pact_consumer_contract_missing_correlation_id".to_string(),
        suite: "Payment Consumer Contract".to_string(),
        track_id: "contract-pact".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 230,
        error_message: Some("PactVerificationError: Body mismatch: Key 'correlation_id' was expected at $.payload but was missing from provider payload".to_string()),
        stack_trace: Some("PactVerificationError: Body mismatch: Key 'correlation_id' was expected at $.payload but was missing from provider payload\n    at PactVerifier.verifyContract (exercises/09_contract_pact/01_consumer_verification/exercise.ts:74:11)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "contract_regression".to_string(),
            latency_ms: 18,
            jitter_ms: 3,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [Pact Verifier] Header 'X-Correlation-ID' missing in HTTP 200 payload from PaymentGatewayMock".to_string()),
            correlated_timestamp: "2026-08-24T18:02:10Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8085/v2/payments".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Load Pact Consumer Contract v2".to_string(), status: TestStatus::Passed, duration_ms: 50, error: None },
            TestStepTelemetry { name: "Verify Provider mock response against contract schema".to_string(), status: TestStatus::Failed, duration_ms: 180, error: Some("Missing mandatory correlation_id field in payload".to_string()) },
        ],
        labels: create_labels("critical", "contract-pact", "Contracts", "01_consumer_verification"),
        root_cause_hint: Some("Provider endpoint dropped mandatory correlation_id field, causing a breaking contract regression.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-106".to_string(),
        name: "test_shopping_cart_integer_overflow_quantity".to_string(),
        suite: "Shopping Cart Math".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 520,
        error_message: Some("AssertionError: Expected total price >= 0, found -$2,147,483.64 when ordering 2,147,483,648 items".to_string()),
        stack_trace: Some("AssertionError: Expected total price >= 0, found -2147483.64\n    at CartPage.verifyTotal (exercises/01_web_playwright_ts/06_cart_checkout/exercise.ts:63:14)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "integer_overflow".to_string(),
            latency_ms: 25,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Backend] Signed 32-bit integer overflow during price calculation: 2147483648 * 100".to_string()),
            correlated_timestamp: "2026-08-24T18:02:25Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8080/cart".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Add item with max integer quantity 2147483648".to_string(), status: TestStatus::Passed, duration_ms: 200, error: None },
            TestStepTelemetry { name: "Inspect cart subtotal and tax".to_string(), status: TestStatus::Failed, duration_ms: 320, error: Some("Total is negative due to arithmetic overflow".to_string()) },
        ],
        labels: create_labels("critical", "playwright-ts", "Cart", "06_cart_checkout"),
        root_cause_hint: Some("Signed 32-bit integer calculation in cart pricing logic wrapped around to negative values.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-107".to_string(),
        name: "test_jwt_auth_token_expiration_clock_drift".to_string(),
        suite: "JWT Token Validation".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 180,
        error_message: Some("AssertionError: Expired JWT token with iat in past and exp expired 1 hour ago was accepted with HTTP 200".to_string()),
        stack_trace: Some("AssertionError: Expected 401 Unauthorized for expired JWT token, got 200 OK\n    File \"exercises/07_cloud_devsecops/03_jwt_validation/exercise.py\", line 42, in test_jwt_auth_token_expiration".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "jwt_bypass".to_string(),
            latency_ms: 15,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Auth Service] JWT signature accepted with expired 'exp' claim due to disabled verify_exp flag".to_string()),
            correlated_timestamp: "2026-08-24T18:02:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/secure/resource".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Generate expired JWT token (exp = now - 3600)".to_string(), status: TestStatus::Passed, duration_ms: 30, error: None },
            TestStepTelemetry { name: "Send Authorization: Bearer <expired_token>".to_string(), status: TestStatus::Failed, duration_ms: 150, error: Some("Expected 401 Unauthorized but received 200 OK".to_string()) },
        ],
        labels: create_labels("blocker", "devsecops-python", "Auth", "03_jwt_validation"),
        root_cause_hint: Some("JWT verification library configured with `verify_exp: False`, bypassing token expiration validation.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-108".to_string(),
        name: "test_sql_injection_search_query_parameter".to_string(),
        suite: "SQL Injection Defense".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 290,
        error_message: Some("AssertionError: SQL syntax error exposed in response body: 'OR 1=1 --'".to_string()),
        stack_trace: Some("AssertionError: Search endpoint leaked internal SQL error in response body\n    File \"exercises/07_cloud_devsecops/04_sqli_protection/exercise.py\", line 55, in test_sql_injection".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Database".to_string(),
            event_type: "sql_syntax_leak".to_string(),
            latency_ms: 35,
            jitter_ms: 4,
            packet_loss_rate: 0.0,
            proxy_log: Some("ERROR [PostgreSQL] syntax error at or near 'OR' at character 48 in query: SELECT * FROM items WHERE name LIKE '%test' OR 1=1 --%'".to_string()),
            correlated_timestamp: "2026-08-24T18:02:55Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/search?q=test%27+OR+1%3D1+--".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Send search query with SQL injection payload".to_string(), status: TestStatus::Failed, duration_ms: 290, error: Some("Raw database error trace leaked in HTTP 500 response".to_string()) },
        ],
        labels: create_labels("blocker", "devsecops-python", "Security", "04_sqli_protection"),
        root_cause_hint: Some("Raw string concatenation used in search SQL query instead of parameterized prepared statements.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-109".to_string(),
        name: "test_loyalty_point_redemption_double_spend".to_string(),
        suite: "Loyalty Points API".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 640,
        error_message: Some("AssertionError: Double-spend successful: User with 500 points redeemed two 500-point coupons concurrently".to_string()),
        stack_trace: Some("com.cherenkov.api.LoyaltyServiceTest.testDoubleSpend(LoyaltyServiceTest.java:95)\n    at org.junit.jupiter.api.AssertionUtils.fail(AssertionUtils.java:39)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Database".to_string(),
            event_type: "race_condition".to_string(),
            latency_ms: 60,
            jitter_ms: 8,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [LoyaltyService] Balance check passed twice before either deduction transaction committed".to_string()),
            correlated_timestamp: "2026-08-24T18:03:10Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/loyalty/redeem".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Seed user account with 500 reward points".to_string(), status: TestStatus::Passed, duration_ms: 80, error: None },
            TestStepTelemetry { name: "Execute concurrent redemption request A and B".to_string(), status: TestStatus::Failed, duration_ms: 560, error: Some("Both redemption requests succeeded without concurrency lock".to_string()) },
        ],
        labels: create_labels("critical", "restassured-java", "Loyalty", "04_loyalty_redemption"),
        root_cause_hint: Some("Missing database SELECT FOR UPDATE row lock allows concurrent double-spending of reward points.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-110".to_string(),
        name: "test_pdf_export_out_of_memory_heap_limit".to_string(),
        suite: "Report Generation Service".to_string(),
        track_id: "k6-js".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::RealBug,
        duration_ms: 2100,
        error_message: Some("HTTP 500 Internal Server Error: java.lang.OutOfMemoryError: Java heap space during PDF table generation".to_string()),
        stack_trace: Some("com.cherenkov.report.PdfGenerator.buildReport(PdfGenerator.java:214)\n    at com.cherenkov.controller.ReportController.exportPdf(ReportController.java:82)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "out_of_memory".to_string(),
            latency_ms: 1800,
            jitter_ms: 200,
            packet_loss_rate: 0.0,
            proxy_log: Some("FATAL [JVM] java.lang.OutOfMemoryError: Java heap space in PdfGenerator.renderRows()".to_string()),
            correlated_timestamp: "2026-08-24T18:03:30Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/reports/export.pdf".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Request 10,000 row PDF transaction report".to_string(), status: TestStatus::Broken, duration_ms: 2100, error: Some("JVM crashed with OutOfMemoryError on PDF buffer allocation".to_string()) },
        ],
        labels: create_labels("blocker", "k6-js", "Reporting", "02_pdf_export"),
        root_cause_hint: Some("Unbounded in-memory buffering of large datasets during PDF report generation exhausts JVM heap.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-111".to_string(),
        name: "test_genai_rag_hallucination_guardrail_violation".to_string(),
        suite: "GenAI RAG Evaluation".to_string(),
        track_id: "genai-qa".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 890,
        error_message: Some("AssertionError: Hallucination rate 0.42 exceeded SLA maximum threshold of 0.05".to_string()),
        stack_trace: Some("AssertionError: Model hallucinated fabricated company policy not present in retrieved vector context\n    at evalHallucination (exercises/06_genai_qa/03_hallucination_eval/exercise.ts:51:9)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "rag_hallucination".to_string(),
            latency_ms: 450,
            jitter_ms: 50,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [RAG Pipeline] Vector chunk similarity score 0.31 below relevance threshold 0.75".to_string()),
            correlated_timestamp: "2026-08-24T18:03:50Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/ai/query".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Retrieve vector embeddings for query".to_string(), status: TestStatus::Passed, duration_ms: 120, error: None },
            TestStepTelemetry { name: "Generate LLM answer with ground truth context".to_string(), status: TestStatus::Passed, duration_ms: 500, error: None },
            TestStepTelemetry { name: "Score faithfulness using LLM-as-judge".to_string(), status: TestStatus::Failed, duration_ms: 270, error: Some("Hallucinated claim detected in response text".to_string()) },
        ],
        labels: create_labels("critical", "genai-qa", "AI-Eval", "03_hallucination_eval"),
        root_cause_hint: Some("Low-quality vector embedding retrieval chunk passes irrelevant context causing LLM hallucination.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-112".to_string(),
        name: "test_genai_prompt_injection_jailbreak_vulnerability".to_string(),
        suite: "GenAI Safety Guardrails".to_string(),
        track_id: "genai-qa".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 760,
        error_message: Some("AssertionError: System prompt leakage detected: Model leaked internal system prompt instructions".to_string()),
        stack_trace: Some("AssertionError: Prompt injection succeeded; output contained 'SYSTEM: You are a helpful assistant'\n    at verifyPromptGuardrail (exercises/06_genai_qa/04_prompt_injection/exercise.ts:63:12)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "prompt_injection".to_string(),
            latency_ms: 380,
            jitter_ms: 40,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Guardrail] Delimiter bypass pattern detected in user prompt input: 'Ignore previous instructions'".to_string()),
            correlated_timestamp: "2026-08-24T18:04:10Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/ai/chat".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Inject adversarial system override prompt".to_string(), status: TestStatus::Failed, duration_ms: 760, error: Some("System prompt was echoed in LLM output".to_string()) },
        ],
        labels: create_labels("blocker", "genai-qa", "AI-Security", "04_prompt_injection"),
        root_cause_hint: Some("Missing prompt sanitization delimiter guardrail allows user input to override system instructions.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-113".to_string(),
        name: "test_accessibility_wcag_color_contrast_ratio_failure".to_string(),
        suite: "WCAG Accessibility Compliance".to_string(),
        track_id: "a11y-axe".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 310,
        error_message: Some("AxeViolation: Elements must have sufficient color contrast (found ratio 2.1:1, required 4.5:1 for #718096 on #ffffff)".to_string()),
        stack_trace: Some("AxeViolation: color-contrast on selector '#btn-secondary-action'\n    at AxeRunner.analyze (exercises/10_a11y_axe/01_color_contrast/exercise.ts:38:15)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "a11y_wcag_violation".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [Axe Core] Rule 'color-contrast' failed on element button#btn-secondary-action".to_string()),
            correlated_timestamp: "2026-08-24T18:04:30Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8080/buttons".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Render UI Button Component".to_string(), status: TestStatus::Passed, duration_ms: 100, error: None },
            TestStepTelemetry { name: "Run Axe-Core WCAG 2.1 AA automated rule audit".to_string(), status: TestStatus::Failed, duration_ms: 210, error: Some("Rule color-contrast failed: ratio 2.1:1 < 4.5:1".to_string()) },
        ],
        labels: create_labels("major", "a11y-axe", "Accessibility", "01_color_contrast"),
        root_cause_hint: Some("Secondary action button CSS text color has insufficient contrast ratio against light background.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-114".to_string(),
        name: "test_jmeter_high_concurrency_thread_pool_exhaustion".to_string(),
        suite: "JMeter Search Load SLA".to_string(),
        track_id: "jmeter".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 3500,
        error_message: Some("JMeterAssertionError: Response time 99th percentile 4850ms exceeded SLA limit 1000ms under 200 VUs".to_string()),
        stack_trace: Some("JMeterAssertionError: SLA breach on /api/v1/search: p99=4850ms, error_rate=14.2%\n    at JMeterRunner.evaluateJtl (exercises/05_perf_jmeter/01_thread_group/exercise.jmx:112)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "thread_pool_exhaustion".to_string(),
            latency_ms: 2800,
            jitter_ms: 300,
            packet_loss_rate: 0.0,
            proxy_log: Some("ERROR [Tomcat] Thread pool maxThreads (200) reached; queue capacity (100) exceeded".to_string()),
            correlated_timestamp: "2026-08-24T18:04:50Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/search".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Ramp up 200 virtual threads".to_string(), status: TestStatus::Passed, duration_ms: 1000, error: None },
            TestStepTelemetry { name: "Execute search workload for 30s".to_string(), status: TestStatus::Failed, duration_ms: 2500, error: Some("P99 response time 4850ms breached 1000ms SLA".to_string()) },
        ],
        labels: create_labels("critical", "jmeter", "Performance", "01_thread_group"),
        root_cause_hint: Some("Synchronous I/O operations block Tomcat worker threads, causing thread pool exhaustion under concurrency.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-115".to_string(),
        name: "test_maestro_biometric_auth_bypass_on_device_rotate".to_string(),
        suite: "Mobile Biometric Security".to_string(),
        track_id: "maestro-mobile".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 1200,
        error_message: Some("MaestroAssertionError: Secret vault was displayed without biometric prompt after orientation change".to_string()),
        stack_trace: Some("MaestroAssertionError: Element 'text: Confidential Vault' visible without 'tapOn: Biometric Prompt'\n    at MaestroEngine.execute (exercises/03_mobile_maestro/01_biometric_auth/exercise.yaml:45)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "activity_lifecycle_bug".to_string(),
            latency_ms: 50,
            jitter_ms: 5,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Android] Activity recreated on orientation change without re-authenticating state".to_string()),
            correlated_timestamp: "2026-08-24T18:05:10Z".to_string(),
            retry_attempts: 0,
            injection_target: "mobile://com.cherenkov.app/vault".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Launch Vault Activity".to_string(), status: TestStatus::Passed, duration_ms: 400, error: None },
            TestStepTelemetry { name: "Simulate device rotation to Landscape".to_string(), status: TestStatus::Passed, duration_ms: 300, error: None },
            TestStepTelemetry { name: "Assert biometric auth modal is visible".to_string(), status: TestStatus::Failed, duration_ms: 500, error: Some("Vault contents leaked without PIN prompt".to_string()) },
        ],
        labels: create_labels("critical", "maestro-mobile", "Mobile", "01_biometric_auth"),
        root_cause_hint: Some("Android Activity configuration change recreation logic skips biometric re-authentication verification.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-116".to_string(),
        name: "test_duplicate_user_registration_case_insensitivity".to_string(),
        suite: "User Registration API".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 280,
        error_message: Some("AssertionError: Expected HTTP 409 Conflict for 'Alice@Example.com' when 'alice@example.com' exists, got 201 Created".to_string()),
        stack_trace: Some("com.cherenkov.api.UserRegistrationTest.testCaseInsensitiveEmail(UserRegistrationTest.java:62)\n    at org.junit.jupiter.api.Assertions.assertEquals(Assertions.java:115)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Database".to_string(),
            event_type: "uniqueness_bug".to_string(),
            latency_ms: 30,
            jitter_ms: 3,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [UserService] Duplicate account created with mixed-case email: Alice@Example.com".to_string()),
            correlated_timestamp: "2026-08-24T18:05:30Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/users".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Register user with email 'alice@example.com'".to_string(), status: TestStatus::Passed, duration_ms: 100, error: None },
            TestStepTelemetry { name: "Register user with email 'Alice@Example.com'".to_string(), status: TestStatus::Failed, duration_ms: 180, error: Some("Expected status 409 but received 201".to_string()) },
        ],
        labels: create_labels("major", "restassured-java", "Auth", "05_registration"),
        root_cause_hint: Some("Database email uniqueness index is case-sensitive, permitting duplicate account creation with mixed casing.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-117".to_string(),
        name: "test_idor_document_access_unauthorized_tenant".to_string(),
        suite: "Multi-Tenant Document Security".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 195,
        error_message: Some("AssertionError: Insecure Direct Object Reference (IDOR): Tenant B accessed Tenant A invoice /documents/inv_10492".to_string()),
        stack_trace: Some("AssertionError: IDOR vulnerability detected: Tenant B retrieved private document of Tenant A\n    File \"exercises/07_cloud_devsecops/05_idor_prevention/exercise.py\", line 47, in test_idor".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "idor_vulnerability".to_string(),
            latency_ms: 20,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [DocService] GET /documents/inv_10492 served to user_id=8812 without tenant ownership check".to_string()),
            correlated_timestamp: "2026-08-24T18:05:50Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/documents/inv_10492".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Authenticate as Tenant B".to_string(), status: TestStatus::Passed, duration_ms: 45, error: None },
            TestStepTelemetry { name: "Fetch Tenant A invoice doc ID inv_10492".to_string(), status: TestStatus::Failed, duration_ms: 150, error: Some("Expected 403 Forbidden but received 200 OK".to_string()) },
        ],
        labels: create_labels("blocker", "devsecops-python", "Security", "05_idor_prevention"),
        root_cause_hint: Some("Document controller fetches record by ID directly without verifying requesting tenant ownership.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-118".to_string(),
        name: "test_contract_pact_field_type_regression_string_to_int".to_string(),
        suite: "Provider State Verification".to_string(),
        track_id: "contract-pact".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 210,
        error_message: Some("PactVerificationError: Type mismatch at $.account_number: expected String, found Number (489102)".to_string()),
        stack_trace: Some("PactVerificationError: Type mismatch at $.account_number: expected String, found Integer\n    at PactVerifier.verify (exercises/09_contract_pact/02_provider_states/exercise.ts:58:14)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "contract_type_regression".to_string(),
            latency_ms: 24,
            jitter_ms: 3,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [Pact] Schema mismatch: field 'account_number' changed from String to Integer".to_string()),
            correlated_timestamp: "2026-08-24T18:06:10Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8085/accounts/1".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Execute Provider pact verification".to_string(), status: TestStatus::Failed, duration_ms: 210, error: Some("Type mismatch on account_number field".to_string()) },
        ],
        labels: create_labels("critical", "contract-pact", "Contracts", "02_provider_states"),
        root_cause_hint: Some("Provider backend refactored account_number from String to Integer, breaking the consumer contract.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "BUG-119".to_string(),
        name: "test_webhook_retry_payload_empty_body_bug".to_string(),
        suite: "Webhook Dispatch Worker".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::RealBug,
        duration_ms: 440,
        error_message: Some("AssertionError: Webhook retry worker delivered empty HTTP POST body on second attempt".to_string()),
        stack_trace: Some("com.cherenkov.api.WebhookWorkerTest.testRetryPayloadIntegrity(WebhookWorkerTest.java:83)\n    at org.junit.jupiter.api.Assertions.assertFalse(Assertions.java:82)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "stream_drain_bug".to_string(),
            latency_ms: 35,
            jitter_ms: 5,
            packet_loss_rate: 0.0,
            proxy_log: Some("ERROR [WebhookWorker] InputStream was already read during initial attempt; body was empty on retry".to_string()),
            correlated_timestamp: "2026-08-24T18:06:30Z".to_string(),
            retry_attempts: 1,
            injection_target: "http://127.0.0.1:8081/webhook/listener".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Trigger webhook delivery to endpoint returning 500".to_string(), status: TestStatus::Passed, duration_ms: 120, error: None },
            TestStepTelemetry { name: "Verify retry payload content length".to_string(), status: TestStatus::Failed, duration_ms: 320, error: Some("Retry body length was 0 bytes".to_string()) },
        ],
        labels: create_labels("major", "restassured-java", "Webhooks", "06_webhook_retry"),
        root_cause_hint: Some("HTTP client request payload stream was not re-buffered before retry attempt, sending empty body.".to_string()),
    });

    // =========================================================================
    // 2. FLAKY INFRASTRUCTURE FAILURES (19 Tests)
    // =========================================================================

    tests.push(ChaosTestResult {
        test_id: "FLAKE-201".to_string(),
        name: "test_k6_high_throughput_chaos_proxy_latency_spike".to_string(),
        suite: "High-Concurrency Spike Profile".to_string(),
        track_id: "k6-js".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 3520,
        error_message: Some("RequestError: Request timed out after 2000ms due to artificial chaos latency injection (3500ms)".to_string()),
        stack_trace: Some("RequestError: Request timed out after 2000ms\n    at http.get (exercises/04_perf_k6_js/02_spike_profile/exercise.js:38:12)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "latency_spike".to_string(),
            latency_ms: 3500,
            jitter_ms: 450,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy] Injected artificial latency: 3500ms (client socket timeout: 2000ms)".to_string()),
            correlated_timestamp: "2026-08-24T18:07:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8086/api/v1/feed".to_string(),
        }),
        flakiness_metrics: Some(FlakinessMetrics {
            iterations: 5,
            passed_iterations: 3,
            failed_iterations: 2,
            flakiness_rate: 0.4,
            avg_duration_ms: 2200,
            duration_stddev_ms: 1100.0,
            historical_flake_score: 0.65,
        }),
        steps: vec![
            TestStepTelemetry { name: "Send high-concurrency burst requests".to_string(), status: TestStatus::Flaky, duration_ms: 3520, error: Some("Socket timeout triggered by proxy latency".to_string()) },
        ],
        labels: create_labels("major", "k6-js", "Chaos", "02_spike_profile"),
        root_cause_hint: Some("Programmable Chaos Proxy injected a 3500ms latency spike exceeding client 2000ms socket timeout.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-202".to_string(),
        name: "test_restassured_tcp_reset_packet_drop".to_string(),
        suite: "Payment Gateway Resiliency".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 120,
        error_message: Some("java.net.SocketException: Connection reset by peer: Chaos Proxy L4 TCP RST packet injected".to_string()),
        stack_trace: Some("java.net.SocketException: Connection reset\n    at java.base/sun.nio.ch.SocketChannelImpl.checkConnect(Native Method)\n    at com.cherenkov.api.PaymentClient.charge(PaymentClient.java:45)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "tcp_reset".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.25,
            proxy_log: Some("WARN [ChaosProxy:L4] Injected TCP RST packet on client socket 127.0.0.1:8086".to_string()),
            correlated_timestamp: "2026-08-24T18:07:20Z".to_string(),
            retry_attempts: 1,
            injection_target: "tcp://127.0.0.1:8086".to_string(),
        }),
        flakiness_metrics: Some(FlakinessMetrics {
            iterations: 5,
            passed_iterations: 4,
            failed_iterations: 1,
            flakiness_rate: 0.2,
            avg_duration_ms: 95,
            duration_stddev_ms: 30.0,
            historical_flake_score: 0.35,
        }),
        steps: vec![
            TestStepTelemetry { name: "Open TCP connection to proxy".to_string(), status: TestStatus::Failed, duration_ms: 120, error: Some("Connection reset by peer (L4 TCP RST)".to_string()) },
        ],
        labels: create_labels("critical", "restassured-java", "NetworkChaos", "01_tcp_resilience"),
        root_cause_hint: Some("Chaos Proxy injected an L4 TCP RST packet, terminating the socket connection during the handshake.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-203".to_string(),
        name: "test_playwright_transient_502_bad_gateway".to_string(),
        suite: "Web Portal Navigation".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 850,
        error_message: Some("page.goto: net::ERR_HTTP_RESPONSE_CODE_FAILURE (HTTP 502 Bad Gateway from reverse proxy)".to_string()),
        stack_trace: Some("Error: page.goto: net::ERR_HTTP_RESPONSE_CODE_FAILURE (502)\n    at exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts:18:14".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "bad_gateway_502".to_string(),
            latency_ms: 120,
            jitter_ms: 10,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy:L7] Synthetic 502 Bad Gateway returned to client during mock upstream restart".to_string()),
            correlated_timestamp: "2026-08-24T18:07:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8086/portal".to_string(),
        }),
        flakiness_metrics: Some(FlakinessMetrics {
            iterations: 5,
            passed_iterations: 3,
            failed_iterations: 2,
            flakiness_rate: 0.4,
            avg_duration_ms: 700,
            duration_stddev_ms: 180.0,
            historical_flake_score: 0.5,
        }),
        steps: vec![
            TestStepTelemetry { name: "Navigate to /portal page".to_string(), status: TestStatus::Flaky, duration_ms: 850, error: Some("502 Bad Gateway from reverse proxy".to_string()) },
        ],
        labels: create_labels("major", "playwright-ts", "ProxyChaos", "04_first_playwright_test"),
        root_cause_hint: Some("Reverse proxy returned transient 502 Bad Gateway while upstream service container was restarting.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-204".to_string(),
        name: "test_devsecops_dns_resolution_timeout".to_string(),
        suite: "Security Scanner Network".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 2500,
        error_message: Some("socket.gaierror: [Errno 11001] getaddrinfo failed: DNS resolution timeout under 40% UDP packet loss".to_string()),
        stack_trace: Some("socket.gaierror: [Errno 11001] getaddrinfo failed\n    File \"exercises/07_cloud_devsecops/02_ast_analyzer/exercise.py\", line 15, in scan_remote".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "dns_timeout".to_string(),
            latency_ms: 2500,
            jitter_ms: 200,
            packet_loss_rate: 0.40,
            proxy_log: Some("WARN [ChaosProxy:DNS] Dropped UDP DNS query packet to 127.0.0.1:53".to_string()),
            correlated_timestamp: "2026-08-24T18:08:00Z".to_string(),
            retry_attempts: 2,
            injection_target: "udp://127.0.0.1:53".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Resolve target hostname internal.service.local".to_string(), status: TestStatus::Failed, duration_ms: 2500, error: Some("DNS resolution timeout".to_string()) },
        ],
        labels: create_labels("major", "devsecops-python", "DNS", "02_ast_analyzer"),
        root_cause_hint: Some("Simulated UDP packet loss on local DNS resolver caused transient name resolution failure.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-205".to_string(),
        name: "test_k6_socket_timeout_keepalive_race".to_string(),
        suite: "HTTP Connection Pool".to_string(),
        track_id: "k6-js".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 1540,
        error_message: Some("GoError: http: server closed idle connection before client sent headers".to_string()),
        stack_trace: Some("GoError: http: server closed idle connection\n    at http.post (exercises/04_perf_k6_js/01_pool_starvation/exercise.js:42:15)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "socket_timeout".to_string(),
            latency_ms: 1500,
            jitter_ms: 50,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [ChaosProxy] Closed idle keepalive socket during race window".to_string()),
            correlated_timestamp: "2026-08-24T18:08:20Z".to_string(),
            retry_attempts: 1,
            injection_target: "http://127.0.0.1:8086/api".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Reuse idle HTTP keep-alive connection".to_string(), status: TestStatus::Failed, duration_ms: 1540, error: Some("Server closed idle connection concurrently".to_string()) },
        ],
        labels: create_labels("minor", "k6-js", "KeepAlive", "01_pool_starvation"),
        root_cause_hint: Some("Server idle timeout closed connection at the exact moment client dispatched request headers.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-206".to_string(),
        name: "test_playwright_network_jitter_asset_timeout".to_string(),
        suite: "Frontend Asset Loading".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 5100,
        error_message: Some("TimeoutError: page.waitForResponse: Timeout 5000ms exceeded while waiting for /assets/bundle.js".to_string()),
        stack_trace: Some("TimeoutError: page.waitForResponse: Timeout 5000ms exceeded\n    at exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts:28:10".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "latency_jitter".to_string(),
            latency_ms: 4800,
            jitter_ms: 1200,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy] Jitter spike +1200ms caused total delay of 4800ms on bundle.js".to_string()),
            correlated_timestamp: "2026-08-24T18:08:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8086/assets/bundle.js".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Wait for client JS bundle download".to_string(), status: TestStatus::Failed, duration_ms: 5100, error: Some("5000ms timeout exceeded due to proxy jitter".to_string()) },
        ],
        labels: create_labels("major", "playwright-ts", "Jitter", "01_hydration_timing"),
        root_cause_hint: Some("High network jitter spike added 1200ms to response time, causing asset download to breach 5s test timeout.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-207".to_string(),
        name: "test_restassured_transient_504_gateway_timeout".to_string(),
        suite: "Order Submission Resiliency".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 5200,
        error_message: Some("java.lang.AssertionError: 1 expectation failed. Expected status code <200> but was <504> Gateway Timeout".to_string()),
        stack_trace: Some("java.lang.AssertionError: Expected status code <200> but was <504>\n    at com.cherenkov.api.OrderTest.submitOrder(OrderTest.java:55)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "gateway_timeout_504".to_string(),
            latency_ms: 5100,
            jitter_ms: 100,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy] Synthetic 504 Gateway Timeout injected on POST /api/orders".to_string()),
            correlated_timestamp: "2026-08-24T18:09:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8086/api/orders".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Post order request to proxy".to_string(), status: TestStatus::Failed, duration_ms: 5200, error: Some("504 Gateway Timeout synthetic response".to_string()) },
        ],
        labels: create_labels("critical", "restassured-java", "GatewayChaos", "03_checkout_flow"),
        root_cause_hint: Some("Synthetic 504 Gateway Timeout injected by Chaos Proxy middleware during order submission.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-208".to_string(),
        name: "test_jmeter_port_exhaustion_time_wait".to_string(),
        suite: "JMeter Socket Pool".to_string(),
        track_id: "jmeter".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 1800,
        error_message: Some("java.net.BindException: Address already in use: connect (ephemeral port exhaustion in TIME_WAIT)".to_string()),
        stack_trace: Some("java.net.BindException: Address already in use: connect\n    at org.apache.http.impl.conn.DefaultHttpClientConnectionOperator.connect(DefaultHttpClientConnectionOperator.java:134)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "port_exhaustion".to_string(),
            latency_ms: 20,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [OS] Socket TIME_WAIT count reached 65534; no ephemeral ports available for outbound connection".to_string()),
            correlated_timestamp: "2026-08-24T18:09:20Z".to_string(),
            retry_attempts: 3,
            injection_target: "tcp://127.0.0.1:8081".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Open high-frequency short-lived TCP sockets".to_string(), status: TestStatus::Failed, duration_ms: 1800, error: Some("Ephemeral port exhaustion in TIME_WAIT state".to_string()) },
        ],
        labels: create_labels("major", "jmeter", "OS-Sockets", "01_thread_group"),
        root_cause_hint: Some("High connection churn exhausted OS ephemeral ports due to sockets lingering in TIME_WAIT state.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-209".to_string(),
        name: "test_maestro_deep_link_cold_start_timeout".to_string(),
        suite: "Mobile Cold Start".to_string(),
        track_id: "maestro-mobile".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 9800,
        error_message: Some("MaestroTimeoutError: Timeout 10000ms waiting for app process com.cherenkov.app to launch on slow emulator".to_string()),
        stack_trace: Some("MaestroTimeoutError: App failed to reach running state within 10000ms\n    at MaestroRunner.launchApp (exercises/03_mobile_maestro/02_deep_link_cold_start/exercise.yaml:12)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "emulator_lag".to_string(),
            latency_ms: 9500,
            jitter_ms: 800,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Emulator] CPU throttling 95% due to host load; cold start delayed".to_string()),
            correlated_timestamp: "2026-08-24T18:09:40Z".to_string(),
            retry_attempts: 1,
            injection_target: "adb://emulator-5554".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Launch mobile app via deep link".to_string(), status: TestStatus::Failed, duration_ms: 9800, error: Some("Emulator CPU lag caused cold start timeout".to_string()) },
        ],
        labels: create_labels("minor", "maestro-mobile", "Emulator", "02_deep_link_cold_start"),
        root_cause_hint: Some("Host machine CPU spike caused Android emulator to throttle, causing app cold start timeout.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-210".to_string(),
        name: "test_genai_streaming_ttft_network_stall".to_string(),
        suite: "GenAI Streaming TTFT".to_string(),
        track_id: "genai-qa".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 3200,
        error_message: Some("AssertionError: Time-to-First-Token (TTFT) was 3200ms (SLA: 800ms) due to proxy buffer stall".to_string()),
        stack_trace: Some("AssertionError: TTFT 3200ms breached SLA threshold 800ms\n    at evalTTFT (exercises/06_genai_qa/05_ttft_streaming/exercise.ts:40:9)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "stream_buffer_stall".to_string(),
            latency_ms: 3200,
            jitter_ms: 150,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy] Chunked stream buffer stalled for 3000ms before flushing first SSE token".to_string()),
            correlated_timestamp: "2026-08-24T18:10:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8081/api/v1/ai/stream".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Initiate SSE stream connection".to_string(), status: TestStatus::Passed, duration_ms: 100, error: None },
            TestStepTelemetry { name: "Measure arrival time of first token chunk".to_string(), status: TestStatus::Failed, duration_ms: 3100, error: Some("Buffer stall inflated TTFT to 3200ms".to_string()) },
        ],
        labels: create_labels("major", "genai-qa", "Streaming", "05_ttft_streaming"),
        root_cause_hint: Some("Proxy intermediate buffer held chunked SSE stream before flushing first token chunk.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-211".to_string(),
        name: "test_pact_mock_server_port_collision".to_string(),
        suite: "Pact Mock Lifecycle".to_string(),
        track_id: "contract-pact".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 350,
        error_message: Some("PactBrokerError: Failed to start Pact Mock Server on port 8089: port already in use".to_string()),
        stack_trace: Some("PactBrokerError: EADDRINUSE 127.0.0.1:8089\n    at PactServer.start (exercises/09_contract_pact/01_consumer_verification/exercise.ts:22:15)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "port_collision".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [PactRunner] Port 8089 occupied by stale process PID 1928".to_string()),
            correlated_timestamp: "2026-08-24T18:10:20Z".to_string(),
            retry_attempts: 1,
            injection_target: "tcp://127.0.0.1:8089".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Bind mock Pact server to port 8089".to_string(), status: TestStatus::Failed, duration_ms: 350, error: Some("EADDRINUSE port collision".to_string()) },
        ],
        labels: create_labels("minor", "contract-pact", "MockServer", "01_consumer_verification"),
        root_cause_hint: Some("Stale orphan test runner process held socket on port 8089 during mock server binding.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-212".to_string(),
        name: "test_devsecops_tls_handshake_timeout".to_string(),
        suite: "TLS Certificate Scanner".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 2800,
        error_message: Some("ssl.SSLError: [SSL: TLSV1_ALERT_INTERNAL_ERROR] tls handshake timeout during certificate validation".to_string()),
        stack_trace: Some("ssl.SSLError: tls handshake timeout\n    File \"exercises/07_cloud_devsecops/03_jwt_validation/exercise.py\", line 12, in verify_tls".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "tls_handshake_timeout".to_string(),
            latency_ms: 2800,
            jitter_ms: 100,
            packet_loss_rate: 0.35,
            proxy_log: Some("WARN [ChaosProxy] Dropped TLS ClientHello packet 2 times".to_string()),
            correlated_timestamp: "2026-08-24T18:10:40Z".to_string(),
            retry_attempts: 2,
            injection_target: "tls://127.0.0.1:8443".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Initiate TLS 1.3 ClientHello handshake".to_string(), status: TestStatus::Failed, duration_ms: 2800, error: Some("Handshake packet dropped by proxy".to_string()) },
        ],
        labels: create_labels("major", "devsecops-python", "TLS", "03_jwt_validation"),
        root_cause_hint: Some("Simulated L4 packet loss dropped TLS ClientHello frames, delaying handshake completion.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-213".to_string(),
        name: "test_k6_redis_failover_transient_spike".to_string(),
        suite: "Cache Cluster Reliability".to_string(),
        track_id: "k6-js".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 490,
        error_message: Some("ResponseError: Redis READONLY You can't write against a read only replica during master failover".to_string()),
        stack_trace: Some("ResponseError: Redis READONLY\n    at http.post (exercises/04_perf_k6_js/03_chaos_sla/exercise.js:52:11)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "redis_failover".to_string(),
            latency_ms: 450,
            jitter_ms: 40,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Redis Sentinel] Master failover in progress; promotion took 250ms".to_string()),
            correlated_timestamp: "2026-08-24T18:11:00Z".to_string(),
            retry_attempts: 1,
            injection_target: "tcp://127.0.0.1:6379".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Execute write command to Redis cache".to_string(), status: TestStatus::Failed, duration_ms: 490, error: Some("Transient READONLY replica error during sentinel failover".to_string()) },
        ],
        labels: create_labels("major", "k6-js", "Redis", "03_chaos_sla"),
        root_cause_hint: Some("Redis Sentinel master failover promotion caused transient 250ms write rejection.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-214".to_string(),
        name: "test_playwright_websocket_connection_drop".to_string(),
        suite: "Live Chat WebSocket".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 620,
        error_message: Some("WebSocketError: WebSocket was closed before the connection was established (code: 1006 abnormal)".to_string()),
        stack_trace: Some("WebSocketError: code 1006 abnormal closure\n    at exercises/01_web_playwright_ts/04_first_playwright_test/exercise.ts:50:18".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L4".to_string(),
            event_type: "ws_disconnect".to_string(),
            latency_ms: 20,
            jitter_ms: 2,
            packet_loss_rate: 0.50,
            proxy_log: Some("WARN [ChaosProxy] Injected abrupt TCP FIN on active WebSocket connection".to_string()),
            correlated_timestamp: "2026-08-24T18:11:20Z".to_string(),
            retry_attempts: 1,
            injection_target: "ws://127.0.0.1:8086/chat/ws".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Open WebSocket connection".to_string(), status: TestStatus::Failed, duration_ms: 620, error: Some("Abrupt TCP FIN injected by proxy".to_string()) },
        ],
        labels: create_labels("major", "playwright-ts", "WebSocket", "04_first_playwright_test"),
        root_cause_hint: Some("Chaos Proxy injected an abrupt TCP FIN packet, closing the WebSocket transport connection.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-215".to_string(),
        name: "test_restassured_transient_503_service_unavailable".to_string(),
        suite: "Rolling Deployment Drain".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 80,
        error_message: Some("java.lang.AssertionError: Expected 200 OK, got 503 Service Unavailable (Backend draining)".to_string()),
        stack_trace: Some("java.lang.AssertionError: Expected 200 but got 503\n    at com.cherenkov.api.DrainTest.testDrain(DrainTest.java:44)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "service_unavailable_503".to_string(),
            latency_ms: 80,
            jitter_ms: 5,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy] Injected 503 Service Unavailable simulating container rolling update".to_string()),
            correlated_timestamp: "2026-08-24T18:11:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8086/api/drain".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Send request during container drain".to_string(), status: TestStatus::Failed, duration_ms: 80, error: Some("503 Service Unavailable".to_string()) },
        ],
        labels: create_labels("minor", "restassured-java", "RollingUpdate", "05_registration"),
        root_cause_hint: Some("Proxy returned synthetic 503 Service Unavailable to simulate rolling deployment container drain.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-216".to_string(),
        name: "test_jmeter_connection_pool_timeout".to_string(),
        suite: "HTTP Client Pool".to_string(),
        track_id: "jmeter".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 3100,
        error_message: Some("org.apache.http.conn.ConnectionPoolTimeoutException: Timeout waiting for connection from pool".to_string()),
        stack_trace: Some("org.apache.http.conn.ConnectionPoolTimeoutException: Timeout waiting for connection from pool\n    at org.apache.http.impl.conn.PoolingHttpClientConnectionManager.leaseConnection(PoolingHttpClientConnectionManager.java:314)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "conn_pool_exhaustion".to_string(),
            latency_ms: 3100,
            jitter_ms: 100,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [HttpClient] Max connections per route (50) exhausted; request waited 3000ms".to_string()),
            correlated_timestamp: "2026-08-24T18:12:00Z".to_string(),
            retry_attempts: 1,
            injection_target: "http://127.0.0.1:8081/api".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Lease connection from HTTP pool".to_string(), status: TestStatus::Failed, duration_ms: 3100, error: Some("Connection pool lease timeout".to_string()) },
        ],
        labels: create_labels("major", "jmeter", "ConnectionPool", "02_rampup_profile"),
        root_cause_hint: Some("Client connection pool reached max capacity (50) under synthetic delay, causing lease timeout.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-217".to_string(),
        name: "test_foundations_file_lock_transient_error".to_string(),
        suite: "File Fixture IO".to_string(),
        track_id: "foundations".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 90,
        error_message: Some("PermissionError: [WinError 32] The process cannot access the file because it is being used by another process".to_string()),
        stack_trace: Some("PermissionError: [WinError 32] File lock collision\n    File \"exercises/00_foundations/01_what_is_a_test/exercise.py\", line 28, in read_fixture".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "os_file_lock".to_string(),
            latency_ms: 10,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [OS] Antivirus / search indexer locked temporary test fixture file".to_string()),
            correlated_timestamp: "2026-08-24T18:12:20Z".to_string(),
            retry_attempts: 1,
            injection_target: "file://temp/fixture.json".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Open test fixture file for read".to_string(), status: TestStatus::Failed, duration_ms: 90, error: Some("WinError 32 file lock collision".to_string()) },
        ],
        labels: create_labels("minor", "foundations", "OS", "01_what_is_a_test"),
        root_cause_hint: Some("Transient OS file lock contention with background system indexer on temporary fixture file.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-218".to_string(),
        name: "test_a11y_axe_iframe_cross_origin_load_timeout".to_string(),
        suite: "Axe Cross-Origin Frame".to_string(),
        track_id: "a11y-axe".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 4200,
        error_message: Some("TimeoutError: Frame loading timed out: Cross-origin sandbox frame failed to load within 4000ms".to_string()),
        stack_trace: Some("TimeoutError: Frame load timeout 4000ms\n    at AxeRunner.analyzeFrames (exercises/10_a11y_axe/02_accessible_forms/exercise.ts:45:12)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "iframe_timeout".to_string(),
            latency_ms: 4200,
            jitter_ms: 200,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [ChaosProxy] 4000ms delay on sandbox iframe CDN script asset".to_string()),
            correlated_timestamp: "2026-08-24T18:12:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "http://127.0.0.1:8086/iframe/widget".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Load cross-origin iframe widget".to_string(), status: TestStatus::Failed, duration_ms: 4200, error: Some("Iframe script asset load timeout".to_string()) },
        ],
        labels: create_labels("minor", "a11y-axe", "iFrame", "02_accessible_forms"),
        root_cause_hint: Some("Cross-origin sandbox iframe asset delayed by proxy network throttling, exceeding frame wait timeout.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "FLAKE-219".to_string(),
        name: "test_genai_rate_limit_429_burst_spike".to_string(),
        suite: "GenAI Rate Limiting".to_string(),
        track_id: "genai-qa".to_string(),
        status: TestStatus::Flaky,
        category: FailureCategory::FlakyInfra,
        duration_ms: 150,
        error_message: Some("APIError: HTTP 429 Too Many Requests: Rate limit exceeded (TPM burst limit reached)".to_string()),
        stack_trace: Some("APIError: HTTP 429 Too Many Requests\n    at evalLLM (exercises/06_genai_qa/01_rag_faithfulness/exercise.ts:32:10)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "L7".to_string(),
            event_type: "rate_limit_429".to_string(),
            latency_ms: 150,
            jitter_ms: 10,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [LLM Gateway] Synthetic 429 Rate Limit injected due to burst window concurrency".to_string()),
            correlated_timestamp: "2026-08-24T18:13:00Z".to_string(),
            retry_attempts: 1,
            injection_target: "http://127.0.0.1:8081/api/v1/ai/generate".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Send LLM generation prompt".to_string(), status: TestStatus::Failed, duration_ms: 150, error: Some("HTTP 429 Too Many Requests".to_string()) },
        ],
        labels: create_labels("minor", "genai-qa", "RateLimit", "01_rag_faithfulness"),
        root_cause_hint: Some("Mock LLM gateway injected synthetic HTTP 429 Rate Limit during concurrent burst window.".to_string()),
    });

    // =========================================================================
    // 3. TEST AUTOMATION ANTI-PATTERNS (18 Tests)
    // =========================================================================

    tests.push(ChaosTestResult {
        test_id: "ANTI-301".to_string(),
        name: "test_playwright_hardcoded_sleep_race".to_string(),
        suite: "Hydration Synchronization".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 540,
        error_message: Some("TimeoutError: page.locator('#profile-data'): Expected text 'John Doe' but found 'Loading...' after sleep(500)".to_string()),
        stack_trace: Some("at exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts:35:10\n// ANTI-PATTERN: await page.waitForTimeout(500);\n// Under 600ms latency, the loading spinner is still active!".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "hardcoded_sleep_timeout".to_string(),
            latency_ms: 620,
            jitter_ms: 50,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [TestRunner] Test relied on hardcoded 500ms sleep, but server response took 620ms".to_string()),
            correlated_timestamp: "2026-08-24T18:13:20Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:35".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Trigger profile fetch".to_string(), status: TestStatus::Passed, duration_ms: 30, error: None },
            TestStepTelemetry { name: "Sleep 500ms (Anti-Pattern)".to_string(), status: TestStatus::Passed, duration_ms: 500, error: None },
            TestStepTelemetry { name: "Assert profile data text without web assertion wait".to_string(), status: TestStatus::Failed, duration_ms: 10, error: Some("Found 'Loading...' instead of 'John Doe'".to_string()) },
        ],
        labels: create_labels("critical", "playwright-ts", "AntiPattern", "01_hydration_timing"),
        root_cause_hint: Some("Brittle hardcoded sleep `waitForTimeout(500)` fails whenever network or rendering latency exceeds 500ms.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-302".to_string(),
        name: "test_playwright_stale_element_reference".to_string(),
        suite: "Dynamic DOM Hydration".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 480,
        error_message: Some("Error: Element is not attached to the DOM (StaleElementReferenceException) at locator.click()".to_string()),
        stack_trace: Some("Error: locator.click: Element is not attached to the DOM\n    at exercises/01_web_playwright_ts/03_stale_element/exercise.ts:42:18\n// ANTI-PATTERN: Stored element handle before React hydration re-render".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "stale_element".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [DOM] Component re-rendered on state update, replacing prior DOM node handle".to_string()),
            correlated_timestamp: "2026-08-24T18:13:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:42".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Query element handle const btn = await page.$('#submit')".to_string(), status: TestStatus::Passed, duration_ms: 80, error: None },
            TestStepTelemetry { name: "Click stale element handle after DOM replacement".to_string(), status: TestStatus::Failed, duration_ms: 400, error: Some("Element is detached from DOM".to_string()) },
        ],
        labels: create_labels("critical", "playwright-ts", "AntiPattern", "03_stale_element"),
        root_cause_hint: Some("Cached raw DOM ElementHandle across React re-render instead of using resilient auto-retrying `page.locator()`.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-303".to_string(),
        name: "test_restassured_missing_assertion_always_passes".to_string(),
        suite: "User API Verification".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 160,
        error_message: Some("AssertionError: Test completed without executing any assertions on response payload (NoAssertionsExecuted)".to_string()),
        stack_trace: Some("com.cherenkov.api.UserApiTest.testCreateUser(UserApiTest.java:45)\n// ANTI-PATTERN: response.then().extract().asString(); without .statusCode(201) or assertNonNull".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "missing_assertion".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Linter] Zero assert statements executed in test method".to_string()),
            correlated_timestamp: "2026-08-24T18:14:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "UserApiTest.java:45".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "POST /api/v1/users".to_string(), status: TestStatus::Passed, duration_ms: 160, error: None },
        ],
        labels: create_labels("major", "restassured-java", "AntiPattern", "01_what_is_a_test"),
        root_cause_hint: Some("Test dispatched network request but omitted all assertions, creating a useless test that passes even on failure.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-304".to_string(),
        name: "test_devsecops_raw_unwrap_panic".to_string(),
        suite: "Security AST Scanner".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 110,
        error_message: Some("KeyError: 'vulnerabilities' - dictionary lookup without .get() or defensive validation on error response".to_string()),
        stack_trace: Some("KeyError: 'vulnerabilities'\n    File \"exercises/07_cloud_devsecops/02_ast_analyzer/exercise.py\", line 28, in scan\n// ANTI-PATTERN: raw indexing response['data']['vulnerabilities']".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "unhandled_unwrap".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Linter] Unvalidated dictionary access crashed on unexpected response shape".to_string()),
            correlated_timestamp: "2026-08-24T18:14:20Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.py:28".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Access response['data']['vulnerabilities'] directly".to_string(), status: TestStatus::Broken, duration_ms: 110, error: Some("KeyError: 'vulnerabilities'".to_string()) },
        ],
        labels: create_labels("major", "devsecops-python", "AntiPattern", "02_ast_analyzer"),
        root_cause_hint: Some("Direct dictionary indexing without status check or defensive `.get()` panics on unexpected error payloads.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-305".to_string(),
        name: "test_playwright_fragile_xpath_locator".to_string(),
        suite: "Checkout Button Locator".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 5050,
        error_message: Some("TimeoutError: page.locator('xpath=/html/body/div[1]/div[2]/div/div[3]/button[2]'): Timeout 5000ms exceeded".to_string()),
        stack_trace: Some("TimeoutError: locator.click: Timeout 5000ms exceeded\n    at exercises/01_web_playwright_ts/05_fragile_locators/exercise.ts:39:15\n// ANTI-PATTERN: Absolute structural XPath broke after header banner was inserted".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "fragile_locator".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [DOM] Page layout index shifted due to promotional notification bar".to_string()),
            correlated_timestamp: "2026-08-24T18:14:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:39".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Click button via absolute XPath /html/body/div[1]/...".to_string(), status: TestStatus::Failed, duration_ms: 5050, error: Some("Element not found: index shifted".to_string()) },
        ],
        labels: create_labels("critical", "playwright-ts", "AntiPattern", "05_fragile_locators"),
        root_cause_hint: Some("Fragile absolute XPath index selector broken by minor DOM structural changes. Use user-facing role or test ID.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-306".to_string(),
        name: "test_k6_missing_error_rate_threshold".to_string(),
        suite: "k6 Performance SLA".to_string(),
        track_id: "k6-js".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 2200,
        error_message: Some("ThresholdError: k6 test passed with 35% HTTP 500 error rate because thresholds only monitored http_req_duration".to_string()),
        stack_trace: Some("ThresholdError: Missing 'http_req_failed' rate threshold in k6 options\n    at exercises/04_perf_k6_js/01_pool_starvation/exercise.js:25:1".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "missing_sla_threshold".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [k6] 35% of requests returned 500 but test passed due to missing http_req_failed SLA".to_string()),
            correlated_timestamp: "2026-08-24T18:15:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.js:25".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Run k6 load test with incomplete thresholds".to_string(), status: TestStatus::Broken, duration_ms: 2200, error: Some("High failure rate unmasked by missing error SLA".to_string()) },
        ],
        labels: create_labels("major", "k6-js", "AntiPattern", "01_pool_starvation"),
        root_cause_hint: Some("k6 script defined duration threshold but omitted `http_req_failed: ['rate<0.01']`, allowing broken endpoints to pass.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-307".to_string(),
        name: "test_playwright_brittle_text_exact_match".to_string(),
        suite: "Cart Badge Validation".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 620,
        error_message: Some("AssertionError: Expected element to have text 'Items in Cart (3)', found 'Items in Cart (3 items)'".to_string()),
        stack_trace: Some("AssertionError: Expected 'Items in Cart (3)', received 'Items in Cart (3 items)'\n    at CartPage.verify (exercises/01_web_playwright_ts/07_text_assertions/exercise.ts:48:21)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "brittle_text_match".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [UI] Minor copy change in button label broke strict exact string assertion".to_string()),
            correlated_timestamp: "2026-08-24T18:15:20Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:48".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Assert strict exact text match 'Items in Cart (3)'".to_string(), status: TestStatus::Failed, duration_ms: 620, error: Some("Exact text mismatch on dynamic copy".to_string()) },
        ],
        labels: create_labels("minor", "playwright-ts", "AntiPattern", "07_text_assertions"),
        root_cause_hint: Some("Strict exact string assertion broke on minor UI copy formatting change. Use regular expression or semantic locator.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-308".to_string(),
        name: "test_restassured_state_leakage_across_tests".to_string(),
        suite: "User Directory Isolation".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 410,
        error_message: Some("AssertionError: Expected 1 user in database, found 14 users created by preceding test methods".to_string()),
        stack_trace: Some("com.cherenkov.api.UserListTest.testSingleUserListing(UserListTest.java:71)\n// ANTI-PATTERN: Tests did not clean up database state or isolate test fixtures".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Database".to_string(),
            event_type: "test_pollution".to_string(),
            latency_ms: 15,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Database] Test executed against dirty database state populated by prior tests".to_string()),
            correlated_timestamp: "2026-08-24T18:15:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "UserListTest.java:71".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Count users in database".to_string(), status: TestStatus::Failed, duration_ms: 410, error: Some("Found 14 users instead of 1 (state pollution)".to_string()) },
        ],
        labels: create_labels("major", "restassured-java", "AntiPattern", "05_registration"),
        root_cause_hint: Some("Test depends on global database state and lacks teardown cleanup or transaction rollback isolation.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-309".to_string(),
        name: "test_playwright_unhandled_promise_race".to_string(),
        suite: "Async Navigation Flow".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 320,
        error_message: Some("Error: page.click() was called before previous navigation promise resolved (UnhandledPromiseRejection)".to_string()),
        stack_trace: Some("UnhandledPromiseRejection: page.goto() was not awaited\n    at exercises/01_web_playwright_ts/08_async_await_flow/exercise.ts:31:5".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "unawaited_promise".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("ERROR [Node] UnhandledPromiseRejectionWarning: Missing await keyword on asynchronous API call".to_string()),
            correlated_timestamp: "2026-08-24T18:16:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:31".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Call page.goto() without await keyword".to_string(), status: TestStatus::Broken, duration_ms: 320, error: Some("UnhandledPromiseRejection: navigation incomplete".to_string()) },
        ],
        labels: create_labels("critical", "playwright-ts", "AntiPattern", "08_async_await_flow"),
        root_cause_hint: Some("Missing `await` keyword on asynchronous Playwright call triggers race conditions and unhandled rejections.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-310".to_string(),
        name: "test_maestro_ambiguous_text_locator".to_string(),
        suite: "Mobile Form Submission".to_string(),
        track_id: "maestro-mobile".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 1100,
        error_message: Some("MaestroAmbiguityError: Found 4 elements matching text 'Submit'; tap target is non-deterministic".to_string()),
        stack_trace: Some("MaestroAmbiguityError: Multiple elements matched 'tapOn: Submit'\n    at exercises/03_mobile_maestro/02_locator_strategies/exercise.yaml:28".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "ambiguous_locator".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Maestro] Non-unique text selector matched modal, footer, and sidebar submit buttons".to_string()),
            correlated_timestamp: "2026-08-24T18:16:20Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.yaml:28".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Execute tapOn: Submit".to_string(), status: TestStatus::Failed, duration_ms: 1100, error: Some("Ambiguous text selector matches 4 elements".to_string()) },
        ],
        labels: create_labels("major", "maestro-mobile", "AntiPattern", "02_locator_strategies"),
        root_cause_hint: Some("Generic text locator matches multiple buttons on screen; use unique semantic accessibility ID.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-311".to_string(),
        name: "test_devsecops_hardcoded_secret_token".to_string(),
        suite: "Secrets Hygiene Lint".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 85,
        error_message: Some("SecurityLintError: High entropy plaintext API key 'ghp_99182831828319' hardcoded in test file".to_string()),
        stack_trace: Some("SecurityLintError: Hardcoded secret detected in test code\n    File \"exercises/07_cloud_devsecops/06_secrets_hygiene/exercise.py\", line 19".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "hardcoded_secret".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("CRITICAL [Gitleaks] Secret token pattern matched in test source code".to_string()),
            correlated_timestamp: "2026-08-24T18:16:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.py:19".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Audit test code for plaintext API credentials".to_string(), status: TestStatus::Broken, duration_ms: 85, error: Some("Plaintext API token hardcoded in test".to_string()) },
        ],
        labels: create_labels("critical", "devsecops-python", "AntiPattern", "06_secrets_hygiene"),
        root_cause_hint: Some("Plaintext production credential hardcoded in test file instead of environment variable injection.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-312".to_string(),
        name: "test_genai_non_deterministic_random_seed".to_string(),
        suite: "LLM Output Consistency".to_string(),
        track_id: "genai-qa".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 920,
        error_message: Some("AssertionError: Model output varied widely between test iterations due to temperature=1.0 and missing seed".to_string()),
        stack_trace: Some("AssertionError: Expected deterministic response matching regex, got random variance\n    at exercises/06_genai_qa/02_llm_flakiness/exercise.ts:44:11".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "unseeded_temperature".to_string(),
            latency_ms: 450,
            jitter_ms: 30,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [LLM] Sampling temperature set to 1.0 with no seed; responses are non-reproducible".to_string()),
            correlated_timestamp: "2026-08-24T18:17:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:44".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Prompt LLM with high temperature (1.0) and no seed".to_string(), status: TestStatus::Failed, duration_ms: 920, error: Some("Non-deterministic output failed assertion".to_string()) },
        ],
        labels: create_labels("major", "genai-qa", "AntiPattern", "02_llm_flakiness"),
        root_cause_hint: Some("Unconstrained sampling temperature without fixed seed creates non-deterministic test assertions.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-313".to_string(),
        name: "test_jmeter_infinite_rampup_time_out".to_string(),
        suite: "JMeter Ramp-Up Config".to_string(),
        track_id: "jmeter".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 1950,
        error_message: Some("JMeterTimeoutException: Test thread group never reached target concurrency before global timeout".to_string()),
        stack_trace: Some("JMeterTimeoutException: Ramp-up period 600s exceeded total test execution window 60s\n    at exercises/05_perf_jmeter/02_rampup_profile/exercise.jmx:82".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "misconfigured_rampup".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [JMeter] Ramp-up time 600 seconds is 10x longer than test duration 60 seconds".to_string()),
            correlated_timestamp: "2026-08-24T18:17:20Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.jmx:82".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Configure 600s ramp-up for 60s test".to_string(), status: TestStatus::Broken, duration_ms: 1950, error: Some("Ramp-up duration exceeds test duration".to_string()) },
        ],
        labels: create_labels("minor", "jmeter", "AntiPattern", "02_rampup_profile"),
        root_cause_hint: Some("JMeter ramp-up duration was configured 10x longer than the total test execution window.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-314".to_string(),
        name: "test_playwright_hover_without_visibility_wait".to_string(),
        suite: "Animated Navigation Menu".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 710,
        error_message: Some("Error: element.hover() clicked invisible submenu before CSS opacity animation completed".to_string()),
        stack_trace: Some("Error: element is not visible or has opacity: 0\n    at NavigationMenu.open (exercises/01_web_playwright_ts/09_menu_hover/exercise.ts:52:16)".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "missing_animation_wait".to_string(),
            latency_ms: 15,
            jitter_ms: 2,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [DOM] CSS transition 'opacity 300ms ease' was mid-animation when click fired".to_string()),
            correlated_timestamp: "2026-08-24T18:17:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:52".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Hover over menu trigger".to_string(), status: TestStatus::Passed, duration_ms: 100, error: None },
            TestStepTelemetry { name: "Click item before CSS opacity transition finishes".to_string(), status: TestStatus::Failed, duration_ms: 610, error: Some("Element not visible (mid-animation)".to_string()) },
        ],
        labels: create_labels("major", "playwright-ts", "AntiPattern", "09_menu_hover"),
        root_cause_hint: Some("Interacting with animated UI element before CSS opacity/height transition reaches final state.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-315".to_string(),
        name: "test_restassured_brittle_timestamp_assertion".to_string(),
        suite: "Audit Log Timestamp".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Failed,
        category: FailureCategory::AntiPattern,
        duration_ms: 230,
        error_message: Some("AssertionError: Expected '2026-08-24T18:00:00.000Z', found '2026-08-24T18:00:00.142Z'".to_string()),
        stack_trace: Some("com.cherenkov.api.AuditLogTest.testTimestamp(AuditLogTest.java:88)\n// ANTI-PATTERN: Exact millisecond string comparison on server-generated timestamp".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "brittle_timestamp".to_string(),
            latency_ms: 10,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("INFO [Test] Server generated timestamp with millisecond precision mismatching hardcoded expectation".to_string()),
            correlated_timestamp: "2026-08-24T18:18:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "AuditLogTest.java:88".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Assert exact timestamp string equality".to_string(), status: TestStatus::Failed, duration_ms: 230, error: Some("Millisecond mismatch on server timestamp".to_string()) },
        ],
        labels: create_labels("minor", "restassured-java", "AntiPattern", "05_registration"),
        root_cause_hint: Some("Asserting exact millisecond timestamp equality rather than relative time window or epoch comparison.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-316".to_string(),
        name: "test_a11y_axe_disabled_rules_anti_pattern".to_string(),
        suite: "Axe Rule Configuration".to_string(),
        track_id: "a11y-axe".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 140,
        error_message: Some("AxeConfigError: Test disabled 14 critical accessibility rules to force passing status".to_string()),
        stack_trace: Some("AxeConfigError: Rules ['color-contrast', 'aria-roles', 'image-alt'] disabled in options\n    at exercises/10_a11y_axe/02_accessible_forms/exercise.ts:29:8".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "disabled_rules".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Axe] Critical accessibility rules explicitly disabled in test configuration".to_string()),
            correlated_timestamp: "2026-08-24T18:18:20Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:29".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Configure Axe-Core audit with disabled rules".to_string(), status: TestStatus::Broken, duration_ms: 140, error: Some("Anti-pattern: suppressing rules to fake green build".to_string()) },
        ],
        labels: create_labels("major", "a11y-axe", "AntiPattern", "02_accessible_forms"),
        root_cause_hint: Some("Anti-pattern of suppressing critical accessibility rules to force a failing test to appear green.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-317".to_string(),
        name: "test_contract_pact_regex_overly_permissive".to_string(),
        suite: "Pact Matcher Validation".to_string(),
        track_id: "contract-pact".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 160,
        error_message: Some("PactContractWarning: Contract matcher '.*' matches anything including null and malicious payloads".to_string()),
        stack_trace: Some("PactContractWarning: Regex '.*' is too permissive; does not validate contract structure\n    at exercises/09_contract_pact/03_flexible_matchers/exercise.ts:41:14".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "permissive_matcher".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Pact] Consumer contract used '.*' matcher disabling contract guarantees".to_string()),
            correlated_timestamp: "2026-08-24T18:18:40Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.ts:41".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Define contract with '.*' wildcard matcher".to_string(), status: TestStatus::Broken, duration_ms: 160, error: Some("Overly permissive regex matcher".to_string()) },
        ],
        labels: create_labels("major", "contract-pact", "AntiPattern", "03_flexible_matchers"),
        root_cause_hint: Some("Using overly permissive wildcard matchers (`.*`) renders contract verification ineffective.".to_string()),
    });

    tests.push(ChaosTestResult {
        test_id: "ANTI-318".to_string(),
        name: "test_foundations_swallowing_exceptions_in_test".to_string(),
        suite: "Assertion Exception Handling".to_string(),
        track_id: "foundations".to_string(),
        status: TestStatus::Broken,
        category: FailureCategory::AntiPattern,
        duration_ms: 80,
        error_message: Some("AssertionError: Exception swallowed in bare 'except Exception: pass' block masking fatal bug".to_string()),
        stack_trace: Some("File \"exercises/00_foundations/02_assertion_mechanics/exercise.py\", line 39\n// ANTI-PATTERN: try: run_test() except: pass".to_string()),
        chaos_event: Some(ChaosEventTelemetry {
            layer: "Runtime".to_string(),
            event_type: "swallowed_exception".to_string(),
            latency_ms: 5,
            jitter_ms: 1,
            packet_loss_rate: 0.0,
            proxy_log: Some("WARN [Linter] Bare exception handler swallowed ZeroDivisionError".to_string()),
            correlated_timestamp: "2026-08-24T18:19:00Z".to_string(),
            retry_attempts: 0,
            injection_target: "exercise.py:39".to_string(),
        }),
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Execute test inside bare try-except pass block".to_string(), status: TestStatus::Broken, duration_ms: 80, error: Some("Bare except block silently swallowed exception".to_string()) },
        ],
        labels: create_labels("major", "foundations", "AntiPattern", "02_assertion_mechanics"),
        root_cause_hint: Some("Catching and silencing exceptions in test code masks real bugs and prevents test failure detection.".to_string()),
    });

    // =========================================================================
    // 4. RESILIENT PASSING TESTS (14 Tests)
    // =========================================================================

    tests.push(ChaosTestResult {
        test_id: "PASS-401".to_string(),
        name: "test_playwright_resilient_locator_and_wait".to_string(),
        suite: "Modern Web Automation".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 450,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: Some(FlakinessMetrics {
            iterations: 5,
            passed_iterations: 5,
            failed_iterations: 0,
            flakiness_rate: 0.0,
            avg_duration_ms: 440,
            duration_stddev_ms: 12.0,
            historical_flake_score: 0.0,
        }),
        steps: vec![
            TestStepTelemetry { name: "Locate button using page.getByRole('button', { name: 'Submit' })".to_string(), status: TestStatus::Passed, duration_ms: 120, error: None },
            TestStepTelemetry { name: "Await web assertion expect(locator).toBeVisible()".to_string(), status: TestStatus::Passed, duration_ms: 330, error: None },
        ],
        labels: create_labels("normal", "playwright-ts", "BestPractices", "01_hydration_timing"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-402".to_string(),
        name: "test_restassured_idempotent_order_creation".to_string(),
        suite: "API Resilience".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 380,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Send POST /orders with Idempotency-Key header".to_string(), status: TestStatus::Passed, duration_ms: 220, error: None },
            TestStepTelemetry { name: "Assert HTTP 201 and valid JSON response schema".to_string(), status: TestStatus::Passed, duration_ms: 160, error: None },
        ],
        labels: create_labels("normal", "restassured-java", "Idempotency", "03_checkout_flow"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-403".to_string(),
        name: "test_maestro_semantic_flow_with_retry".to_string(),
        suite: "Mobile Semantic Automation".to_string(),
        track_id: "maestro-mobile".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 1450,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Launch app com.cherenkov.app".to_string(), status: TestStatus::Passed, duration_ms: 600, error: None },
            TestStepTelemetry { name: "tapOn: id: 'btn_login'".to_string(), status: TestStatus::Passed, duration_ms: 350, error: None },
            TestStepTelemetry { name: "assertVisible: text: 'Welcome back'".to_string(), status: TestStatus::Passed, duration_ms: 500, error: None },
        ],
        labels: create_labels("normal", "maestro-mobile", "Mobile", "01_biometric_auth"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-404".to_string(),
        name: "test_k6_adaptive_load_ramp_profile".to_string(),
        suite: "k6 Performance Benchmarks".to_string(),
        track_id: "k6-js".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 1850,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Ramp up to 50 VUs over 10s".to_string(), status: TestStatus::Passed, duration_ms: 1000, error: None },
            TestStepTelemetry { name: "Verify p95 response time < 300ms".to_string(), status: TestStatus::Passed, duration_ms: 850, error: None },
        ],
        labels: create_labels("normal", "k6-js", "Performance", "03_chaos_sla"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-405".to_string(),
        name: "test_devsecops_bandit_security_ast_scan".to_string(),
        suite: "DevSecOps Static Analysis".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 220,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Scan Python codebase for B101, B105, B301 violations".to_string(), status: TestStatus::Passed, duration_ms: 220, error: None },
        ],
        labels: create_labels("normal", "devsecops-python", "Security", "01_rbac_security"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-406".to_string(),
        name: "test_genai_rag_faithfulness_within_sla".to_string(),
        suite: "GenAI Quality Guardrails".to_string(),
        track_id: "genai-qa".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 680,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Retrieve top 3 vector chunks".to_string(), status: TestStatus::Passed, duration_ms: 180, error: None },
            TestStepTelemetry { name: "Evaluate answer faithfulness score (0.96 >= 0.85 SLA)".to_string(), status: TestStatus::Passed, duration_ms: 500, error: None },
        ],
        labels: create_labels("normal", "genai-qa", "GenAI", "01_rag_faithfulness"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-407".to_string(),
        name: "test_jmeter_p95_percentile_sla_verification".to_string(),
        suite: "JMeter Baseline SLA".to_string(),
        track_id: "jmeter".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 1250,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Execute JMeter plan at 100 TPS".to_string(), status: TestStatus::Passed, duration_ms: 1250, error: None },
        ],
        labels: create_labels("normal", "jmeter", "Performance", "01_thread_group"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-408".to_string(),
        name: "test_contract_pact_strict_provider_verification".to_string(),
        suite: "Pact Verification".to_string(),
        track_id: "contract-pact".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 290,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Verify all consumer pact interactions".to_string(), status: TestStatus::Passed, duration_ms: 290, error: None },
        ],
        labels: create_labels("normal", "contract-pact", "Contracts", "02_provider_states"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-409".to_string(),
        name: "test_a11y_axe_wcag_full_compliance_pass".to_string(),
        suite: "WCAG 2.1 AA Audit".to_string(),
        track_id: "a11y-axe".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 380,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Analyze DOM with full Axe-Core rule engine".to_string(), status: TestStatus::Passed, duration_ms: 380, error: None },
        ],
        labels: create_labels("normal", "a11y-axe", "Accessibility", "01_color_contrast"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-410".to_string(),
        name: "test_foundations_boundary_value_analysis".to_string(),
        suite: "Foundations QA Mechanics".to_string(),
        track_id: "foundations".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 120,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Evaluate min, min+1, nominal, max-1, max boundary points".to_string(), status: TestStatus::Passed, duration_ms: 120, error: None },
        ],
        labels: create_labels("normal", "foundations", "Foundations", "01_what_is_a_test"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-411".to_string(),
        name: "test_tool_decisions_ui_vs_api_suitability".to_string(),
        suite: "Architecture Decisions".to_string(),
        track_id: "tool-decisions".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 95,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Evaluate test pyramid tier cost vs speed trade-offs".to_string(), status: TestStatus::Passed, duration_ms: 95, error: None },
        ],
        labels: create_labels("normal", "tool-decisions", "Architecture", "01_ui_vs_api_test"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-412".to_string(),
        name: "test_playwright_network_idle_and_auto_wait".to_string(),
        suite: "Web Auto-Waiting Flow".to_string(),
        track_id: "playwright-ts".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 410,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Execute search with network idle synchronization".to_string(), status: TestStatus::Passed, duration_ms: 410, error: None },
        ],
        labels: create_labels("normal", "playwright-ts", "AutoWait", "04_first_playwright_test"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-413".to_string(),
        name: "test_restassured_json_schema_validation_strict".to_string(),
        suite: "JSON Schema Contract".to_string(),
        track_id: "restassured-java".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 310,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Validate response matches JSON Schema Draft-07".to_string(), status: TestStatus::Passed, duration_ms: 310, error: None },
        ],
        labels: create_labels("normal", "restassured-java", "Schema", "03_checkout_flow"),
        root_cause_hint: None,
    });

    tests.push(ChaosTestResult {
        test_id: "PASS-414".to_string(),
        name: "test_devsecops_dependency_vulnerability_audit".to_string(),
        suite: "Software Supply Chain".to_string(),
        track_id: "devsecops-python".to_string(),
        status: TestStatus::Passed,
        category: FailureCategory::None,
        duration_ms: 270,
        error_message: None,
        stack_trace: None,
        chaos_event: None,
        flakiness_metrics: None,
        steps: vec![
            TestStepTelemetry { name: "Audit package dependencies against OSV and CVE database".to_string(), status: TestStatus::Passed, duration_ms: 270, error: None },
        ],
        labels: create_labels("normal", "devsecops-python", "SupplyChain", "06_secrets_hygiene"),
        root_cause_hint: None,
    });

    tests
}

fn create_labels(severity: &str, track: &str, feature: &str, drill: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("severity".to_string(), severity.to_string());
    map.insert("track".to_string(), track.to_string());
    map.insert("suite".to_string(), track.to_string());
    map.insert("subSuite".to_string(), drill.to_string());
    map.insert("feature".to_string(), feature.to_string());
    map.insert("framework".to_string(), "cherenkov-matrix".to_string());
    map
}

/// Find a specific test in the dataset by its ID or name
pub fn get_test_by_id(test_id: &str) -> Option<ChaosTestResult> {
    let clean_id = test_id.trim().to_lowercase();
    generate_chaos_dataset().into_iter().find(|t| {
        t.test_id.to_lowercase() == clean_id || t.name.to_lowercase() == clean_id || t.name.to_lowercase().contains(&clean_id)
    })
}

/// Return all tests that have a failing, broken, or flaky status
pub fn get_failing_tests() -> Vec<ChaosTestResult> {
    generate_chaos_dataset()
        .into_iter()
        .filter(|t| t.status != TestStatus::Passed && t.status != TestStatus::Skipped)
        .collect()
}

/// Return all tests belonging to a specific failure category
pub fn get_tests_by_category(category: FailureCategory) -> Vec<ChaosTestResult> {
    generate_chaos_dataset()
        .into_iter()
        .filter(|t| t.category == category)
        .collect()
}

/// Return all tests belonging to a specific track ID
pub fn get_tests_by_track(track_id: &str) -> Vec<ChaosTestResult> {
    let clean_track = track_id.trim().to_lowercase();
    generate_chaos_dataset()
        .into_iter()
        .filter(|t| t.track_id.to_lowercase() == clean_track)
        .collect()
}
