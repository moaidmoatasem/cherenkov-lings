# Drill 03: Solution -- Arrange-Act-Assert

def process_payment(amount: float, currency: str, card_last4: str) -> dict:
    if amount <= 0:
        return {"status": "error", "message": "Amount must be positive"}
    return {"status": "success", "amount": amount, "currency": currency, "masked_card": f"****{card_last4}"}

def test_successful_payment_returns_masked_card_and_amount():
    # ARRANGE: Set up the inputs
    amount = 99.99
    currency = "USD"
    card_last4 = "4242"

    # ACT: Do the one thing we are testing
    result = process_payment(amount, currency, card_last4)

    # ASSERT: Check each outcome separately
    assert result["status"] == "success"
    assert result["amount"] == amount
    assert result["currency"] == currency
    assert result["masked_card"] == "****4242"

def test_zero_amount_returns_error_status():
    # ARRANGE
    amount = 0.0

    # ACT
    result = process_payment(amount, "USD", "1234")

    # ASSERT
    assert result["status"] == "error"
