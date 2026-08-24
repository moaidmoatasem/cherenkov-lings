# Hints: Drill 05 - Locator Hierarchy

## Hint 1 (Concept)
The Locator Quality dimension of the 4D Feedback Matrix scores your selectors. CSS class selectors score 40/100 because they are coupled to implementation. If a developer renames btn-primary-submit to btn-submit, your test breaks with zero application change.

## Hint 2 (API Pattern)
The Playwright locator hierarchy (best to worst):
  1. page.getByRole("button", { name: "Pay Now" })       -- 100pts
  2. page.getByTestId("checkout-btn")                    -- 85pts
  3. page.getByText("Pay Now")                           -- 90pts
  4. page.locator(".btn-primary-submit")                 -- 40pts (fragile)
  5. page.locator("/html/body/div/button")               -- 0pts (never use)

## Hint 3 (Code Diff)
Replace: page.locator("div.checkout-container > button.btn-primary-submit")
With:    page.getByRole("button", { name: /Pay Now/i })
