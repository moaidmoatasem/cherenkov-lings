# Theoretical Context: Cross-Origin Iframes & Sandboxed Contexts

## Production Incident: Shopify & Stripe 3D Secure Frame Drop (2020)

During Black Friday 2020, several high-volume Shopify merchants experienced an automation and payment verification outage when updating their Stripe 3D Secure (3DS2) authentication flows. Automated staging tests executing pre-deployment checkout verifications began failing intermittently. The test scripts attempted to interact with credit card number and PIN input fields embedded inside Stripe's sandboxed, cross-origin `<iframe>` element. Because the legacy automation framework treated the main page DOM as a single flat hierarchy and lacked cross-origin security context awareness, element queries failed to cross the browser security boundary, falsely reporting that payment inputs were missing and triggering false rollbacks of production-ready payment fixes.

## The Underlying Mechanism

The browser's Same-Origin Policy (SOP) strictly separates execution contexts between different origins (protocol, domain, and port):

1. **Iframe Isolation**: An `<iframe>` creates a nested browsing context with its own distinct `window` and `document` object hierarchy. If the iframe is hosted on a different origin (e.g., `https://js.stripe.com` inside `https://myshopify.com`), the parent document cannot directly access the child iframe's DOM via JavaScript due to cross-origin security restrictions.
2. **Context Switching in Automation**: Attempting to locate elements inside an iframe using `page.locator('#card-number')` fails because the selector query executes against the parent document root.
3. **Playwright `frameLocator` Solution**: Playwright's `frameLocator` API provides a seamless, auto-waiting mechanism to pierce iframe boundaries. It maintains proper execution context switching internally, waiting for the iframe to load its document before resolving child locators.

```
[Cross-Origin Iframe Execution Context Barrier]
Parent Document Context (https://checkout.store.com)
 ├── <div id="app">
 └── <iframe src="https://secure.stripe.com/frame" id="stripe-frame">
          ▲
          │ [Same-Origin Boundary Barrier]
          ▼
     Child Iframe Context (https://secure.stripe.com)
      └── <input id="card-pin" type="password" />

❌ page.locator('#card-pin') ──> NULL (Element not in parent document)
✅ page.frameLocator('#stripe-frame').locator('#card-pin') ──> TARGET ACQUIRED
```

Using `frameLocator` ensures tests reliably traverse sandboxed payment gateways, OAuth modals, and third-party widgets without compromising browser security contexts.

You will now simulate this in the Crucible: target and automate secure input fields embedded inside nested cross-origin iframes using Playwright's `frameLocator`.
