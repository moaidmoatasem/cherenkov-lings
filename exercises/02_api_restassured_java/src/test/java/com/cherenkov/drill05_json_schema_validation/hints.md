# Hints: Drill 05 - JSON Schema Validation

## Hint 1 (Architectural Nudge)
Checking individual JSON properties (`body("name", equalTo("Item"))`) is fragile and fails to catch schema violations like missing mandatory fields or type changes. JSON Schema validates the entire structure in one assertion.

## Hint 2 (API Pattern)
Use REST Assured's `matchesJsonSchemaInClasspath()` from `io.restassured.module.jsv.JsonSchemaValidator`.

## Hint 3 (Code Diff)
```diff
- .body("products[0].id", equalTo("prod-1"))
+ .body(matchesJsonSchemaInClasspath("schemas/product-schema.json"))
```
