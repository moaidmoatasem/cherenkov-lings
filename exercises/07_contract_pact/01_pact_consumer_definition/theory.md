# Theoretical Context: Consumer-Driven Contract Testing with Pact

## Real-World Incident Case Study
At Soundcloud and Netflix, maintaining dedicated multi-service staging environments for hundreds of microservices created massive deployment queues. Pact contract testing enabled microservices to deploy independently without full-system integration tests.

## Protocol & Runtime Mechanism
Consumer-Driven Contracts invert traditional API testing: the API consumer generates a JSON contract ("Pact file"), and the provider verifies against that contract in its own build pipeline:

```
  [ Consumer Build ] --? Generates Pact JSON --? [ Pact Broker / Repo ]
                                                          ¦
                                                          ?
  [ Provider Build ] ?-- Replays Contract Against Provider Endpoints
```

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=contract-pact` and verify the consumer contract.
