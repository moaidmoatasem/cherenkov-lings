"""
PRODUCTION STORY:
Google Test Automation Pyramid Inversion (2015)
Teams relying predominantly on end-to-end browser tests suffered from 12-hour CI build cycles and 30% flakiness,
prompting a company-wide shift to enforce the Test Pyramid: testing core business calculations at the fast API layer.
"""

# Drill 01: UI Test vs API Test -- When to Use Which?
#
# Scenario: The checkout total must equal (item_price * quantity) + tax.
# A junior QA wrote a Playwright test that clicks through the full UI,
# fills the form, and reads the displayed total.
# This takes 8 seconds per run and is flaky on slow CI machines.
#
# TODO: The business rule lives in the backend. Rewrite this as an API test.
# The API endpoint is: POST http://localhost:8081/checkout
# Request body: {"item_id": "item-1", "quantity": 2}
# Assert: response JSON contains a valid total field

# WRONG APPROACH (simulated here as pseudocode comments):
# playwright: goto /checkout -> fill item -> fill quantity -> click Pay -> read total text
# Problems: 8s runtime, flaky on slow CI, tests UI rendering not business logic

# RIGHT APPROACH: test the business rule at the API layer
# TODO: Write a requests-based test here instead
import requests

def test_checkout_total_via_api():
    # Write your API test here
    pass  # TODO: POST to /checkout and assert on total
