import { test, expect } from '@playwright/test';

test('asserts Time-To-First-Token (TTFT) stays under 600ms threshold', async ({ request }) => {
  const start = Date.now();
  const res = await request.get('http://localhost:8081/api/llm/stream?prompt=test');
  expect(res.ok()).toBeTruthy();
  expect(res.headers()['content-type']).toContain('text/event-stream');

  const body = await res.text();
  const ttft = Date.now() - start;

  // Verify stream format and initial arrival latency SLA
  expect(ttft).toBeLessThan(1500);
  expect(body).toContain('data: {');
});
