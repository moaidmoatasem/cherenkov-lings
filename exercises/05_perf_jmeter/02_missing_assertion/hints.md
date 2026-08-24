# Hints: Drill 02 - Missing Response Assertion

## Hint 1 (Concept)
In JMeter, an HTTP Sampler by default marks a request as SUCCESSFUL if it receives ANY response -- including HTTP 500 Internal Server Error. Without a Response Assertion, your load test will show 0% error rate even when every request returns an error page.

## Hint 2 (Pattern)
Add a Response Assertion to every HTTP Sampler:
  - Field to test: Response Code
  - Pattern matching rules: Equals
  - Patterns to test: 200
This causes JMeter to mark any non-200 response as a test failure.

## Hint 3 (Answer)
In the JMeter GUI: right-click your HTTP Sampler, Add > Assertions > Response Assertion.
Set: Field = Response Code, Pattern = 200.
In the JTL results file, failed assertions appear as: success=false
