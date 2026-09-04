# Theoretical Context: Algorithm Confusion and JWT Signature Forgery

## Disclosed Vulnerability: the `alg: "none"` bypass in JWT libraries (2015)

In March 2015 a review of JSON Web Token implementations found the same flaw in
libraries across several languages: the token itself was allowed to name the
algorithm used to verify it. RFC 7519 defines `"none"` as a valid `alg` value
for unsecured tokens, and affected libraries honoured it during verification.
An attacker could therefore take a legitimate token, rewrite the payload to
`{"role": "admin"}`, set the header to `{"alg": "none"}`, drop the signature
entirely, and have the server accept it as valid.

The same class of bug had a second form: a server holding an RSA public key for
`RS256` verification could be sent a token claiming `HS256`, and would then use
that public key -- a value the attacker also has -- as the HMAC secret. Both
variants come from the same mistake, which is letting untrusted input choose the
verification path.

The lesson is that a token's header is attacker-controlled data. The set of
acceptable algorithms belongs in the server's configuration, never in the token.

## The Underlying Mechanism

JSON Web Tokens (JWT, RFC 7519) are compact, URL-safe tokens used for stateless authorization across distributed architectures:

1. **JWT Cryptographic Structure**:
   `HEADER.PAYLOAD.SIGNATURE`
   - Header: Algorithm specification (e.g., `{"alg": "HS256", "typ": "JWT"}`)
   - Payload: The claims set (e.g., `{"sub": "123", "role": "user", "exp": 1700000000}`)
   - Signature: HMAC-SHA256 hash calculated as `HMAC-SHA256(Base64URL(Header) + "." + Base64URL(Payload), SecretKey)`
2. **The Algorithm Confusion Vulnerability**:
   - Verification code that reads `alg` from the token and trusts it lets the caller choose how -- or whether -- the signature is checked. With `alg: "none"` accepted, the signature is not checked at all.
   - The fix is an allowlist held by the server: decode with an explicit `algorithms=["HS256"]` (or `["RS256"]`) argument, so a token naming anything else is rejected before its claims are read.
3. **The Weak Symmetric Key Vulnerability**:
   - When developers use predictable or short secret keys (such as `"secret"`, `"password"`, `"jwt_key_123"`), attackers can capture any valid token and perform offline dictionary or brute-force cracking using high-speed GPU tools (e.g., Hashcat or John the Ripper) at rates exceeding billions of hashes per second.
   - Once the secret key is discovered, the attacker can forge custom JWT tokens with `{"role": "admin"}` and sign them locally. The server's signature verification will pass completely.
4. **Resilient DevSecOps Standards**:
   - **High-Entropy Symmetric Keys**: HS256 secrets must have a minimum of 256 bits (32 bytes) of cryptographically secure random entropy.
   - **Asymmetric Cryptography (RS256 / ES256)**: Sign tokens with a private RSA/ECDSA key that never leaves the auth server; verify tokens with a public key.
   - **Reject Insecure Algorithms**: Explicitly disallow `alg: "none"` and enforce strict algorithm whitelisting during token verification.

```
[Insecure Anti-Pattern: Weak Symmetric Secret Cracking]
Victim User JWT: [eyJ...].[{"role":"user"}].[Signature]
       │
       ▼ (Attacker Captures Token)
[GPU Offline Brute-Force: Hashcat] ──► Cracked Secret in 0.4s: "secret123"
       │
       ▼ (Attacker Forges Admin Token)
Forged JWT: [eyJ...].[{"role":"admin"}].[Signature Signed with "secret123"]
       │
       ▼
Target API Server: Verifies Signature (Valid!) ──► PRIVILEGE ESCALATION ❌

[Resilient DevSecOps Pattern: Cryptographically Secure Asymmetric Keys]
Auth Server (Holds RS256 Private Key) ──► Signs JWT with 2048-bit RSA Private Key
                                                 │
                                                 ▼
API Server (Holds Public Key) ──────────► Verifies Signature with RS256 Public Key
                                          (Impossible to Forge without Private Key!) ✅
```

Auditing token verification logic and enforcing high-entropy cryptographic keys prevents offline signature recovery and protects distributed systems against administrative privilege escalation.

You will now simulate this in the Crucible: audit JWT authentication handlers so that `alg: "none"` is rejected, only an explicit allowlist of algorithms is accepted, and signing keys carry real entropy.
