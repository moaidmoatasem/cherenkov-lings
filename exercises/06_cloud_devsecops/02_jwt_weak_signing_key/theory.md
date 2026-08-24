# Theoretical Context: Cryptographic Weak Keys and JWT Signature Forgery

## Production Incident: Capital One AWS S3 Data Breach (2019)

In 2019, a massive security breach at financial giant Capital One exposed the personal financial records and Social Security numbers of over 106 million credit card applicants and customers. Forensic post-incident investigations revealed that alongside a Server-Side Request Forgery (SSRF) vulnerability in an open-source web application firewall, internal microservices utilized weak, predictable symmetric signing keys and default authorization tokens across internal microservice meshes. Once initial network access was obtained, weak authentication key entropy allowed attackers to forge valid internal authorization tokens, escalate privileges across AWS IAM roles, and query sensitive S3 buckets containing customer databases without raising security alarms.

## The Underlying Mechanism

JSON Web Tokens (JWT, RFC 7519) are compact, URL-safe tokens used for stateless authorization across distributed architectures:

1. **JWT Cryptographic Structure**:
   `HEADER.PAYLOAD.SIGNATURE`
   - Header: Algorithm specification (e.g., `{"alg": "HS256", "typ": "JWT"}`)
   - Payload: Claims claims set (e.g., `{"sub": "123", "role": "user", "exp": 1700000000}`)
   - Signature: HMAC-SHA256 hash calculated as `HMAC-SHA256(Base64URL(Header) + "." + Base64URL(Payload), SecretKey)`
2. **The Weak Symmetric Key Vulnerability**:
   - When developers use predictable or short secret keys (such as `"secret"`, `"password"`, `"jwt_key_123"`), attackers can capture any valid token and perform offline dictionary or brute-force cracking using high-speed GPU tools (e.g., Hashcat or John the Ripper) at rates exceeding billions of hashes per second.
   - Once the secret key is discovered, the attacker can forge custom JWT tokens with `{"role": "admin"}` and sign them locally. The server's signature verification will pass completely.
3. **Resilient DevSecOps Standards**:
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

You will now simulate this in the Crucible: audit JWT authentication handlers to detect weak signing keys, reject algorithm confusion attacks, and enforce secure signature verification.
