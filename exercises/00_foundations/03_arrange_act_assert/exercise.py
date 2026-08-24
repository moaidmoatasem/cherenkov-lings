# Drill 03: The Arrange-Act-Assert Pattern
# Every test in the world -- Playwright, REST Assured, k6, Postman -- follows AAA.
# ARRANGE: Set up the data and state you need.
# ACT:     Do the one thing you are testing.
# ASSERT:  Check the result.
#
# This test mixes setup, action, and assertion together.
# It is hard to read and hard to debug when it fails.
# TODO: Reorganize into clear ARRANGE / ACT / ASSERT sections using comments.

def process_payment(amount: float, currency: str, card_last4: str) -> dict:
    if amount <= 0:
        return {"status": "error", "message": "Amount must be positive"}
    return {"status": "success", "amount": amount, "currency": currency, "masked_card": f"****{card_last4}"}

def test_payment_success():
    result = process_payment(99.99, "USD", "4242")
    assert result["status"] == "success"
    amount = 99.99
    currency = "USD"
    card = "4242"
    assert result["amount"] == amount and result["currency"] == currency and result["masked_card"] == f"****{card}"
