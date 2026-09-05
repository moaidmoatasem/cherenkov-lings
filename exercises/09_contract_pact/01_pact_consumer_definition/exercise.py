"""
PRODUCTION STORY:
Monolith Decomposition E2E Testing Bottleneck (2017)
When microservices scale, end-to-end integration environments become brittle and bottleneck deployment velocity.
Consumer-Driven Contracts allow services to test compatibility in isolation.
"""
import requests

def test_consumer_contract_definition():
    # Anti-pattern: Relying on live third-party dependencies without contract specifications
    # TODO: Use pact-python's `Pact` + `match` builders to define this consumer's
    # expectations against a Pact mock provider (see pact.serve()), instead of
    # sending the request straight to the live OrdersService below.
    res = requests.get("http://localhost:8081/api/pact/orders")
    assert res.status_code == 200
