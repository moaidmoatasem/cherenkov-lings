# I AM NOT DONE
# Drill 03: Arrange-Act-Assert (AAA) Pattern
#
# Production Context:
# In the 2014 Toyota ETCS unintended acceleration case, entangled test state masked stack
# overflows and race conditions because preparation, mutation, and validation were interleaved.
#
# Your Goal:
# Refactor the tangled test below into two clean AAA tests:
# 1. test_successful_payment_returns_masked_card_and_amount()
#    - # ARRANGE: set amount = 99.99, currency = "USD", card_last4 = "4242"
#    - # ACT: result = process_payment(amount, currency, card_last4)
#    - # ASSERT: check status, amount, currency, masked_card ("****4242")
# 2. test_zero_amount_returns_error_status()
#    - # ARRANGE: set amount = 0.0
#    - # ACT: result = process_payment(amount, "USD", "1234")
#    - # ASSERT: check status == "error"

def process_payment(amount: float, currency: str, card_last4: str) -> dict:
    if amount <= 0:
        return {"status": "error", "message": "Amount must be positive"}
    return {"status": "success", "amount": amount, "currency": currency, "masked_card": f"****{card_last4}"}

def test_payment_procedural_mess():
    # Anti-pattern: Interleaved state setup and assertion spaghetti without clear AAA structure
    # TODO: Refactor into test_successful_payment_returns_masked_card_and_amount with # ARRANGE, # ACT, # ASSERT
    # TODO: Add test_zero_amount_returns_error_status
    res = process_payment(99.99, "USD", "4242")
    assert res["status"] == "success"
    assert False, "TODO: Restructure into AAA pattern and separate tests"
