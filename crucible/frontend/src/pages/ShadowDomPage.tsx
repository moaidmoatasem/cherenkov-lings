import React from 'react';
import { PaymentFrame } from '../components/PaymentFrame';
import { apiUrl } from '../lib/api';
import '../components/ChaosVault'; // Ensure custom element is registered

export const ShadowDomPage: React.FC = () => {
  return (
    <div className="page-container" data-testid="shadow-dom-page">
      <div className="page-header">
        <span className="badge purple">Encapsulation Barrier: Closed Shadow DOM & Cross-Origin Iframes</span>
        <h1>Shadow DOM & Cross-Origin Sandbox</h1>
        <p className="page-description">
          Demonstrates browser DOM encapsulation boundaries. Standard XPath locators cannot cross
          closed shadow roots or iframe boundaries. Playwright locators natively pierce open/closed shadow
          roots with CSS engines and provide explicit `frameLocator` APIs for iframes.
        </p>
      </div>

      <div className="grid-2-col">
        <div className="card showcase-card">
          <h2 className="card-title">1. Closed Shadow DOM Custom Element</h2>
          <p className="component-desc">
            The <code>&lt;chaos-vault&gt;</code> element below encapsulates its internal DOM inside a closed shadow root.
          </p>

          <div className="vault-embed-wrapper" data-testid="vault-wrapper">
            <chaos-vault></chaos-vault>
          </div>

          <div className="divider"></div>

          <h2 className="card-title">2. Cross-Origin Payment Gateway (Iframe)</h2>
          <p className="component-desc">
            An embedded payment authorization iframe hosted on port 8081 (origin: <code>http://localhost:8081</code>).
          </p>

          <PaymentFrame src={apiUrl('/embed/payment-frame')} />
        </div>

        <div className="card diagnostic-card">
          <h2 className="card-title">Encapsulation & Locator Inspector</h2>
          <div className="state-indicators">
            <div className="indicator-row">
              <span>Shadow DOM Mode:</span>
              <code>mode: 'closed'</code>
            </div>
            <div className="indicator-row">
              <span>Expected Secret Token:</span>
              <code>CHERENKOV_SECRET_9876</code>
            </div>
            <div className="indicator-row">
              <span>Iframe Origin:</span>
              <code>http://localhost:8081</code>
            </div>
          </div>

          <div className="code-hint">
            <h4>Piercing Shadow DOM Locators</h4>
            <p><strong>Anti-pattern:</strong> <code>page.locator('/html/body/.../chaos-vault/div/span[2]')</code> (Fails because XPath stops at shadow boundary).</p>
            <p><strong>Fix:</strong> <code>page.locator('chaos-vault').locator('[data-testid="vault-secret"]')</code></p>
          </div>

          <div className="code-hint">
            <h4>Iframe Frame Piercing</h4>
            <p><strong>Playwright Frame Locator:</strong> <code>{"page.frameLocator('[data-testid=\"payment-frame\"]').getByRole('button', { name: 'Authorize Payment' })"}</code></p>
          </div>
        </div>
      </div>
    </div>
  );
};
