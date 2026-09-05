"""
PRODUCTION STORY:
The 2003 Northeast Blackout -- A Silent Alarm
A race-condition bug in FirstEnergy's alarm processor silenced the control
room's audible and visual warnings for over an hour while a cascading grid
failure unfolded. Operators kept watching a board that looked fine, because
nothing was left to tell them it wasn't. Fifty million people lost power.
An automated check that never reports what it actually found is that same
silent alarm, just smaller.
"""


def total_price(unit_price: float, quantity: int) -> float:
    return unit_price * quantity


def test_total_price_for_three_items():
    # TODO: This assertion expects the wrong number on purpose. Save this
    # file, look at what the watcher/pytest prints when an assertion fails
    # -- it shows you both sides of the comparison -- and use that printed
    # value, not a guess, to put the correct number in the assertion.
    assert total_price(unit_price=4.00, quantity=3) == 13.00
