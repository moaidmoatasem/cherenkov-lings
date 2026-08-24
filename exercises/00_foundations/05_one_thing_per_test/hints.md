# Hints: Drill 05 - One Thing Per Test

## Hint 1 (Concept)
A test with 5 assertions is like a manual test case with 20 steps and one PASS/FAIL result. When it fails on step 12, you must re-run everything from step 1. Automated tests should be cheap to run, so make each one tiny and focused.

## Hint 2 (Pattern)
The rule: one test function = one reason to fail. If you find yourself writing "and also..." you probably need a second test function.

## Hint 3 (Code Diff)
Split test_search_api() into 5 functions:
  def test_search_returns_200_for_empty_query
  def test_search_returns_200_for_valid_query
  def test_search_results_field_present_in_response
  def test_search_returns_at_least_one_result
  def test_search_echoes_query_in_response
Each function has exactly one assert statement.
