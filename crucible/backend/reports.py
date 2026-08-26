"""Allure Chaos Reporting Engine and Dataset Generator.

Generates 70+ deterministic chaotic test executions with rich L4/L7 network telemetry,
flakiness statistics, failure taxonomies, Allure summary metrics, and an interactive HTML report.
"""

from __future__ import annotations

import functools
import json

from crucible.backend.models import (
    AllureSummaryResponse,
    ChaosEventTelemetry,
    ChaosTestResultItem,
    FlakinessMetrics,
    TestStepTelemetry,
)


@functools.lru_cache(maxsize=1)
def generate_chaos_dataset() -> list[ChaosTestResultItem]:
    """Generate the deterministic dataset of 70 chaotic test executions."""
    tests: list[ChaosTestResultItem] = []

    # =========================================================================
    # 1. GENUINE PRODUCT DEFECTS (19 Tests)
    # =========================================================================

    tests.append(
        ChaosTestResultItem(
            test_id="BUG-101",
            name="test_auth_role_privilege_escalation",
            suite="RBAC Security Suite",
            track_id="devsecops-python",
            status="failed",
            category="real_bug",
            duration_ms=145,
            error_message="AssertionError: Expected HTTP 403 Forbidden for non-admin role, received HTTP 200 OK with admin access token",
            stack_trace="File 'exercises/06_cloud_devsecops/01_rbac_security/exercise.py', line 58, in test_auth_role_privilege_escalation\n    assert response.status_code == 403\nAssertionError: Expected HTTP 403 Forbidden",
            chaos_event=ChaosEventTelemetry(
                layer="L7",
                event_type="rbac_bypass",
                latency_ms=22,
                jitter_ms=4,
                packet_loss_rate=0.0,
                proxy_log="INFO [L7 Proxy] POST /api/v1/auth/elevate -> 200 OK (Missing RBAC middleware check on role parameter)",
                correlated_timestamp="2026-08-24T18:01:10Z",
                retry_attempts=0,
                injection_target="http://127.0.0.1:8081/api/v1/auth/elevate",
            ),
            flakiness_metrics=FlakinessMetrics(
                iterations=5,
                passed_iterations=0,
                failed_iterations=5,
                flakiness_rate=0.0,
                avg_duration_ms=145,
                duration_stddev_ms=3.2,
                historical_flake_score=0.0,
            ),
            steps=[
                TestStepTelemetry(name="Authenticate as standard user", status="passed", duration_ms=40),
                TestStepTelemetry(name="Send role elevation request to /api/v1/auth/elevate", status="passed", duration_ms=45),
                TestStepTelemetry(name="Assert HTTP status code is 403 Forbidden", status="failed", duration_ms=60, error="Expected status 403 but got 200"),
            ],
            labels={"severity": "critical", "track": "devsecops-python", "tier": "Security"},
            root_cause_hint="Missing RBAC authorization check in authorization middleware allowing unprivileged users to claim admin role.",
        )
    )

    tests.append(
        ChaosTestResultItem(
            test_id="BUG-102",
            name="test_concurrent_account_balance_transfer_deadlock",
            suite="Account Banking Service",
            track_id="restassured-java",
            status="failed",
            category="real_bug",
            duration_ms=820,
            error_message="java.sql.SQLException: Deadlock detected when trying to get lock; try restarting transaction: UPDATE accounts SET balance = balance - 100 WHERE id = 1042",
            stack_trace="com.cherenkov.api.TransferServiceTest.testConcurrentTransfer(TransferServiceTest.java:114)\n    at org.springframework.dao.CannotAcquireLockException: Deadlock detected",
            chaos_event=ChaosEventTelemetry(
                layer="Database",
                event_type="deadlock",
                latency_ms=110,
                jitter_ms=15,
                packet_loss_rate=0.0,
                proxy_log="WARN [PostgreSQL] Process 4192 waiting for ExclusiveLock on transaction 88192; blocked by process 4193",
                correlated_timestamp="2026-08-24T18:01:25Z",
                retry_attempts=1,
                injection_target="tcp://127.0.0.1:5432",
            ),
            flakiness_metrics=FlakinessMetrics(
                iterations=5,
                passed_iterations=1,
                failed_iterations=4,
                flakiness_rate=0.8,
                avg_duration_ms=820,
                duration_stddev_ms=45.0,
                historical_flake_score=0.4,
            ),
            steps=[
                TestStepTelemetry(name="Setup account balance A=1000, B=1000", status="passed", duration_ms=120),
                TestStepTelemetry(name="Spawn parallel transfer thread 1: A -> B ($100)", status="passed", duration_ms=300),
                TestStepTelemetry(name="Spawn parallel transfer thread 2: B -> A ($200)", status="failed", duration_ms=400, error="Deadlock detected on resource accounts"),
            ],
            labels={"severity": "blocker", "track": "restassured-java", "tier": "Banking"},
            root_cause_hint="Unordered resource lock acquisition across concurrent transfer threads leading to database deadlocks.",
        )
    )

    tests.append(
        ChaosTestResultItem(
            test_id="BUG-103",
            name="test_order_checkout_foreign_key_constraint_violation",
            suite="Order Checkout API",
            track_id="restassured-java",
            status="broken",
            category="real_bug",
            duration_ms=340,
            error_message="java.lang.RuntimeException: HTTP 500 Internal Server Error: insert or update on table 'order_items' violates foreign key constraint 'fk_product_id'",
            stack_trace="com.cherenkov.api.OrderCheckoutTest.testCheckoutNonExistentSku(OrderCheckoutTest.java:78)\n    at io.restassured.internal.ResponseSpecificationImpl.statusCode(ResponseSpecificationImpl.groovy:139)",
            chaos_event=ChaosEventTelemetry(
                layer="Database",
                event_type="constraint_violation",
                latency_ms=45,
                jitter_ms=5,
                packet_loss_rate=0.0,
                proxy_log="ERROR [Database] ForeignKeyViolation: key (product_id)=(99999) is not present in table 'products'",
                correlated_timestamp="2026-08-24T18:01:40Z",
                retry_attempts=0,
                injection_target="http://127.0.0.1:8081/api/v1/orders",
            ),
            steps=[
                TestStepTelemetry(name="Create cart with deleted product SKU 99999", status="passed", duration_ms=90),
                TestStepTelemetry(name="Submit checkout payload", status="broken", duration_ms=250, error="HTTP 500 Unhandled DB constraint violation"),
            ],
            labels={"severity": "critical", "track": "restassured-java", "tier": "Orders"},
            root_cause_hint="Checkout service fails to validate SKU existence before executing SQL insert, triggering raw 500 constraint exception.",
        )
    )

    # Additional Real Bugs
    real_bug_templates = [
        ("BUG-104", "test_ssrf_internal_metadata_exfiltration", "Security Suite", "devsecops-python", "SSRF vulnerability: /api/security/fetch-url retrieved AWS IMDS 169.254.169.254"),
        ("BUG-105", "test_jwt_none_algorithm_signature_bypass", "Auth API", "devsecops-python", "Signature bypass: JWT token with alg: none was accepted with 200 OK"),
        ("BUG-106", "test_sql_injection_blind_time_delay", "Data API", "devsecops-python", "SQL injection: sleep(5) payload delayed response by 5.2s"),
        ("BUG-107", "test_cors_wildcard_credential_leak", "Web API", "devsecops-python", "CORS misconfiguration: Access-Control-Allow-Origin: * combined with credentials: true"),
        ("BUG-108", "test_inventory_negative_oversell_race", "Inventory API", "k6-js", "Race condition: Inventory quantity dropped to -3 under 100 concurrent requests"),
        ("BUG-109", "test_pact_provider_missing_field_contract_break", "Pact Contract", "contract-pact", "Contract mismatch: Provider omitted mandatory field 'customer_tier' in /api/pact/orders"),
        ("BUG-110", "test_rag_hallucination_ungrounded_claims", "GenAI QA", "genai-qa", "GenAI Hallucination: Model invented unsupported factual assertions not in grounding document"),
        ("BUG-111", "test_prompt_injection_system_instructions_override", "GenAI Agent", "genai-qa", "Prompt injection exploit: System guardrails breached by adversarial prompt"),
        ("BUG-112", "test_accessibility_color_contrast_below_wcag_aa", "A11y Suite", "a11y-axe", "WCAG 2.1 AA Violation: Button text contrast ratio is 2.8:1, required >= 4.5:1"),
        ("BUG-113", "test_keyboard_focus_trap_modal_escape", "A11y Suite", "a11y-axe", "Keyboard navigation bug: Tab focus escapes modal dialog to background DOM"),
        ("BUG-114", "test_streaming_sse_premature_close_truncated_payload", "Perf Suite", "k6-js", "SSE stream terminated abruptly before delimiter was sent"),
        ("BUG-115", "test_graphql_depth_limit_denial_of_service", "GraphQL API", "restassured-java", "GraphQL DoS: Deeply nested query (depth=25) exhausted heap memory"),
        ("BUG-116", "test_idempotency_key_replay_double_charge", "Payment Gateway", "restassured-java", "Double charge: Duplicate idempotency key processed twice within 500ms window"),
        ("BUG-117", "test_null_pointer_on_empty_shipping_address", "Checkout Service", "restassured-java", "NullPointerException in ShippingCalculator when postal_code is null"),
        ("BUG-118", "test_rate_limiter_bypass_x_forwarded_for_spoof", "Security Gateway", "devsecops-python", "Rate limiter bypassed by spoofing X-Forwarded-For header"),
        ("BUG-119", "test_cache_invalidation_stale_product_price", "Pricing Engine", "playwright-ts", "Stale cache: Updated price $129.00 still showed as $149.00 after 10 minutes"),
    ]

    for tid, name, suite, track, desc in real_bug_templates:
        tests.append(
            ChaosTestResultItem(
                test_id=tid,
                name=name,
                suite=suite,
                track_id=track,
                status="failed",
                category="real_bug",
                duration_ms=180 + (int(tid.split("-")[1]) * 15) % 300,
                error_message=f"Defect Verified: {desc}",
                stack_trace=f"Traceback at {suite} -> {name}: {desc}",
                chaos_event=ChaosEventTelemetry(
                    layer="L7",
                    event_type="product_bug",
                    latency_ms=30,
                    jitter_ms=5,
                    packet_loss_rate=0.0,
                    proxy_log=f"INFO [L7 Proxy] {name} -> 500/Assertion Failure ({desc})",
                    correlated_timestamp="2026-08-24T18:02:00Z",
                    retry_attempts=0,
                ),
                flakiness_metrics=FlakinessMetrics(
                    iterations=5,
                    passed_iterations=0,
                    failed_iterations=5,
                    flakiness_rate=0.0,
                    avg_duration_ms=210,
                    duration_stddev_ms=5.0,
                    historical_flake_score=0.0,
                ),
                steps=[
                    TestStepTelemetry(name="Setup test fixture", status="passed", duration_ms=50),
                    TestStepTelemetry(name="Execute target operation", status="failed", duration_ms=130, error=desc),
                ],
                labels={"severity": "critical", "track": track},
                root_cause_hint=desc,
            )
        )

    # =========================================================================
    # 2. FLAKY INFRASTRUCTURE FAILURES (25 Tests)
    # =========================================================================

    tests.append(
        ChaosTestResultItem(
            test_id="FLAKE-201",
            name="test_checkout_payment_gateway_proxy_timeout_504",
            suite="Payment Gateway E2E",
            track_id="playwright-ts",
            status="broken",
            category="flaky_infra",
            duration_ms=5200,
            error_message="Error: page.waitForResponse: Timeout 5000ms exceeded while waiting for POST /api/checkout. Proxy injected L7 504 Gateway Timeout",
            stack_trace="at CheckoutPage.submitPayment (exercises/01_web_playwright_ts/04_checkout/CheckoutPage.ts:45)\n    at test_checkout_payment_gateway_proxy_timeout_504 (exercise.ts:18)",
            chaos_event=ChaosEventTelemetry(
                layer="L7",
                event_type="gateway_timeout_504",
                latency_ms=5400,
                jitter_ms=600,
                packet_loss_rate=0.15,
                proxy_log="WARN [ChaosProxy:8086] INJECTED FAULT: HTTP 504 Gateway Timeout (latency=5400ms) on POST /api/checkout",
                correlated_timestamp="2026-08-24T18:05:12Z",
                retry_attempts=2,
                injection_target="http://127.0.0.1:8086/api/checkout",
            ),
            flakiness_metrics=FlakinessMetrics(
                iterations=5,
                passed_iterations=2,
                failed_iterations=3,
                flakiness_rate=0.6,
                avg_duration_ms=3400,
                duration_stddev_ms=1800.0,
                historical_flake_score=0.72,
            ),
            steps=[
                TestStepTelemetry(name="Navigate to checkout page", status="passed", duration_ms=450),
                TestStepTelemetry(name="Fill payment details", status="passed", duration_ms=300),
                TestStepTelemetry(name="Submit payment via ChaosProxy", status="broken", duration_ms=4450, error="HTTP 504 Gateway Timeout"),
            ],
            labels={"severity": "flaky", "track": "playwright-ts", "tier": "E2E"},
            root_cause_hint="Chaos Proxy artificial latency spike (5400ms) exceeded client HTTP read timeout before retry backoff engaged.",
        )
    )

    flaky_infra_templates = [
        ("FLAKE-202", "test_tcp_connection_reset_by_peer", "L4 Network", "k6-js", "read: connection reset by peer (Chaos Proxy dropped raw TCP connection)"),
        ("FLAKE-203", "test_dns_lookup_timeout_unresolved_host", "DNS Service", "restassured-java", "UnknownHostException: Temporary failure in name resolution in CI docker bridge"),
        ("FLAKE-204", "test_ssl_handshake_timeout_under_cpu_throttle", "TLS Handshake", "playwright-ts", "SSL routines:ssl3_read_bytes:tlsv1 alert internal error during spike"),
        ("FLAKE-205", "test_redis_connection_pool_exhaustion_latency", "Cache Layer", "k6-js", "Redis timeout: Could not get a resource from the pool within 3000ms"),
        ("FLAKE-206", "test_kafka_consumer_group_rebalance_lag", "Event Bus", "restassured-java", "CommitFailedException: Group rebalance took longer than max.poll.interval.ms"),
        ("FLAKE-207", "test_browser_context_crash_oom_in_container", "UI Matrix", "playwright-ts", "Target closed / crash detected: Browser renderer process was killed (OOM)"),
        ("FLAKE-208", "test_ephemeral_port_exhaustion_in_parallel_matrix", "Socket Pool", "k6-js", "Cannot assign requested address (EADDRNOTAVAIL) on high concurrency port bind"),
        ("FLAKE-209", "test_stale_docker_volume_cache_drift", "DevSecOps", "devsecops-python", "File not found in /tmp/test-volume: Stale container layer drift in CI runner"),
        ("FLAKE-210", "test_chaos_proxy_packet_drop_burst", "Network Chaos", "playwright-ts", "net::ERR_NETWORK_CHANGED: Chaos proxy dropped 25% burst packets"),
        ("FLAKE-211", "test_s3_mock_slowdown_503_slow_down", "Storage Mock", "devsecops-python", "AWS S3 503 Slow Down: Request rate exceeded capacity in MinIO container"),
        ("FLAKE-212", "test_browser_font_rendering_jitter", "Visual Testing", "a11y-axe", "Visual snapshot pixel delta 4.2% due to asynchronous font loading"),
        ("FLAKE-213", "test_db_connection_pool_timeout", "Database", "restassured-java", "HikariPool-1 - Connection is not available, request timed out after 30000ms"),
        ("FLAKE-214", "test_maestro_adb_device_offline_flakiness", "Mobile Emulator", "maestro-mobile", "ADB server dropped connection: device emulator-5554 offline"),
        ("FLAKE-215", "test_jmeter_gc_pause_response_time_outlier", "JMeter Engine", "jmeter", "Response time 8420ms spiked during JVM Full Garbage Collection pause"),
        ("FLAKE-216", "test_pact_broker_network_partition", "Pact Broker", "contract-pact", "Failed to retrieve pact specification: Broker connection refused on 8080"),
        ("FLAKE-217", "test_ollama_mock_local_socket_timeout", "LLM Mock", "genai-qa", "HTTPConnectionPool(host='localhost', port=11434): Read timed out (5.0s)"),
        ("FLAKE-218", "test_playwright_ws_cdp_disconnect", "CDP Protocol", "playwright-ts", "WebSocket connection to ws://127.0.0.1:9222/devtools/page was closed"),
        ("FLAKE-219", "test_cross_tenant_db_lock_contention", "Multi-Tenant DB", "restassured-java", "Lock wait timeout exceeded; try restarting transaction"),
        ("FLAKE-220", "test_disk_io_throttling_on_shared_ci_runner", "CI Runner", "foundations", "IOError: [Errno 28] No space left on device or IOPS quota exhausted"),
        ("FLAKE-221", "test_grpc_deadline_exceeded_proxy_jitter", "gRPC Transport", "restassured-java", "StatusRuntime: DEADLINE_EXCEEDED (Client deadline was 2000ms, elapsed 2150ms)"),
        ("FLAKE-222", "test_shadow_dom_hydration_race_on_mobile", "Mobile Web", "maestro-mobile", "Element not attached to DOM during mobile viewport emulation reflow"),
        ("FLAKE-223", "test_cors_preflight_options_drop", "Gateway Proxy", "playwright-ts", "Failed to load resource: Response to preflight request doesn't pass access control"),
        ("FLAKE-224", "test_oauth_token_refresh_clock_skew", "Auth Service", "devsecops-python", "JWT expired: clock skew between CI runner and auth server is 12 seconds"),
        ("FLAKE-225", "test_parallel_runner_sqlite_lock", "Local DB", "foundations", "sqlite3.OperationalError: database is locked across 4 parallel pytest workers"),
    ]

    for tid, name, suite, track, desc in flaky_infra_templates:
        tests.append(
            ChaosTestResultItem(
                test_id=tid,
                name=name,
                suite=suite,
                track_id=track,
                status="flaky",
                category="flaky_infra",
                duration_ms=1200 + (int(tid.split("-")[1]) * 40) % 2500,
                error_message=f"Flaky Infra Anomaly: {desc}",
                stack_trace=f"Infrastructure Telemetry in {suite}: {desc}",
                chaos_event=ChaosEventTelemetry(
                    layer="Network/Infra",
                    event_type="infra_jitter",
                    latency_ms=1500,
                    jitter_ms=450,
                    packet_loss_rate=0.08,
                    proxy_log=f"WARN [Infra Chaos] {desc}",
                    correlated_timestamp="2026-08-24T18:06:00Z",
                    retry_attempts=2,
                ),
                flakiness_metrics=FlakinessMetrics(
                    iterations=5,
                    passed_iterations=2,
                    failed_iterations=3,
                    flakiness_rate=0.6,
                    avg_duration_ms=1600,
                    duration_stddev_ms=600.0,
                    historical_flake_score=0.55,
                ),
                steps=[
                    TestStepTelemetry(name="Initialize connection", status="passed", duration_ms=120),
                    TestStepTelemetry(name="Transmit request over network buffer", status="flaky", duration_ms=1400, error=desc),
                ],
                labels={"severity": "flaky", "track": track},
                root_cause_hint=desc,
            )
        )

    # =========================================================================
    # 3. TEST AUTOMATION ANTI-PATTERNS (26 Tests)
    # =========================================================================

    tests.append(
        ChaosTestResultItem(
            test_id="ANTI-301",
            name="test_search_autocomplete_hardcoded_sleep_race",
            suite="Search UI Drills",
            track_id="playwright-ts",
            status="failed",
            category="anti_pattern",
            duration_ms=3150,
            error_message="AssertionError: Expected search results count >= 3, found 0. Hardcoded sleep (1000ms) expired before debounced API responded under 1200ms latency",
            stack_trace="at test_search_autocomplete_hardcoded_sleep_race (exercises/01_web_playwright_ts/01_hydration/exercise.ts:32)\n    await page.waitForTimeout(1000); // ANTI-PATTERN",
            chaos_event=ChaosEventTelemetry(
                layer="L7",
                event_type="timing_race",
                latency_ms=1200,
                jitter_ms=150,
                packet_loss_rate=0.0,
                proxy_log="INFO [ChaosProxy:8086] GET /api/search?q=Playwright completed in 1200ms (client sleep expired at 1000ms)",
                correlated_timestamp="2026-08-24T18:10:05Z",
                retry_attempts=0,
                injection_target="http://127.0.0.1:8086/api/search",
            ),
            flakiness_metrics=FlakinessMetrics(
                iterations=5,
                passed_iterations=1,
                failed_iterations=4,
                flakiness_rate=0.8,
                avg_duration_ms=2100,
                duration_stddev_ms=320.0,
                historical_flake_score=0.85,
            ),
            steps=[
                TestStepTelemetry(name="Fill search input with 'Playwright'", status="passed", duration_ms=150),
                TestStepTelemetry(name="Arbitrary sleep waitForTimeout(1000)", status="passed", duration_ms=1000),
                TestStepTelemetry(name="Assert search results visible", status="failed", duration_ms=2000, error="Timed out waiting for .search-results"),
            ],
            labels={"severity": "anti-pattern", "track": "playwright-ts", "tier": "Foundations"},
            root_cause_hint="Hardcoded sleep (1000ms) raced against debounced search query taking 1200ms under injected proxy latency.",
        )
    )

    anti_pattern_templates = [
        ("ANTI-302", "test_transfer_brittle_absolute_xpath_selector", "Transfer Form", "playwright-ts", "Brittle XPath /html/body/div[2]/div[3]/table/tbody/tr[1]/td[2]/input failed after DOM wrapper refactor"),
        ("ANTI-303", "test_unawaited_floating_promise_race", "Catalog Page", "playwright-ts", "Missing await on page.click() caused runner to exit before button click was dispatched"),
        ("ANTI-304", "test_vacuous_assertion_false_confidence", "Foundations", "foundations", "Vacuous assertion: expect(true).toBe(true) ignored 100% of underlying business logic"),
        ("ANTI-305", "test_stale_dom_reference_element_handle", "Web UI", "playwright-ts", "StaleElementReference: Target element was detached from DOM during client-side hydration"),
        ("ANTI-306", "test_hardcoded_admin_password_plaintext", "DevSecOps", "devsecops-python", "Hardcoded credential: password = 'super_secret_123' committed directly in test fixture"),
        ("ANTI-307", "test_shared_global_state_pollution", "Parallel Suite", "foundations", "Test pollution: Mutating global DEFAULT_ACCOUNTS modified state for subsequent tests"),
        ("ANTI-308", "test_sleep_in_k6_virtual_users", "Load Suite", "k6-js", "Anti-pattern: sleep(10) inside VU iteration artificially lowered RPS metrics"),
        ("ANTI-309", "test_deep_css_chain_utility_class", "Checkout UI", "playwright-ts", "Brittle selector .btn.btn-primary.large.blue broke when Tailwind styling changed"),
        ("ANTI-310", "test_missing_arrange_act_assert_separation", "Foundations", "foundations", "Anti-pattern: 15 mixed assertions without clear AAA boundary in single 200-line test"),
        ("ANTI-311", "test_testing_the_mock_tautology", "Foundations", "foundations", "Tautological mock: Asserted return value of mock method without exercising real code path"),
        ("ANTI-312", "test_raw_unwrap_panic_in_rust_fixture", "Rust Tests", "foundations", "called `Result::unwrap()` on an `Err` value without diagnostic assertion"),
        ("ANTI-313", "test_missing_awaitility_polling_in_java", "API Drills", "restassured-java", "Thread.sleep(3000) used instead of event-driven Awaitility polling"),
        ("ANTI-314", "test_ignoring_ssl_verification_globally", "Security Suite", "devsecops-python", "Anti-pattern: verify=False disabled SSL validation across entire session"),
        ("ANTI-315", "test_maestro_tap_on_raw_coordinates", "Mobile Drills", "maestro-mobile", "Brittle tapOn: point: '450, 890' broke across different screen densities"),
        ("ANTI-316", "test_blind_assertion_on_http_200_only", "API Suite", "restassured-java", "Smoke only: Asserted 200 OK without validating response payload or business fields"),
        ("ANTI-317", "test_relying_on_test_execution_order", "Test Runner", "foundations", "Order dependency: Test B failed when run in isolation without Test A"),
        ("ANTI-318", "test_auto_generated_dynamic_id_selector", "Web UI", "playwright-ts", "Selector '#input-a7f9b2' targeted dynamic hash regenerated on every build"),
        ("ANTI-319", "test_missing_cleanup_temporary_files", "File System", "foundations", "Test leaked 50 MB temporary files in /tmp without fixture teardown"),
        ("ANTI-320", "test_overly_broad_try_except_swallowing_errors", "Python Suite", "foundations", "except Exception: pass silently masked assertion failures"),
        ("ANTI-321", "test_unpinned_docker_image_latest_tag", "CI Workflow", "devsecops-python", "Workflow used node:latest instead of deterministic SHA or LTS version"),
        ("ANTI-322", "test_polling_without_timeout_boundary", "Async Tests", "playwright-ts", "Infinite while loop without max timeout budget froze CI runner"),
        ("ANTI-323", "test_direct_database_mutation_bypassing_api", "E2E Suite", "restassured-java", "Anti-pattern: Inserting DB row directly bypassed business validation layer"),
        ("ANTI-324", "test_hardcoded_port_conflict_in_tests", "Sandbox", "foundations", "Hardcoded port 8080 caused binding conflicts during parallel test execution"),
        ("ANTI-325", "test_asserting_entire_json_blob_fragile", "API Drills", "restassured-java", "Full JSON string match broke when timestamp or ordering changed"),
        ("ANTI-326", "test_missing_error_context_in_custom_assert", "Custom Assert", "foundations", "Custom matcher failed with generic False without diff context"),
    ]

    for tid, name, suite, track, desc in anti_pattern_templates:
        tests.append(
            ChaosTestResultItem(
                test_id=tid,
                name=name,
                suite=suite,
                track_id=track,
                status="failed",
                category="anti_pattern",
                duration_ms=250 + (int(tid.split("-")[1]) * 20) % 600,
                error_message=f"Anti-Pattern Violation: {desc}",
                stack_trace=f"Test Suite Anti-Pattern in {suite}: {desc}",
                chaos_event=ChaosEventTelemetry(
                    layer="Test Code",
                    event_type="anti_pattern",
                    latency_ms=10,
                    jitter_ms=2,
                    packet_loss_rate=0.0,
                    proxy_log=f"WARN [AST Lint] {desc}",
                    correlated_timestamp="2026-08-24T18:11:00Z",
                    retry_attempts=0,
                ),
                flakiness_metrics=FlakinessMetrics(
                    iterations=5,
                    passed_iterations=1,
                    failed_iterations=4,
                    flakiness_rate=0.8,
                    avg_duration_ms=450,
                    duration_stddev_ms=80.0,
                    historical_flake_score=0.7,
                ),
                steps=[
                    TestStepTelemetry(name="Execute brittle test code", status="failed", duration_ms=250, error=desc)
                ],
                labels={"severity": "anti-pattern", "track": track},
                root_cause_hint=desc,
            )
        )

    return tests


def summarize_dataset(
    dataset: list[ChaosTestResultItem],
    results_dir: str = "target/allure-results",
    report_html_path: str = "target/allure-report/index.html",
) -> AllureSummaryResponse:
    """Compute summary statistics across chaotic test dataset."""
    total = len(dataset)
    passed = sum(1 for t in dataset if t.status == "passed")
    failed = sum(1 for t in dataset if t.status == "failed")
    broken = sum(1 for t in dataset if t.status == "broken")
    flaky = sum(1 for t in dataset if t.status == "flaky")
    skipped = sum(1 for t in dataset if t.status == "skipped")

    real_bugs = sum(1 for t in dataset if t.category == "real_bug")
    flaky_infra = sum(1 for t in dataset if t.category == "flaky_infra")
    anti_patterns = sum(1 for t in dataset if t.category == "anti_pattern")
    none_cat = sum(1 for t in dataset if t.category in ("none", "healthy"))

    total_duration = sum(t.duration_ms for t in dataset)
    pass_pct = round((passed / total * 100.0), 1) if total > 0 else 0.0

    taxonomy_breakdown = {
        "real_bug": real_bugs,
        "flaky_infra": flaky_infra,
        "anti_pattern": anti_patterns,
        "none": none_cat,
    }

    return AllureSummaryResponse(
        total_tests=total,
        passed=passed,
        failed=failed,
        broken=broken,
        flaky=flaky,
        skipped=skipped,
        real_bugs=real_bugs,
        flaky_infra=flaky_infra,
        anti_patterns=anti_patterns,
        duration_ms=total_duration,
        pass_percentage=pass_pct,
        results_dir=results_dir,
        report_html_path=report_html_path,
        generated_at="2026-08-24T18:30:00Z",
        tests=dataset,
        taxonomy_breakdown=taxonomy_breakdown,
    )


def render_html_report_string(dataset: list[ChaosTestResultItem]) -> str:
    """Generate self-contained, responsive Allure HTML report."""
    summary = summarize_dataset(dataset)
    tests_json = json.dumps([t.model_dump() for t in dataset])

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Enterprise Allure Chaos Report — Cherenkov-Lings</title>
  <style>
    :root {{
      --bg-primary: #0b0f19;
      --bg-card: #111827;
      --bg-card-hover: #1f2937;
      --border-color: #374151;
      --text-primary: #f9fafb;
      --text-secondary: #9ca3af;
      --accent-blue: #38bdf8;
      --accent-green: #22c55e;
      --accent-red: #ef4444;
      --accent-yellow: #eab308;
      --accent-purple: #a855f7;
    }}
    * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }}
    body {{ background: var(--bg-primary); color: var(--text-primary); padding: 24px; }}
    .header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px; padding-bottom: 16px; border-bottom: 1px solid var(--border-color); }}
    .header h1 {{ font-size: 24px; color: var(--accent-blue); display: flex; align-items: center; gap: 8px; }}
    .header-badges {{ display: flex; gap: 12px; }}
    .badge {{ padding: 6px 12px; border-radius: 9999px; font-size: 13px; font-weight: 600; }}
    .badge-pass {{ background: rgba(34, 197, 94, 0.15); color: var(--accent-green); border: 1px solid var(--accent-green); }}
    .badge-fail {{ background: rgba(239, 68, 68, 0.15); color: var(--accent-red); border: 1px solid var(--accent-red); }}
    .badge-flaky {{ background: rgba(234, 179, 8, 0.15); color: var(--accent-yellow); border: 1px solid var(--accent-yellow); }}
    .badge-bug {{ background: rgba(239, 68, 68, 0.2); color: #f87171; }}
    .badge-infra {{ background: rgba(56, 189, 248, 0.2); color: #38bdf8; }}
    .badge-anti {{ background: rgba(168, 85, 247, 0.2); color: #c084fc; }}
    
    .stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 24px; }}
    .stat-card {{ background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; padding: 18px; }}
    .stat-label {{ font-size: 12px; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.05em; }}
    .stat-value {{ font-size: 28px; font-weight: 700; margin-top: 6px; }}
    
    .controls {{ display: flex; gap: 12px; margin-bottom: 20px; flex-wrap: wrap; }}
    .search-input {{ flex: 1; min-width: 250px; background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 6px; padding: 8px 14px; color: #fff; }}
    .filter-btn {{ background: var(--bg-card); border: 1px solid var(--border-color); color: var(--text-secondary); padding: 8px 16px; border-radius: 6px; cursor: pointer; font-size: 13px; font-weight: 500; }}
    .filter-btn.active {{ background: var(--accent-blue); color: #000; font-weight: 600; border-color: var(--accent-blue); }}
    
    .table-container {{ background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; overflow: hidden; }}
    table {{ width: 100%; border-collapse: collapse; text-align: left; font-size: 13px; }}
    th {{ background: #1e293b; padding: 12px 16px; color: var(--text-secondary); font-weight: 600; border-bottom: 1px solid var(--border-color); }}
    td {{ padding: 12px 16px; border-bottom: 1px solid rgba(55, 65, 81, 0.5); }}
    tr:hover {{ background: var(--bg-card-hover); }}
    .test-id {{ font-family: monospace; font-weight: 600; color: var(--accent-blue); }}
    .btn-triage {{ background: #2563eb; color: #fff; border: none; padding: 4px 10px; border-radius: 4px; font-size: 12px; cursor: pointer; }}
    .btn-triage:hover {{ background: #1d4ed8; }}
    
    .modal {{ display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.7); justify-content: center; align-items: center; z-index: 100; }}
    .modal-content {{ background: var(--bg-card); border: 1px solid var(--border-color); border-radius: 8px; width: 90%; max-width: 700px; max-height: 85vh; overflow-y: auto; padding: 24px; }}
    .modal-header {{ display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }}
    .modal-close {{ background: none; border: none; color: #fff; font-size: 20px; cursor: pointer; }}
    pre {{ background: #0b0f19; padding: 12px; border-radius: 6px; font-size: 12px; overflow-x: auto; color: #38bdf8; margin-top: 8px; }}
  </style>
</head>
<body>
  <div class="header">
    <h1>⚡ Allure Chaos Test Report</h1>
    <div class="header-badges">
      <span class="badge badge-pass">{summary.passed} Passed</span>
      <span class="badge badge-fail">{summary.failed} Failed</span>
      <span class="badge badge-fail">{summary.broken} Broken</span>
      <span class="badge badge-flaky">{summary.flaky} Flaky</span>
    </div>
  </div>

  <div class="stats-grid">
    <div class="stat-card">
      <div class="stat-label">Total Tests</div>
      <div class="stat-value" style="color: var(--accent-blue);">{summary.total_tests}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Pass Percentage</div>
      <div class="stat-value" style="color: var(--accent-green);">{summary.pass_percentage}%</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Product Bugs</div>
      <div class="stat-value" style="color: var(--accent-red);">{summary.real_bugs}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Flaky Infrastructure</div>
      <div class="stat-value" style="color: var(--accent-yellow);">{summary.flaky_infra}</div>
    </div>
    <div class="stat-card">
      <div class="stat-label">Anti-Patterns</div>
      <div class="stat-value" style="color: var(--accent-purple);">{summary.anti_patterns}</div>
    </div>
  </div>

  <div class="controls">
    <input type="text" id="searchInput" class="search-input" placeholder="Search tests by ID, name, track, or error message..." oninput="filterTests()" />
    <button class="filter-btn active" onclick="setCategoryFilter('all', this)">All ({summary.total_tests})</button>
    <button class="filter-btn" onclick="setCategoryFilter('real_bug', this)">Product Bugs ({summary.real_bugs})</button>
    <button class="filter-btn" onclick="setCategoryFilter('flaky_infra', this)">Flaky Infra ({summary.flaky_infra})</button>
    <button class="filter-btn" onclick="setCategoryFilter('anti_pattern', this)">Anti-Patterns ({summary.anti_patterns})</button>
  </div>

  <div class="table-container">
    <table>
      <thead>
        <tr>
          <th>Test ID</th>
          <th>Test Name & Suite</th>
          <th>Track</th>
          <th>Status</th>
          <th>Taxonomy Category</th>
          <th>Duration</th>
          <th>Action</th>
        </tr>
      </thead>
      <tbody id="testTableBody">
      </tbody>
    </table>
  </div>

  <div id="telemetryModal" class="modal" onclick="closeModal(event)">
    <div class="modal-content" onclick="event.stopPropagation()">
      <div class="modal-header">
        <h3 id="modalTitle">Test Investigation</h3>
        <button class="modal-close" onclick="closeModalDirect()">&times;</button>
      </div>
      <div id="modalBody"></div>
    </div>
  </div>

  <script>
    const testsData = {tests_json};
    let currentCategory = 'all';

    function renderTable() {{
      const query = (document.getElementById('searchInput').value || '').toLowerCase();
      const tbody = document.getElementById('testTableBody');
      tbody.innerHTML = '';

      const filtered = testsData.filter(t => {{
        const matchesCategory = currentCategory === 'all' || t.category === currentCategory;
        const matchesSearch = !query || 
          t.test_id.toLowerCase().includes(query) || 
          t.name.toLowerCase().includes(query) || 
          t.track_id.toLowerCase().includes(query) ||
          (t.error_message && t.error_message.toLowerCase().includes(query));
        return matchesCategory && matchesSearch;
      }});

      for (const t of filtered) {{
        const row = document.createElement('tr');
        
        let statusBadge = `<span class="badge badge-pass">PASSED</span>`;
        if (t.status === 'failed') statusBadge = `<span class="badge badge-fail">FAILED</span>`;
        if (t.status === 'broken') statusBadge = `<span class="badge badge-fail">BROKEN</span>`;
        if (t.status === 'flaky') statusBadge = `<span class="badge badge-flaky">FLAKY</span>`;

        let catBadge = `<span class="badge badge-pass">Healthy</span>`;
        if (t.category === 'real_bug') catBadge = `<span class="badge badge-bug">Product Bug</span>`;
        if (t.category === 'flaky_infra') catBadge = `<span class="badge badge-infra">Flaky Infra</span>`;
        if (t.category === 'anti_pattern') catBadge = `<span class="badge badge-anti">Anti-Pattern</span>`;

        row.innerHTML = `
          <td class="test-id">${{t.test_id}}</td>
          <td>
            <div style="font-weight: 600;">${{t.name}}</div>
            <div style="color: #64748b; font-size: 11px;">${{t.suite}}</div>
          </td>
          <td><code>${{t.track_id}}</code></td>
          <td>${{statusBadge}}</td>
          <td>${{catBadge}}</td>
          <td>${{t.duration_ms}}ms</td>
          <td>
            <button class="btn-triage" onclick="inspectTest('${{t.test_id}}')">Inspect Telemetry</button>
          </td>
        `;
        tbody.appendChild(row);
      }}
    }}

    function setCategoryFilter(cat, btn) {{
      currentCategory = cat;
      document.querySelectorAll('.filter-btn').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      renderTable();
    }}

    function filterTests() {{
      renderTable();
    }}

    function inspectTest(testId) {{
      const test = testsData.find(t => t.test_id === testId);
      if (!test) return;

      document.getElementById('modalTitle').innerText = `${{test.test_id}}: ${{test.name}}`;
      const body = document.getElementById('modalBody');
      body.innerHTML = `
        <p><strong>Suite:</strong> ${{test.suite}} (<code>${{test.track_id}}</code>)</p>
        <p><strong>Category:</strong> ${{test.category}} | <strong>Duration:</strong> ${{test.duration_ms}}ms</p>
        ${{test.error_message ? `<div style="margin-top: 12px;"><strong>Error Message:</strong><pre>${{test.error_message}}</pre></div>` : ''}}
        ${{test.chaos_event && test.chaos_event.proxy_log ? `<div style="margin-top: 12px;"><strong>Chaos Proxy L4/L7 Telemetry:</strong><pre>${{test.chaos_event.proxy_log}}</pre></div>` : ''}}
        ${{test.stack_trace ? `<div style="margin-top: 12px;"><strong>Stack Trace:</strong><pre>${{test.stack_trace}}</pre></div>` : ''}}
        ${{test.root_cause_hint ? `<div style="margin-top: 12px; color: #a855f7;"><strong>Root Cause Analysis:</strong><p style="margin-top: 4px; font-size: 13px;">${{test.root_cause_hint}}</p></div>` : ''}}
      `;
      document.getElementById('telemetryModal').style.display = 'flex';
    }}

    function closeModal(e) {{
      if (e.target.id === 'telemetryModal') {{
        document.getElementById('telemetryModal').style.display = 'none';
      }}
    }}

    function closeModalDirect() {{
      document.getElementById('telemetryModal').style.display = 'none';
    }}

    // Initial render
    renderTable();
  </script>
</body>
</html>"""
