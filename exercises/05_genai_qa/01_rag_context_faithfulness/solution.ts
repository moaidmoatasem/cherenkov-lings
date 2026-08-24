import { test, expect } from '@playwright/test';

test('RAG response is faithful to the source document', async ({ request }) => {
  const response = await request.get('http://localhost:8081/api/rag?query=Cherenkov');
  expect(response.status()).toBe(200);
  const body = await response.json();

  // Assert structural grounding fields -- not the raw LLM text
  expect(body.grounded).toBe(true);
  expect(body.document_title).toBe('Cherenkov Radiation Primer');

  // Assert that key facts appear in the source_facts list (faithfulness check)
  const facts: string[] = body.source_facts;
  expect(facts.length).toBeGreaterThan(0);
  const combinedFacts = facts.join(' ').toLowerCase();
  expect(combinedFacts).toContain('cherenkov');
  expect(combinedFacts).toContain('particle');

  // Soft-assert on answer content: key facts must be present, exact phrasing is irrelevant
  expect(body.answer.toLowerCase()).toContain('cherenkov');
});
