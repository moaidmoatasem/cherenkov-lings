# Theoretical Context: Status Code Assertions & The Vacuous Test

## Production Incident: The Knight Capital Deployment (2012)

Knight Capital Group lost approximately $460 million in 45 minutes when a deployment left obsolete code active on one of eight production servers. The failure is usually told as a deployment story, but the deeper lesson is about verification: the release passed its automated checks because those checks confirmed that the deployment process completed, not that the deployed system behaved correctly. Green checks that assert on the wrong thing — or on nothing at all — are how catastrophic regressions reach production wearing a passing badge.

## The Underlying Mechanism

A pytest function signals failure by raising an exception. It signals success by returning. This means the default outcome of any test body that runs to completion is **pass**:

```
[Anatomy of a Vacuous Test]

  def test_health():
      res = requests.get(...)     # network call happens
      pass                        # ← no exception raised
                                  # ← pytest reports PASSED

  Server returns 200  →  PASSED   ✅ (correct, by accident)
  Server returns 500  →  PASSED   ❌ (false confidence)
  Server returns HTML →  PASSED   ❌ (false confidence)
  Server is on fire   →  PASSED   ❌ (false confidence)
```

The request is genuinely issued, so the test is not obviously inert — it consumes CI time, it touches the system under test, and it appears in the report alongside real tests. Only an assertion converts an observation into a verdict.

Status code checking is the foundation of the API testing pyramid for two reasons. First, it is the one part of the contract that is defined by the protocol rather than by your application, so it is stable across refactors of the response body. Second, it short-circuits misleading downstream failures: if you parse JSON before checking the status, a 500 returning an HTML error page surfaces as a confusing `JSONDecodeError` rather than the server error it actually is.

Assert the exact code the contract specifies. `response.ok` is true for every 2xx and 3xx, so it accepts a `301 Moved Permanently` or a `204 No Content` as a healthy `200` — a distinction that matters the moment someone changes a route.

You will now simulate this in the Crucible: fix a vacuous health check by asserting the exact status code, with a failure message that reports what you actually received.
