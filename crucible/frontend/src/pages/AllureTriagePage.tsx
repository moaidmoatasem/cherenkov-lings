import React, { useEffect, useMemo, useState } from 'react';
import { apiUrl } from '../lib/api';
import {
  fetchTriageTests,
  submitTriage,
  fetchAllureSummary,
  type TriageVerdict,
  type AllureMetrics,
} from '../lib/triageApi';

export type FailureCategory = 'ProductBug' | 'FlakyInfra' | 'AntiPattern' | 'Passed';
export type TestStatus = 'passed' | 'failed' | 'broken' | 'flaky';

export interface TestCaseResult {
  id: string;
  name: string;
  track: string;
  suite: string;
  status: TestStatus;
  durationMs: number;
  category: FailureCategory;
  errorMessage?: string;
  stackTrace?: string;
  chaosLogs: string[];
  os: string;
  shard: string;
  retries: number;
  groundTruthExplanation: string;
  groundTruthRemediation: string;
}

const CHAOS_TEST_CASES: TestCaseResult[] = [
  {
    id: 'tc-01',
    name: 'test_checkout_with_ssr_hydration_delay',
    track: '01_web_playwright_ts',
    suite: 'Hydration & Client State Suite',
    status: 'flaky',
    durationMs: 4200,
    category: 'AntiPattern',
    errorMessage: 'locator.click: Timeout 4000ms exceeded waiting for #checkout-btn',
    stackTrace: `Error: locator.click: Timeout 4000ms exceeded.
    at /exercises/01_web_playwright_ts/01_hydration/exercise.ts:14:28
    at async TestRunner.runSingle (node_modules/@playwright/test/runner.js:842:12)`,
    chaosLogs: [
      '[08:14:02.100] [ChaosProxy:8086] INJECTED HTTP 200 with SSR artificial delay: 350ms on /checkout',
      '[08:14:02.455] [DOM Inspector] Button rendered without click listener (hydration in-flight)',
      '[08:14:02.650] [TestRunner] Naive waitForTimeout(200) expired before React hydration completed',
      '[08:14:06.652] [TestRunner] Click dispatched to unhydrated DOM node; event dropped silently'
    ],
    os: 'ubuntu-latest',
    shard: '1/4',
    retries: 2,
    groundTruthExplanation:
      'The test relies on a static 200ms sleep that is shorter than the SSR hydration delay (350ms under chaos proxy), causing the click to be dropped before event handlers are attached.',
    groundTruthRemediation:
      'Replace the arbitrary sleep with a web-first locator assertion: await expect(page.getByTestId("order-status")).toHaveText("Order Confirmed") which auto-polls until hydrated.'
  },
  {
    id: 'tc-02',
    name: 'test_payment_gateway_504_retry_backoff',
    track: '02_api_rest_assured_java',
    suite: 'Payment Gateway Fault Injection',
    status: 'failed',
    durationMs: 8500,
    category: 'FlakyInfra',
    errorMessage: 'java.net.SocketTimeoutException: 504 Gateway Timeout on POST /api/checkout',
    stackTrace: `java.net.SocketTimeoutException: Read timed out
    at java.base/sun.nio.ch.SocketDispatcher.read0(Native Method)
    at com.crucible.payment.GatewayClient.charge(GatewayClient.java:88)
    at com.crucible.payment.PaymentTest.test_payment_gateway_504_retry(PaymentTest.java:42)`,
    chaosLogs: [
      '[08:14:10.012] [ChaosProxy:8086] INJECTED FAULT: HTTP 504 Gateway Timeout (latency=1450ms, jitter=300ms) on POST /api/checkout',
      '[08:14:11.465] [ChaosProxy:8086] Downstream payment microservice connection closed unexpectedly (L7 proxy drop)',
      '[08:14:14.200] [Client Telemetry] Request aborted after 3000ms read timeout'
    ],
    os: 'ubuntu-latest',
    shard: '2/4',
    retries: 3,
    groundTruthExplanation:
      'The test failed because Chaos Proxy injected an L7 HTTP 504 Gateway Timeout, simulating an upstream microservice network partition or gateway latency anomaly.',
    groundTruthRemediation:
      'Implement an exponential backoff retry policy (resilience4j / tenacity) with jitter, and configure circuit breakers to handle transient 504s gracefully.'
  },
  {
    id: 'tc-03',
    name: 'test_inventory_balance_underflow_race',
    track: '04_contract_pact_python',
    suite: 'Transactional Integrity Suite',
    status: 'broken',
    durationMs: 1950,
    category: 'ProductBug',
    errorMessage: 'AssertionError: Balance went negative (-$45.00) after concurrent debit operations',
    stackTrace: `AssertionError: Expected balance >= 0, but got -45.00
    at tests/test_inventory.py:58: assert balance >= 0
    at test_concurrent_transfers (tests/test_inventory.py:64)`,
    chaosLogs: [
      '[08:14:15.890] [ChaosProxy:8086] INJECTED L4 TCP packet jitter: 120ms during concurrent POST /transfer',
      '[08:14:16.020] [Crucible DB] Concurrent isolation level: READ COMMITTED (Row lock missing on balance table)',
      '[08:14:16.050] [Crucible DB] Double-spend debit executed: Account 101 debited twice concurrently'
    ],
    os: 'windows-latest',
    shard: '3/4',
    retries: 1,
    groundTruthExplanation:
      'This is a genuine product defect: the backend balance debit operation lacks pessimistic locking (SELECT FOR UPDATE) or atomic CAS constraints, permitting double-spend under concurrent load.',
    groundTruthRemediation:
      'Enforce atomic database transaction isolation with SELECT FOR UPDATE or an optimistic version lock (@Version in JPA / SQLAlchemy) on the account balance entity.'
  },
  {
    id: 'tc-04',
    name: 'test_search_input_debounce_race_condition',
    track: '01_web_playwright_ts',
    suite: 'Search Async Timing Suite',
    status: 'flaky',
    durationMs: 3100,
    category: 'AntiPattern',
    errorMessage: 'locator.fill: Target closed / unhandled floating Promise',
    stackTrace: `Error: Target page, context or browser has been closed
    at /exercises/01_web_playwright_ts/03_search/exercise.ts:18:10
    at async Worker.runTest (playwright-core/lib/runner.js:290)`,
    chaosLogs: [
      '[08:14:22.400] [ChaosProxy:8086] Network delay 400ms on GET /search?q=Wireless',
      '[08:14:22.410] [Browser Console] Unhandled Promise Rejection: fetch("/search") superseded by next keypress',
      '[08:14:22.750] [Test Runner] Missing await on page.locator("#search-query").fill() caused early suite tearDown'
    ],
    os: 'macos-latest',
    shard: '4/4',
    retries: 2,
    groundTruthExplanation:
      'The test contains floating promises (missing await keyword on async fill/click operations), causing the browser context to tear down while network requests were still inflight.',
    groundTruthRemediation:
      'Always await every asynchronous Playwright action and assert using auto-waiting locators: await page.locator("#search-query").fill("Wireless");'
  },
  {
    id: 'tc-05',
    name: 'test_kafka_order_event_stream_lag',
    track: '04_contract_pact_python',
    suite: 'Kafka Event Consumer Suite',
    status: 'flaky',
    durationMs: 6200,
    category: 'FlakyInfra',
    errorMessage: 'TimeoutException: Kafka consumer record did not arrive within 5000ms',
    stackTrace: `TimeoutException: Failed to receive record on topic 'order-events' within 5000ms
    at org.apache.kafka.clients.consumer.KafkaConsumer.poll(KafkaConsumer.java:1200)
    at com.crucible.events.ConsumerTest.test_kafka_order_event_stream(ConsumerTest.java:77)`,
    chaosLogs: [
      '[08:14:30.120] [ChaosProxy:8086] INJECTED Kafka Broker Consumer Group Rebalance Latency (3200ms)',
      '[08:14:33.325] [Kafka Mock] Consumer group [order-audit] rebalance completed',
      '[08:14:35.150] [Test Runner] Test timeout 5000ms exceeded before event arrived at consumer'
    ],
    os: 'ubuntu-latest',
    shard: '1/4',
    retries: 3,
    groundTruthExplanation:
      'The failure was caused by simulated infrastructure latency: a Kafka consumer group rebalance artificially prolonged event arrival beyond the default 5s assertion window.',
    groundTruthRemediation:
      'Use Awaitility / dynamic polling with a 10s poll window for event-driven async assertions, or mock consumer partition assignment in unit integration tests.'
  },
  {
    id: 'tc-06',
    name: 'test_shadow_dom_closed_root_piercing',
    track: '01_web_playwright_ts',
    suite: 'Shadow DOM & Isolation Suite',
    status: 'passed',
    durationMs: 1450,
    category: 'Passed',
    chaosLogs: [
      '[08:14:40.050] [ChaosProxy:8086] Normal latency baseline (15ms)',
      '[08:14:40.350] [Playwright Engine] Pierced ShadowRoot using semantic getByRole("button", { name: "Submit" })',
      '[08:14:41.500] [Test Runner] 100% assertions satisfied'
    ],
    os: 'ubuntu-latest',
    shard: '2/4',
    retries: 0,
    groundTruthExplanation: 'Test passed cleanly using semantic piercing locators.',
    groundTruthRemediation: 'No remediation required.'
  },
  {
    id: 'tc-07',
    name: 'test_security_sql_injection_sanitization',
    track: '03_devsecops_zap_python',
    suite: 'Security Vulnerability Suite',
    status: 'failed',
    durationMs: 2300,
    category: 'ProductBug',
    errorMessage: "SecurityAssertionError: Backend leaked raw SQLite error: 'unrecognized token near --'",
    stackTrace: `SecurityAssertionError: Expected sanitized 400 Bad Request, but received 500 Internal Server Error with SQL syntax leak
    at tests/security/test_sqli.py:44
    at pytest_pyfunc_call (pytest/runner.py:321)`,
    chaosLogs: [
      '[08:14:45.200] [ChaosProxy:8086] Injected payload: "admin\' OR 1=1 --"',
      '[08:14:45.350] [Crucible API] 500 Internal Server Error returned: sqlite3.OperationalError: near "OR": syntax error',
      '[08:14:45.360] [Security Scanner] Critical CWE-89 (SQL Injection) detected in /api/security/user-lookup'
    ],
    os: 'ubuntu-latest',
    shard: '3/4',
    retries: 1,
    groundTruthExplanation:
      'Genuine product security vulnerability (CWE-89): The user lookup endpoint concatenates raw user strings into SQL queries without parameterized prepared statements.',
    groundTruthRemediation:
      'Refactor database query to use parameterized queries / ORM binding: cursor.execute("SELECT * FROM users WHERE username = ?", (username,))'
  }
];

export const AllureTriagePage: React.FC = () => {
  // Seeded with the bundled cases so the page still reads offline; replaced by
  // the backend's 70-case chaos dataset as soon as /api/triage/tests answers.
  const [testCases, setTestCases] = useState<TestCaseResult[]>(CHAOS_TEST_CASES);
  const [datasetIsLive, setDatasetIsLive] = useState<boolean>(false);
  const [selectedStatusFilter, setSelectedStatusFilter] = useState<string>('all');
  const [selectedCategoryFilter, setSelectedCategoryFilter] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [expandedTestId, setExpandedTestId] = useState<string | null>('tc-01');

  // Interactive Triage Challenge State
  const [triageTargetTestId, setTriageTargetTestId] = useState<string>('tc-01');
  const [chosenCategory, setChosenCategory] = useState<FailureCategory | null>(null);
  const [studentExplanation, setStudentExplanation] = useState<string>('');
  const [studentRemediation, setStudentRemediation] = useState<string>('');
  const [triageEvaluation, setTriageEvaluation] = useState<TriageVerdict | null>(null);
  const [isSubmitting, setIsSubmitting] = useState<boolean>(false);

  const [earnedXP, setEarnedXP] = useState<number>(0);
  const [toastMessage, setToastMessage] = useState<string | null>(null);

  useEffect(() => {
    const ctrl = new AbortController();
    fetchTriageTests(ctrl.signal)
      .then((cases) => {
        if (cases.length === 0) return;
        setTestCases(cases);
        setDatasetIsLive(true);
        // The seeded ids (tc-01) do not exist in the real dataset, so the
        // selection has to move with it or every submission would 404.
        setExpandedTestId(cases[0].id);
        setTriageTargetTestId(cases[0].id);
      })
      .catch(() => {
        // Offline: keep the seeded cases. The header says which one is showing.
      });

    // The running total is whatever the backend has persisted, not a number
    // this page invented — it is the same figure Mission Control shows.
    fetch(apiUrl('/api/progress'), { signal: ctrl.signal })
      .then((res) => (res.ok ? res.json() : null))
      .then((data) => {
        if (typeof data?.total_xp === 'number') setEarnedXP(data.total_xp);
      })
      .catch(() => {});

    return () => ctrl.abort();
  }, []);

  // Filtered Test Cases
  const filteredTestCases = useMemo(() => {
    return testCases.filter((tc) => {
      const matchStatus = selectedStatusFilter === 'all' || tc.status === selectedStatusFilter;
      const matchCategory = selectedCategoryFilter === 'all' || tc.category === selectedCategoryFilter;
      const matchSearch =
        tc.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        tc.suite.toLowerCase().includes(searchQuery.toLowerCase()) ||
        (tc.errorMessage && tc.errorMessage.toLowerCase().includes(searchQuery.toLowerCase()));
      return matchStatus && matchCategory && matchSearch;
    });
  }, [testCases, selectedStatusFilter, selectedCategoryFilter, searchQuery]);

  // Allure KPI Metrics: seeded offline, replaced by GET /api/reports/allure
  // (the same 70-case chaos dataset the triage tests below come from) as soon
  // as it answers.
  const [metrics, setMetrics] = useState<AllureMetrics>({
    total: 68,
    passed: 52,
    flaky: 10,
    failed: 6,
    passRate: '76.5',
    flakyRate: '14.7',
    productBugs: 6,
    flakyInfra: 7,
    antiPatterns: 3,
    stabilityTrend: [
      { label: 'Least reliable quintile', passPct: 65 },
      { label: 'Below-average quintile', passPct: 71 },
      { label: 'Median quintile', passPct: 82 },
      { label: 'Above-average quintile', passPct: 94 },
      { label: 'Most reliable quintile', passPct: 100 },
    ],
  });

  useEffect(() => {
    const ctrl = new AbortController();
    fetchAllureSummary(ctrl.signal)
      .then(setMetrics)
      .catch(() => {
        // Offline: keep the seeded KPIs.
      });
    return () => ctrl.abort();
  }, []);

  // Donut chart slices, sized from the real KPI metrics rather than baked-in
  // percentages -- so the picture moves when /api/reports/allure does.
  const DONUT_CIRCUMFERENCE = 2 * Math.PI * 45;
  const donutSegments = useMemo(() => {
    const total = metrics.total || 1;
    const slices = [
      { key: 'passed', label: 'Passed', color: '#4ade80', value: metrics.passed },
      { key: 'productBugs', label: 'Product Bugs', color: '#f87171', value: metrics.productBugs },
      { key: 'flakyInfra', label: 'Flaky Infra', color: '#fbbf24', value: metrics.flakyInfra },
      { key: 'antiPatterns', label: 'Anti-Patterns', color: '#c084fc', value: metrics.antiPatterns },
    ];
    let offset = 0;
    return slices.map((slice) => {
      const length = (slice.value / total) * DONUT_CIRCUMFERENCE;
      const seg = { ...slice, length, dashOffset: -offset };
      offset += length;
      return seg;
    });
  }, [metrics]);

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  const activeTriageTest = useMemo(() => {
    return testCases.find((t) => t.id === triageTargetTestId) || testCases[0];
  }, [testCases, triageTargetTestId]);

  // Submit Triage Hypothesis Evaluator Engine
  const handleEvaluateTriage = async () => {
    if (!chosenCategory) {
      showToast('⚠️ Please select a failure category before submitting.');
      return;
    }

    setIsSubmitting(true);
    try {
      // The backend owns the scoring model AND persists the award, so the XP
      // shown here is the same XP Mission Control reads back.
      const verdict = await submitTriage({
        testId: triageTargetTestId,
        category: chosenCategory,
        explanation: studentExplanation,
        fix: studentRemediation,
      });

      setTriageEvaluation(verdict);
      setEarnedXP(verdict.totalXp ?? ((prev) => prev + verdict.xpEarned)(earnedXP));
      showToast(
        `Triage evaluated: +${verdict.xpEarned} XP, saved to your record`
      );
    } catch (err) {
      // Never silently score it locally instead — a number that does not persist
      // is worse than an honest failure.
      showToast(
        `Could not reach the triage service (${
          err instanceof Error ? err.message : 'unknown error'
        }). Nothing was recorded.`
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="page-container allure-triage-page">
      {/* Toast */}
      {toastMessage && (
        <div className="allure-toast" role="alert">
          <span className="toast-icon">📊</span>
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Header Banner */}
      <div className="allure-header">
        <div className="header-left">
          <div className="badge-row">
            <span className="badge info">R3: Enterprise Allure Reports</span>
            <span className="badge purple">L4/L7 Chaos Telemetry</span>
            <span className="badge green">Interactive Triage Challenge</span>
          </div>
          <h1 className="page-title">Enterprise Allure Reports & Triage Station</h1>
          <p className="page-description">
            Correlate chaotic test failures against L4/L7 proxy logs, distinguish product defects from flaky infrastructure,
            and submit root-cause hypotheses to level up your SDET diagnostic skills.
          </p>
        </div>

        <div className="header-actions">
          <div className="xp-counter-card">
            <span className="xp-icon">🏆</span>
            <div className="xp-details">
              <span className="xp-val">{earnedXP} XP</span>
              <span className="xp-label" data-live={datasetIsLive}>
                {datasetIsLive
                  ? `Triage Detective Rank · ${testCases.length} live cases`
                  : 'Triage Detective Rank · offline sample'}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Allure KPI Metrics Row */}
      <div className="allure-kpi-grid">
        <div className="kpi-card total">
          <span className="kpi-label">Total Tests Run</span>
          <span className="kpi-value">{metrics.total}</span>
          <span className="kpi-sub">Across 4 Shards & 2 OS</span>
        </div>
        <div className="kpi-card pass">
          <span className="kpi-label">Pass Rate</span>
          <span className="kpi-value text-green">{metrics.passRate}%</span>
          <span className="kpi-sub">{metrics.passed} Tests Passing</span>
        </div>
        <div className="kpi-card flaky">
          <span className="kpi-label">Flaky Rate (Chaos)</span>
          <span className="kpi-value text-amber">{metrics.flakyRate}%</span>
          <span className="kpi-sub">{metrics.flaky} Intermittent Runs</span>
        </div>
        <div className="kpi-card failed">
          <span className="kpi-label">Broken / Failed</span>
          <span className="kpi-value text-red">{metrics.failed}</span>
          <span className="kpi-sub">Requires SDET Triage</span>
        </div>
      </div>

      {/* Visual Charts Row */}
      <div className="charts-grid">
        {/* Donut Chart */}
        <div className="chart-card">
          <h3 className="chart-title">Root-Cause Failure Taxonomy</h3>
          <div className="donut-chart-wrapper">
            <svg className="donut-svg" viewBox="0 0 120 120">
              <circle cx="60" cy="60" r="45" fill="none" stroke="#1e293b" strokeWidth="18" />
              {donutSegments.map((seg) => (
                <circle
                  key={seg.key}
                  cx="60"
                  cy="60"
                  r="45"
                  fill="none"
                  stroke={seg.color}
                  strokeWidth="18"
                  strokeDasharray={`${seg.length} ${DONUT_CIRCUMFERENCE}`}
                  strokeDashoffset={seg.dashOffset}
                />
              ))}
            </svg>

            <div className="donut-legend">
              <div className="legend-item">
                <span className="legend-dot green"></span>
                <span className="legend-name">Passed ({metrics.passed})</span>
              </div>
              <div className="legend-item">
                <span className="legend-dot red"></span>
                <span className="legend-name">Product Bugs ({metrics.productBugs})</span>
              </div>
              <div className="legend-item">
                <span className="legend-dot amber"></span>
                <span className="legend-name">Flaky Infra / 504 ({metrics.flakyInfra})</span>
              </div>
              <div className="legend-item">
                <span className="legend-dot purple"></span>
                <span className="legend-name">Anti-Patterns / Sleeps ({metrics.antiPatterns})</span>
              </div>
            </div>
          </div>
        </div>

        {/* Chaos Stability Trend */}
        <div className="chart-card">
          <h3 className="chart-title">Test Reliability Distribution</h3>
          <p className="chart-sub">
            Real avg. pass rate across each test&apos;s 5 chaos iterations, tests grouped into
            quintiles from least to most reliable
          </p>
          <div className="trend-bars-container">
            {metrics.stabilityTrend.map((bar, bIdx) => {
              const color = bar.passPct >= 90 ? 'green' : bar.passPct >= 75 ? 'amber' : 'red';
              return (
                <div key={bIdx} className="trend-bar-row">
                  <span className="bar-label">{bar.label}</span>
                  <div className="bar-track">
                    <div className={`bar-fill ${color}`} style={{ width: `${bar.passPct}%` }}></div>
                  </div>
                  <span className="bar-pct">{bar.passPct}%</span>
                </div>
              );
            })}
          </div>
        </div>
      </div>

      {/* Main Section: Expandable Test Cases Table */}
      <div className="test-cases-section">
        <div className="section-header-row">
          <div className="section-title-wrap">
            <h2>Test Telemetry & Chaos Logs</h2>
            <span className="section-sub">Click any row to expand stack traces and correlated L4/L7 proxy logs</span>
          </div>

          <div className="filters-bar">
            {/* Status Filter */}
            <select
              className="select-input"
              value={selectedStatusFilter}
              onChange={(e) => setSelectedStatusFilter(e.target.value)}
            >
              <option value="all">All Statuses</option>
              <option value="failed">Failed Only</option>
              <option value="flaky">Flaky Only</option>
              <option value="passed">Passed Only</option>
              <option value="broken">Broken Only</option>
            </select>

            {/* Category Filter */}
            <select
              className="select-input"
              value={selectedCategoryFilter}
              onChange={(e) => setSelectedCategoryFilter(e.target.value)}
            >
              <option value="all">All Root-Causes</option>
              <option value="ProductBug">Product Bugs</option>
              <option value="FlakyInfra">Flaky Infrastructure</option>
              <option value="AntiPattern">Test Anti-Patterns</option>
            </select>

            {/* Search Input */}
            <input
              type="text"
              className="search-input"
              placeholder="Search test name or error..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {/* Table */}
        <div className="table-responsive-wrapper">
          <table className="allure-table">
            <thead>
              <tr>
                <th>Status</th>
                <th>Test Case Name</th>
                <th>Suite / Track</th>
                <th>Duration</th>
                <th>Root-Cause Category</th>
                <th>Chaos Environment</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {filteredTestCases.map((tc) => {
                const isExpanded = expandedTestId === tc.id;
                return (
                  <React.Fragment key={tc.id}>
                    <tr
                      className={`test-row ${tc.status} ${isExpanded ? 'expanded-row' : ''}`}
                      onClick={() => setExpandedTestId(isExpanded ? null : tc.id)}
                    >
                      <td>
                        <span className={`status-badge ${tc.status}`}>{tc.status.toUpperCase()}</span>
                      </td>
                      <td className="test-name-cell">
                        <span className="name-text">{tc.name}</span>
                        {tc.errorMessage && <span className="error-preview">{tc.errorMessage}</span>}
                      </td>
                      <td>
                        <span className="suite-text">{tc.suite}</span>
                        <span className="track-sub">{tc.track}</span>
                      </td>
                      <td className="duration-cell">{tc.durationMs}ms</td>
                      <td>
                        <span className={`category-tag ${tc.category}`}>{tc.category}</span>
                      </td>
                      <td>
                        <span className="env-meta">
                          {tc.os} | Shard {tc.shard}
                        </span>
                      </td>
                      <td>
                        <button
                          className="triage-action-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            setTriageTargetTestId(tc.id);
                            setChosenCategory(null);
                            setStudentExplanation('');
                            setStudentRemediation('');
                            setTriageEvaluation(null);
                            // Scroll to triage challenge
                            document.getElementById('triage-challenge-section')?.scrollIntoView({ behavior: 'smooth' });
                          }}
                        >
                          🎯 Triage
                        </button>
                      </td>
                    </tr>

                    {/* Expandable Details Drawer */}
                    {isExpanded && (
                      <tr className="expanded-details-row">
                        <td colSpan={7}>
                          <div className="test-details-panel">
                            {/* Stack Trace */}
                            {tc.stackTrace && (
                              <div className="detail-block">
                                <h4 className="detail-heading">
                                  <span className="detail-icon">🛑</span>
                                  <span>Stack Trace & Assertion Failure</span>
                                </h4>
                                <pre className="stack-trace-box">
                                  <code>{tc.stackTrace}</code>
                                </pre>
                              </div>
                            )}

                            {/* Correlated Chaos Logs */}
                            <div className="detail-block">
                              <h4 className="detail-heading">
                                <span className="detail-icon">🌀</span>
                                <span>Correlated L4/L7 Chaos Proxy Logs (Port 8086)</span>
                              </h4>
                              <div className="chaos-logs-box">
                                {tc.chaosLogs.map((log, lIdx) => (
                                  <div key={lIdx} className="chaos-log-line">
                                    <code>{log}</code>
                                  </div>
                                ))}
                              </div>
                            </div>

                            {/* Telemetry Footer */}
                            <div className="details-footer-row">
                              <div className="meta-pills">
                                <span className="pill">Retries: {tc.retries}</span>
                                <span className="pill">Runner: {tc.os}</span>
                                <span className="pill">Shard: {tc.shard}</span>
                              </div>
                              <button
                                className="primary-btn launch-triage-btn"
                                onClick={() => {
                                  setTriageTargetTestId(tc.id);
                                  setChosenCategory(null);
                                  setStudentExplanation('');
                                  setStudentRemediation('');
                                  setTriageEvaluation(null);
                                  document.getElementById('triage-challenge-section')?.scrollIntoView({ behavior: 'smooth' });
                                }}
                              >
                                🎯 Launch Hypothesis Challenge for this Test
                              </button>
                            </div>
                          </div>
                        </td>
                      </tr>
                    )}
                  </React.Fragment>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* Interactive Triage Hypothesis Challenge Section */}
      <div id="triage-challenge-section" className="triage-challenge-card">
        <div className="triage-header">
          <div className="triage-title-group">
            <span className="triage-icon">🎯</span>
            <div>
              <h3>Interactive Triage Hypothesis Challenge</h3>
              <p>Formulate and test your diagnostic hypothesis against enterprise failure taxonomies.</p>
            </div>
          </div>

          <div className="target-test-badge">
            <span className="target-label">Target Test:</span>
            <span className="target-name">{activeTriageTest.name}</span>
          </div>
        </div>

        <div className="triage-form-body">
          {/* Step 1: Select Category */}
          <div className="triage-step">
            <span className="step-tag">Step 1</span>
            <label className="step-label">Classify the Root-Cause Failure Taxonomy:</label>
            <div className="category-selection-grid">
              <button
                type="button"
                className={`category-option-card ${chosenCategory === 'ProductBug' ? 'selected' : ''}`}
                onClick={() => setChosenCategory('ProductBug')}
              >
                <span className="opt-icon">🐛</span>
                <span className="opt-title">Genuine Product Bug</span>
                <p className="opt-desc">Application code logic flaw, unhandled state, or security vulnerability.</p>
              </button>

              <button
                type="button"
                className={`category-option-card ${chosenCategory === 'FlakyInfra' ? 'selected' : ''}`}
                onClick={() => setChosenCategory('FlakyInfra')}
              >
                <span className="opt-icon">🌀</span>
                <span className="opt-title">Flaky Infrastructure Chaos</span>
                <p className="opt-desc">Network latency spike, HTTP 504 gateway timeout, or Kafka lag.</p>
              </button>

              <button
                type="button"
                className={`category-option-card ${chosenCategory === 'AntiPattern' ? 'selected' : ''}`}
                onClick={() => setChosenCategory('AntiPattern')}
              >
                <span className="opt-icon">⚠️</span>
                <span className="opt-title">Test Anti-Pattern</span>
                <p className="opt-desc">Hardcoded sleep timeout, fragile XPath locator, or missing await.</p>
              </button>
            </div>
          </div>

          {/* Step 2: Root-Cause Explanation */}
          <div className="triage-step">
            <span className="step-tag">Step 2</span>
            <label className="step-label">Provide Technical Root-Cause Explanation:</label>
            <p className="step-hint">Explain the sequence of events leading to failure based on the chaos logs.</p>
            <textarea
              className="triage-textarea"
              placeholder="e.g., The test failed because Chaos Proxy injected an L7 504 Gateway Timeout on /api/checkout, exceeding the client read timeout before retry backoff could engage..."
              value={studentExplanation}
              onChange={(e) => setStudentExplanation(e.target.value)}
              rows={3}
            />
          </div>

          {/* Step 3: Remediation Fix */}
          <div className="triage-step">
            <span className="step-tag">Step 3</span>
            <label className="step-label">Suggested Architectural Remediation / Fix:</label>
            <p className="step-hint">What changes should be made to either the application code or the test suite?</p>
            <textarea
              className="triage-textarea"
              placeholder="e.g., Implement exponential backoff retry in the API client and replace arbitrary sleep with web-first locator assertion..."
              value={studentRemediation}
              onChange={(e) => setStudentRemediation(e.target.value)}
              rows={3}
            />
          </div>

          {/* Submit Action */}
          <div className="triage-submit-row">
            <button
              className="primary-btn submit-hypothesis-btn"
              onClick={handleEvaluateTriage}
              disabled={isSubmitting}
            >
              <span className="btn-icon">⚡</span>
              <span>{isSubmitting ? 'Evaluating…' : 'Submit Hypothesis & Evaluate'}</span>
            </button>
          </div>

          {/* Evaluation Results Card */}
          {triageEvaluation && (
            <div
              className={`evaluation-result-card ${
                triageEvaluation.isCorrectCategory ? 'passed' : 'needs-improvement'
              }`}
            >
              <div className="eval-header">
                <div className="eval-score-wrap">
                  <span className="score-circle">{triageEvaluation.score}</span>
                  <div className="eval-title-wrap">
                    <h4>
                      {!triageEvaluation.isCorrectCategory
                        ? '⚠️ Wrong Category — Review Telemetry'
                        : triageEvaluation.score >= 140
                        ? '🏆 Master SDET Diagnosis'
                        : '✅ Valid Hypothesis Accepted'}
                    </h4>
                    <span className="xp-earned-badge">+{triageEvaluation.xpEarned} XP Earned</span>
                  </div>
                </div>

                {triageEvaluation.badgeUnlocked && (
                  <span className="badge green unlocked-badge-pill">
                    🎖️ Unlocked: {triageEvaluation.badgeUnlocked}
                  </span>
                )}
              </div>

              <div className="eval-body">
                <p className="eval-feedback">{triageEvaluation.feedback}</p>

                {/* Contrastive Feedback Table */}
                <div className="contrastive-box">
                  <h5 className="contrastive-title">Senior SDET Ground Truth Reference:</h5>
                  <div className="contrastive-grid">
                    <div className="contrast-col">
                      <span className="contrast-label">Ground Truth Root-Cause:</span>
                      <p className="contrast-text">{activeTriageTest.groundTruthExplanation}</p>
                    </div>
                    <div className="contrast-col">
                      <span className="contrast-label">Recommended Remediation:</span>
                      {/* The evaluator returns its reasoning; the bundled cases
                          carry a written remediation. Show whichever exists. */}
                      {triageEvaluation.detailedReasons.length > 0 ? (
                        <ul className="contrast-text">
                          {triageEvaluation.detailedReasons.map((reason, i) => (
                            <li key={i}>{reason}</li>
                          ))}
                        </ul>
                      ) : (
                        <p className="contrast-text">
                          {activeTriageTest.groundTruthRemediation || 'No remediation recorded.'}
                        </p>
                      )}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
