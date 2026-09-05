## Hint 1 (Architectural Nudge)
Dynamic content changes must be announced to assistive technologies using `role="status"` or `aria-live="polite"`. axe-core cannot detect a *missing* live region, but it does validate that whatever ARIA markup is present is well-formed.

## Hint 2 (API Pattern)
Use `page.getByRole('status')` to locate the accessible live region, then run `new AxeBuilder({ page }).include(...).analyze()` scoped to that region and assert `results.violations` is empty.

## Hint 3 (Code Diff)
```diff
- const status = page.locator('#transfer-status');
- await expect(status).toBeAttached();
+ const statusBox = page.getByRole('status');
+ await expect(statusBox).toBeVisible();
+
+ const results = await new AxeBuilder({ page }).include('.transfer-status-box').analyze();
+ expect(results.violations).toEqual([]);
```
