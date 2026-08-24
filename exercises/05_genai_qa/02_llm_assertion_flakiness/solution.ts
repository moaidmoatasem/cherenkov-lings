import { test, expect } from '@playwright/test';

test('LLM response describes transfer status correctly', async ({ request }) => {
  const response = await request.get('http://localhost:8081/api/llm?prompt=What+is+the+status+of+my+transfer');
  expect(response.status()).toBe(200);
  const body = await response.json();

  // Assert on STABLE structured fields, not the variable raw_text
  expect(body.intent).toBe('transfer_status_inquiry');
  expect(body.entities.action).toBe('transfer');
  expect(body.entities.status).toBe('pending');
  expect(body.entities.domain).toBe('ledger');
  expect(body.confidence).toBeGreaterThan(0.8);

  // Optional: verify raw_text contains key concepts without exact-matching
  expect(body.raw_text.toLowerCase()).toContain('transfer');
});
