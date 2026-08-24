# Theoretical Context: Server-Side Request Forgery (SSRF) & Cloud Metadata Protection

## Real-World Incident Case Study
In the 2019 Capital One breach, an attacker leveraged a misconfigured WAF on AWS to issue an SSRF request to `http://169.254.169.254/latest/meta-data/iam/security-credentials/`, compromising 100M+ customer records.

## Protocol & Runtime Mechanism
Link-local addresses (`169.254.0.0/16`) provide metadata services in AWS, GCP, and Azure. Unrestricted webhook fetchers allow external callers to induce internal requests:

```
  [ Attacker ] --? POST /fetch-url {"url": "http://169.254.169.254"}
                           ¦
                           ?
                    [ Target Server ]
                           +--? IP in Private CIDR Block?
                           +--? YES: Terminate with HTTP 403 Forbidden
```

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=devsecops-python` and verify SSRF protection.
