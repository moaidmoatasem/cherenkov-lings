# Theoretical Context: Keyboard Navigation & Focus Management

## Real-World Incident Case Study
Web applications frequently break keyboard navigation by using non-interactive `<div>` tags with `onClick` listeners or failing to manage focus during modal transitions.

## Protocol & Runtime Mechanism
Browsers maintain active element focus via `document.activeElement`. The Tab order follows the DOM sequence:

```
  [ Recipient Input ] --? Tab Key --? [ Amount Input ] --? Tab Key --? [ Submit Button ]
           ?                                                                   ¦
           +-------------------------- Shift + Tab ----------------------------+
```

## You will now simulate this in the Crucible
Run `cherenkov-lings watch --track=a11y-axe` and verify keyboard focus.
