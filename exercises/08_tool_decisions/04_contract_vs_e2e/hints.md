# Hints: Drill 04 - Contract Testing vs E2E

## Hint 1 (Decision Framework)
Ask: "Can this risk be verified between two API specifications without spinning up full browser and database infrastructure?"
- API boundary schema compatibility between microservices -> Contract Testing (Pact)
- Cross-application visual user journeys and third-party web redirects -> E2E Testing

## Hint 2 (Pattern)
- Consumer-Driven Contracts record expectations as lightweight JSON contracts and verify them against mock provider responses in <1 second.
- E2E tests exercise the integrated stack end-to-end to catch deep integration wiring bugs.

## Hint 3 (Code Diff)
Set values:
- `microservices_frequent_independent_deployments`: `"contract_testing"`
- `critical_end_to_end_user_checkout_journey`: `"e2e_testing"`
- `fast_pr_builds_without_spinning_up_50_services`: `"contract_testing"`
- `visual_layout_and_third_party_gateway_redirects`: `"e2e_testing"`
