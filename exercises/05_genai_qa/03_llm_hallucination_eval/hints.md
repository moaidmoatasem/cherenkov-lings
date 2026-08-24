## Hint 1 (Architectural Nudge)
Testing LLM RAG pipelines requires measuring *Faithfulness* — ensuring every claim in the response can be attributed to the retrieved source chunks.

## Hint 2 (API Pattern)
Assert on the `grounded` boolean, citation metadata in `sources`, and forbidden hallucination terms.

## Hint 3 (Code Diff)
```diff
- expect(data.answer.length).toBeGreaterThan(10);
+ expect(data.grounded).toBe(true);
+ expect(data.sources).toContain('policy_v2.pdf');
```
