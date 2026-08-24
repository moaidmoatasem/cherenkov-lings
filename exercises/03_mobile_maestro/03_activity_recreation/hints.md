# Hints: Drill 03 - Activity Recreation

## Hint 1 (Architectural Nudge)
On Android, rotating the screen destroys and recreates the Activity by default. Any state stored in instance variables (not in a ViewModel or savedInstanceState) is lost. A test that only asserts before rotation gives you no confidence that your app handles configuration changes correctly.

## Hint 2 (API Pattern)
Maestro's setOrientation command triggers a device rotation:
setOrientation: { orientation: landscape }
Use this before re-asserting the same UI state to verify Activity recreation resilience.

## Hint 3 (Code Diff)
Add after the first assertVisible:
  setOrientation:
    orientation: landscape
  assertVisible:
    text: "Account Balance: USD 1000"
  setOrientation:
    orientation: portrait
  assertVisible:
    text: "Account Balance: USD 1000"

