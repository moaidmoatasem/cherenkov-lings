# ⚡ Cherenkov-lings Quick Reference & Cheat Sheet

> **The zero-BS, tactical guide to mastering modern test automation with cherenkov-lings.**

---

## 🛠️ Essential CLI Commands

```bash
# 1. Start learning on any track
cherenkov-lings watch --track=getting-started
cherenkov-lings watch --track=foundations
cherenkov-lings watch --track=api-pytest
cherenkov-lings watch --track=playwright-ts
cherenkov-lings watch --track=restassured-java
cherenkov-lings watch --track=maestro-mobile
cherenkov-lings watch --track=k6-js
cherenkov-lings watch --track=jmeter
cherenkov-lings watch --track=genai-qa
cherenkov-lings watch --track=devsecops-python
cherenkov-lings watch --track=tool-decisions
cherenkov-lings watch --track=ci-pipeline
cherenkov-lings watch --track=contract-pact
cherenkov-lings watch --track=a11y-axe

# 2. View your rank, XP, badges, and streak
cherenkov-lings dashboard

# 3. Verify curriculum integrity (100% health check)
cherenkov-lings audit

# 4. Diagnose anti-patterns in any exercise file (static source analysis)
cherenkov-lings diagnose --file=exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts

# 5. Scaffold a brand-new drill
cherenkov-lings new-drill --track=playwright-ts --name="11_websockets" --title="WebSocket Reconnection"

# 6. Start the standalone Layer 4/7 Chaos Proxy
cherenkov-lings proxy --port=8086 --upstream="127.0.0.1:8081" --latency=500 --drop-rate=0.1

# 7. Start the MCP Server for Cursor / VS Code Copilot
cherenkov-lings mcp

# 8. Run the AI Senior QA code review engine on a file
cherenkov-lings review --file=exercises/01_web_playwright_ts/01_hydration_timing/exercise.ts

# 9. Validate or simulate-run a GitHub Actions workflow
cherenkov-lings pipeline run .github/workflows/ci.yml

# 10. Launch the interactive Allure chaos triage challenge
cherenkov-lings triage

# 11. Generate an Allure-compatible chaos test report
cherenkov-lings report --output-dir=target/allure-report
```

---

## 📊 The 4D Feedback Matrix Formula

$$\text{Total Score} = (0.35 \times \text{Correctness}) + (0.35 \times \text{Flakiness}) + (0.15 \times \text{LocatorQuality}) + (0.15 \times \text{Speed})$$

| Dimension | Weight | Target | How it is evaluated |
|---|:---:|:---:|---|
| **Correctness** | 35% | 100/100 | Test passes standard assertions under clean conditions |
| **Flakiness Guard** | 35% | 100/100 | Test executes **5 consecutive times** under injected chaos (`X-Chaos: delay=200ms;jitter=75ms`). Penalized if `waitForTimeout` or `Thread.sleep` is detected |
| **Locator Quality** | 15% | 100/100 | Static source analysis of element selectors (see scoring below) |
| **Speed Benchmark**| 15% | 100/100 | Wall-clock execution time vs. pre-calibrated baseline |

**Passing Threshold**: `Total Score >= 85.0` to complete drill and earn XP.

---

## 🎯 Locator Quality Scoring Table

Scoring is a straight per-locator classification, not a running deduction — this table mirrors `LocatorKind::score()` in `src/feedback.rs` exactly.

| Selector Pattern | Score | Classification | Why |
|---|:---:|:---:|---|
| `page.getByRole('button', { name: '...' })` | **100** | Semantic Role | Mirrors real user interaction; resilient to DOM restructuring |
| `page.getByText(...)` / `getByLabel(...)` / `getByPlaceholder(...)` / `getByAltText(...)` / `getByTitle(...)` | **90** | Text / Label | User-visible content; bound to accessibility semantics, not markup |
| `page.getByTestId('...')` | **85** | Explicit Test ID | Dedicated automation hook, but not user-facing |
| `page.locator('.btn-primary')` | **40** | CSS Class / Element | Highly brittle across styling refactors |
| `page.locator('//div[2]/table/tr[3]/td[1]')`| **0** | Absolute XPath | Extremely brittle; breaks on any DOM layout shift |

---

## 🚫 Common Anti-Patterns vs. ✅ Resilient Fixes

### 1. Web UI (Playwright TS)
```typescript
// ❌ ANTI-PATTERN: Brittle arbitrary sleep
await page.waitForTimeout(2000);
await page.click('.checkout-btn');

// ✅ RESILIENT: Web-first assertions & auto-waiting semantic locators
const checkoutBtn = page.getByRole('button', { name: 'Confirm Purchase' });
await expect(checkoutBtn).toBeVisible({ timeout: 5000 });
await checkoutBtn.click();
```

### 2. API Testing (REST Assured Java)
```java
// ❌ ANTI-PATTERN: Naive single-call assertion on async ledger
given().get("/balance").then().body("balance", equalTo(750.0f));

// ✅ RESILIENT: Eventual consistency polling loop
await().atMost(5, SECONDS).pollInterval(200, MILLISECONDS).untilAsserted(() -> {
    given().get("/balance").then().body("balance", equalTo(750.0f));
});
```

### 3. High-Concurrency Performance (k6 JS)
```javascript
// ❌ ANTI-PATTERN: Static VU burst causing DB connection pool starvation
export const options = { vus: 100, duration: '10s' };

// ✅ RESILIENT: Gradual staged ramp-up with trend metrics and p99 threshold
export const options = {
  stages: [
    { duration: '30s', target: 20 },
    { duration: '1m', target: 50 },
    { duration: '20s', target: 0 },
  ],
  thresholds: {
    'http_req_duration{status:200}': ['p(99)<500'],
  },
};
```

---

## ☢️ Micro-Crucible Chaos Injection Headers

Send these headers against `http://localhost:8081` (or test them live in `http://localhost:8080/mission-control`):

| Header | Example Value | Injected Failure Mode |
|---|---|---|
| `X-Chaos` | `delay=500ms` | Injects artificial server response latency |
| `X-Chaos` | `delay=1000ms;jitter=300ms` | Simulates high-jitter wireless/cellular packet spikes |
| `X-Chaos` | `kafka_lag=1500` | Simulates 1500ms asynchronous ledger processing delay |
| `X-Chaos` | `token_expire=immediate` | Simulates mid-session JWT token invalidation (401) |
| `X-Chaos` | `drop_partial=true` | Simulates dropped connection during multipart file upload |
| `X-Chaos` | `drop_after=3` | Drops Server-Sent Events (SSE) stream after 3 events |
| `X-Chaos` | `stale_dom=true` | Forces frontend DOM nodes to remount mid-interaction |

---

## 🎮 Levels & XP Progression

| Rank / Title | XP Required | Tier Multiplier |
|---|:---:|:---:|
| 🌱 **Trainee** | 0 XP | 1.0x |
| 🔍 **Junior QA** | 500 XP | 1.0x |
| ⚡ **Mid QA** | 1,500 XP | 1.0x |
| 🔥 **Senior QA** | 3,000 XP | 1.5x |
| 🎯 **Lead QA** | 6,000 XP | 1.5x |
| 🏗️ **QA Architect** | 10,000 XP | 2.0x |
| ⚛️ **SDET Master** | 20,000 XP | 2.0x |

**XP Formula**: `Earned XP = 100 * (Score / 100) * Tier Multiplier`
