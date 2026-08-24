package com.cherenkov.drill04_pagination_boundary;

import io.restassured.RestAssured;
import io.restassured.http.ContentType;
import io.restassured.response.Response;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import static io.restassured.RestAssured.given;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.*;

/**
 * SDET Resilient Pattern: Complete Pagination Boundary Traversal
 * Iteratively traverses all pages until currentPage > totalPages,
 * verifying that the aggregated product count matches the reported total.
 */
public class Solution {

    @BeforeAll
    static void setup() {
        RestAssured.baseURI = System.getProperty("base.url", System.getenv().getOrDefault("TARGET_URL", "http://localhost:8081"));
    }

    @Test
    @DisplayName("Verify product catalog with complete pagination traversal (RESILIENT)")
    void testCatalogCompletePagination() {
        int currentPage = 1;
        int perPage = 2;
        int totalPages = 1;
        int totalReported = 0;
        List<Map<String, Object>> allProducts = new ArrayList<>();

        do {
            Response response = given()
                .contentType(ContentType.JSON)
                .queryParam("page", currentPage)
                .queryParam("per_page", perPage)
            .when()
                .get("/products")
            .then()
                .statusCode(200)
                .extract().response();

            totalPages = response.path("total_pages");
            totalReported = response.path("total");
            List<Map<String, Object>> pageProducts = response.path("products");

            if (pageProducts != null) {
                allProducts.addAll(pageProducts);
            }

            currentPage++;
        } while (currentPage <= totalPages);

        // Assert that every single item across all pages was successfully retrieved
        assertThat("Total collected products must match total count reported by server",
            allProducts.size(), equalTo(totalReported));

        // Assert all products have non-empty IDs and names
        for (Map<String, Object> product : allProducts) {
            assertThat(product.get("id"), notNullValue());
            assertThat(product.get("name"), notNullValue());
        }
    }
}
