# I AM NOT DONE
# Drill 02: Test Naming Matters
# Refactor vague test names (test_1, test_2, test_3) into descriptive business names:
# test_{what_the_system_does}_{under_what_condition}

def apply_discount(price: float, discount_pct: float) -> float:
    if discount_pct < 0 or discount_pct > 100:
        raise ValueError("Discount must be between 0 and 100")
    return price * (1 - discount_pct / 100)

def test_1():
    # TODO: rename this to say what it proves -- a percentage discount brings
    #       the price down. Follow the formula in the header.
    assert apply_discount(100.0, 20.0) == 80.0

def test_2():
    # TODO: rename this one. It proves a discount of zero leaves the price
    #       exactly as it was.
    assert apply_discount(50.0, 0.0) == 50.0

def test_3():
    # TODO: rename this one. It proves an out-of-range discount is refused
    #       rather than quietly applied.
    try:
        apply_discount(100.0, -5.0)
        assert False, "Should have raised ValueError"
    except ValueError:
        pass
