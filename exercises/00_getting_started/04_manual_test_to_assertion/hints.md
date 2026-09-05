# Hints: Drill 04 - From Manual Test Case to Automated Assertion

## Hint 1 (Architectural Nudge)
TC-014's "Expected result" line is already, in prose, exactly what an assertion needs to say: one value should equal the sum of two others. The translation work is not inventing a new check -- it's finding where the manual tester's three read-the-page steps map onto the three fields the API already returns (`subtotal`, `tax`, `total`), and writing the comparison the human was already doing in their head every time they ran this by hand.

## Hint 2 (API Pattern)
Resist the temptation to run this once, see `{"total": 160.92}`, and write `assert checkout["total"] == 160.92`. That check would pass today and pass for the wrong reason tomorrow -- it verifies a hardcoded number matches itself, not that the total is actually *computed correctly* from the subtotal and tax. Compute the expected value from the response's own fields (`checkout["subtotal"] + checkout["tax"]`) so the assertion still means something the day the price changes. Round both sides to two decimal places before comparing floats, since `subtotal + tax` can land on `160.91999999999998` for reasons that have nothing to do with a bug.

## Hint 3 (Code Diff)
```diff
  def test_checkout_total_includes_tax():
      checkout = requests.get("http://localhost:8081/checkout").json()
-     pass
+     expected_total = round(checkout["subtotal"] + checkout["tax"], 2)
+     assert round(checkout["total"], 2) == expected_total, (
+         f"Expected total {expected_total} (subtotal + tax), got {checkout['total']}"
+     )
```
