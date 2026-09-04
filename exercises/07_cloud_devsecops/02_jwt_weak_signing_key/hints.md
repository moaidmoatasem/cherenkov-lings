# Hints: Drill 02 - JWT Weak Signing Algorithm

## Hint 1 (Architectural Nudge)
The `alg` field in a JWT header is attacker-controlled data. If the server trusts whatever algorithm the token claims, an attacker can set `alg: "none"`, drop the signature entirely, and forge any payload -- including `{"role": "admin"}`.

## Hint 2 (API Pattern)
Build a token whose header is `{"alg": "none", "typ": "JWT"}` and whose signature segment is empty, send it to `/auth/me` as a Bearer token, and assert the server rejects it with HTTP 401 rather than trusting the forged claims.

## Hint 3 (Code Diff)
```diff
+ def _make_alg_none_token(payload: dict) -> str:
+     def b64url(data: bytes) -> str:
+         return base64.urlsafe_b64encode(data).rstrip(b"=").decode()
+     header = b64url(json.dumps({"alg": "none", "typ": "JWT"}).encode())
+     body = b64url(json.dumps(payload).encode())
+     return f"{header}.{body}."
+
+ forged = requests.get(
+     "http://localhost:8081/auth/me",
+     headers={"Authorization": f"Bearer {_make_alg_none_token({'sub': 'attacker', 'role': 'admin'})}"},
+ )
+ assert forged.status_code == 401
```
