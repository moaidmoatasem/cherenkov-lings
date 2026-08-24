# Hints: Drill 01 - Hydration Timing

## Hint 1 (Architectural Nudge)
Modern frontends (React 19, Next.js, Remix) stream HTML to the browser before JavaScript bundles finish downloading and attaching event listeners (hydration). Clicking elements during this hydration gap silently drops user events. Hardcoded timeouts (`page.waitForTimeout`) fail predictably under CI load, CPU throttling, or network jitter.

## Hint 2 (API Pattern)
Instead of arbitrary sleep durations, observe how the application signals readiness. Look for the `data-hydrated` attribute on the checkout button. Use Playwright's auto-retrying assertion:
```typescript
await expect(locator).toHaveAttribute('data-hydrated', 'true');
```

## Hint 3 (Code Diff)
```diff
- await page.waitForTimeout(200);
- await page.locator('#checkout-btn').click();
+ const checkoutBtn = page.getByRole('button', { name: /Pay Now/i });
+ await expect(checkoutBtn).toHaveAttribute('data-hydrated', 'true');
+ await checkoutBtn.click();
```
