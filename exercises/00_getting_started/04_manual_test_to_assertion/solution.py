"""
PRODUCTION STORY:
Therac-25 (1985-1987)
Operators learned exactly what to watch for through repeated use, but that
knowledge was never captured as a repeatable check anyone else could run.
"""

import requests

# --- Manual Test Case TC-014 (from the test plan, written by a QA engineer) ---
# Title: Checkout total includes tax
# Expected result: total equals subtotal plus tax, to the cent.
# --------------------------------------------------------------------------


def test_checkout_total_includes_tax():
    checkout = requests.get("http://localhost:8081/checkout").json()
    expected_total = round(checkout["subtotal"] + checkout["tax"], 2)
    assert round(checkout["total"], 2) == expected_total, (
        f"Expected total {expected_total} (subtotal + tax), got {checkout['total']}"
    )
