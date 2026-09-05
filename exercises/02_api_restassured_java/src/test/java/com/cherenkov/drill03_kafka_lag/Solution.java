package com.cherenkov.drill03_kafka_lag;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static java.util.concurrent.TimeUnit.MILLISECONDS;
import static java.util.concurrent.TimeUnit.SECONDS;
import static org.awaitility.Awaitility.await;
import static org.hamcrest.Matchers.equalTo;

/**
 * Drill 03: Kafka Lag Assertions (Resilient Solution)
 *
 * RESILIENT PATTERN:
 * Replaces static sleeps with Awaitility-based polling assertions (await().atMost(5, SECONDS).untilAsserted(...)).
 * This dynamically waits for the asynchronous message broker to settle the ledger entry,
 * passing reliably regardless of network jitter or CI latency.
 */
public class Solution {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
        RestAssured.config = io.restassured.config.RestAssuredConfig.config()
            .httpClient(io.restassured.config.HttpClientConfig.httpClientConfig()
                .setParam("http.connection.timeout", 5000)
                .setParam("http.socket.timeout", 5000));
    }

    @Test
    @DisplayName("Assert ledger balance after async transfer using Awaitility polling")
    void testTransferSettlementWithAwaitility() {
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

        // RESILIENT PATTERN: Poll until ledger reaches eventual consistency (up to 5 seconds)
        await()
            .atMost(5, SECONDS)
            .pollInterval(100, MILLISECONDS)
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

        // Also verify recipient balance settled to $750.00
        given()
            .queryParam("account_id", "ACC-002")
        .when()
            .get("/balance")
        .then()
            .statusCode(200)
            .body("balance", equalTo(750.0f));
    }
}
