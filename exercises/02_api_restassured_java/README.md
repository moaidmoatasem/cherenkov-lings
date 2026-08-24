# Cherenkov Lings — REST Assured (Java) Track

This module contains API resilience and security testing drills implemented with **REST Assured**, **JUnit 5 Jupiter**, **AssertJ**, and **Awaitility**.

## Drills

1. **Drill 01: Idempotency Collisions** (`com.cherenkov.drill01_idempotency`)
   - `Exercise.java`: Demonstrates anti-pattern naively expecting 200 OK without idempotency conflict handling.
   - `Solution.java`: Resilient implementation using dynamic UUIDs and 409 Conflict reconciliation.
   - `hints.md`: Progressive guidance on idempotency keys.

2. **Drill 02: JWT Refresh Filters** (`com.cherenkov.drill02_jwt_auth`)
   - `Exercise.java`: Demonstrates anti-pattern failing with 401 Unauthorized upon mid-session token expiry.
   - `Solution.java`: Resilient implementation using custom REST Assured `Filter` for transparent re-authentication.
   - `hints.md`: Progressive guidance on token refresh interceptors.

3. **Drill 03: Kafka Lag Assertions** (`com.cherenkov.drill03_kafka_lag`)
   - `Exercise.java`: Demonstrates brittle anti-pattern using fixed `Thread.sleep(100)` against eventual consistency.
   - `Solution.java`: Resilient implementation using Awaitility `await().atMost(5, SECONDS).untilAsserted(...)`.
   - `hints.md`: Progressive guidance on asynchronous polling.

## Build and Test

```bash
# Compile test sources
mvn test-compile

# Run specific solution
mvn test -Dtest=com.cherenkov.drill01_idempotency.Solution
mvn test -Dtest=com.cherenkov.drill02_jwt_auth.Solution
mvn test -Dtest=com.cherenkov.drill03_kafka_lag.Solution
```
