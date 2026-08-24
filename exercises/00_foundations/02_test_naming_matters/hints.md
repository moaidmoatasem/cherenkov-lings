# Hints: Drill 02 - Test Naming Matters

## Hint 1 (Concept)
When a CI pipeline fails at 2am and wakes someone up, the test name is the first thing they read. test_1 tells you nothing. test_discount_reduces_price_by_percentage tells you the business rule that broke.

## Hint 2 (Pattern)
Good test names follow this formula: test_{what_the_system_does}_{under_what_condition}
Examples: test_login_fails_with_wrong_password, test_cart_total_includes_tax

## Hint 3 (Code Diff)
Rename: test_1  -->  test_discount_reduces_price_by_percentage
Rename: test_2  -->  test_zero_discount_returns_original_price
Rename: test_3  -->  test_negative_discount_raises_value_error
