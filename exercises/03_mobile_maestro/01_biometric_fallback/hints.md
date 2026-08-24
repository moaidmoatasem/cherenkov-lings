# Hints: Drill 01 - Biometric Fallback

## Hint 1 (Architectural Nudge)
Mobile biometric authentication is inherently environment-dependent. In CI environments, on simulators, or when the hardware sensor fails, the app falls back to a PIN entry screen. A flow that assumes biometric always succeeds is a flaky test waiting to happen.

## Hint 2 (API Pattern)
Maestro's runFlow command accepts a when: condition that checks for a visible element before running the nested flow. This is the idiomatic way to handle optional UI states:
runFlow: { when: { visible: { text: 'Biometric unavailable' } }, file: pin_fallback_flow.yaml }

## Hint 3 (Code Diff)
Add after the tapOn biometric step:
  runFlow:
    when:
      visible:
        text: Biometric unavailable
    file: pin_fallback_flow.yaml
This runs pin_fallback_flow.yaml only if the biometric failure message is visible.
