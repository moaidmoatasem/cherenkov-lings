# I AM NOT DONE
# Drill 02: Test Naming Matters
# Refactor vague test names (test_1, test_2, test_3) into descriptive business names:
# test_{what_the_system_does}_{under_what_condition}

def apply_discount(price: float, discount_pct: float) -> float:
    if discount_pct < 0 or discount_pct > 100:
        raise ValueError("Discount must be between 0 and 100")
    return price * (1 - discount_pct / 100)

def test_1():
    # TODO: Rename to test_discount_reduces_price_by_percentage
    assert apply_discount(100.0, 20.0) == 80.0

def test_2():
    # TODO: Rename to test_zero_discount_returns_original_price
    assert apply_discount(50.0, 0.0) == 50.0

def test_3():
    # TODO: Rename to test_negative_discount_raises_value_error
    try:
        apply_discount(100.0, -5.0)
        assert False, "Should have raised ValueError"
    except ValueError:
        pass
