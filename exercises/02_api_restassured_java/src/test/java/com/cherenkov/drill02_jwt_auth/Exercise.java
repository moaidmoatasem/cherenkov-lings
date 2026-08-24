package com.cherenkov.drill02_jwt_auth;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;

/**
 * Drill 02: JWT Refresh Filters (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * Twitter / X OAuth Expiration Cascade (2022)
 * Automated integration test suites and backend workers crashed when short-lived access tokens expired
 * mid-session under load without transparent automatic token refresh handlers.
 *
 * ANTI-PATTERN:
 * This test acquires an authentication token with no automatic refresh handling.
 * When the server issues an expired token (simulating mid-session token expiry),
 * subsequent requests to protected endpoints fail with HTTP 401 Unauthorized.
 *
 * GOAL:
 * Observe the failure when the token expires mid-session, then build a transparent
 * REST Assured Filter in Solution.java to catch 401 and refresh the token automatically.
 */
public class Exercise {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Fetch user profile with expired token fails without refresh filter (FLAWED)")
    void testGetProfileWithExpiredToken() {
        // Authenticate with chaos directive causing immediate token expiry
        String expiredToken = given()
            .contentType(ContentType.JSON)
            .header("X-Chaos", "token_expire=immediate")
            .body("""
                {
                    "username": "sdet_student",
                    "password": "secret_password"
                }
                """)
        .when()
            .post("/auth/login")
        .then()
            .statusCode(200)
            .extract().path("access_token");

        // ANTI-PATTERN: Calling protected endpoint with expired token without an interceptor / filter
        given()
            .header("Authorization", "Bearer " + expiredToken)
        .when()
            .get("/auth/me")
        .then()
            .statusCode(200) // FAILS: Micro-Crucible returns HTTP 401 Unauthorized
            .body("username", equalTo("sdet_student"))
            .body("role", equalTo("sdet_engineer"));
    }
}
