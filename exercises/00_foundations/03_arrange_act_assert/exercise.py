# I AM NOT DONE
# Drill 03: Arrange-Act-Assert (AAA) Pattern
#
# Production Context:
# In the 2014 Toyota ETCS unintended acceleration case, entangled test state masked stack
# overflows and race conditions because preparation, mutation, and validation were interleaved.
#
# Your Goal:
# The test below checks two unrelated behaviours in one tangled block. Split it
# into two tests, each covering one behaviour of process_payment:
#   - the successful path, where a positive amount is charged
#   - the rejected path, where an amount of zero is refused
#
# Structure each one with explicit # ARRANGE, # ACT and # ASSERT sections, and
# name it after the behaviour it proves. On the successful path, assert on
# everything the caller is promised -- including how the card is masked.

def process_payment(amount: float, currency: str, card_last4: str) -> dict:
    if amount <= 0:
        return {"status": "error", "message": "Amount must be positive"}
    return {"status": "success", "amount": amount, "currency": currency, "masked_card": f"****{card_last4}"}

def test_payment_procedural_mess():
    # Anti-pattern: setup, call and checks interleaved in one block, covering
    # two behaviours at once. When this fails, you cannot tell which one broke.
    # TODO: replace this with the two AAA-structured tests described above.
    res = process_payment(99.99, "USD", "4242")
    assert res["status"] == "success"
    assert False, "TODO: split into two tests, each with ARRANGE / ACT / ASSERT"
