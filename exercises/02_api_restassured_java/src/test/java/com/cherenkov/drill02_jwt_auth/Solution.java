package com.cherenkov.drill02_jwt_auth;

import io.restassured.RestAssured;
import io.restassured.filter.Filter;
import io.restassured.filter.FilterContext;
import io.restassured.http.ContentType;
import io.restassured.response.Response;
import io.restassured.specification.FilterableRequestSpecification;
import io.restassured.specification.FilterableResponseSpecification;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.equalTo;

/**
 * Drill 02: JWT Refresh Filters (Resilient Solution)
 *
 * RESILIENT PATTERN:
 * Implements a transparent REST Assured `Filter` (interceptor) that catches HTTP 401 Unauthorized,
 * performs an automatic token refresh login, updates the Authorization header, and replays
 * the original request seamlessly.
 */
public class Solution {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    /**
     * Transparent JWT Refresh Filter for REST Assured.
     */
    public static class JwtRefreshFilter implements Filter {
        private String token;
        private final String username;
        private final String password;

        public JwtRefreshFilter(String initialToken, String username, String password) {
            this.token = initialToken;
            this.username = username;
            this.password = password;
        }

        public String getToken() {
            return token;
        }

        @Override
        public Response filter(FilterableRequestSpecification requestSpec,
                               FilterableResponseSpecification responseSpec,
                               FilterContext ctx) {
            if (token != null && !token.isEmpty()) {
                requestSpec.replaceHeader("Authorization", "Bearer " + token);
            }

            Response response = ctx.next(requestSpec, responseSpec);

            if (response.getStatusCode() == 401) {
                // Transparently obtain a fresh token without chaos expiration
                String freshToken = given()
                    .contentType(ContentType.JSON)
                    .body(String.format("{\"username\":\"%s\",\"password\":\"%s\"}", username, password))
                .when()
                    .post("/auth/login")
                .then()
                    .statusCode(200)
                    .extract().path("access_token");

                this.token = freshToken;
                requestSpec.replaceHeader("Authorization", "Bearer " + this.token);
                // Replay original request with refreshed token using FilterContext send
                response = ctx.send(requestSpec);
            }

            return response;
        }
    }

    @Test
    @DisplayName("Fetch user profile succeeds transparently using JwtRefreshFilter when token expires")
    void testGetProfileWithRefreshFilter() {
        // Step 1: Obtain an initial expired token simulating mid-session expiration
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

        // Step 2: Configure the resilient filter
        JwtRefreshFilter refreshFilter = new JwtRefreshFilter(expiredToken, "sdet_student", "secret_password");

        // Step 3: Execute request to protected endpoint - filter transparently catches 401 and refreshes
        given()
            .filter(refreshFilter)
        .when()
            .get("/auth/me")
        .then()
            .statusCode(200)
            .body("username", equalTo("sdet_student"))
            .body("role", equalTo("sdet_engineer"))
            .body("status", equalTo("active"));
    }
}
