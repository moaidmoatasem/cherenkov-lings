package com.cherenkov.drill07_request_spec_reuse;

import io.restassured.builder.RequestSpecBuilder;
import io.restassured.builder.ResponseSpecBuilder;
import io.restassured.filter.log.LogDetail;
import io.restassured.http.ContentType;
import io.restassured.specification.RequestSpecification;
import io.restassured.specification.ResponseSpecification;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.notNullValue;

/**
 * SDET Resilient Pattern: Centralized Request & Response Specifications
 * RequestSpecBuilder standardizes common HTTP configurations (baseUri, headers,
 * auth tokens, logging) across the entire test suite, enabling zero-effort global updates.
 */
public class Solution {

    private static RequestSpecification defaultRequestSpec;
    private static ResponseSpecification defaultResponseSpec;

    @BeforeAll
    static void setupSpecs() {
        String baseUrl = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
        String authToken = "sdet-valid-test-token";

        defaultRequestSpec = new RequestSpecBuilder()
            .setBaseUri(baseUrl)
            .setContentType(ContentType.JSON)
            .addHeader("Authorization", "Bearer " + authToken)
            .addHeader("X-Client-Version", "2.0.0")
            .log(LogDetail.URI)
            .build();

        defaultResponseSpec = new ResponseSpecBuilder()
            .expectStatusCode(200)
            .expectContentType(ContentType.JSON)
            .build();
    }

    @Test
    @DisplayName("Get products reusing shared request and response specifications (RESILIENT)")
    void testGetProductsWithSharedSpec() {
        given()
            .spec(defaultRequestSpec)
        .when()
            .get("/products")
        .then()
            .spec(defaultResponseSpec)
            .body("total", notNullValue());
    }

    @Test
    @DisplayName("Get balance reusing shared request and response specifications (RESILIENT)")
    void testGetBalanceWithSharedSpec() {
        given()
            .spec(defaultRequestSpec)
            .queryParam("account_id", "ACC-001")
        .when()
            .get("/balance")
        .then()
            .spec(defaultResponseSpec)
            .body("account_id", equalTo("ACC-001"));
    }
}
