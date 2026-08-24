# Hints: Drill 04 - Pagination Boundary

## Hint 1 (Architectural Nudge)
Testing only the first page of a paginated API leaves downstream pages, boundary conditions, and total count calculations untested. Always traverse the full pagination cursor or index.

## Hint 2 (API Pattern)
Use a `do-while` loop in REST Assured that inspects `response.path("total_pages")` and increments `page` until all pages are consumed.

## Hint 3 (Code Diff)
```diff
- given().queryParam("page", 1).get("/products").then().body("products", hasSize(2));
+ int page = 1;
+ List<Map<String, Object>> allItems = new ArrayList<>();
+ do {
+   Response res = given().queryParam("page", page).get("/products");
+   allItems.addAll(res.path("products"));
+   page++;
+ } while (page <= res.path("total_pages"));
+ assertThat(allItems.size(), equalTo(res.path("total")));
```
