# Theoretical Context: One Thing per Test (Single Responsibility Principle in Testing)

## Production Incident: Therac-25 Radiation Overdose Accidents (1985–1987)

Between 1985 and 1987, the Therac-25 computerized medical linear accelerator delivered massive radiation overdoses (up to 100 times the intended therapeutic dose) to six cancer patients, resulting in severe injuries and fatalities. The root cause was a subtle race condition in the software controller between keyboard input interpretation and mechanical collimator positioning. When operators rapidly entered prescription parameters and corrected typos within an 8-second window, the software failed to engage the physical tungsten radiation shield before activating the high-intensity 25 MeV electron beam. The manufacturer's automated testing had relied on massive, monolithic test routines that executed entire 20-minute patient therapy workflows in a single procedural test block. When an assertion failed halfway through the test, execution halted immediately, preventing tests for subsequent edge cases—such as fast editing sequences and asynchronous safety interlocks—from ever running.

## The Underlying Mechanism

Monolithic test functions containing multiple independent assertions violate the Single Responsibility Principle for tests and suffer from early-exit masking:

1. **Short-Circuit Masking**: In standard xUnit assertion frameworks (such as Pytest or JUnit), an assertion failure immediately raises an `AssertionError` exception, terminating test execution. All subsequent assertions in that block are skipped.
2. **Defect Concealment**: If a test validates user creation, profile update, order placement, and receipt email generation in a single method, a failure in profile update prevents any verification of order placement or receipt dispatch. Critical regressions in downstream modules remain completely undetected until the upstream issue is resolved.

```
[Anti-Pattern: Monolithic Test Cascade Masking]
Test: test_complete_e2e_flow()
  ├── Assert 1: User created        [PASS]
  ├── Assert 2: Profile updated      [FAIL] ──> Test ABORTS!
  ├── Assert 3: Order processed      [SKIPPED / UNTESTED - Critical Bug Hidden!]
  └── Assert 4: Payment confirmed    [SKIPPED / UNTESTED - Data Leak Hidden!]

[Resilient SDET Pattern: Atomic Single-Concept Tests]
Test A: test_user_creation_succeeds()              ──> [PASS]
Test B: test_profile_update_persists_changes()     ──> [FAIL - Clear Diagnosis]
Test C: test_order_processing_deducts_inventory()  ──> [PASS - Independently Verified]
Test D: test_payment_confirmation_sends_receipt()  ──> [PASS - Independently Verified]
```

Splitting tests by discrete concept ensures complete coverage reporting, pinpoint fault isolation, and independent test execution across continuous integration workers.

You will now simulate this in the Crucible: decompose monolithic multi-assertion test blobs into focused, atomic tests that isolate failures with precision.
