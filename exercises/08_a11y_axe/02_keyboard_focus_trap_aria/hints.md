## Hint 1 (Architectural Nudge)
Keyboard navigation is essential for accessibility. Users must be able to Tab through all interactive elements in logical DOM order.

## Hint 2 (API Pattern)
Use `page.keyboard.press('Tab')` and `await expect(locator).toBeFocused()`.

## Hint 3 (Code Diff)
```diff
+ await recipientInput.focus();
+ await page.keyboard.press('Tab');
+ await expect(amountInput).toBeFocused();
```
