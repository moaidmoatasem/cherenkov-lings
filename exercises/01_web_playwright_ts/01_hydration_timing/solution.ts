import { test, expect } from '@playwright/test';

test('checkout completes successfully despite hydration delay', async ({ page }) => {
  await page.goto('/checkout');

  const checkoutBtn = page.getByRole('button', { name: /Pay Now/i });

  // Web-first assertion: Wait for React hydration marker before interaction
  await expect(checkoutBtn).toHaveAttribute('data-hydrated', 'true');

  // Interactive click once listeners are safely attached
  await checkoutBtn.click();

  // Verify order confirmation
  await expect(page.getByTestId('order-status')).toHaveText('Order Confirmed');
});
