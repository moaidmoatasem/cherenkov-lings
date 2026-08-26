# Hints: Drill 06 - Login Flow & Selector Durability

## Hint 1 (Architectural Nudge)
The flow launches the app and then stops. Everything after `launchApp` is a comment, so the test proves only that the package installs and the process starts — it never reaches the authenticated state the rest of your mobile suite depends on. Before writing the taps, think about what you are tapping *by*. Maestro will happily match a visible label like "Username", but visible labels are the least stable identifier a screen has: they change with copy edits, they differ per locale, and on a localized build the same flow fails in every language but one.

## Hint 2 (API Pattern)
Maestro commands are a YAML list, each entry a single verb. `tapOn` accepts either a bare string (matched against visible text and accessibility identifiers) or a map with explicit fields — `id:` targets the accessibility identifier directly and survives copy changes. `inputText` types into whatever currently holds focus, so it must follow the `tapOn` that focused the field; the two are a pair, and reversing them types into nothing. Maestro auto-waits for an element to appear before interacting, so explicit sleeps are unnecessary and mark the flow as flaky by construction. Close the flow with an `assertVisible` on something only the authenticated screen renders — without it you have automated the taps but verified nothing.

## Hint 3 (Code Diff)
Replace the three TODO comments with the tap/input pairs and a terminal assertion:

    appId: com.example.app
    ---
    - launchApp
    - tapOn: "Username"
    - inputText: "user1"
    - tapOn: "Password"
    - inputText: "pass1"
    - tapOn: "Login"
    - assertVisible: "Welcome"

Prefer `tapOn: { id: "username_field" }` over the bare text form once your app exposes accessibility identifiers — the text form is the version that breaks on the first copy change.
