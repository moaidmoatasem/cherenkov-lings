import { test, expect } from "@playwright/test";

// Drill 05: Locator Hierarchy -- Why Your Selectors Keep Breaking
// This test uses a fragile CSS path that will break the moment a developer
// renames a class, wraps the button in a div, or refactors the component.
// TODO: Replace the fragile CSS selector with a semantic getByRole locator.

test("checkout button is clickable", async ({ page }) => {
  await page.goto("http://localhost:8080/checkout");

  // Anti-pattern: tightly coupled to CSS implementation detail
  // Breaks when: class renamed, element moved, CSS-in-JS changes hash
  const btn = page.locator("div.checkout-container > button.btn-primary-submit");

  await expect(btn).toBeVisible();
});
