## Hint 1 (Architectural Nudge)
Keyboard navigation is essential for accessibility. axe-core can catch static markup that would break Tab order — most notably a positive `tabindex` — but it never actually presses a key, so the real sequence still needs a behavioral check.

## Hint 2 (API Pattern)
Run `new AxeBuilder({ page }).withRules(['tabindex']).analyze()` for the static check. Then use `page.keyboard.press('Tab')` and `await expect(locator).toBeFocused()` to verify the real sequence.

## Hint 3 (Code Diff)
```diff
+ const axeResults = await new AxeBuilder({ page }).withRules(['tabindex']).analyze();
+ expect(axeResults.violations).toEqual([]);
+
+ await recipientInput.focus();
+ await page.keyboard.press('Tab');
+ await expect(amountInput).toBeFocused();
```
