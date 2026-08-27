import { test, expect } from "@playwright/test";

// Drill 05: Solution -- Semantic Locators Survive Refactoring
// getByRole queries the accessibility tree, not the DOM structure.
// It survives class renames, CSS refactors, and component rewrites.
// It also tests accessibility as a side-effect.

test("checkout button is clickable", async ({ page }) => {
  await page.goto("/checkout");

  // Semantic locator: finds the button by its accessible role and name
  // Survives any CSS or DOM structure changes
  const btn = page.getByRole("button", { name: /Pay Now/i });

  await expect(btn).toBeVisible();
});
