# Theoretical Context: Consumer-Driven Provider Verification & Schema Evolution

## Real-World Incident Case Study
In a high-profile 2019 outage at a Fortune 500 retail platform, backend engineers refactored an internal microservice and renamed the order identifier field from `order_id` to `id` to conform with internal REST conventions. Although all backend unit and integration tests passed (because mock test fixtures were updated in tandem), thousands of mobile clients crashed on startup because the mobile consumer contract was violated without warning.

## Protocol & Runtime Mechanism
In monolithic architectures, breaking changes are caught by compilation or unified test suites. In distributed microservice architectures, consumers (web apps, mobile apps, downstream ETL pipelines) and providers (API services) evolve on independent release schedules.

Provider verification closes this safety gap by replaying consumer contracts directly against provider pull requests before merge:

```
  [ Consumer Repo ] ----------? Generates Pact JSON ----------? [ Pact Broker ]
                                                                      │
                                                                      ?
  [ Provider CI Pipeline ] ?-- Replays Contract Against Provider -----+
            │
            +--? If Schema Matches: Provider Verified (Safe to Deploy)
            +--? If Field Missing: CI Fails -> Blocks Breaking PR
```

Key verification rules:
1. **Field Existence**: All keys requested by the consumer must exist in the provider response payload.
2. **Type Invariance**: Data types (e.g. `String`, `Number`, `Boolean`) must strictly match consumer expectations.
3. **Status Enums**: Response status codes and state transitions must adhere to allowable contract sets.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=contract-pact` and verify that provider endpoints strictly adhere to contract expectations across all order entries.
