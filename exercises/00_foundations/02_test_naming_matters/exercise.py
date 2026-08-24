# Drill 02: Test Naming Matters
# Good test names are the first line of documentation for your team.
# When a test fails in CI, the name tells you WHAT broke WITHOUT reading the code.
#
# TODO: Rename test_1, test_2, test_3 to describe exactly what they verify.

def apply_discount(price: float, discount_pct: float) -> float:
    if discount_pct < 0 or discount_pct > 100:
        raise ValueError("Discount must be between 0 and 100")
    return price * (1 - discount_pct / 100)

def test_1():
    assert apply_discount(100.0, 20.0) == 80.0

def test_2():
    assert apply_discount(50.0, 0.0) == 50.0

def test_3():
    try:
        apply_discount(100.0, -5.0)
        assert False, "Should have raised"
    except ValueError:
        pass
