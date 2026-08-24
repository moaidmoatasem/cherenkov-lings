# Theoretical Context: Keyboard Navigation & Focus Management

## Real-World Incident Case Study
In 2020, a major banking application shipped a modal dialog that trapped mouse users but released keyboard users into the background page. Keyboard-only users (including many screen reader operators) could interact with hidden form fields behind the modal, submitting unintended transactions. The defect was discovered only after customer complaints revealed that tab order bypassed the modal overlay entirely. The root cause was a missing `inert` attribute and absent `focus trap` implementation. This class of defect is classified under WCAG 2.1 Success Criterion 2.4.3 (Focus Order) and 2.1.2 (No Keyboard Trap), both Level A requirements.

## Protocol & Runtime Mechanism
Browsers maintain active element focus via `document.activeElement`. The Tab order follows the DOM sequence, with `tabindex` values overriding natural order:

```
  [ Recipient Input ] --Tab→ [ Amount Input ] --Tab→ [ Submit Button ]
           ← Shift+Tab ←                          ← Shift+Tab ←
```

When a modal opens, the focus trap must:
1. Move focus to the first focusable element inside the modal
2. Cycle Tab from the last element back to the first
3. Cycle Shift+Tab from the first element back to the last
4. Return focus to the triggering element when the modal closes

Without this trap, Tab keypresses escape the modal and reach background elements. The `inert` attribute (now supported in all major browsers) makes background elements non-interactive and removes them from the tab order, providing a declarative alternative to manual focus management.

## Testing Focus Traps with Playwright
Playwright provides `page.keyboard.press('Tab')` and `page.evaluate(() => document.activeElement)` to verify focus behavior. Automated tests should:
- Press Tab repeatedly and confirm focus cycles within the modal
- Verify `document.activeElement` never escapes to background elements
- Confirm Escape key returns focus to the triggering button
- Assert that focus restoration works after modal close and reopen

These checks catch regressions that visual testing misses, since the modal may look correct while being completely keyboard-inaccessible.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=a11y-axe` and verify keyboard focus management by tabbing through the Crucible sandbox forms.
