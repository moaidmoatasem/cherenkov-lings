package com.cherenkov.drill05_json_schema_validation;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import io.restassured.module.jsv.JsonSchemaValidator;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static io.restassured.module.jsv.JsonSchemaValidator.matchesJsonSchemaInClasspath;

/**
 * SDET Resilient Pattern: Complete JSON Schema Validation
 * Validates the entire payload contract against a formal JSON Schema definition,
 * guaranteeing required keys, correct data types, and array structure.
 */
public class Solution {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Verify product response against JSON schema contract (RESILIENT)")
    void testProductJsonSchemaContract() {
        given()
            .contentType(ContentType.JSON)
            .queryParam("page", 1)
            .queryParam("per_page", 5)
        .when()
            .get("/products")
        .then()
            .statusCode(200)
            .contentType(ContentType.JSON)
            .body(matchesJsonSchemaInClasspath("schemas/product-schema.json"));
    }
}
