"""
PRODUCTION STORY:
Therac-25 (1985-1987)
Operators of the Therac-25 radiation therapy machine learned through
repeated use that entering an edit command and correcting it within eight
seconds could trigger a race condition that silently bypassed a safety
interlock. Several patients received massive radiation overdoses before
the cause was found. The operators' own knowledge of exactly what to check
for existed -- it lived in their hands and habits, and was never captured
as a repeatable check anyone else could run against the next release.
"""

import requests

# --- Manual Test Case TC-014 (from the test plan, written by a QA engineer) ---
# Title: Checkout total includes tax
# Steps:
#   1. Open the checkout page with one item in the cart.
#   2. Note the subtotal and the tax shown on the page.
#   3. Note the total shown on the page.
# Expected result: total equals subtotal plus tax, to the cent.
# --------------------------------------------------------------------------


def test_checkout_total_includes_tax():
    checkout = requests.get("http://localhost:8081/checkout").json()
    # TODO: TC-014 above is a real step in this project's manual test plan --
    # someone runs those three steps by hand today. Turn its "Expected
    # result" line into one assertion against the fields already sitting in
    # `checkout` (subtotal, tax, total). Compute the expected total FROM
    # subtotal and tax -- don't hardcode the specific numbers you happen to
    # see when you run this once, or the check stops meaning anything the
    # moment the price changes.
    pass
