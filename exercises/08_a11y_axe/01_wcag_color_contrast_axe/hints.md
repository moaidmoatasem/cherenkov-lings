## Hint 1 (Architectural Nudge)
WCAG 2.1 AA requires all interactive controls to expose a compute-accessible name and role in the browser accessibility tree.

## Hint 2 (API Pattern)
Use `page.getByRole('button', { name: '...' })` rather than CSS `#checkout-btn`.

## Hint 3 (Code Diff)
```diff
- const btn = page.locator('#checkout-btn');
- await expect(btn).toBeAttached();
+ const checkoutBtn = page.getByRole('button', { name: 'Confirm Purchase' });
+ await expect(checkoutBtn).toBeVisible();
```
