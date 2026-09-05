import requests
from pact import Pact, match


def test_consumer_contract_definition(tmp_path):
    pact = Pact("OrdersWebClient", "OrdersService")
    (
        pact.upon_receiving("a request for the current orders")
        .given("orders exist")
        .with_request("GET", "/api/pact/orders")
        .will_respond_with(200)
        .with_header("Content-Type", "application/json")
        .with_body(
            {
                "orders": match.each_like(
                    {
                        "id": match.like("ORD-101"),
                        "total": match.like(149.0),
                        "status": match.regex(
                            "COMPLETED", regex=r"COMPLETED|PENDING|CANCELLED"
                        ),
                        "currency": "USD",
                    }
                ),
                "count": match.like(2),
            }
        )
    )

    # The interaction above spins up a real mock HTTP server; the consumer
    # code under test talks to *that* mock, never to the live OrdersService.
    # This is the whole point of a consumer-driven contract: the consumer
    # records its own expectations in isolation.
    with pact.serve() as mock_server:
        res = requests.get(f"{mock_server.url}/api/pact/orders")
        assert res.status_code == 200
        data = res.json()

        # Assert structural contract schema expectations
        assert "orders" in data
        assert isinstance(data["orders"], list)
        assert len(data["orders"]) > 0
        order = data["orders"][0]
        assert "id" in order and "total" in order and "status" in order

    # This is the artifact a real CI pipeline would publish to a Pact Broker
    # (or hand directly to the provider team) for provider verification.
    pact.write_file(tmp_path, overwrite=True)
    contract = tmp_path / "OrdersWebClient-OrdersService.json"
    assert contract.exists(), "Pact did not write the consumer contract to disk"
