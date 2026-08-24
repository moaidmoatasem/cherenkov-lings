# Hints: Drill 02 - Deep Link Cold Start

## Hint 1 (Architectural Nudge)
Deep links are deceptive. openLink works when the app is already warm in memory, but in CI the app starts cold. On Android, an unregistered deeplink scheme falls back to a browser. On iOS, it may silently fail. Your test passing in dev and failing in CI is the classic symptom of a warm-device assumption.

## Hint 2 (API Pattern)
Maestro's launchApp command supports an arguments.deeplink field that handles the cold-start case natively:
launchApp: { appId: com.myapp, arguments: { deeplink: 'myapp://home' } }

## Hint 3 (Code Diff)
Replace:
  openLink:
    link: cherenkov://account/ACC-001
With:
  launchApp:
    appId: com.cherenkov.bankapp
    clearState: true
    arguments:
      deeplink: cherenkov://account/ACC-001
