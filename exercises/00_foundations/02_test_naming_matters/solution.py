# Drill 02: Solution -- Test Naming Matters
# Each test name answers: "What should be true about the system?"
# When test_discount_reduces_price_by_percentage fails in CI,
# you know exactly what broke without opening the file.

def apply_discount(price: float, discount_pct: float) -> float:
    if discount_pct < 0 or discount_pct > 100:
        raise ValueError("Discount must be between 0 and 100")
    return price * (1 - discount_pct / 100)

def test_discount_reduces_price_by_percentage():
    assert apply_discount(100.0, 20.0) == 80.0

def test_zero_discount_returns_original_price():
    assert apply_discount(50.0, 0.0) == 50.0

def test_negative_discount_raises_value_error():
    try:
        apply_discount(100.0, -5.0)
        assert False, "Should have raised ValueError"
    except ValueError:
        pass
