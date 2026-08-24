package com.cherenkov.drill01_idempotency;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Drill 01: Idempotency Collisions (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * Stripe Duplicate Payment Double-Charge (2017)
 * Network timeout retry storms lacking unique idempotency keys caused thousands of customers
 * to be double-charged during a downstream payment gateway partition.
 *
 * ANTI-PATTERN:
 * This test naively assumes every checkout request and retry succeeds with HTTP 200 OK,
 * ignoring distributed transaction collisions and 409 Conflict responses.
 *
 * GOAL:
 * Observe the failure when the backend returns HTTP 409 Conflict under idempotency chaos,
 * then implement the resilient retry/reconciliation pattern in Solution.java.
 */
public class Exercise {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Checkout with static idempotency key should succeed (FLAWED)")
    void testCheckoutWithStaticKey() {
        String payload = """
            {
                "item_id": "course-sdet-masterclass",
                "customer_name": "Ada Lovelace",
                "payment_method": "credit_card"
            }
            """;

        // ANTI-PATTERN: Naively asserts 200 OK even when idempotency conflict is triggered
        given()
            .contentType(ContentType.JSON)
            .header("Idempotency-Key", "static-checkout-key-999")
            .header("X-Chaos", "idempotency_conflict=true")
            .body(payload)
        .when()
            .post("/checkout")
        .then()
            .statusCode(200) // FAILS: Micro-Crucible returns HTTP 409 Conflict
            .body("status", equalTo("success"))
            .body("order_id", notNullValue());
    }
}
