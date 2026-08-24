# Theoretical Context: WCAG Accessibility & Semantic Accessibility Trees

## Real-World Incident Case Study
In *Robles v. Domino's Pizza LLC (2019)*, the US Ninth Circuit Court of Appeals ruled that the Americans with Disabilities Act (ADA) applies to websites and mobile apps, mandating accessible UI controls. Domino's argued that because their website was not specifically mentioned in the ADA, they had no obligation to make it accessible. The court rejected this, establishing that places of public accommodation must provide effective communication through accessible digital interfaces. The case set a precedent that now applies to every e-commerce platform, banking application, and government service portal in the United States.

## Protocol & Runtime Mechanism
Browsers translate the HTML DOM tree into a parallel **Accessibility Tree** consumed by assistive technologies (Screen Readers). Each DOM node produces an accessibility node with a computed role, name, and state:

```
  HTML DOM:           <button id="btn-12">Confirm</button>
                               ↓
  Accessibility Tree: Role: button | Name: "Confirm" | Focusable: true
                               ↓
  Screen Reader:      "Confirm, button" (spoken to user)
```

When developers use non-semantic elements like `<div onClick>` instead of `<button>`, the accessibility tree has no role information, and screen readers treat the element as static text. ARIA attributes (`role`, `aria-label`, `aria-describedby`) can restore semantic meaning, but semantic HTML remains the gold standard because it provides correct behavior with zero additional markup.

## WCAG 2.1 Success Criteria for Testing
The axe-core engine automates three critical success criteria:
- **1.1.1 Non-text Content**: Images must have `alt` text; decorative images use `alt=""`
- **1.3.1 Info and Relationships**: Form inputs must have associated `<label>` elements
- **4.1.2 Name, Role, Value**: Interactive components must expose role and state to the accessibility tree

These criteria are auditable by automated tools because they map directly to DOM structure. Subjective criteria like "1.4.3 Contrast (Minimum)" require computed style analysis, which axe-core also provides by calculating luminance ratios between foreground and background colors.

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=a11y-axe` and verify semantic accessibility by inspecting the accessibility tree of the Crucible sandbox pages.
