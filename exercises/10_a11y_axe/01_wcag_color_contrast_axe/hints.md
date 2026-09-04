## Hint 1 (Architectural Nudge)
WCAG 2.1 AA requires interactive controls to expose a computed accessible name and role in the browser accessibility tree, and to meet a minimum 4.5:1 color-contrast ratio (Success Criterion 1.4.3). Neither is something `toBeAttached()` can verify — you need a real accessibility engine to compute contrast from rendered styles.

## Hint 2 (API Pattern)
Use `page.getByRole('button', { name: '...' })` for name/role. For contrast, run `new AxeBuilder({ page }).withRules(['color-contrast']).analyze()` and assert `results.violations` is an empty array.

## Hint 3 (Code Diff)
```diff
- const btn = page.locator('#checkout-btn');
- await expect(btn).toBeAttached();
+ const checkoutBtn = page.getByRole('button', { name: 'Confirm Purchase' });
+ await expect(checkoutBtn).toBeVisible();
+
+ const results = await new AxeBuilder({ page })
+   .include('#confirm-purchase')
+   .withRules(['color-contrast'])
+   .analyze();
+ expect(results.violations).toEqual([]);
```
