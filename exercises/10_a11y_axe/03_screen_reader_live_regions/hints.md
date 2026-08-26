## Hint 1 (Architectural Nudge)
Dynamic content changes must be announced to assistive technologies using `role="status"` or `aria-live="polite"`.

## Hint 2 (API Pattern)
Use `page.getByRole('status')` to locate accessible live regions.

## Hint 3 (Code Diff)
```diff
- const status = page.locator('#transfer-status');
+ const statusBox = page.getByRole('status');
+ await expect(statusBox).toBeVisible();
```
