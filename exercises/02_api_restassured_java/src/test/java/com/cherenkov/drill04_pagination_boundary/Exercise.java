package com.cherenkov.drill04_pagination_boundary;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static io.restassured.RestAssured.given;
import static org.hamcrest.Matchers.*;

/**
 * Drill 04: Pagination Boundary Traversals (Broken Anti-Pattern)
 *
 * PRODUCTION STORY:
 * BestBuy Inventory Sync Omission (2020)
 * A critical pre-holiday integration test only asserted against page 1 of the product catalog.
 * A pagination boundary bug in the backend dropped subsequent pages (2 through 40), resulting
 * in phantom inventory shortages for thousands of SKUs.
 *
 * ANTI-PATTERN:
 * This test only queries page 1 with a small per_page limit and asserts that the whole catalog
 * has been validated, ignoring total_pages and pagination boundaries.
 *
 * GOAL:
 * Traverse through all paginated results iteratively in Solution.java until all items are verified.
 */
public class Exercise {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Verify product catalog with single-page fetch (FLAWED)")
    void testCatalogSinglePageOnly() {
        // ANTI-PATTERN: Querying only page 1 with per_page=2 and assuming the full catalog is validated
        given()
            .contentType(ContentType.JSON)
            .queryParam("page", 1)
            .queryParam("per_page", 2)
        .when()
            .get("/products")
        .then()
            .statusCode(200)
            .body("page", equalTo(1))
            .body("products", hasSize(2)) // Fails to verify the remaining products on subsequent pages!
            .body("total", greaterThan(2));
    }
}
