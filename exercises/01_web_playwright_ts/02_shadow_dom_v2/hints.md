# Hints: Drill 02 - Shadow DOM v2

## Hint 1 (Architectural Nudge)
Absolute XPaths (`xpath=/html/body/...`) are brittle and tightly coupled to the exact DOM hierarchy. Furthermore, standard XPath specifications and browser DOM traversal engines cannot cross closed Shadow DOM boundaries into encapsulated custom elements.

## Hint 2 (API Pattern)
Playwright's locator engine seamlessly pierces Shadow DOM boundaries by default when using CSS locators, `getByRole`, `getByTestId`, or scoped chained locators (`host.locator(...)`).

## Hint 3 (Code Diff)
```diff
- const secretElement = page.locator('xpath=/html/body/div/div/div/chaos-vault/div/span[2]');
- await expect(secretElement).toHaveText('CHERENKOV_SECRET_9876');
- const unlockBtn = page.locator('xpath=/html/body/div/div/div/chaos-vault/div/button');
- await unlockBtn.click();
- const statusElement = page.locator('xpath=/html/body/div/div/div/chaos-vault/div/span[3]');
- await expect(statusElement).toHaveText('Unlocked');
+ const vault = page.locator('chaos-vault');
+ await expect(vault.locator('[data-testid="vault-secret"]')).toHaveText('CHERENKOV_SECRET_9876');
+ await vault.getByRole('button', { name: 'Unlock' }).click();
+ await expect(vault.locator('[data-testid="vault-status"]')).toHaveText('Unlocked');
```
