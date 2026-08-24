# Theoretical Context: OS Permission Dialogs & Conditional Flow Handling

## Production Incident: Slack iOS 14 Permission Modal Freeze (2020)

In September 2020, Apple released iOS 14, introducing mandatory runtime permission prompts for push notifications, local network access, and App Tracking Transparency (ATT). Immediately following the iOS update, Slack's continuous integration test matrix—spanning 50 parallel real-device cloud test runners—ground to a complete halt. When test runners launched fresh app instances, native OS permission dialogs appeared asynchronously over the login form. Because the test scripts expected to type credentials into the underlying application inputs immediately and lacked conditional flow handlers to dismiss native system popups, the tests hung indefinitely, waiting for login inputs that were visually and functionally obstructed by the OS modal.

## The Underlying Mechanism

Modern mobile platforms enforce privacy-by-design by delegating permission grants to modal system dialogs:

1. **System Modal Interruption**: Unlike in-app views, system permission dialogs run in a separate system-level process (e.g., `SpringBoard` on iOS, `com.android.permissioncontroller` on Android). When active, they capture all touch events, preventing interaction with the background application.
2. **Non-Deterministic Timing**: Permission modals may appear on the first app launch, upon navigating to specific features, or never (if the test runner reuses an already-permissioned simulator image).
3. **Maestro Conditional Flows**: To handle non-deterministic system interruptions without breaking test linearity, Maestro provides conditional sub-flows:
   ```yaml
   - runFlow:
       when:
         visible: "Allow.*notifications"
       commands:
         - tapOn: "Allow"
   ```
   If the permission dialog is present, the flow taps "Allow" and clears the modal; if absent, execution proceeds immediately without error or timeout penalty.

```
[System Permission Modal Interruption Flow]
App Launches ──> Native OS Modal: "Allow App to send Notifications?"
                   │
         ┌─────────┴──────────────────────────────┐
         ▼                                        ▼
   [Unconditional Test]                     [Maestro runFlow: when: visible]
Attempts to tap "Login"                  Detects Permission Modal
Fails / Blocked by Modal! ❌             Taps "Allow" ──> Clears Dialog ✅
Test times out after 60s                 Taps "Login" ──> Test Continues!
```

Integrating conditional permission handlers ensures mobile test automation runs seamlessly across both pristine fresh-install environments and persistent test devices.

You will now simulate this in the Crucible: handle asynchronous OS permission prompts using Maestro's conditional `runFlow` directives.
