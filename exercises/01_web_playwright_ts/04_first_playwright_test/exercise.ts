/**
 * PRODUCTION STORY:
 * Basecamp Architecture Overhaul (2021)
 * Teams moving from legacy Selenium suites had dozens of test cases that navigated to pages
 * without asserting final DOM states, silently passing in CI even when pages rendered 500 error banners.
 */

import { test, expect } from "@playwright/test";

// Drill 04: Your First Playwright Test
// You are a Manual QA Engineer. Your job: verify that the homepage loads.
// Right now this test navigates to the page but never checks anything.
// It will always pass -- even if the page is blank!
// TODO: Add an expect() call to verify the page title or a key element is visible.

test("homepage loads successfully", async ({ page }) => {
  await page.goto("http://localhost:8080");
  // Right now we go to the page but never check anything.
  // Add: await expect(page).toHaveTitle(/Cherenkov|SDET|checkout/i);
  // Or:  await expect(page.getByRole("main")).toBeVisible();
});
