# Hints: Drill 07 - Request & Response Specification Reuse

## Hint 1 (Architectural Nudge)
Duplicating baseUrl, auth headers, and common settings in every test leads to massive maintenance debt. Centralize these configs using REST Assured's specification builders.

## Hint 2 (API Pattern)
Use `RequestSpecBuilder` and `ResponseSpecBuilder` in a `@BeforeAll` setup method to build reusable `RequestSpecification` and `ResponseSpecification` objects.

## Hint 3 (Code Diff)
```diff
- given().baseUri("http://localhost:8081").header("Authorization", "Bearer ...").get("/products").then().statusCode(200);
+ static RequestSpecification spec = new RequestSpecBuilder().setBaseUri(baseUrl).addHeader("Authorization", "...").build();
+ given().spec(spec).get("/products").then().statusCode(200);
```
