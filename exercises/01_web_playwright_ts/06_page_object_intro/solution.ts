import { test, expect, type Page, type Locator } from '@playwright/test';

/**
 * SDET Resilient Pattern: Page Object Model (POM)
 * Encapsulates UI selectors, interactions, and business assertions in a dedicated class.
 * When DOM changes occur, updates are isolated to a single class rather than hundreds of tests.
 */
export class CheckoutPage {
  readonly page: Page;
  readonly itemIdInput: Locator;
  readonly quantityInput: Locator;
  readonly shippingSelect: Locator;
  readonly addressInput: Locator;
  readonly payButton: Locator;
  readonly orderStatus: Locator;

  constructor(page: Page) {
    this.page = page;
    this.itemIdInput = page.getByLabel('Item ID').or(page.locator('#item-id'));
    this.quantityInput = page.getByLabel('Quantity').or(page.locator('#quantity'));
    this.shippingSelect = page.getByLabel('Shipping').or(page.locator('#shipping-type'));
    this.addressInput = page.getByLabel('Address').or(page.locator('#address'));
    this.payButton = page.getByRole('button', { name: /Pay|Complete Order/i }).or(page.locator('#pay-btn'));
    this.orderStatus = page.getByTestId('order-status').or(page.locator('#order-status'));
  }

  async goto() {
    await this.page.goto('/checkout');
  }

  async fillOrder(itemId: string, quantity: number) {
    await this.itemIdInput.fill(itemId);
    await this.quantityInput.fill(quantity.toString());
  }

  async setShipping(shippingType: string, address: string) {
    await this.shippingSelect.selectOption(shippingType);
    await this.addressInput.fill(address);
  }

  async submitPayment() {
    await this.payButton.click();
  }

  async expectOrderConfirmed() {
    await expect(this.orderStatus).toHaveText('Order Confirmed', { timeout: 5000 });
  }
}

test('complete checkout flow with Page Object Model (RESILIENT)', async ({ page }) => {
  const checkoutPage = new CheckoutPage(page);

  await checkoutPage.goto();
  await checkoutPage.fillOrder('item-1', 2);
  await checkoutPage.setShipping('express', '742 Evergreen Terrace');
  await checkoutPage.submitPayment();
  await checkoutPage.expectOrderConfirmed();
});
