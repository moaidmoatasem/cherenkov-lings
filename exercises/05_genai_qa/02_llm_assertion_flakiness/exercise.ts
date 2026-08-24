/**
 * PRODUCTION STORY:
 * Google Gemini Customer Intent Drift (2023)
 * Non-deterministic phrasing variations across temperature-enabled LLM endpoints broke 75% of CI builds
 * when test suites asserted on exact raw response strings rather than structured semantic intent and entity fields.
 */

import { test, expect } from '@playwright/test';

test('LLM response describes transfer status correctly', async ({ request }) => {
  const response = await request.get('http://localhost:8081/api/llm?prompt=What+is+the+status+of+my+transfer');
  expect(response.status()).toBe(200);
  const body = await response.json();

  // Anti-pattern: Asserting on the raw_text which varies slightly on every call
  // TODO: Replace raw_text assertion with structured intent/entity field checks
  expect(body.raw_text).toBe('The transfer was successfully initiated and is pending ledger settlement.');
});  // Fails ~75% of the time as raw_text cycles through 4 different phrasings
