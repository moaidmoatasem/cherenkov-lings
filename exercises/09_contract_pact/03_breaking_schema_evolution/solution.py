import pytest
import requests
from pact import Pact, Verifier, match

PROVIDER_URL = "http://localhost:8081"


def _write_contract(directory, consumer_name, order_shape):
    pact = Pact(consumer_name, "OrdersService")
    (
        pact.upon_receiving("a request for the current orders")
        .given("orders exist")
        .with_request("GET", "/api/pact/orders")
        .will_respond_with(200)
        .with_body({"orders": match.each_like(order_shape), "count": match.like(2)})
    )
    with pact.serve() as mock_server:
        requests.get(f"{mock_server.url}/api/pact/orders")
    pact.write_file(directory, overwrite=True)
    return next(directory.glob("*.json"))


def test_additive_schema_change_is_backward_compatible(tmp_path):
    """A consumer that only ever asked for `id`/`total` keeps passing when
    the provider adds `status`/`currency` alongside them -- additive changes
    are safe."""
    pact_file = _write_contract(
        tmp_path / "safe",
        "ReportingServiceV1",
        {"id": match.like("ORD-101"), "total": match.like(149.0)},
    )

    verifier = (
        Verifier("OrdersService").add_transport(url=PROVIDER_URL).add_source(pact_file)
    )
    verifier.verify()

    assert verifier.results["result"] is True


def test_breaking_field_rename_is_rejected(tmp_path):
    """A consumer still wired to the pre-rename field name (`order_ref`
    instead of `id`) fails verification -- that failure is the safety gate
    working, not a broken test."""
    pact_file = _write_contract(
        tmp_path / "breaking",
        "LegacyReportingConsumer",
        {"order_ref": match.like("ORD-101"), "total": match.like(149.0)},
    )

    verifier = (
        Verifier("OrdersService").add_transport(url=PROVIDER_URL).add_source(pact_file)
    )

    with pytest.raises(RuntimeError):
        verifier.verify()

    mismatches = verifier.results["errors"][0]["mismatch"]["mismatches"]
    assert any("order_ref" in m["mismatch"] for m in mismatches)
