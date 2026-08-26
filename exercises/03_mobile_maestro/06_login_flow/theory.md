# Theoretical Context: Selector Durability & The Unasserted Flow

## Production Incident: The Aadhaar mAadhaar Localization Failures (2018)

India's mAadhaar identity application ships across a dozen-plus official languages against an identity database of over a billion residents. Localized Android builds repeatedly surfaced authentication defects that reached users in regional-language builds while the English build tested clean. The root cause pattern is the same one every large mobile programme eventually meets: automation that identifies controls by their rendered label validates exactly one locale, and silently stops covering every other build the moment the string table diverges.

## The Underlying Mechanism

Mobile UI automation resolves an on-screen control through one of three identifier classes, and their durability is not remotely equal:

1. **Rendered text** — "Login", "Sign in", "Continue". Zero setup cost, and it breaks on copy edits, A/B tests, and every non-default locale.
2. **Accessibility identifier** — `id: login_button`. Set deliberately by the developer, invisible to the user, unaffected by translation. It is also the same attribute screen readers consume, so keeping it correct serves accessibility and automation at once.
3. **Coordinates / index position** — `tapOn: { point: "50%,80%" }`. Survives nothing: a font-scale change, a new banner, or a different screen density invalidates it.

```
[Same flow, three selector strategies, after a copy change to "Sign in"]

  tapOn: "Login"              → element not found      ❌
  tapOn: { id: login_button } → resolves, taps         ✅
  tapOn: { point: 50%,80% }   → taps whatever moved
                                into that spot         ☠️ (worse: passes)
```

The second failure mode in this drill is subtler. A flow that taps through a login and then ends has *performed* an action without *verifying* an outcome. Maestro reports success as long as no command errors, so a login that lands on an "Invalid credentials" screen still passes — every command found its target. A terminal `assertVisible` on an element unique to the post-login state is what converts the flow from a macro into a test.

You will now simulate this in the Crucible: complete a login flow by tapping and typing into both credential fields, submitting, and asserting on the authenticated screen.
