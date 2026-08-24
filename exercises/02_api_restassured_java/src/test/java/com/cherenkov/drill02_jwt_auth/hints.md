# Drill 02: JWT Refresh Filters — Progressive Hints

### Hint 1: Dealing with Mid-Session Expiration
In long-running test suites or real-world API interactions, authentication tokens expire asynchronously. Hardcoding manual re-login before every API call clutters test code and fails when token TTL varies across environments. Transparent interceptors (filters) inspect incoming responses and automatically renew credentials without breaking the calling test.

### Hint 2: The REST Assured `Filter` Interface
REST Assured provides the `io.restassured.filter.Filter` interface:
```java
public Response filter(FilterableRequestSpecification requestSpec,
                       FilterableResponseSpecification responseSpec,
                       FilterContext ctx) {
    Response response = ctx.next(requestSpec, responseSpec);
    if (response.getStatusCode() == 401) {
        // Refresh token, update requestSpec header, and replay via ctx.send(requestSpec)
    }
    return response;
}
```

### Hint 3: Code Diff Solution
```java
public static class JwtRefreshFilter implements Filter {
    private String token;
    private final String username;
    private final String password;

    public JwtRefreshFilter(String initialToken, String username, String password) {
        this.token = initialToken;
        this.username = username;
        this.password = password;
    }

    @Override
    public Response filter(FilterableRequestSpecification req, FilterableResponseSpecification res, FilterContext ctx) {
        req.replaceHeader("Authorization", "Bearer " + token);
        Response response = ctx.next(req, res);
        if (response.getStatusCode() == 401) {
            String freshToken = given()
                .contentType(ContentType.JSON)
                .body("{\"username\":\"" + username + "\",\"password\":\"" + password + "\"}")
            .when()
                .post("/auth/login")
            .then()
                .statusCode(200)
                .extract().path("access_token");

            this.token = freshToken;
            req.replaceHeader("Authorization", "Bearer " + this.token);
            response = ctx.send(req);
        }
        return response;
    }
}
```
Attach with `.filter(new JwtRefreshFilter(expiredToken, "sdet_student", "secret_password"))`.
