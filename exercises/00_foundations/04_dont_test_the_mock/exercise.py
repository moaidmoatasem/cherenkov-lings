# I AM NOT DONE
"""
PRODUCTION STORY:
Knight Capital Group ($440M Trading Loss, 2012)
Engineers repurposed an old testing flag in high-frequency trading code and tested it solely against
internal mock engines that echoed back expected test data, concealing catastrophic live routing behavior.
"""

# Drill 04: Do Not Test the Mock
# When you stub or fake every part of the system, you only prove
# that your fake works correctly -- not your real code.
# This test stubs the entire payment gateway and asserts on the stub.
# It gives 100% false confidence.
# TODO: Change the test to call the REAL Crucible API endpoint instead.
import requests

class FakePaymentGateway:
    def charge(self, amount):
        return {}

def test_payment_gateway_charges_correctly():
    gateway = FakePaymentGateway()
    result = gateway.charge(50.00)
    assert result["status"] == "success"
    assert result["amount"] == 50.00
