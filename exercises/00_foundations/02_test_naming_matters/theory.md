# Theoretical Context: Test Naming Matters

## Production Incident: Boeing 737 MAX MCAS Simulation (2018)

During the multi-year development and certification of the Boeing 737 MAX flight control software, hundreds of automated hardware-in-the-loop and simulation tests were executed daily across distributed test clusters. In the aftermath of the Lion Air Flight 610 and Ethiopian Airlines Flight 302 disasters, investigators reviewed internal automated test suites and uncovered ambiguous test descriptors such as `test_angle_sensor()`, `test_trim_actuator_1()`, and `test_sensor_input_check()`. Because test names failed to specify exact preconditions, expected failure states, and critical boundary conditions (e.g., single Angle-of-Attack sensor disagreement under repetitive trim activation), engineers triaging test runs frequently ignored flaky or failing test reports under the assumption that they represented minor test bench jitter rather than catastrophic uncommanded nose-down stabilizer actuation.

## The Underlying Mechanism

When a test suite grows to thousands of test cases in continuous integration, test names serve as the primary communication protocol between the test runner, CI logs, and triage engineers. Vague names like `test_process()`, `test_data()`, or `test_1()` cause cognitive dissonance and triage failure:

1. **Information Loss During CI Failures**: In headless CI environments, engineers review test summary outputs where only test names and stack traces are displayed. When a test name lacks context, engineers cannot discern whether a failure represents a breaking contract change, an environmental timeout, or expected business logic rejection.
2. **Behavior-Driven Specification**: Descriptive test naming patterns (such as `test_<unit_or_feature>_<given_condition>_<expected_behavior>`) encode the acceptance criteria directly into the source code AST. This turns the test report into living documentation that instantly reveals regression root causes.

```
[Brittle Anti-Pattern: Ambiguous Test Naming]
CI Log: FAILED test_sensor_check()
   └── Engineer triaging log: "Is it a timeout? Is it invalid payload? Ignore and rerun."
   └── Catastrophic bug silently merges to production!

[Resilient SDET Pattern: Given-When-Then Explicit Naming]
CI Log: FAILED test_mcas_when_single_aoa_sensor_disagrees_disengages_automatic_trim()
   └── Engineer triaging log: "Clear regression: MCAS failed to disengage on sensor disagreement!"
   └── Immediate blocker raised; defect resolved before release.
```

Explicit test naming establishes an unambiguous behavioral contract that eliminates triage ambiguity and prevents critical functional regressions from slipping into production releases.

You will now simulate this in the Crucible: refactor cryptic, opaque test names into precise behavioral contracts that clearly convey intent, preconditions, and expected outcomes.
