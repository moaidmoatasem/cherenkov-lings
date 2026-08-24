# Theoretical Context: HTTP Idempotency & Retry Storm Protection

## Production Incident: Stripe Duplicate Payment Double-Charge Outage (2017)

In 2017, payment infrastructure giant Stripe experienced an intermittent network connectivity disruption between their edge routing proxies and internal payment processing ledgers. As client requests encountered 504 Gateway Timeouts, client applications automatically initiated rapid retry loops. Because some upstream client integrations did not supply unique `Idempotency-Key` headers on their `POST /v1/charges` requests, the payment gateway treated each retry as an independent transaction request. Over several hours, thousands of end customers were billed multiple times for single purchases, triggering millions of dollars in unexpected charges, widespread merchant panic, and thousands of manual dispute reconciliations.

## The Underlying Mechanism

In RESTful HTTP architecture, HTTP methods carry explicit idempotency semantics under RFC 7231 / RFC 9110:

1. **Idempotent vs. Non-Idempotent Methods**: `GET`, `PUT`, and `DELETE` are defined as idempotent—executing them $N$ times produces the identical server state as executing them once. In contrast, `POST` is non-idempotent; executing `POST` twice typically creates two distinct resources.
2. **The Distributed Network Uncertainty Window**: In distributed systems, when an HTTP client experiences a network timeout or dropped connection, the client cannot determine whether:
   - The request failed before reaching the server.
   - The server processed the request, but the response was lost in transit.
3. **Idempotency Keys**: Supplying an `Idempotency-Key` header allows the server to cache the response of the initial execution in a distributed cache (e.g., Redis). When a duplicate request arrives with the same key within a validity window, the server returns the cached response with an HTTP 200 or 409 Conflict without re-executing state mutation.

```
[Non-Idempotent Retry Storm vs. Idempotent Key Resolution]
Non-Idempotent POST /charges (No Key):
Client ──[POST $100]──> Server (Processes $100, response drops)
Client ──[POST $100]──> Server (Processes ANOTHER $100! ❌ Double Charge!)

Idempotent POST /charges (Idempotency-Key: uuid-1234):
Client ──[POST Key: uuid-1234]──> Server (Stores Key, Deducts $100)
Client ──[POST Key: uuid-1234]──> Server (Detects Key ──> Returns Cached 200 OK ✅)
```

Automated API resilience suites must verify that re-sending identical requests with idempotency keys prevents duplicate side-effects.

You will now simulate this in the Crucible: assert API idempotency protections using REST Assured to verify that retry requests prevent duplicate resource mutations.
