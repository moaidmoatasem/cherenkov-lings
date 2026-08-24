# Theoretical Context: JWT Authentication & Token Expiration Lifecycles

## Production Incident: Twitter OAuth & JWT Expiration Cascade (2022)

In early 2022, Twitter deployed a security hardening update that tightened access token lifecycles across its developer API platform, reducing JSON Web Token (JWT) lifetimes from 24 hours to 15 minutes. Following the deployment, enterprise analytics clients and internal microservices suffered massive authentication cascade failures, throwing thousands of HTTP 401 Unauthorized errors every hour. Automated API integration test suites had previously generated static JWT bearer tokens during suite initialization (`@BeforeAll`) and reused them across hundreds of test methods spanning 30 minutes of runtime. Because the test suites lacked proactive token expiration detection and automated refresh flows, tests failed randomly whenever suite execution exceeded the 15-minute token TTL.

## The Underlying Mechanism

JSON Web Tokens (JWT, RFC 7519) are stateless, cryptographically signed credentials composed of three Base64URL-encoded segments separated by dots: `Header.Payload.Signature`:

1. **Claims and TTL**: The payload contains claims such as `sub` (subject), `iat` (issued at), and `exp` (expiration time as a Unix epoch timestamp).
2. **Stateless Server Validation**: The resource server validates token authenticity by checking the cryptographic signature and evaluating `currentTime >= exp`. If expired, the server rejects the request with HTTP 401.
3. **Test Suite Lifecycle Pitfalls**:
   - Hardcoding static tokens in test properties causes inevitable expiration failures.
   - Long-running automated suites that authenticate once at start-up fail as soon as token lifespan is exceeded.
   - API tests must test both valid authentication and expected token expiration responses (e.g., verifying 401 Unauthorized when tokens expire or signatures are corrupted).

```
[JWT Structure & Expiration Verification]
┌──────────────────┐.┌───────────────────────────────┐.┌────────────────────┐
│ Header: Alg=HS256│ │ Payload: sub=user1, exp=170000│ │ HMAC-SHA256 Sig   │
└──────────────────┘ └───────────────────────────────┘ └────────────────────┘
                                     │
               ┌─────────────────────┴─────────────────────┐
               ▼                                           ▼
      [currentTime < exp]                         [currentTime >= exp]
      Server returns 200 OK                       Server returns 401 Unauthorized
```

Resilient API testing requires encapsulating token acquisition, attaching dynamic `Authorization: Bearer <token>` headers, and verifying proper 401 rejection under expired token conditions.

You will now simulate this in the Crucible: handle dynamic JWT authentication, header injection, and expired token validation using REST Assured.
