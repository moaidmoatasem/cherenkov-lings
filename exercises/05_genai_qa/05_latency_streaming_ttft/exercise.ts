/**
 * PRODUCTION STORY:
 * ChatGPT March 2023 Streaming Disruption
 * When OpenAI rolled out token streaming, client buffering caused perceived UI freezes
 * where Time-To-First-Token exceeded 15 seconds despite total generation time being nominal.
 */
import { test, expect } from '@playwright/test';

// Anti-pattern: Waiting for the full stream to complete before evaluating latency
// TODO: Measure Time-To-First-Token (TTFT) and token inter-arrival jitter
test('measures full response instead of streaming token latency', async ({ request }) => {
  const start = Date.now();
  const res = await request.get('http://localhost:8081/api/llm/stream?prompt=test');
  const body = await res.text();
  const duration = Date.now() - start;

  // Brittle check: Measures total stream time rather than initial token arrival (TTFT)
  expect(body.length).toBeGreaterThan(0);
});
