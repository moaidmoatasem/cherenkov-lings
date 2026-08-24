package com.cherenkov.drill05_json_schema_validation;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;

/**
 * Drill 05: JSON Schema Validation (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * Twitter / X API v2 Optional Field Stripping (2022)
 * When backend services refactored response payloads, individual field assertions passed for the
 * few fields explicitly checked, but failed to catch missing mandatory nested fields and data type
 * mutations, causing widespread crashes in downstream mobile and web clients.
 *
 * ANTI-PATTERN:
 * Asserting only individual fields with hardcoded equality instead of validating the overall JSON Schema contract.
 *
 * GOAL:
 * Replace brittle field-by-field assertions with io.restassured.module.jsv.JsonSchemaValidator in Solution.java.
 */
public class Exercise {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Verify product response with brittle individual field checks (FLAWED)")
    void testProductBrittleFieldChecks() {
        // ANTI-PATTERN: Verifying only a couple of hardcoded fields; misses structural regressions and type mismatches
        given()
            .contentType(ContentType.JSON)
            .queryParam("page", 1)
            .queryParam("per_page", 5)
        .when()
            .get("/products")
        .then()
            .statusCode(200)
            .body("page", equalTo(1))
            .body("products[0].id", equalTo("prod-1")); // Brittle: ignores required schema properties, data types, and total fields
    }
}
