# Drill 01: Idempotency Collisions — Progressive Hints

### Hint 1: Understanding Idempotency Conflicts
Distributed payment gateways and checkout endpoints use client-provided `Idempotency-Key` headers to guarantee that retried network requests do not cause duplicate billing or double fulfillment. When concurrent requests collide or a transaction lock exists for an in-flight key, the server responds with `HTTP 409 Conflict` rather than `HTTP 200 OK`.

### Hint 2: Dynamic Keys vs. Collision Reconciliation
- **Fresh transactions**: Never hardcode static idempotency keys. Always generate dynamic keys using `UUID.randomUUID().toString()`.
- **Retried / colliding transactions**: Handle `409 Conflict` gracefully. Inspect the response payload (`error: "IDEMPOTENCY_CONFLICT"`, `order_id`) to reconcile the transaction rather than blindly expecting `200 OK`.

### Hint 3: Code Diff Solution
```java
// Replace static keys and blind 200 assertion with dynamic keys and conflict reconciliation:
String idempotencyKey = UUID.randomUUID().toString();

Response response = given()
    .contentType(ContentType.JSON)
    .header("Idempotency-Key", idempotencyKey)
    .header("X-Chaos", "idempotency_conflict=true")
    .body(payload)
.when()
    .post("/checkout");

if (response.getStatusCode() == 409) {
    response.then()
        .body("status", equalTo("conflict"))
        .body("error", equalTo("IDEMPOTENCY_CONFLICT"))
        .body("order_id", notNullValue());
} else {
    response.then()
        .statusCode(200)
        .body("status", equalTo("success"));
}
```
