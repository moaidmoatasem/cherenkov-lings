import React, { useState, useMemo } from 'react';
import { StreamViewer } from '../components/StreamViewer';

export interface AstViolation {
  id: string;
  rule_id: string;
  rule_name: string;
  severity: 'error' | 'warning' | 'info';
  line_number: number;
  message: string;
  code_snippet: string;
  suggested_fix?: string;
  socratic_prompt: string;
  socratic_answer: string;
  fix_label: string;
  replacement_code: string;
}

export interface CodeTemplate {
  id: string;
  title: string;
  track: string;
  language: string;
  difficulty: 'Junior' | 'Mid' | 'Senior';
  description: string;
  initialCode: string;
}

const CODE_TEMPLATES: CodeTemplate[] = [
  {
    id: 'sleep_anti_pattern',
    title: 'Hydration Timing: Hardcoded Sleep',
    track: '01_web_playwright_ts',
    language: 'TypeScript',
    difficulty: 'Junior',
    description: 'Relies on arbitrary sleep timeout (5000ms) instead of web-first locator assertion.',
    initialCode: `import { test, expect } from '@playwright/test';

// I AM NOT DONE
test('user can complete checkout order', async ({ page }) => {
  await page.goto('http://localhost:8080/checkout');

  await page.locator('#item-select').selectOption('prod_499');
  await page.locator('#btn-add-cart').click();

  // ANTI-PATTERN: Hardcoded sleep causes flakiness under CPU throttle or wasted CI time
  await page.waitForTimeout(5000);

  const status = await page.locator('.status-badge').textContent();
  expect(status).toContain('Order Confirmed');
});`
  },
  {
    id: 'fragile_locators',
    title: 'Checkout Form: Fragile XPath & Deep CSS',
    track: '01_web_playwright_ts',
    language: 'TypeScript',
    difficulty: 'Mid',
    description: 'Uses brittle DOM-path XPath and chained utility classes vulnerable to UI redesigns.',
    initialCode: `import { test, expect } from '@playwright/test';

// I AM NOT DONE
test('submits transfer with fragile selectors', async ({ page }) => {
  await page.goto('http://localhost:8080/transfer');

  // ANTI-PATTERN: Brittle XPath tightly coupled to exact DOM hierarchy
  await page.locator('/html/body/div[2]/div[3]/table/tbody/tr[1]/td[2]/input').fill('ACC-9942');

  // ANTI-PATTERN: Class selector tied to layout framework styling
  await page.locator('.btn.btn-primary.submit-button-large.theme-blue').click();

  await expect(page.locator('xpath=//div[contains(@class, "receipt")]/span[2]')).toBeVisible();
});`
  },
  {
    id: 'floating_promise',
    title: 'Search Filter: Floating Promises (Missing Await)',
    track: '01_web_playwright_ts',
    language: 'TypeScript',
    difficulty: 'Senior',
    description: 'Async Playwright action calls without await cause race conditions and dropped events.',
    initialCode: `import { test, expect } from '@playwright/test';

// I AM NOT DONE
test('searches catalog with debounced input', async ({ page }) => {
  await page.goto('http://localhost:8080/search');

  // ANTI-PATTERN: Missing await causes floating promise and race condition
  page.locator('#search-query').fill('Wireless Headphones');

  // ANTI-PATTERN: Missing await on click before navigation finishes
  page.locator('button[type="submit"]').click();

  await expect(page.getByTestId('search-results')).toBeVisible();
});`
  },
  {
    id: 'missing_assertions',
    title: 'Payment Gateway: Missing Assertions & Blind Clicks',
    track: '02_api_rest_assured_java',
    language: 'TypeScript',
    difficulty: 'Junior',
    description: 'Performs payment state changes without verifying resulting transaction state or API receipts.',
    initialCode: `import { test, expect } from '@playwright/test';

// I AM NOT DONE
test('processes credit card payment', async ({ page }) => {
  await page.goto('http://localhost:8080/payment');

  await page.getByLabel('Card Number').fill('4111222233334444');
  await page.getByLabel('CVV').fill('123');
  await page.getByRole('button', { name: 'Submit Payment' }).click();

  // ANTI-PATTERN: Missing assertion - test exits without verifying receipt or charge confirmation
  console.log('Payment form submitted');
});`
  },
  {
    id: 'unsafe_unwrap',
    title: 'Kafka Consumer: Unsafe Unwrap / Force Unchecked',
    track: '04_contract_pact_python',
    language: 'TypeScript',
    difficulty: 'Mid',
    description: 'Unsafely unwraps nullable response objects without verifying error boundaries or payload schema.',
    initialCode: `import { test, expect } from '@playwright/test';

// I AM NOT DONE
test('parses order stream payload', async ({ request }) => {
  const response = await request.get('http://localhost:8081/api/pact/orders');
  const payload = (await response.json()) as any;

  // ANTI-PATTERN: Direct unwrap of deeply nested nullable property crashes test on unexpected null
  const transactionId = payload.data.orders[0].receipt.transactionId!;
  expect(transactionId).not.toBeNull();
});`
  },
  {
    id: 'mastered_clean',
    title: 'Clean Reference Solution (100/100 Benchmark)',
    track: '01_web_playwright_ts',
    language: 'TypeScript',
    difficulty: 'Senior',
    description: 'Resilient test using semantic web-first assertions, proper async awaits, and auto-waiting.',
    initialCode: `import { test, expect } from '@playwright/test';

test('user can complete checkout order', async ({ page }) => {
  await page.goto('http://localhost:8080/checkout');

  await page.getByLabel('Select Product').selectOption('prod_499');
  await page.getByRole('button', { name: 'Add to Cart' }).click();

  // Resilient web-first assertion with automatic polling and retry
  const orderStatus = page.getByTestId('order-status');
  await expect(orderStatus).toHaveText('Order Confirmed', { timeout: 8000 });
  await expect(page.getByRole('heading', { name: 'Receipt Summary' })).toBeVisible();
});`
  }
];

export const CodeReviewPage: React.FC = () => {
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>(CODE_TEMPLATES[0].id);
  const [code, setCode] = useState<string>(CODE_TEMPLATES[0].initialCode);
  const [activeRightTab, setActiveRightTab] = useState<'violations' | 'mentor' | 'wizard'>('violations');
  const [activeDiffMode, setActiveDiffMode] = useState<boolean>(false);
  const [selectedViolationForFix, setSelectedViolationForFix] = useState<AstViolation | null>(null);
  const [toastMessage, setToastMessage] = useState<string | null>(null);
  const [expandedSocraticId, setExpandedSocraticId] = useState<string | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState<boolean>(false);
  const [viewMode, setViewMode] = useState<'Embedded' | 'Native'>('Embedded');

  const activeTemplate = useMemo(() => {
    return CODE_TEMPLATES.find((t) => t.id === selectedTemplateId) || CODE_TEMPLATES[0];
  }, [selectedTemplateId]);

  // Client-Side AST Rule Scanner Engine
  const { violations, score, scoreCategories } = useMemo(() => {
    const detected: AstViolation[] = [];
    const lines = code.split('\n');

    let flakinessDeduction = 0;
    let locatorDeduction = 0;
    let asyncDeduction = 0;
    let assertionDeduction = 0;

    lines.forEach((lineText, idx) => {
      const lineNum = idx + 1;
      const trimmed = lineText.trim();

      // Rule 1: Hardcoded Sleeps
      if (
        lineText.includes('waitForTimeout') ||
        lineText.includes('Thread.sleep') ||
        lineText.includes('time.sleep') ||
        lineText.includes('setTimeout(')
      ) {
        flakinessDeduction += 35;
        detected.push({
          id: `sleep-${lineNum}`,
          rule_id: 'SDET-R01-NO-HARDCODED-SLEEPS',
          rule_name: 'Hardcoded Sleep Anti-Pattern',
          severity: 'error',
          line_number: lineNum,
          message: 'Hardcoded arbitrary sleep found. Causes flakiness in congested CI or wastes execution time.',
          code_snippet: lineText.trim(),
          suggested_fix: "await expect(page.getByTestId('order-status')).toHaveText('Order Confirmed');",
          socratic_prompt:
            'What happens when CI runner latency spikes from 100ms to 6500ms? How does Playwright auto-waiting assertions solve this?',
          socratic_answer:
            'Arbitrary sleeps are static time bombs: if the server takes 5001ms, a 5000ms sleep fails. If the server takes 50ms, it wastes 4950ms. Web-first assertions poll dynamically until the predicate holds true.',
          fix_label: 'Replace with web-first locator assertion',
          replacement_code: `  // Resilient web-first locator assertion with auto-polling
  await expect(page.getByTestId('order-status')).toHaveText('Order Confirmed', { timeout: 8000 });`
        });
      }

      // Rule 2: Fragile Locators (XPath / deep CSS / brittle class names)
      if (
        lineText.includes('/html/') ||
        lineText.includes('xpath=') ||
        lineText.includes('div > div') ||
        lineText.includes('tbody/tr[') ||
        lineText.includes('tbody > tr') ||
        lineText.includes('.btn.btn-primary.')
      ) {
        locatorDeduction += 30;
        detected.push({
          id: `locator-${lineNum}`,
          rule_id: 'SDET-R02-FRAGILE-DOM-LOCATOR',
          rule_name: 'Fragile DOM-Coupled Locator',
          severity: 'error',
          line_number: lineNum,
          message: 'Brittle XPath or structural CSS selector tightly coupled to layout implementation.',
          code_snippet: lineText.trim(),
          suggested_fix: "await page.getByLabel('Account Number').fill('ACC-9942');",
          socratic_prompt:
            'If a frontend engineer wraps the input field in a new <div> container, why will this test break even if the product works?',
          socratic_answer:
            'Hierarchical XPaths (/div/tr/td/input) break on every minor DOM refactor. Semantic user-facing locators (getByRole, getByLabel, getByTestId) remain resilient across redesigns.',
          fix_label: 'Refactor to user-facing semantic getByLabel / getByRole',
          replacement_code: `  // Semantic resilient locator targeting user-visible label
  await page.getByLabel('Account Number').fill('ACC-9942');`
        });
      }

      // Rule 3: Floating Promise (Missing await on page actions)
      if (
        (trimmed.startsWith('page.locator(') ||
          trimmed.startsWith('page.click(') ||
          trimmed.startsWith('page.fill(') ||
          trimmed.startsWith('page.goto(')) &&
        !trimmed.startsWith('await ') &&
        !trimmed.startsWith('const ') &&
        !trimmed.startsWith('let ') &&
        !trimmed.startsWith('return ')
      ) {
        asyncDeduction += 35;
        detected.push({
          id: `await-${lineNum}`,
          rule_id: 'SDET-R03-FLOATING-PROMISE',
          rule_name: 'Floating Promise (Missing await)',
          severity: 'error',
          line_number: lineNum,
          message: 'Async Playwright API called without await. Execution will race ahead and drop events.',
          code_snippet: lineText.trim(),
          suggested_fix: `await ${lineText.trim()}`,
          socratic_prompt:
            'What is the difference between firing an asynchronous Promise and awaiting its fulfillment before the next action?',
          socratic_answer:
            'Without await, JavaScript continues to execute the next line immediately before the previous action (typing, clicking) has completed in the browser process, resulting in intermittent race conditions.',
          fix_label: 'Add await keyword to asynchronous action',
          replacement_code: `  await ${lineText.trim()}`
        });
      }

      // Rule 4: Unsafe unwrap / force non-null operator
      if (
        (lineText.includes('.unwrap()') || lineText.includes('!') || lineText.includes('as any')) &&
        !lineText.includes('!==') &&
        !lineText.includes('!=') &&
        !lineText.includes('!re.')
      ) {
        if (!lineText.includes('//') && lineText.includes('!')) {
          flakinessDeduction += 15;
          detected.push({
            id: `unwrap-${lineNum}`,
            rule_id: 'SDET-R04-UNSAFE-UNWRAP',
            rule_name: 'Unsafe Force Unwrap / Nullable Access',
            severity: 'warning',
            line_number: lineNum,
            message: 'Unchecked non-null assertion (!) or raw unwrap bypasses error handling on null/undefined.',
            code_snippet: lineText.trim(),
            suggested_fix: 'expect(payload?.data?.orders?.[0]?.receipt?.transactionId).toBeDefined();',
            socratic_prompt:
              'Why is asserting defensive optional chaining (?.) superior to force-unwrapping (!) in distributed integration tests?',
            socratic_answer:
              'Force-unwrapping throws unhandled runtime TypeError crash exceptions that abort the entire suite. Defensive checks produce clear assertion failure messages showing the actual vs expected payload.',
            fix_label: 'Use safe optional chaining with explicit assertion',
            replacement_code: `  const order = payload?.data?.orders?.[0];
  expect(order?.receipt?.transactionId).toBeTruthy();`
          });
        }
      }
    });

    // Rule 5: Missing Assertions Check
    const hasExpect = code.includes('expect(') || code.includes('assert ') || code.includes('Assert.');
    const hasAction =
      code.includes('.click(') || code.includes('.fill(') || code.includes('.goto(') || code.includes('.selectOption(');

    if (hasAction && !hasExpect) {
      assertionDeduction += 40;
      detected.push({
        id: 'missing-assertions-global',
        rule_id: 'SDET-R05-MISSING-ASSERTIONS',
        rule_name: 'Missing Verification Assertion',
        severity: 'error',
        line_number: lines.length - 1,
        message: 'Test executes browser mutations without asserting UI state or server response.',
        code_snippet: 'console.log("Payment form submitted");',
        suggested_fix: "await expect(page.getByRole('alert')).toHaveText('Payment successful');",
        socratic_prompt:
          'If a test clicks a button but never verifies the consequence, can it catch a silent failure where the server returned 500?',
        socratic_answer:
          'No! Without assertions, tests merely act as "headless tourists" — they click elements and pass as long as no fatal JS exception was thrown, masking critical silent bugs.',
        fix_label: 'Add explicit UI state and receipt assertions',
        replacement_code: `  // Verify transaction receipt and confirmed state
  await expect(page.getByRole('alert')).toContainText('Payment processed successfully');
  await expect(page.getByTestId('receipt-id')).toBeVisible();`
      });
    }

    const totalDeductions = flakinessDeduction + locatorDeduction + asyncDeduction + assertionDeduction;
    const finalScore = Math.max(0, 100 - totalDeductions);

    const categories = {
      correctness: Math.max(0, 100 - assertionDeduction),
      flakinessRisk: Math.max(0, 100 - flakinessDeduction),
      locatorResilience: Math.max(0, 100 - locatorDeduction),
      asyncSafety: Math.max(0, 100 - asyncDeduction)
    };

    return {
      violations: detected,
      score: finalScore,
      scoreCategories: categories
    };
  }, [code]);

  // Handle template selection
  const handleSelectTemplate = (templateId: string) => {
    setSelectedTemplateId(templateId);
    const tmpl = CODE_TEMPLATES.find((t) => t.id === templateId);
    if (tmpl) {
      setCode(tmpl.initialCode);
      setSelectedViolationForFix(null);
      showToast(`Loaded template: ${tmpl.title}`);
    }
  };

  const showToast = (msg: string) => {
    setToastMessage(msg);
    setTimeout(() => setToastMessage(null), 3500);
  };

  // One-Click Automated Fix Applier
  const handleApplyFix = (violation: AstViolation) => {
    const lines = code.split('\n');
    let newCode = code;

    if (violation.id === 'missing-assertions-global') {
      // Append assertion before closing brace
      const lastIndex = lines.lastIndexOf('});');
      if (lastIndex !== -1) {
        lines.splice(lastIndex, 0, violation.replacement_code);
        newCode = lines.join('\n');
      } else {
        newCode = code + '\n' + violation.replacement_code;
      }
    } else if (violation.line_number > 0 && violation.line_number <= lines.length) {
      lines[violation.line_number - 1] = violation.replacement_code;
      newCode = lines.join('\n');
    }

    // Clean up '// I AM NOT DONE' if all violations resolved
    if (violations.length <= 1) {
      newCode = newCode.replace(/\/\/\s*I\s+AM\s+NOT\s+DONE\n?/g, '');
    }

    setCode(newCode);
    setSelectedViolationForFix(null);
    showToast(`Applied fix for: ${violation.rule_name} (+Score boost!)`);
  };

  const handleRunASTReview = () => {
    setIsAnalyzing(true);
    setTimeout(() => {
      setIsAnalyzing(false);
      showToast(`AST Static Analysis Complete: Score ${score}/100`);
    }, 450);
  };

  // Generate unified diff for wizard preview
  const generateDiffPreview = (violation: AstViolation) => {
    return {
      originalLine: violation.code_snippet,
      fixedLine: violation.replacement_code.trim()
    };
  };

  return (
    <div className="page-container code-review-page">
      {/* Toast Notification */}
      {toastMessage && (
        <div className="review-toast" role="alert">
          <span className="toast-icon">✨</span>
          <span>{toastMessage}</span>
        </div>
      )}

      {/* Header Banner */}
      <div className="review-header">
        <div className="header-left">
          <div className="badge-row">
            <span className="badge info">R1: AST Review Engine</span>
            <span className="badge purple">Socratic AI Mentor</span>
            <span className="badge warning">Fix-It-Together</span>
          </div>
          <h1 className="page-title">Code Review & Senior QA Mentor</h1>
          <p className="page-description">
            Evaluate test automation code against strict Enterprise SDET AST rules, receive Socratic architectural
            critiques, and patch anti-patterns in real time.
          </p>
        </div>

        <div className="header-actions">
          <button
            className="secondary-btn"
            onClick={() => setViewMode(viewMode === 'Embedded' ? 'Native' : 'Embedded')}
          >
            <span className="btn-icon">📺</span>
            <span>View Mode: {viewMode}</span>
          </button>
          <button className="secondary-btn" onClick={handleRunASTReview} disabled={isAnalyzing}>
            <span className="btn-icon">⚡</span>
            <span>{isAnalyzing ? 'Analyzing AST...' : 'Re-Run AST Review'}</span>
          </button>
          <button
            className={`secondary-btn ${activeDiffMode ? 'active-toggle' : ''}`}
            onClick={() => setActiveDiffMode(!activeDiffMode)}
          >
            <span className="btn-icon">🔄</span>
            <span>{activeDiffMode ? 'Editor View' : 'Diff Preview'}</span>
          </button>
        </div>
      </div>

      {/* Template Selector Bar */}
      <div className="template-selector-card">
        <div className="selector-header">
          <span className="selector-title">📌 Test Anti-Pattern Presets:</span>
          <span className="selector-hint">Select a drill to analyze common SDET anti-patterns</span>
        </div>
        <div className="template-pills-row">
          {CODE_TEMPLATES.map((tmpl) => (
            <button
              key={tmpl.id}
              className={`template-pill ${selectedTemplateId === tmpl.id ? 'active' : ''}`}
              onClick={() => handleSelectTemplate(tmpl.id)}
            >
              <span className="pill-lang">{tmpl.language === 'TypeScript' ? 'TS' : 'PY'}</span>
              <span className="pill-title">{tmpl.title}</span>
              <span className={`pill-diff ${tmpl.difficulty.toLowerCase()}`}>{tmpl.difficulty}</span>
            </button>
          ))}
        </div>
      </div>

      {/* Main Workspace Layout */}
      <div className="review-workspace-grid">
        {/* Left Column: Code Editor / Diff View */}
        <div className="code-editor-column">
          {viewMode === 'Embedded' ? (
            <StreamViewer />
          ) : (
            <div className="native-mode-message" style={{ padding: '20px', backgroundColor: '#f0f4f8', border: '1px solid #cce0ff', borderRadius: '4px', marginBottom: '20px', color: '#0056b3' }}>
              Tests are running in external native windows.
            </div>
          )}
          <div className="editor-card">
            <div className="editor-topbar">
              <div className="file-info">
                <span className="file-icon">📄</span>
                <span className="file-name">{activeTemplate.track}/exercise.ts</span>
                <span className="lang-tag">{activeTemplate.language}</span>
              </div>
              <div className="editor-meta">
                <span className="line-count">{code.split('\n').length} lines</span>
                <button
                  className="reset-code-btn"
                  title="Reset code to starter preset"
                  onClick={() => setCode(activeTemplate.initialCode)}
                >
                  ↺ Reset
                </button>
              </div>
            </div>

            {/* View Mode: Interactive Code Editor vs Side-by-Side Diff */}
            {!activeDiffMode ? (
              <div className="code-editor-wrapper">
                <div className="line-numbers">
                  {code.split('\n').map((_, i) => {
                    const lineNum = i + 1;
                    const lineViolation = violations.find((v) => v.line_number === lineNum);
                    return (
                      <div key={i} className={`line-num ${lineViolation ? 'has-violation' : ''}`}>
                        {lineViolation && <span className="line-marker-icon">⚠️</span>}
                        <span>{lineNum}</span>
                      </div>
                    );
                  })}
                </div>
                <textarea
                  className="code-textarea"
                  value={code}
                  onChange={(e) => setCode(e.target.value)}
                  spellCheck={false}
                  autoComplete="off"
                  aria-label="Code Editor"
                />
              </div>
            ) : (
              <div className="diff-view-wrapper">
                <div className="diff-header">
                  <span className="diff-col-title before">❌ Before (Starter with Anti-Patterns)</span>
                  <span className="diff-col-title after">✅ Current Workspace</span>
                </div>
                <div className="diff-split-grid">
                  <pre className="diff-pane before-pane">
                    <code>{activeTemplate.initialCode}</code>
                  </pre>
                  <pre className="diff-pane after-pane">
                    <code>{code}</code>
                  </pre>
                </div>
              </div>
            )}

            {/* Bottom Status Bar */}
            <div className="editor-bottom-bar">
              <div className="ast-status-indicator">
                <span className={`status-dot ${violations.length === 0 ? 'clean' : 'has-errors'}`}></span>
                <span>
                  {violations.length === 0
                    ? '100% Clean AST — Zero Anti-Patterns Detected'
                    : `${violations.length} AST Violation${violations.length > 1 ? 's' : ''} Identified`}
                </span>
              </div>
              <span className="sentinel-badge">
                {code.includes('I AM NOT DONE') ? '⚠️ Sentinel Active (// I AM NOT DONE)' : '✨ Sentinel Cleared'}
              </span>
            </div>
          </div>
        </div>

        {/* Right Column: Intelligence & Mentor Panel */}
        <div className="intelligence-column">
          {/* Tabs Switcher */}
          <div className="panel-tab-headers">
            <button
              className={`panel-tab-btn ${activeRightTab === 'violations' ? 'active' : ''}`}
              onClick={() => setActiveRightTab('violations')}
            >
              <span className="tab-icon">📊</span>
              <span>AST Score ({score})</span>
              {violations.length > 0 && <span className="tab-counter">{violations.length}</span>}
            </button>
            <button
              className={`panel-tab-btn ${activeRightTab === 'mentor' ? 'active' : ''}`}
              onClick={() => setActiveRightTab('mentor')}
            >
              <span className="tab-icon">🤖</span>
              <span>Senior QA Mentor</span>
            </button>
            <button
              className={`panel-tab-btn ${activeRightTab === 'wizard' ? 'active' : ''}`}
              onClick={() => setActiveRightTab('wizard')}
            >
              <span className="tab-icon">🪄</span>
              <span>Fix-It Wizard</span>
            </button>
          </div>

          <div className="panel-tab-content">
            {/* TAB 1: AST SCORE & VIOLATIONS */}
            {activeRightTab === 'violations' && (
              <div className="violations-tab-view">
                {/* Score Gauge Card */}
                <div className="score-gauge-card">
                  <div className="gauge-flex">
                    <div className="circular-gauge-container">
                      <svg className="gauge-svg" viewBox="0 0 100 100">
                        <circle className="gauge-bg" cx="50" cy="50" r="42" strokeWidth="8" />
                        <circle
                          className={`gauge-bar ${score >= 85 ? 'green' : score >= 50 ? 'amber' : 'red'}`}
                          cx="50"
                          cy="50"
                          r="42"
                          strokeWidth="8"
                          strokeDasharray={264}
                          strokeDashoffset={264 - (264 * score) / 100}
                        />
                      </svg>
                      <div className="gauge-score-value">
                        <span className="num">{score}</span>
                        <span className="denom">/100</span>
                      </div>
                    </div>

                    <div className="gauge-summary">
                      <h3 className="gauge-heading">
                        {score >= 85
                          ? '🎉 SDET Enterprise Grade'
                          : score >= 50
                          ? '⚠️ Refactoring Needed'
                          : '❌ High Flakiness Risk'}
                      </h3>
                      <p className="gauge-subtext">
                        {score >= 85
                          ? 'Code meets production resilience benchmarks against latency, chaos, and DOM refactors.'
                          : 'Static AST rules detected fragile patterns likely to cause intermittent test failures.'}
                      </p>

                      <div className="dimension-mini-bars">
                        <div className="dim-row">
                          <span className="dim-name">Correctness:</span>
                          <div className="dim-track">
                            <div className="dim-fill green" style={{ width: `${scoreCategories.correctness}%` }}></div>
                          </div>
                          <span className="dim-val">{scoreCategories.correctness}%</span>
                        </div>
                        <div className="dim-row">
                          <span className="dim-name">Chaos Stability:</span>
                          <div className="dim-track">
                            <div className="dim-fill amber" style={{ width: `${scoreCategories.flakinessRisk}%` }}></div>
                          </div>
                          <span className="dim-val">{scoreCategories.flakinessRisk}%</span>
                        </div>
                        <div className="dim-row">
                          <span className="dim-name">Locator Resilience:</span>
                          <div className="dim-track">
                            <div className="dim-fill cyan" style={{ width: `${scoreCategories.locatorResilience}%` }}></div>
                          </div>
                          <span className="dim-val">{scoreCategories.locatorResilience}%</span>
                        </div>
                        <div className="dim-row">
                          <span className="dim-name">Async Safety:</span>
                          <div className="dim-track">
                            <div className="dim-fill purple" style={{ width: `${scoreCategories.asyncSafety}%` }}></div>
                          </div>
                          <span className="dim-val">{scoreCategories.asyncSafety}%</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Violations List */}
                <div className="violations-container">
                  <h4 className="violations-heading">
                    <span>Detected Violations</span>
                    <span className="badge info">{violations.length} Items</span>
                  </h4>

                  {violations.length === 0 ? (
                    <div className="clean-code-banner">
                      <span className="clean-icon">🌟</span>
                      <div className="clean-text">
                        <h4>Zero Anti-Patterns Found</h4>
                        <p>Your test code utilizes robust semantic locators, async safety, and web-first auto-waiting.</p>
                      </div>
                    </div>
                  ) : (
                    <div className="violation-cards-list">
                      {violations.map((v) => (
                        <div key={v.id} className={`violation-card ${v.severity}`}>
                          <div className="vcard-header">
                            <div className="vcard-title-group">
                              <span className={`severity-badge ${v.severity}`}>{v.severity.toUpperCase()}</span>
                              <span className="rule-id-text">{v.rule_id}</span>
                            </div>
                            <span className="line-tag">Line {v.line_number}</span>
                          </div>

                          <h5 className="vcard-name">{v.rule_name}</h5>
                          <p className="vcard-desc">{v.message}</p>

                          <div className="vcard-snippet">
                            <code>{v.code_snippet}</code>
                          </div>

                          <div className="vcard-actions">
                            <button
                              className="wizard-fix-btn"
                              onClick={() => {
                                setSelectedViolationForFix(v);
                                setActiveRightTab('wizard');
                              }}
                            >
                              <span>🪄 Fix with Wizard</span>
                            </button>
                            <button className="apply-direct-btn" onClick={() => handleApplyFix(v)}>
                              <span>⚡ 1-Click Fix</span>
                            </button>
                          </div>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* TAB 2: SENIOR QA MENTOR */}
            {activeRightTab === 'mentor' && (
              <div className="mentor-tab-view">
                <div className="mentor-profile-card">
                  <div className="mentor-avatar">
                    <span className="avatar-emoji">👨‍💻</span>
                    <span className="mentor-status-badge">AI Senior SDET</span>
                  </div>
                  <div className="mentor-info">
                    <h3 className="mentor-name">Dr. Cherenkov</h3>
                    <p className="mentor-role">Principal QA Architect & Chaos Engineering Lead</p>
                    <span className="mentor-tagline">"Tests that sleep are tests that fail in production."</span>
                  </div>
                </div>

                <div className="mentor-critique-box">
                  <div className="critique-header">
                    <span className="critique-icon">💬</span>
                    <span className="critique-title">Socratic Architecture Review</span>
                  </div>

                  {violations.length === 0 ? (
                    <div className="mentor-speech clean">
                      <p>
                        "Superb work! Your suite adheres to clean SDET principles. Notice how by eliminating static sleeps and
                        using semantic locator assertions, your test execution is both faster and resilient against network
                        jitter."
                      </p>
                    </div>
                  ) : (
                    <div className="mentor-speech">
                      <p>
                        "I reviewed your test implementation for <strong>{activeTemplate.title}</strong>. I've noted{' '}
                        <strong>{violations.length} critical anti-patterns</strong> that will cause false alarms during
                        high-concurrency CI pipeline runs."
                      </p>
                      <p>
                        "Let's explore the underlying mechanics together. Click below to inspect why these anti-patterns
                        degrade reliability:"
                      </p>
                    </div>
                  )}
                </div>

                {/* Socratic Dialogue Q&A Pills */}
                <div className="socratic-accordion-list">
                  {violations.map((v) => {
                    const isExpanded = expandedSocraticId === v.id;
                    return (
                      <div key={v.id} className={`socratic-card ${isExpanded ? 'expanded' : ''}`}>
                        <button
                          className="socratic-header-btn"
                          onClick={() => setExpandedSocraticId(isExpanded ? null : v.id)}
                        >
                          <span className="socratic-badge">Q</span>
                          <span className="socratic-question">{v.socratic_prompt}</span>
                          <span className="accordion-chevron">{isExpanded ? '▲' : '▼'}</span>
                        </button>

                        {isExpanded && (
                          <div className="socratic-body">
                            <div className="socratic-answer">
                              <span className="mentor-mini-icon">💡</span>
                              <p>{v.socratic_answer}</p>
                            </div>
                            <div className="socratic-footer">
                              <button
                                className="primary-btn wizard-cta"
                                onClick={() => {
                                  setSelectedViolationForFix(v);
                                  setActiveRightTab('wizard');
                                }}
                              >
                                🪄 Launch Fix Wizard for this Violation
                              </button>
                            </div>
                          </div>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            )}

            {/* TAB 3: FIX-IT-TOGETHER WIZARD */}
            {activeRightTab === 'wizard' && (
              <div className="wizard-tab-view">
                <div className="wizard-header-card">
                  <div className="wizard-icon">🪄</div>
                  <div className="wizard-title-group">
                    <h3>Interactive Fix-It-Together Wizard</h3>
                    <p>Select an anti-pattern, preview the surgical patch diff, and apply it with one click.</p>
                  </div>
                </div>

                {violations.length === 0 ? (
                  <div className="wizard-all-clear">
                    <span className="all-clear-icon">🎉</span>
                    <h4>All Violations Resolved!</h4>
                    <p>Your workspace code is 100% compliant with Enterprise SDET standards.</p>
                    <button className="secondary-btn" onClick={() => setActiveRightTab('violations')}>
                      Return to Score Overview
                    </button>
                  </div>
                ) : (
                  <div className="wizard-steps-container">
                    {/* Step 1: Select Violation */}
                    <div className="wizard-step">
                      <span className="step-num">Step 1</span>
                      <span className="step-title">Select Anti-Pattern to Remediate:</span>
                      <div className="wizard-violation-chips">
                        {violations.map((v) => {
                          const isSelected = selectedViolationForFix?.id === v.id;
                          return (
                            <button
                              key={v.id}
                              className={`vchip ${isSelected ? 'active' : ''}`}
                              onClick={() => setSelectedViolationForFix(v)}
                            >
                              <span className={`chip-dot ${v.severity}`}></span>
                              <span className="chip-name">{v.rule_name}</span>
                              <span className="chip-line">L{v.line_number}</span>
                            </button>
                          );
                        })}
                      </div>
                    </div>

                    {/* Step 2: Suggestion Chips & Diff Preview */}
                    {selectedViolationForFix && (
                      <div className="wizard-step">
                        <span className="step-num">Step 2</span>
                        <span className="step-title">Review Remediation Strategy:</span>

                        <div className="strategy-chip">
                          <span className="strategy-label">Recommended Refactor:</span>
                          <span className="strategy-action">{selectedViolationForFix.fix_label}</span>
                        </div>

                        {/* Side-by-Side Diff Preview */}
                        <div className="patch-diff-preview">
                          <div className="diff-header-mini">
                            <span className="diff-title">Unified Code Diff Preview</span>
                            <span className="diff-target">Target: Line {selectedViolationForFix.line_number}</span>
                          </div>
                          <div className="diff-lines">
                            <div className="diff-line diff-del">
                              <span className="diff-sign">-</span>
                              <code>{generateDiffPreview(selectedViolationForFix).originalLine}</code>
                            </div>
                            <div className="diff-line diff-add">
                              <span className="diff-sign">+</span>
                              <code>{generateDiffPreview(selectedViolationForFix).fixedLine}</code>
                            </div>
                          </div>
                        </div>

                        {/* Step 3: Apply Button */}
                        <div className="wizard-actions-row">
                          <button
                            className="primary-btn apply-fix-large-btn"
                            onClick={() => handleApplyFix(selectedViolationForFix)}
                          >
                            <span className="btn-icon">⚡</span>
                            <span>Apply Fix to Code Editor</span>
                          </button>
                        </div>
                      </div>
                    )}

                    {!selectedViolationForFix && violations.length > 0 && (
                      <div className="wizard-prompt-select">
                        <span>👈 Please click on one of the anti-pattern chips above to preview its fix.</span>
                      </div>
                    )}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
};
