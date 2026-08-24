# Hints: Drill 09 - Visual Regression Testing

## Hint 1 (Architectural Nudge)
Pixel-by-pixel visual snapshot comparison is fragile when rendered against non-deterministic dynamic data such as timestamps, user session IDs, or CSS animation phases.

## Hint 2 (API Pattern)
Use Playwright's `mask` array option in `toHaveScreenshot()` to overlay neutral pink bounding boxes over volatile elements, and specify `maxDiffPixelRatio`.

> **Note:** The very first execution intentionally fails with "A snapshot doesn't exist ... writing actual." — that run generates your local baseline. Run the solution a second time to perform the real masked comparison.

## Hint 3 (Code Diff)
```diff
- await expect(page).toHaveScreenshot('dashboard-baseline.png');
+ await expect(page).toHaveScreenshot('dashboard-baseline.png', {
+   maxDiffPixelRatio: 0.05,
+   mask: [page.getByTestId('live-clock'), page.getByTestId('session-id')],
+   animations: 'disabled',
+ });
```
