package com.cherenkov.drill03_kafka_lag;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;

/**
 * Drill 03: Kafka Lag Assertions (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * Revolut Async Ledger Balance Drift (2021)
 * High-frequency ledger transfers processed asynchronously through message queues caused immediate
 * read-after-write assertions to fail because read replicas lagged behind the Kafka ingestion pipeline.
 *
 * ANTI-PATTERN:
 * This test naively relies on a hardcoded, brittle Thread.sleep(100) to wait for an
 * asynchronous ledger transfer to settle. Because the message queue introduces 1500ms
 * of lag, the sleep finishes prematurely and the balance assertion fails.
 *
 * GOAL:
 * Observe the failure due to asynchronous eventual consistency, then replace Thread.sleep
 * with event-driven polling using Awaitility in Solution.java.
 */
public class Exercise {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Assert ledger balance after transfer using static sleep (FLAWED)")
    void testTransferSettlementWithStaticSleep() throws InterruptedException {
        // Step 1: Reset ledger accounts (ACC-001 = $1000.00, ACC-002 = $500.00)
        given()
        .when()
            .post("/reset")
        .then()
            .statusCode(200);

        // Step 2: Queue transfer of $250.00 with 1500ms Kafka lag
        given()
            .contentType(ContentType.JSON)
            .header("X-Chaos", "kafka_lag=1500ms")
            .body("""
                {
                    "from_account": "ACC-001",
                    "to_account": "ACC-002",
                    "amount": 250.00
                }
                """)
        .when()
            .post("/transfer")
        .then()
            .statusCode(200)
            .body("status", equalTo("QUEUED_LEDGER"));

        // ANTI-PATTERN: Fixed 100ms sleep is insufficient for a 1500ms asynchronous queue lag
        Thread.sleep(100);

        // Step 3: Check balance immediately - fails because transfer is still pending
        given()
            .queryParam("account_id", "ACC-001")
        .when()
            .get("/balance")
        .then()
            .statusCode(200)
            .body("balance", equalTo(750.0f)) // FAILS: Still 1000.0f
            .body("pending_count", equalTo(0)); // FAILS: Still 1
    }
}
