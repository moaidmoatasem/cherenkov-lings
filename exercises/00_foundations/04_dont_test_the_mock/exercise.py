# Drill 04: Do Not Test the Mock
# When you stub or fake every part of the system, you only prove
# that your fake works correctly -- not your real code.
# This test stubs the entire payment gateway and asserts on the stub.
# It gives 100% false confidence.
# TODO: Change the test to call the REAL Crucible API endpoint instead.
import requests

class FakePaymentGateway:
    def charge(self, amount):
        return {"status": "success", "amount": amount}

def test_payment_gateway_charges_correctly():
    # Anti-pattern: we are testing our own fake, not the real system
    gateway = FakePaymentGateway()
    result = gateway.charge(50.00)
    # This will ALWAYS pass because we wrote both the fake and the assertion
    assert result["status"] == "success"
    assert result["amount"] == 50.00
