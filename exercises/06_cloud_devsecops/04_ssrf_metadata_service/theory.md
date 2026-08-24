# Theoretical Context: Server-Side Request Forgery (SSRF) & Cloud Metadata Protection

## Real-World Incident Case Study
In March 2019, Capital One suffered a breach affecting 100 million customer records when attacker Paige Thompson exploited a misconfigured web application firewall (WAF) on AWS. The application accepted user-supplied URLs for server-side fetching, and the attacker replaced the target URL with `http://169.254.169.254/latest/meta-data/iam/security-credentials/`. Because the application ran with an overly permissive IAM role, the metadata service returned temporary AWS credentials that granted access to S3 buckets containing sensitive data. The total cost exceeded $150 million in settlements and remediation. This incident remains the canonical example of SSRF in cloud environments.

## Protocol & Runtime Mechanism
Cloud providers expose instance metadata via a link-local HTTP service at `169.254.169.254` (AWS), `metadata.google.internal` (GCP), and `169.254.169.254` (Azure). These services are accessible from within the instance network but must never be reachable from application code processing untrusted input:

```
  [ Attacker ] ──→ POST /fetch-url {"url": "http://169.254.169.254/latest/meta-data/iam/security-credentials/"}
                                        ↓
                              [ Target Server ]
                                 ├── IP in Private CIDR Block (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16)?
                                 ├── IP is link-local (169.254.0.0/16)?
                                 ├── YES → Terminate with HTTP 403 Forbidden
                                 └── NO → Forward request normally
```

## Defense Layers
1. **URL validation**: Parse the URL and reject requests targeting private IP ranges, link-local addresses, and localhost. Use a URL parser that normalizes IPv6 and DNS rebinding attempts.
2. **Network segmentation**: Place metadata services on a separate network interface with IMDSv2 (requires a PUT request with a session token, blocking simple GET-based exfiltration).
3. **IAM least privilege**: Application roles should have minimal permissions. Even if SSRF succeeds, the returned credentials should grant access only to the specific S3 buckets or services the application needs.
4. **Egress filtering**: Configure firewall rules to block outbound traffic to metadata IP ranges from application containers.

## Testing SSRF Protection
Effective SSRF tests should cover:
- Direct IP address requests to `169.254.169.254` and `127.0.0.1`
- DNS rebinding attacks where a domain resolves to a private IP
- URL parser bypasses using IPv6 notation (`[::ffff:169.254.169.254]`)
- Redirect-based SSRF where the initial URL is public but the server follows a 302 redirect to a private address

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=devsecops-python` and verify SSRF protection by testing URL fetch endpoints against metadata service addresses.
