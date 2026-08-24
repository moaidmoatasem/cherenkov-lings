# Theoretical Context: Backward Compatibility & Non-Breaking Schema Evolution

## Real-World Incident Case Study
In 2020, a SaaS analytics company modified an API response structure by transforming a flat JSON dictionary into a nested object to support internationalization. While web clients received the frontend update simultaneously, third-party webhook integrations and enterprise API consumers experienced immediate parsing failures, leading to data loss across 200+ partner organizations.

## Protocol & Runtime Mechanism
When microservice APIs evolve, teams must adhere to Postel’s Law (*The Robustness Principle*): *"Be conservative in what you do, be liberal in what you accept from others."*

In contract testing, changes are classified into two categories:

$$\text{Additive (Safe)}: \text{Schema}_{\text{v2}} = \text{Schema}_{\text{v1}} \cup \{\text{new\_optional\_field}\}$$
$$\text{Breaking (Unsafe)}: \text{Schema}_{\text{v2}} \not\supseteq \text{Schema}_{\text{v1}}$$

```
  Additive Evolution (Safe):
  v1 Consumer Expects:  { id: "101", total: 149.0 }
  v2 Provider Returns:  { id: "101", total: 149.0, discount: 10.0 } 
  -> v1 Consumer ignores extra "discount" field -> SUCCESS

  Destructive Evolution (Breaking):
  v1 Consumer Expects:  { id: "101", total: 149.0 }
  v2 Provider Returns:  { order_ref: "101", total: 149.0 }
  -> Missing "id" key causes NullPointer / KeyError in Consumer -> FAILURE
```

Automated contract testing prevents destructive evolution by ensuring that every newly added capability is strictly additive and existing fields retain their names, semantics, and structural types.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=contract-pact` and verify non-breaking backward compatibility.
