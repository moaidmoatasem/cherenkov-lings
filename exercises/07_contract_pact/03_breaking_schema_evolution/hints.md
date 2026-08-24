## Hint 1 (Architectural Nudge)
Additive changes (adding new optional fields) are non-breaking. Modifying or deleting existing fields breaks consumers.

## Hint 2 (API Pattern)
Verify both payload content and response metadata (`count`).

## Hint 3 (Code Diff)
```diff
+ assert "count" in data
+ assert data["count"] == len(data["orders"])
```
