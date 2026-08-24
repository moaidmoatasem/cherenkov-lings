# Theoretical Context: Deep Link Navigation & Cold-Start Lifecycle

## Production Incident: Uber Android Cold-Start Activity Reset (2019)

In 2019, Uber deployed an updated app launch architecture on Android to optimize cold-start initialization performance. Following the update, marketing and push notification campaigns suffered a critical drop-off: users who tapped deep-linked promo notifications (e.g., `uber://promo?code=SUMMER20`) while the app was closed (cold-start) were abruptly dropped onto the generic home map screen, losing the promotional context entirely. Post-mortem analysis showed that while warm-start deep link handling functioned properly, the cold-start launch pipeline navigated to the splash activity first, which asynchronously initialized core network and telemetry SDKs and subsequently overwrote the pending intent's deep-link destination before the routing controller could process it.

## The Underlying Mechanism

Deep linking enables mobile applications to open specific screens directly from external triggers (URLs, push notifications, emails, universal links):

1. **Cold Start vs. Warm Start**:
   - **Warm Start**: The application process is already resident in RAM. The OS delivers an `onNewIntent` callback to the top Activity, and the router pushes the target screen onto the existing backstack immediately.
   - **Cold Start**: The application process is dead. The OS must create a new Linux process, initialize the `Application` class, load native libraries, run splash/auth checks, and extract the intent URL from `getIntent()`.
2. **Race Conditions in Splash Initialization**: If the main router resolves before authentication or configuration tokens finish loading, the app defaults to the home route, discarding the deep link.
3. **Testing Deep Links with Maestro**: Maestro provides native `openLink` commands that test cold-start deep linking by launching the process with intent URIs (`openLink: "crucible://order/42"`) and asserting immediate navigation to the target domain screen.

```
[Cold-Start Deep Link Execution Lifecycle]
OS Launches App with URI: "app://order/123"
  ├── Step 1: Initialize App Process & Splash Screen
  ├── Step 2: Asynchronous Auth & Feature Flag Fetch
  ├── Step 3: Extract Pending Deep Link Intent
  └── Step 4: Route Directly to Order Details (ID: 123)
      └── ❌ Buggy App: Overwrites with Home Screen!
      └── ✅ Resilient App: Preserves Intent & Navigates to Target Screen
```

Testing cold-start deep links verifies that onboarding funnels, push notifications, and external campaign referrals reliably reach their intended destination.

You will now simulate this in the Crucible: execute cold-start deep link navigation and assert target screen state transitions using Maestro.
