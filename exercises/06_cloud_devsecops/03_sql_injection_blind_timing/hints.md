## Hint 1 (Architectural Nudge)
SQL injection occurs when user-supplied input is directly concatenated into database query strings instead of bound as parameters.

## Hint 2 (API Pattern)
Verify that input payloads like `1 OR 1=1` are treated as literal strings rather than executable SQL syntax.

## Hint 3 (Code Diff)
```diff
- res = requests.get(f"http://localhost:8081/api/security/user-lookup?user_id={user_id}")
+ res_sqli = requests.get("http://localhost:8081/api/security/user-lookup?user_id=1%20OR%201=1")
+ assert res_sqli.json()["id"] == "1 OR 1=1"
```
