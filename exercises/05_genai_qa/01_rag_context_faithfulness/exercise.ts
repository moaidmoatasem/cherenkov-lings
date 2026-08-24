/**
 * PRODUCTION STORY:
 * Air Canada Chatbot False Refund Advice (2022)
 * A generative customer service chatbot hallucinated a bereavement discount policy not present in the
 * company's official knowledge base. Exact-match string assertions in QA failed to detect semantic unfaithfulness.
 */

import { test, expect } from '@playwright/test';

test('RAG response is faithful to the source document', async ({ request }) => {
  const response = await request.get('http://localhost:8081/api/rag?query=Cherenkov');
  expect(response.status()).toBe(200);
  const body = await response.json();

  // Anti-pattern: Exact string match on the full LLM answer
  // TODO: Replace rigid exact-match with key-fact presence checks and grounding assertions
  expect(body.answer).toBe(
    'Based on the Cherenkov Radiation Primer: Cherenkov radiation occurs when a charged particle moves faster than light in a medium. The radiation was discovered by Pavel Cherenkov in 1934.'
  );
});  // This fails on any rephrasing, punctuation change, or fact reordering
