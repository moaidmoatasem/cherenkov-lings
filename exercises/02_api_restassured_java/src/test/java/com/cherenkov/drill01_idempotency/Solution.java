package com.cherenkov.drill01_idempotency;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import io.restassured.response.Response;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.UUID;

import static io.restassured.RestAssured.given;
import static org.assertj.core.api.Assertions.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.startsWith;

/**
 * Drill 01: Idempotency Collisions (Resilient Solution)
 *
 * RESILIENT PATTERN:
 * 1. Generates unique dynamic UUID idempotency keys for fresh checkout requests.
 * 2. Properly handles HTTP 409 Conflict responses on duplicate/colliding retries.
 * 3. Reconciles existing order state from the conflict response without creating duplicate charges.
 */
public class Solution {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Fresh checkout with dynamic UUID idempotency key succeeds with 200 OK")
    void testFreshCheckoutWithDynamicKey() {
        String idempotencyKey = "key-" + UUID.randomUUID();
        String payload = """
            {
                "item_id": "course-sdet-masterclass",
                "customer_name": "Ada Lovelace",
                "payment_method": "credit_card"
            }
            """;

        given()
            .contentType(ContentType.JSON)
            .header("Idempotency-Key", idempotencyKey)
            .body(payload)
        .when()
            .post("/checkout")
        .then()
            .statusCode(200)
            .body("status", equalTo("success"))
            .body("order_id", startsWith("ORD-"))
            .body("total_charged", equalTo(160.92f));
    }

    @Test
    @DisplayName("Resiliently handle 409 Conflict on idempotency collision and reconcile order state")
    void testIdempotencyCollisionHandling() {
        String idempotencyKey = "key-reconciled-" + UUID.randomUUID();
        String payload = """
            {
                "item_id": "course-sdet-masterclass",
                "customer_name": "Ada Lovelace",
                "payment_method": "credit_card"
            }
            """;

        // Issue request under chaos idempotency conflict simulation
        Response response = given()
            .contentType(ContentType.JSON)
            .header("Idempotency-Key", idempotencyKey)
            .header("X-Chaos", "idempotency_conflict=true")
            .body(payload)
        .when()
            .post("/checkout");

        int statusCode = response.getStatusCode();
        if (statusCode == 409) {
            // Reconcile conflict: server already processed or detected collision for this key
            String status = response.jsonPath().getString("status");
            String error = response.jsonPath().getString("error");
            String orderId = response.jsonPath().getString("order_id");

            assertThat(status).isEqualTo("conflict");
            assertThat(error).isEqualTo("IDEMPOTENCY_CONFLICT");
            assertThat(orderId).isNotEmpty().startsWith("ORD-");
        } else {
            // Fresh execution succeeded
            assertThat(statusCode).isEqualTo(200);
            assertThat(response.jsonPath().getString("status")).isEqualTo("success");
        }
    }
}
