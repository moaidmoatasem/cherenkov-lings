# Drill 01: What is an Automated Test?
# As a Manual QA engineer you know how to CHECK something manually.
# An automated test is just that same check -- written as code.
#
# Right now this "test" just runs the checkout logic but never checks anything.
# The result could be wrong and nobody would know!
# TODO: Add an assert statement to check that the total price is correct.

def calculate_total(item_price: float, quantity: int, tax_rate: float) -> float:
    """Calculates the final checkout total including tax."""
    subtotal = item_price * quantity
    return subtotal + (subtotal * tax_rate)

def test_checkout_total():
    total = calculate_total(item_price=10.00, quantity=3, tax_rate=0.1)
    # This test always passes even if calculate_total is completely broken!
    # A test without an assert is just code that runs silently.
    pass  # TODO: Replace this with: assert total == 33.0
