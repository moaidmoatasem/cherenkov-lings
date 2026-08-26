## Hint 1 (Architectural Nudge)
Server-Side Request Forgery (SSRF) exploits server-side URL fetchers to reach internal infrastructure or cloud link-local addresses.

## Hint 2 (API Pattern)
Assert on HTTP 403 Forbidden and check for `error: "SSRF_ATTEMPT_PREVENTED"`.

## Hint 3 (Code Diff)
```diff
+ res_ssrf = requests.post("http://localhost:8081/api/security/fetch-url", json={"url": "http://169.254.169.254/latest/meta-data/"})
+ assert res_ssrf.status_code == 403
```
