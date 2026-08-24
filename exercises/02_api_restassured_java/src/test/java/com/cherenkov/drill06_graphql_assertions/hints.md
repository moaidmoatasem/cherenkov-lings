# Hints: Drill 06 - GraphQL Assertions

## Hint 1 (Architectural Nudge)
GraphQL responses always wrap results in a top-level `data` object (or `errors` on failure). If a query uses an alias like `me: user { ... }`, the response key is `me`, not `user`.

## Hint 2 (API Pattern)
Use JSONPath in REST Assured to extract nested fields under the alias:
`.body("data.me.name", equalTo("sdet_student"))`

## Hint 3 (Code Diff)
```diff
- .body("name", equalTo("sdet_student"))
+ .body("data.me.name", equalTo("sdet_student"))
+ .body("data.me.role", equalTo("sdet_engineer"))
```
