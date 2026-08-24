# Theoretical Context: WCAG Accessibility & Semantic Accessibility Trees

## Real-World Incident Case Study
In *Robles v. Domino's Pizza LLC (2019)*, the US Ninth Circuit Court of Appeals ruled that the Americans with Disabilities Act (ADA) applies to websites and mobile apps, mandating accessible UI controls.

## Protocol & Runtime Mechanism
Browsers translate the HTML DOM tree into a parallel **Accessibility Tree** consumed by assistive technologies (Screen Readers):

```
  HTML DOM:           <button id="btn-12">Confirm</button>
                               ¦
                               ?
  Accessibility Tree: Role: button | Name: "Confirm" | Focusable: true
```

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=a11y-axe` and verify semantic accessibility.
