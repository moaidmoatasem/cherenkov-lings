import { test, expect } from '@playwright/test';

test('chatbot answer is strictly grounded in ground truth policy documents', async ({ request }) => {
  const res = await request.get('http://localhost:8081/api/rag?query=bereavement');
  expect(res.ok()).toBeTruthy();
  const data = await res.json();

  // Verify citation grounding and ensure forbidden hallucination terms are absent
  expect(data).toHaveProperty('grounded', true);
  expect(data.document_title).toContain('Cherenkov Radiation Primer');
  expect(data.source_facts.length).toBeGreaterThan(0);
  expect(data.answer.toLowerCase()).not.toContain('free unlimited retroactive refund');
});
