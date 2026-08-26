# Theoretical Context: Consumer-Driven Contract Testing with Pact

## Real-World Incident Case Study
At Soundcloud and Netflix, maintaining dedicated multi-service staging environments for hundreds of microservices created massive deployment queues. A single breaking API change in one service could cascade failures across dozens of consumers, but the defect was only discovered during full-system integration testing that took hours to run. Teams began shipping "defensive" code that handled every possible response shape, adding complexity and hiding real bugs. Pact contract testing solved this by enabling microservices to deploy independently: each consumer generates a JSON contract describing its expectations, and the provider verifies against that contract in its own CI pipeline. Soundcloud reduced integration testing time from hours to minutes while catching 94% of breaking changes before deployment.

## Protocol & Runtime Mechanism
Consumer-Driven Contracts invert traditional API testing: the API consumer generates a JSON contract ("Pact file"), and the provider verifies against that contract in its own build pipeline:

```
  [ Consumer Test ] ──→ Records HTTP interactions ──→ Pact JSON file
                                                          │
  [ Pact Broker / Git ] ←── Stores contract ──────────────┘
                                                          │
  [ Provider Build ] ←── Replays contract against provider endpoints
                         ├── All interactions match → Pass
                         └── Mismatch detected → Fail with diff
```

The Pact JSON records each expected request-response pair: the HTTP method, path, headers, body (with matching rules for variable fields), and the expected response status, headers, and body. This contract captures not just the schema but the exact semantic expectations of the consumer.

## Why Contracts Beat Integration Tests
- **Speed**: Contracts verify a single service in seconds; integration tests require the full stack
- **Isolation**: Each consumer's expectations are independent; one consumer's contract doesn't block another
- **Early feedback**: Breaking changes are caught when the provider's PR runs CI, before merge
- **Documentation**: The Pact file serves as machine-readable API documentation that stays in sync with actual consumer usage

## Pact Verification Modes
Pact supports two verification strategies: **stateful** (the provider sets up database state before replaying requests) and **stateless** (the provider's endpoints must work without setup). Stateful verification is more realistic but requires the provider to implement setup/teardown hooks. Stateless verification is faster and catches more edge cases because it tests the endpoint's behavior under arbitrary conditions.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=contract-pact` and verify the consumer contract by generating a Pact file from the consumer test and verifying it against the Crucible backend.
