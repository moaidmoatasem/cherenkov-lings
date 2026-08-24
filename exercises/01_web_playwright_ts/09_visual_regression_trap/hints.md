# Hints: Drill 09 - Visual Regression Testing

## Hint 1 (Architectural Nudge)
Pixel-by-pixel visual snapshot comparison is fragile when rendered against non-deterministic dynamic data such as timestamps, user session IDs, or CSS animation phases.

## Hint 2 (API Pattern)
Use Playwright's `mask` array option in `toHaveScreenshot()` to overlay neutral pink bounding boxes over volatile elements, and specify `maxDiffPixelRatio`.

## Hint 3 (Code Diff)
```diff
- await expect(page).toHaveScreenshot('dashboard-baseline.png');
+ await expect(page).toHaveScreenshot('dashboard-baseline.png', {
+   maxDiffPixelRatio: 0.05,
+   mask: [page.getByTestId('live-clock'), page.getByTestId('session-id')],
+   animations: 'disabled',
+ });
```
