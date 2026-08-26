# Drill 04: Solution -- Contract Testing vs E2E Testing Decision Matrix
# Rule:
# Use Contract Testing (Pact) for: Microservice boundaries, fast PR checks, preventing breaking API changes.
# Use End-to-End (E2E) for: Business critical user flows, third-party redirects, visual rendering.

scenarios = {
    "microservices_frequent_independent_deployments": "contract_testing",
    "critical_end_to_end_user_checkout_journey": "e2e_testing",
    "fast_pr_builds_without_spinning_up_50_services": "contract_testing",
    "visual_layout_and_third_party_gateway_redirects": "e2e_testing",
}

def test_contract_vs_e2e_decisions():
    assert scenarios["microservices_frequent_independent_deployments"] == "contract_testing", (
        "Consumer-driven contract testing lets squads deploy independently without shared environment dependencies"
    )
    assert scenarios["critical_end_to_end_user_checkout_journey"] == "e2e_testing", (
        "Core revenue journeys require complete end-to-end integration validation across all subsystems"
    )
    assert scenarios["fast_pr_builds_without_spinning_up_50_services"] == "contract_testing", (
        "Contract tests execute in milliseconds per microservice without orchestrating complex staging clusters"
    )
    assert scenarios["visual_layout_and_third_party_gateway_redirects"] == "e2e_testing", (
        "Contract testing validates JSON payloads, while E2E tests are required for browser UI and external redirects"
    )
