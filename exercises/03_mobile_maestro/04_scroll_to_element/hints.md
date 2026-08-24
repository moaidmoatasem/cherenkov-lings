# Hints: Drill 04 - Scroll Until Visible

## Hint 1 (Architectural Nudge)
Mobile device viewports vary drastically in height. Hardcoded tap coordinates or immediate assertions on off-screen list elements will fail on standard and small phone screens.

## Hint 2 (API Pattern)
Use Maestro's `scrollUntilVisible` command, specifying the target element, scroll direction, and maximum retries:
```yaml
- scrollUntilVisible:
    element:
      id: btn-checkout
    direction: DOWN
    maxRetries: 5
```

## Hint 3 (Code Diff)
```diff
- - tapOn:
-     id: btn-checkout
+ - scrollUntilVisible:
+     element:
+       id: btn-checkout
+     direction: DOWN
+     maxRetries: 5
+ - tapOn:
+     id: btn-checkout
```
