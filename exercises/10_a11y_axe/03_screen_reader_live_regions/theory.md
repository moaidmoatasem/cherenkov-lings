# Theoretical Context: ARIA Live Regions & Dynamic Screen Reader Announcements

## Real-World Incident Case Study
In 2021, a fintech payment application displayed a real-time transaction confirmation toast ("Payment of $45.00 sent to Alice") that was visually prominent but completely invisible to screen reader users. The toast was rendered by a React component that appended a new DOM node after the initial page render. Because no `aria-live` region wrapped the toast container, the browser's accessibility subsystem never notified assistive technologies of the change. Visually impaired users had no way to confirm their payment succeeded, leading to duplicate transaction attempts and customer support escalation. The fix was a single attribute: `aria-live="polite"` on the toast container.

## Protocol & Runtime Mechanism
ARIA Live Regions instruct the browser accessibility subsystem to announce DOM mutations without requiring a full page reload. When a live region's subtree changes, the browser queues an `AccessibilityEvent` of type `AX_REGION_CHANGED`:

```
  DOM Mutation: <div role="status">Transfer Confirmed</div>
                               ↓
  Accessibility API: AccessibilityEvent(TYPE_ANNOUNCEMENT, "Transfer Confirmed")
                               ↓
  Screen Reader: Speaks "Transfer Confirmed" to user
```

Three urgency levels control announcement behavior:
- **`aria-live="polite"`**: Announces after the current announcement finishes. Use for non-critical updates like status messages and form validation feedback.
- **`aria-live="assertive"`**: Interrupts the current announcement. Use sparingly for errors and urgent alerts that require immediate attention.
- **`aria-live="off"`** (default): No announcement. Dynamic content inside these regions is invisible to screen readers.

Common ARIA roles that imply live regions include `role="status"`, `role="alert"`, and `role="log"`. Using these implicit roles reduces markup verbosity while achieving the same announcement behavior.

## Testing Live Regions with axe-core and Playwright
axe-core validates that live regions exist and use correct urgency levels. Playwright can verify announcements by intercepting accessibility events or by checking DOM state after mutations. A robust test pattern:
1. Trigger the dynamic update (e.g., click a submit button)
2. Wait for the live region's text content to change
3. Assert the expected announcement text matches what was rendered
4. Verify the `aria-live` attribute is set to the appropriate urgency level

This catches regressions where refactored components inadvertently remove live region wrappers, silently breaking screen reader accessibility for dynamic content.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=a11y-axe` and verify live region announcements by triggering dynamic updates in the Crucible sandbox.
