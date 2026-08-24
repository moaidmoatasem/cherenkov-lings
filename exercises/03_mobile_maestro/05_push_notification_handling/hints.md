# Hints: Drill 05 - Push Notification Permission Handling

## Hint 1 (Architectural Nudge)
OS-level permission dialogs appear non-deterministically depending on device OS version, previous test state, or simulator settings. Hardcoding a tap on the dialog will fail if the dialog does not appear, and omitting it will fail if it does.

## Hint 2 (API Pattern)
Use Maestro's conditional execution block `runFlow: when: visible: ...`:
```yaml
- runFlow:
    when:
      visible: "Allow.*notifications"
    commands:
      - tapOn: "Allow"
```

## Hint 3 (Code Diff)
```diff
  - launchApp:
      appId: com.cherenkov.bankapp
+ - runFlow:
+     when:
+       visible: "Allow.*notifications|Allow"
+     commands:
+       - tapOn: "Allow"
  - tapOn:
      text: View Alerts
```
