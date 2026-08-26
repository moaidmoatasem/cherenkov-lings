# Hints: Drill 01 - Health Endpoint Status Code

## Hint 1 (Architectural Nudge)
Look closely at what the starter test actually proves. It issues the request, binds the response to a name, and then falls off the end of the function body. A pytest test passes unless it raises, so a test with no assertion is green no matter what the server returned — 200, 500, or an HTML error page. This is the vacuous assertion anti-pattern, and it is worse than having no test at all: it occupies a slot in your suite and reports confidence it never earned. The status code is the coarsest contract an HTTP API offers, and it is the first thing worth pinning down.

## Hint 2 (API Pattern)
The `requests` library returns a `Response` object. Its integer HTTP status lives on the `status_code` attribute — not a method, so no parentheses. Compare it against the exact code the contract promises rather than a truthy range check like `res.ok`, which quietly accepts any 2xx or 3xx and would let a 301 redirect or a 204 No Content pass as healthy. When the assertion fails, an f-string message that reports the code you actually received turns a bare `AssertionError` into a diagnosis you can act on without re-running under a debugger.

## Hint 3 (Code Diff)
Replace the `pass` placeholder with a real assertion:

    def test_health_check_status_code():
        response = requests.get("http://localhost:8081/health")
        assert response.status_code == 200, f"Expected 200, got {response.status_code}"

Delete the `pass` line entirely — leaving it after the assertion is dead code, and leaving it instead of the assertion is the bug you were sent here to fix.
