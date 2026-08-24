# Hints: Drill 03 - Debounce Race Condition

## Hint 1 (Architectural Nudge)
Debounced autocomplete fields fire multiple asynchronous network requests as characters are typed. In real-world networks, an earlier slow response can resolve AFTER a newer fast response, clobbering the search state if not synchronized. Fixed sleeps cannot prevent this race condition.

## Hint 2 (API Pattern)
Synchronize on a deterministic state marker provided by the UI (e.g., `data-testid="active-query"` matching the target search term), ensuring assertions only evaluate against the intended query state:
```typescript
await expect(page.getByTestId('active-query')).toHaveText('playwright');
```

## Hint 3 (Code Diff)
```diff
- await page.waitForTimeout(600);
- const firstResult = page.locator('[data-testid="result-item"]').first();
- await expect(firstResult).toHaveText('Playwright', { timeout: 1000 });
+ await expect(page.getByTestId('active-query')).toHaveText('playwright');
+ const results = page.getByTestId('search-results');
+ await expect(results.getByTestId('result-item').first()).toContainText('Playwright');
```
