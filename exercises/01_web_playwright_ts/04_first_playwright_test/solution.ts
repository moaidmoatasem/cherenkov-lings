import { test, expect } from "@playwright/test";

// Drill 04: Solution -- Your First Playwright Test
// We navigate AND verify. A test that only navigates is not a test.

test("homepage loads and displays main content", async ({ page }) => {
  // ACT: go to the app
  await page.goto("http://localhost:8080");

  // ASSERT: verify the page actually rendered something meaningful
  await expect(page).toHaveURL(/localhost:8080/);
  await expect(page.getByRole("main")).toBeVisible();
});
