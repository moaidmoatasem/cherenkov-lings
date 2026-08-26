# Hints: Drill 02 - LLM Assertion Flakiness

## Hint 1 (Architectural Nudge)
LLM outputs vary even for identical inputs. Vendor API temperature settings, system prompt updates, and context window changes all produce different phrasings. A test that asserts exact string equality on LLM output will flake on every model update and will create noise that erodes trust in your test suite.

## Hint 2 (API Pattern)
Well-designed LLM APIs return structured metadata alongside raw text: intent classification, entity extraction, confidence scores. These fields are stable across rephrasing and are the correct assertion targets. The /api/llm endpoint exposes: intent, entities (action, status, domain), and confidence.

## Hint 3 (Code Diff)
Replace: expect(body.raw_text).toBe('The transfer was...')
With:
  expect(body.intent).toBe('transfer_status_inquiry');
  expect(body.entities.action).toBe('transfer');
  expect(body.entities.status).toBe('pending');
  expect(body.confidence).toBeGreaterThan(0.8);
  expect(body.raw_text.toLowerCase()).toContain('transfer');
