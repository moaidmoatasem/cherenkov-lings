package com.cherenkov.drill06_graphql_assertions;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.notNullValue;

/**
 * SDET Resilient Pattern: GraphQL Query & Aliased Response Assertions
 * Navigates the GraphQL `data` response envelope using JSONPath expressions
 * targeting the alias identifier (`data.me.name`, `data.me.role`).
 */
public class Solution {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Query user profile via GraphQL with aliased envelope assertions (RESILIENT)")
    void testGraphQLWithAliasedEnvelopeAssertions() {
        String graphqlPayload = """
            {
                "query": "query { me: user { id name email role status } }"
            }
            """;

        given()
            .contentType(ContentType.JSON)
            .body(graphqlPayload)
        .when()
            .post("/graphql")
        .then()
            .statusCode(200)
            .contentType(ContentType.JSON)
            .body("data.me.id", notNullValue())
            .body("data.me.name", equalTo("sdet_student"))
            .body("data.me.email", equalTo("student@cherenkov.qa"))
            .body("data.me.role", equalTo("sdet_engineer"))
            .body("data.me.status", equalTo("active"));
    }
}
