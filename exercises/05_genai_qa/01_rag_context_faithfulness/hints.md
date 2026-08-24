# Hints: Drill 01 - RAG Context Faithfulness

## Hint 1 (Architectural Nudge)
LLM outputs are non-deterministic by nature. Even a 'deterministic' mock endpoint may change phrasing, reorder sentences, or vary punctuation. An exact-equality assertion on the raw answer string is the most brittle test you can write for a generative system. The real question is: does the answer stay grounded in the source document?

## Hint 2 (API Pattern)
The /api/rag endpoint returns structured fields designed for grounding checks:
- grounded: boolean -- was the response derived from the source doc?
- source_facts: string[] -- which specific facts were cited?
- document_title: string -- which document was used?
Assert on these fields, not on body.answer directly.

## Hint 3 (Code Diff)
Replace: expect(body.answer).toBe('...')
With:
  expect(body.grounded).toBe(true);
  expect(body.source_facts.length).toBeGreaterThan(0);
  const facts = body.source_facts.join(' ').toLowerCase();
  expect(facts).toContain('cherenkov');
  expect(body.answer.toLowerCase()).toContain('cherenkov');
