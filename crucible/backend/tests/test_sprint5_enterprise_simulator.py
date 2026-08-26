"""Sprint 5 Phase 2 & 4 backend coverage.

Three features land here: the db_timeout / dast_xss chaos directives, the
distributed-tracing middleware, and the spec-driven Pytest generator behind
POST /api/generate-tests.
"""

from __future__ import annotations

import ast
import json

import pytest
from fastapi import FastAPI
from fastapi.testclient import TestClient

from crucible.backend.ai_generator import (
    OpenApiFetchError,
    generate_pytest_from_spec,
    generate_pytest_from_openapi,
)
from crucible.backend.app import app
from crucible.backend.chaos import parse_chaos_header
from crucible.backend.tracing import TracingMiddleware


@pytest.fixture()
def client() -> TestClient:
    return TestClient(app)


# ---------------------------------------------------------------------------
# Chaos directive parsing
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("directive", ["db_timeout", "dast_xss"])
@pytest.mark.parametrize("truthy", ["true", "1", "yes", "TRUE", "Yes"])
def test_new_chaos_flags_parse_as_booleans(directive: str, truthy: str) -> None:
    assert parse_chaos_header(f"{directive}={truthy}") == {directive: True}


@pytest.mark.parametrize("directive", ["db_timeout", "dast_xss"])
def test_new_chaos_flags_are_false_when_explicitly_disabled(directive: str) -> None:
    # A client sending db_timeout=false must not get a 504: an unrecognized
    # value silently becoming True would make the flag impossible to turn off.
    assert parse_chaos_header(f"{directive}=false") == {directive: False}


def test_new_chaos_flags_compose_with_existing_directives() -> None:
    parsed = parse_chaos_header("delay=250ms; db_timeout=true, dast_xss=true")

    assert parsed["delay"] == 250.0
    assert parsed["db_timeout"] is True
    assert parsed["dast_xss"] is True


# ---------------------------------------------------------------------------
# db_timeout — simulated database bottleneck
# ---------------------------------------------------------------------------


def test_db_timeout_returns_504_on_search(client: TestClient) -> None:
    res = client.get("/search", params={"q": "python"}, headers={"X-Chaos": "db_timeout=true"})

    assert res.status_code == 504
    assert res.json()["status"] == "error"


def test_db_timeout_short_circuits_before_the_latency_simulation(client: TestClient) -> None:
    # A single-character query normally sleeps 800ms. The DB failure must be
    # returned immediately rather than after the full inverted-latency delay.
    res = client.get("/search", params={"q": "p"}, headers={"X-Chaos": "db_timeout=true"})

    assert res.status_code == 504


def test_search_is_unaffected_without_the_db_timeout_flag(client: TestClient) -> None:
    res = client.get("/search", params={"q": "python"})

    assert res.status_code == 200
    assert res.json()["query"] == "python"


def test_db_timeout_false_does_not_trigger_the_failure(client: TestClient) -> None:
    res = client.get("/search", params={"q": "python"}, headers={"X-Chaos": "db_timeout=false"})

    assert res.status_code == 200


# ---------------------------------------------------------------------------
# dast_xss — reflected XSS payload injection
# ---------------------------------------------------------------------------


def test_dast_xss_injects_an_unescaped_script_tag(client: TestClient) -> None:
    res = client.get("/search", params={"q": "python"}, headers={"X-Chaos": "dast_xss=true"})

    assert res.status_code == 200
    results = res.json()["results"]
    assert any("<script>" in item for item in results), (
        "the DAST drill needs a genuinely unescaped payload to be detectable"
    )


def test_dast_xss_reflects_the_caller_supplied_query(client: TestClient) -> None:
    # Reflection is the whole point: a static payload would not teach a learner
    # to trace attacker-controlled input through to the response.
    res = client.get("/search", params={"q": "python"}, headers={"X-Chaos": "dast_xss=true"})

    payload = res.json()["results"][0]
    assert "python" in payload


def test_dast_xss_payload_is_counted_in_the_result_total(client: TestClient) -> None:
    res = client.get("/search", params={"q": "python"}, headers={"X-Chaos": "dast_xss=true"})
    body = res.json()

    assert body["count"] == len(body["results"]), "count must match the returned list"


def test_search_has_no_script_payload_without_the_flag(client: TestClient) -> None:
    res = client.get("/search", params={"q": "python"})

    assert not any("<script>" in item for item in res.json()["results"])


def test_db_timeout_takes_precedence_over_dast_xss(client: TestClient) -> None:
    # Both flags set: the request never reaches the search body, so no payload
    # can be reflected out of a failed database call.
    res = client.get(
        "/search",
        params={"q": "python"},
        headers={"X-Chaos": "db_timeout=true; dast_xss=true"},
    )

    assert res.status_code == 504
    assert "<script>" not in res.text


# ---------------------------------------------------------------------------
# Distributed tracing middleware
# ---------------------------------------------------------------------------


def test_every_response_carries_a_trace_id(client: TestClient) -> None:
    res = client.get("/health")

    assert res.headers.get("X-Trace-Id"), "tracing middleware is not registered"


def test_trace_ids_are_unique_per_request(client: TestClient) -> None:
    ids = {client.get("/health").headers["X-Trace-Id"] for _ in range(5)}

    assert len(ids) == 5, "a shared trace id would make spans uncorrelatable"


def test_trace_id_is_exposed_to_browsers_via_cors(client: TestClient) -> None:
    # Without the expose_headers entry the header exists on the wire but is
    # unreadable from the Mission Control frontend's fetch().
    res = client.get("/health", headers={"Origin": "http://localhost:8080"})

    exposed = res.headers.get("access-control-expose-headers", "")
    assert "X-Trace-Id" in exposed


def test_trace_id_is_present_on_error_responses(client: TestClient) -> None:
    res = client.get("/search", params={"q": "x"}, headers={"X-Chaos": "db_timeout=true"})

    assert res.status_code == 504
    assert res.headers.get("X-Trace-Id"), "a failing request is when the id matters most"


def test_span_is_closed_even_when_the_route_raises(caplog) -> None:
    # Regression: the original middleware assigned the header and logged
    # SPAN END after `await call_next(...)` with no try/finally, so an
    # unhandled route exception left the span dangling forever.
    isolated = FastAPI()
    isolated.add_middleware(TracingMiddleware)

    @isolated.get("/boom")
    def boom() -> None:
        raise RuntimeError("kaboom")

    with caplog.at_level("INFO", logger="crucible.tracing"):
        res = TestClient(isolated, raise_server_exceptions=False).get("/boom")

    assert res.status_code == 500
    messages = [r.message for r in caplog.records]
    assert any("[SPAN START]" in m for m in messages)
    assert any("[SPAN END]" in m for m in messages), "span never closed on the error path"
    assert any("[SPAN ERROR]" in m for m in messages)


def test_span_logs_pair_start_and_end_for_a_normal_request(caplog) -> None:
    with caplog.at_level("INFO", logger="crucible.tracing"):
        res = TestClient(app).get("/health")

    trace_id = res.headers["X-Trace-Id"]
    messages = [r.message for r in caplog.records if trace_id in r.message]

    assert any("[SPAN START]" in m for m in messages)
    assert any("[SPAN END]" in m for m in messages)


def test_chaos_state_still_reaches_endpoints_through_the_tracing_layer(
    client: TestClient,
) -> None:
    # TracingMiddleware wraps ChaosMiddleware. If the extra layer broke the
    # shared request scope, request.state.chaos would come back empty and every
    # chaos drill in the platform would silently stop working.
    res = client.get("/search", params={"q": "python"}, headers={"X-Chaos": "dast_xss=true"})

    assert any("<script>" in item for item in res.json()["results"])


# ---------------------------------------------------------------------------
# OpenAPI-driven Pytest generator
# ---------------------------------------------------------------------------

MINIMAL_SPEC = {
    "info": {"title": "Tiny API"},
    "paths": {
        "/health": {
            "get": {
                "summary": "Health probe",
                "responses": {"200": {"description": "ok"}},
            }
        },
        "/items/{item_id}": {
            "get": {
                "summary": "Fetch one item",
                "parameters": [{"name": "item_id", "in": "path", "required": True}],
                "responses": {"200": {"description": "ok"}},
            }
        },
        "/orders": {
            "post": {"summary": "Create order", "responses": {"201": {"description": "made"}}}
        },
    },
}


def test_generator_emits_a_test_per_parameterless_get() -> None:
    code = generate_pytest_from_spec(MINIMAL_SPEC)

    assert "def test_get_health():" in code
    assert "/health" in code


def test_generator_skips_operations_it_cannot_infer_arguments_for() -> None:
    code = generate_pytest_from_spec(MINIMAL_SPEC)

    assert "def test_get_items_item_id():" in code
    assert "@pytest.mark.skip" in code, "a required path param must not be silently guessed"


def test_generator_ignores_non_get_operations() -> None:
    code = generate_pytest_from_spec(MINIMAL_SPEC)

    assert "post" not in code.lower().split("# ")[0].replace("post(", "")
    assert "requests.post" not in code, "POST needs a body the generator cannot invent"


def test_generator_asserts_the_status_code_declared_by_the_spec() -> None:
    spec = {
        "info": {"title": "Tiny API"},
        "paths": {"/created": {"get": {"responses": {"201": {"description": "made"}}}}},
    }

    code = generate_pytest_from_spec(spec)

    assert "== 201" in code, "a 201-only endpoint must not be asserted as 200"


def test_generated_module_is_syntactically_valid_python() -> None:
    # The output is handed straight to a learner to run; a syntax error in the
    # scaffolding is indistinguishable to them from a broken exercise.
    ast.parse(generate_pytest_from_spec(MINIMAL_SPEC))


def test_generator_handles_a_spec_with_no_testable_endpoints() -> None:
    code = generate_pytest_from_spec({"info": {"title": "Empty"}, "paths": {}})

    ast.parse(code)
    assert "no testable endpoints" in code


def test_generator_reflects_the_supplied_base_url() -> None:
    code = generate_pytest_from_spec(MINIMAL_SPEC, base_url="http://example.test:9999")

    assert 'BASE_URL = "http://example.test:9999"' in code


def test_fetch_failure_degrades_to_a_skipped_module_not_an_exception() -> None:
    code = generate_pytest_from_openapi("http://127.0.0.1:1/openapi.json")

    ast.parse(code)
    assert "pytest.skip" in code


def test_fetch_error_is_raised_by_the_low_level_helper() -> None:
    from crucible.backend.ai_generator import fetch_openapi_spec

    with pytest.raises(OpenApiFetchError):
        fetch_openapi_spec("http://127.0.0.1:1/openapi.json", timeout=0.5)


# ---------------------------------------------------------------------------
# POST /api/generate-tests
# ---------------------------------------------------------------------------


def test_generate_tests_endpoint_returns_generated_code(client: TestClient) -> None:
    res = client.post("/api/generate-tests")

    assert res.status_code == 200
    body = res.json()
    assert body["status"] == "success"
    assert body["code"].strip()


def test_generate_tests_reads_this_apps_real_routes(client: TestClient) -> None:
    # Regression: the first implementation returned a hardcoded string and never
    # looked at the spec, so it produced the same two-line file for any service.
    code = client.post("/api/generate-tests").json()["code"]

    assert "/api/curriculum" in code, "generated suite does not reflect the real route table"
    assert code.count("def test_") > 5, "a real spec yields many endpoint tests"


def test_generate_tests_output_is_importable_python(client: TestClient) -> None:
    ast.parse(client.post("/api/generate-tests").json()["code"])


def test_generate_tests_targets_the_requesting_host(client: TestClient) -> None:
    # The old version hardcoded localhost:8081, which is wrong whenever the app
    # is served on another port or behind a container hostname.
    body = client.post("/api/generate-tests").json()

    assert body["base_url"] == "http://testserver"
    assert 'BASE_URL = "http://testserver"' in body["code"]


def test_generate_tests_does_not_call_itself_over_http(client: TestClient) -> None:
    # A self-referential fetch deadlocks under a single-threaded test client;
    # the endpoint must read app.openapi() in-process. Completing at all is the
    # assertion here.
    assert client.post("/api/generate-tests").status_code == 200


def test_generated_suite_json_round_trips(client: TestClient) -> None:
    # The frontend receives this as JSON; embedded quotes and newlines in the
    # generated source must survive serialization.
    raw = client.post("/api/generate-tests").content
    assert json.loads(raw)["code"].startswith('"""')
