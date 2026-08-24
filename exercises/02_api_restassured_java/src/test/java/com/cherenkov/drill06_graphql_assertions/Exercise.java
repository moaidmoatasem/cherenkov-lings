package com.cherenkov.drill06_graphql_assertions;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;

/**
 * Drill 06: GraphQL Queries & Aliased Envelopes (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * GitHub GraphQL API Migration (2017)
 * During the migration from REST v3 to GraphQL v4, multiple automated test frameworks failed
 * because they assumed top-level JSON fields rather than navigating through GraphQL nested `data`
 * envelopes and query-specific alias keys.
 *
 * ANTI-PATTERN:
 * Sending a GET request or querying top-level JSON fields without accounting for GraphQL `data` envelopes and field aliases.
 *
 * GOAL:
 * POST a GraphQL query to `/graphql` with field aliasing (`me: user { ... }`) and assert nested fields via JSONPath in Solution.java.
 */
public class Exercise {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Query user profile with naive REST assumption (FLAWED)")
    void testGraphQLWithNaiveRestAssertion() {
        String graphqlPayload = """
            {
                "query": "query { me: user { id name role } }"
            }
            """;

        // ANTI-PATTERN: Fails to assert within the data.me envelope, expecting top-level REST fields
        given()
            .contentType(ContentType.JSON)
            .body(graphqlPayload)
        .when()
            .post("/graphql")
        .then()
            .statusCode(200)
            .body("name", equalTo("sdet_student")); // FAILS: Field is nested under data.me.name
    }
}
