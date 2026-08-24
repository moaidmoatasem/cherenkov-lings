/**
 * PRODUCTION STORY:
 * Air Canada Chatbot Pricing Hallucination (2022)
 * An airline chatbot hallucinated a bereavement discount policy that did not exist.
 * The tribunal held the airline liable for its chatbot's ungrounded outputs.
 */
import { test, expect } from '@playwright/test';

// Anti-pattern: Naive string inclusion without hallucination tolerance bounds
// TODO: Implement semantic G-Eval fact-checking against ground truth context
test('chatbot answer does not hallucinate non-existent discount policies', async ({ request }) => {
  const res = await request.get('http://localhost:8081/api/rag?query=bereavement');
  const data = await res.json();
  
  // Brittle check: Assumes arbitrary output length instead of grounded citation matching
  expect(data.answer.length).toBeGreaterThan(10);
});
