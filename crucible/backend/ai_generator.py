"""Spec-driven Pytest scaffolding.

Reads an OpenAPI document and emits one Pytest function per GET operation that
takes no required parameters — the subset that can be validated with nothing
but a base URL. Anything needing a body, a path parameter, or auth is reported
as a skipped stub rather than silently dropped, so the learner can see what the
generator declined to guess at.
"""

from __future__ import annotations

import json
import re
import urllib.error
import urllib.request
from typing import Any

# A generated module that hammers a live service should not hang a CI job.
FETCH_TIMEOUT_SECONDS = 5.0

DEFAULT_BASE_URL = "http://localhost:8081"


class OpenApiFetchError(RuntimeError):
    """Raised when the OpenAPI document cannot be retrieved or parsed."""


def fetch_openapi_spec(openapi_url: str, timeout: float = FETCH_TIMEOUT_SECONDS) -> dict[str, Any]:
    """Fetch and parse an OpenAPI document.

    Raises OpenApiFetchError for transport failures, non-JSON bodies, and
    documents whose root is not a JSON object — the caller decides whether that
    is fatal or worth degrading on.
    """
    try:
        with urllib.request.urlopen(openapi_url, timeout=timeout) as response:
            raw = response.read()
    except (urllib.error.URLError, OSError, ValueError) as exc:
        raise OpenApiFetchError(f"could not fetch {openapi_url}: {exc}") from exc

    try:
        spec = json.loads(raw)
    except json.JSONDecodeError as exc:
        raise OpenApiFetchError(f"{openapi_url} did not return valid JSON: {exc}") from exc

    if not isinstance(spec, dict):
        raise OpenApiFetchError(f"{openapi_url} returned {type(spec).__name__}, expected an object")

    return spec


def _test_name(path: str, method: str) -> str:
    """Turn '/api/checkout' + 'get' into 'test_get_api_checkout'."""
    slug = re.sub(r"[^a-zA-Z0-9]+", "_", path).strip("_").lower()
    return f"test_{method.lower()}_{slug or 'root'}"


def _requires_arguments(operation: dict[str, Any]) -> bool:
    """True when the operation needs input the generator cannot invent."""
    if operation.get("requestBody"):
        return True
    return any(param.get("required") for param in operation.get("parameters", []) or [])


def _expected_status(operation: dict[str, Any]) -> int:
    """The success code the spec declares, defaulting to 200."""
    responses = operation.get("responses", {}) or {}
    for code in sorted(responses):
        if isinstance(code, str) and code.isdigit() and 200 <= int(code) < 300:
            return int(code)
    return 200


def generate_pytest_from_spec(spec: dict[str, Any], base_url: str = DEFAULT_BASE_URL) -> str:
    """Render a Pytest module from an already-parsed OpenAPI document."""
    title = (spec.get("info") or {}).get("title", "API")
    paths = spec.get("paths") or {}

    lines: list[str] = [
        '"""Auto-generated endpoint validation tests.',
        "",
        f"Source spec: {title}",
        f"Base URL:    {base_url}",
        "",
        "Generated scaffolding, not a substitute for hand-written tests: it",
        "asserts only that each endpoint answers with its declared status code.",
        '"""',
        "",
        "import pytest",
        "import requests",
        "",
        f'BASE_URL = "{base_url}"',
        "",
    ]

    generated = 0
    skipped = 0

    for path in sorted(paths):
        operations = paths[path]
        if not isinstance(operations, dict):
            continue
        for method in sorted(operations):
            if method.lower() not in ("get", "head"):
                continue
            operation = operations[method]
            if not isinstance(operation, dict):
                continue

            name = _test_name(path, method)
            summary = operation.get("summary") or operation.get("operationId") or path

            if _requires_arguments(operation):
                skipped += 1
                lines += [
                    '@pytest.mark.skip(reason="requires request data the generator cannot infer")',
                    f"def {name}():",
                    f'    """{summary}"""',
                    f'    raise NotImplementedError("supply parameters for {path}")',
                    "",
                ]
                continue

            generated += 1
            lines += [
                f"def {name}():",
                f'    """{summary}"""',
                f'    response = requests.{method.lower()}(f"{{BASE_URL}}{path}", timeout=5)',
                f"    assert response.status_code == {_expected_status(operation)}, (",
                f'        f"{method.upper()} {path} returned {{response.status_code}}"',
                "    )",
                "",
            ]

    if generated == 0 and skipped == 0:
        lines += [
            "def test_spec_exposed_no_testable_endpoints():",
            '    """The spec declared no parameterless GET/HEAD operations."""',
            '    pytest.skip("no testable endpoints found in the OpenAPI document")',
            "",
        ]

    lines.append(f"# {generated} test(s) generated, {skipped} skipped as under-specified.")
    return "\n".join(lines) + "\n"


def generate_pytest_from_openapi(
    openapi_url: str, base_url: str = DEFAULT_BASE_URL
) -> str:
    """Fetch an OpenAPI document and render a Pytest module from it.

    Fetch failures are returned as a commented, importable module rather than
    raised: the caller is an HTTP endpoint whose job is to hand the learner
    something readable, not a 500.
    """
    try:
        spec = fetch_openapi_spec(openapi_url)
    except OpenApiFetchError as exc:
        return (
            '"""Generation failed."""\n\n'
            "import pytest\n\n"
            f"# Could not read the OpenAPI document at {openapi_url}.\n"
            f"# {exc}\n\n"
            "def test_openapi_spec_unavailable():\n"
            f'    pytest.skip("OpenAPI spec unavailable: {openapi_url}")\n'
        )

    return generate_pytest_from_spec(spec, base_url=base_url)
