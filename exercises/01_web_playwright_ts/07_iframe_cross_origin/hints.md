# Hints: Drill 07 - Iframe Cross-Origin Handling

## Hint 1 (Architectural Nudge)
Browsers isolate iframes into distinct browsing contexts with independent DOM trees and separate origins. Top-level `page.locator()` queries will never find elements rendered within an iframe.

## Hint 2 (API Pattern)
Use `page.frameLocator(selector)` to obtain a FrameLocator reference, then chain child locators from it:
```typescript
const frame = page.frameLocator('iframe#stripe-frame');
await frame.getByLabel('Card Number').fill('4242424242424242');
```

## Hint 3 (Code Diff)
```diff
- const cardInput = page.locator('#card-number');
- await cardInput.fill('4242424242424242');
+ const frame = page.frameLocator('iframe#stripe-frame');
+ const cardInput = frame.locator('#card-number');
+ await cardInput.fill('4242424242424242');
```
