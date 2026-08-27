# Hints: Drill 08 - Network Intercept & API Mocking

## Hint 1 (Architectural Nudge)
When tests depend directly on live backend services with variable latency, end-to-end test pipelines become slow and flaky. Mocking at the network layer with Playwright intercepts HTTP traffic before it leaves the browser.

## Hint 2 (API Pattern)
Use `page.route(urlPattern, handler)` to intercept incoming network calls:
```typescript
await page.route('**/products*', async (route) => {
  await route.fulfill({
    status: 200,
    contentType: 'application/json',
    body: JSON.stringify({ products: [...] }),
  });
});
```

## Hint 3 (Code Diff)
```diff
+ await page.route('**/products*', async (route) => {
+   await route.fulfill({
+     status: 200,
+     contentType: 'application/json',
+     body: JSON.stringify({ total: 3, products: [...] }),
+   });
+ });
  await page.goto('/products');
```
