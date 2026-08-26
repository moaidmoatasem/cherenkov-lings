# Hints: Drill 01 - UI Test vs API Test

## Hint 1 (Decision Framework)
Ask: "Where does this business rule live?"
  Backend logic (calculation, validation, auth) -> API test
  User interaction, visual layout, accessibility -> UI test
  User journey across multiple pages -> E2E UI test (use sparingly, they are expensive)

## Hint 2 (Pattern)
The Test Pyramid: many unit tests at the base, fewer API tests in the middle, very few E2E UI tests at the top.
A checkout total calculation belongs at the API layer -- fast, reliable, no browser needed.

## Hint 3 (Code Diff)
Replace: Playwright page.goto + form filling (8s)
With:    requests.post("http://localhost:8081/checkout", json={"item_id": "item-1", "quantity": 2}, timeout=5)
