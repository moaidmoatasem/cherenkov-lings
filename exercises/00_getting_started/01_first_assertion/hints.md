# Hints: Drill 01 - Your First Assertion

## Hint 1 (Architectural Nudge)
Save this file. The watcher notices, runs it through pytest, and prints a scorecard back at you within moments -- that loop, save-then-read-the-result, is the entire mechanic you will use for every drill in this platform, in every language it covers. Right now the result it prints is a failure, and that is fine: a failure is information, not a penalty. Read what pytest actually says before changing anything. It does not just say "failed" -- it shows you the exact values on both sides of the `==` that didn't match.

## Hint 2 (API Pattern)
pytest rewrites plain `assert` statements so that a failure shows you both operands, not just the word `False`. `assert total_price(unit_price=4.00, quantity=3) == 13.00` fails by printing something like `assert 12.0 == 13.0` -- the left side is what your code actually produced; the right side is what the test expected. When those two numbers disagree, one of them is wrong. Here, the code is right and the expectation was written wrong on purpose. Trust the number pytest computed over the number already sitting in the assertion.

## Hint 3 (Code Diff)
```diff
- assert total_price(unit_price=4.00, quantity=3) == 13.00
+ assert total_price(unit_price=4.00, quantity=3) == 12.00
```
Four dollars, three times, is twelve -- not thirteen. Save the file again and confirm the watcher now reports a pass.
