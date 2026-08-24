package com.cherenkov.drill07_request_spec_reuse;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Drill 07: RequestSpecBuilder & Shared Specification Reuse (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * Uber Microservice Auth Header Modernization (2019)
 * A security protocol upgrade required injecting HMAC signatures and tenant tracking headers
 * across all microservice API calls. Teams without centralized RequestSpecifications had to
 * manually update over 4,200 individual test files, delaying security compliance by weeks.
 *
 * ANTI-PATTERN:
 * Copy-pasting baseUrl, content type, auth tokens, and common headers across every single test method.
 *
 * GOAL:
 * Encapsulate reusable configuration into a RequestSpecification built via RequestSpecBuilder in Solution.java.
 */
public class Exercise {

    private static final String BASE_URL = "http://localhost:8081";
    private static final String AUTH_TOKEN = "sdet-valid-test-token";

    @Test
    @DisplayName("Get products with duplicated request headers (BOILERPLATE)")
    void testGetProductsWithDuplicatedConfig() {
        // ANTI-PATTERN: Manually duplicating base URI, content type, and auth headers
        given()
            .baseUri(BASE_URL)
            .contentType(ContentType.JSON)
            .header("Authorization", "Bearer " + AUTH_TOKEN)
            .header("X-Client-Version", "2.0.0")
        .when()
            .get("/products")
        .then()
            .statusCode(200)
            .body("total", notNullValue());
    }

    @Test
    @DisplayName("Get balance with duplicated request headers (BOILERPLATE)")
    void testGetBalanceWithDuplicatedConfig() {
        // ANTI-PATTERN: Duplicated headers again
        given()
            .baseUri(BASE_URL)
            .contentType(ContentType.JSON)
            .header("Authorization", "Bearer " + AUTH_TOKEN)
            .header("X-Client-Version", "2.0.0")
            .queryParam("account_id", "ACC-001")
        .when()
            .get("/balance")
        .then()
            .statusCode(200)
            .body("account_id", equalTo("ACC-001"));
    }
}
