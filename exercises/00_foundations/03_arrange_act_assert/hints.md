# Hints: Drill 03 - Arrange-Act-Assert

## Hint 1 (Concept)
AAA is universal. Whether you are writing a Playwright test, a REST Assured test, or a k6 script, you are always: setting something up, doing something, and checking the result. Separating these three phases makes every failure instantly obvious.

## Hint 2 (Pattern)
Add these three comments to structure your test:
  # ARRANGE
  # ACT
  # ASSERT
Then move your code under the right section.

## Hint 3 (Code Diff)
Before: all mixed together in one block
After:
  # ARRANGE
  amount = 99.99
  currency = "USD"
  card_last4 = "4242"
  # ACT
  result = process_payment(amount, currency, card_last4)
  # ASSERT
  assert result["status"] == "success"
