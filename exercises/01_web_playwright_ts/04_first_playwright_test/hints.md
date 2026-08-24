# Hints: Drill 04 - Your First Playwright Test

## Hint 1 (Concept)
page.goto() is the ACT. But without an expect(), there is no ASSERT. A Playwright test that only navigates will pass even if the server returns a blank page or a 500 error.

## Hint 2 (API Pattern)
Playwright assertions use expect(). Two useful ones for checking a page loaded:
  await expect(page).toHaveURL(/localhost:8080/);
  await expect(page.getByRole("main")).toBeVisible();

## Hint 3 (Code Diff)
Add after page.goto():
  await expect(page).toHaveURL(/localhost:8080/);
  await expect(page.getByRole("main")).toBeVisible();
