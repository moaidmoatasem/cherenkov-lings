"""
PRODUCTION STORY:
The 2003 Northeast Blackout -- A Silent Alarm
A race-condition bug in FirstEnergy's alarm processor silenced the control
room's audible and visual warnings for over an hour while a cascading grid
failure unfolded. An automated check that never reports what it actually
found is that same silent alarm, just smaller.
"""


def total_price(unit_price: float, quantity: int) -> float:
    return unit_price * quantity


def test_total_price_for_three_items():
    assert total_price(unit_price=4.00, quantity=3) == 12.00
