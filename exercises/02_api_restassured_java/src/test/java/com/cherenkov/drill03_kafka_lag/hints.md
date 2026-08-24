# Drill 03: Kafka Lag Assertions — Progressive Hints

### Hint 1: Eventual Consistency and Asynchronous Ledgers
In distributed, event-driven architectures (using Kafka, RabbitMQ, or SQS), mutations like financial transfers are published to a message queue and processed asynchronously. Immediate queries to `/balance` will return stale data until consumer workers finish processing the message.

### Hint 2: The Flakiness of `Thread.sleep(...)`
Using fixed `Thread.sleep(100)` or arbitrary sleep intervals is brittle. If network congestion or server load increases queue processing time past the sleep window, the test fails. Conversely, setting large sleeps (e.g. `Thread.sleep(5000)`) unnecessarily slows down test suites.

### Hint 3: Code Diff Solution with Awaitility
Replace `Thread.sleep(...)` with Awaitility's event-driven polling:
```java
import static java.util.concurrent.TimeUnit.SECONDS;
import static org.awaitility.Awaitility.await;

// Wait up to 5 seconds, polling the balance endpoint until the condition succeeds
await()
    .atMost(5, SECONDS)
    .untilAsserted(() -> {
        given()
            .queryParam("account_id", "ACC-001")
        .when()
            .get("/balance")
        .then()
            .statusCode(200)
            .body("balance", equalTo(750.0f))
            .body("pending_count", equalTo(0));
    });
```
