# I AM NOT DONE
# Drill 01: What is an Automated Test?
# Fix this test by adding an assertion that verifies the checkout total.

def calculate_total(item_price: float, quantity: int, tax_rate: float) -> float:
    """Calculates the final checkout total including tax."""
    subtotal = item_price * quantity
    return subtotal + (subtotal * tax_rate)

def test_checkout_total():
    total = calculate_total(item_price=10.00, quantity=3, tax_rate=0.1)
    # 10.00 * 3 = 30.00, plus 10% tax = 33.00
    # TODO: Replace the line below with: assert total == 33.0, f"Expected 33.0 but got {total}"
    assert False, "TODO: Implement assert statement to verify checkout total"
