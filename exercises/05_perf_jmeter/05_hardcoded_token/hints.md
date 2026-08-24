# Hints: Drill 05 - Hardcoded CSRF/Auth Token (Correlation)

## Hint 1 (Concept)
Modern web applications generate unique CSRF tokens, session IDs, and OAuth tokens on every login. If you record a session in JMeter and hardcode the captured token, every subsequent test run will fail with 403 Forbidden because the token has expired or is tied to a previous session.

## Hint 2 (Pattern)
Use JMeter Extractors to capture dynamic values from responses:
  1. Regular Expression Extractor: captures value from response body using regex
  2. JSON Extractor: captures value from JSON response
  3. CSS/JQuery Extractor: captures value from HTML
Then use ${variable_name} in your next request to reference the captured value.

## Hint 3 (Answer)
Example: Capture CSRF token from login response.
Add a JSON Extractor to your Login request:
  Variable Name: csrf_token
  JSON Path: $.csrfToken
Use it in the next request header: X-CSRF-Token: ${csrf_token}
