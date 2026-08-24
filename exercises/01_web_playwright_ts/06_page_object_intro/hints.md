# Hints: Drill 06 - Page Object Model

## Hint 1 (Architectural Nudge)
Directly querying DOM selectors (`page.locator('div > input#id')`) inside your test steps couples your tests to markup details. When developers rename or restructure DOM elements, dozens of tests fail. The Page Object Model encapsulates UI structure and business actions into reusable classes.

## Hint 2 (API Pattern)
Create a `CheckoutPage` class that receives the Playwright `Page` in its constructor:
```typescript
class CheckoutPage {
  constructor(private page: Page) {}
  async submitPayment() {
    await this.page.getByRole('button', { name: 'Pay' }).click();
  }
}
```

## Hint 3 (Code Diff)
```diff
- await page.locator('div.checkout-container input#item-id').fill('item-1');
- await page.locator('div.checkout-container button#pay-btn').click();
+ const checkoutPage = new CheckoutPage(page);
+ await checkoutPage.goto();
+ await checkoutPage.fillOrder('item-1', 2);
+ await checkoutPage.submitPayment();
```
