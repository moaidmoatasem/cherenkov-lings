# Hints: Drill 01 - What is an Automated Test?

## Hint 1 (Concept)
You already know how to test manually: you open the app, do something, and CHECK that the result is correct. An automated test does the same thing in code. The CHECK is the `assert` statement.

## Hint 2 (Pattern)
The Python `assert` statement works like this:
  assert actual_value == expected_value, "Error message shown if it fails"
If the condition is True, the test passes silently. If False, pytest marks it as FAILED and shows you the values.

## Hint 3 (Code Diff)
Replace: pass
With:    assert total == 33.0, f"Expected 33.0 but got {total}"
