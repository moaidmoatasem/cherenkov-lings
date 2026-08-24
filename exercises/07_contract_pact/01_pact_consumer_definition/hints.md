## Hint 1 (Architectural Nudge)
Consumer-Driven Contracts formalize the shape, types, and required fields that a consumer depends on.

## Hint 2 (API Pattern)
Assert on types (`isinstance(data['orders'], list)`) and mandatory schema keys (`id`, `total`, `status`).

## Hint 3 (Code Diff)
```diff
- assert res.status_code == 200
+ assert "orders" in data
+ assert "id" in order and "total" in order
```
