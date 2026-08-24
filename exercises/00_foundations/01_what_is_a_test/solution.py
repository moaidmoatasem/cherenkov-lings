# Drill 01: Solution -- What is an Automated Test?
# An assert statement is what turns code into a TEST.
# If the condition is False, pytest marks the test FAILED and shows you exactly why.

def calculate_total(item_price: float, quantity: int, tax_rate: float) -> float:
    """Calculates the final checkout total including tax."""
    subtotal = item_price * quantity
    return subtotal + (subtotal * tax_rate)

def test_checkout_total():
    total = calculate_total(item_price=10.00, quantity=3, tax_rate=0.1)
    # 10.00 * 3 = 30.00, plus 10% tax = 33.00
    assert total == 33.0, f"Expected 33.0 but got {total}"
