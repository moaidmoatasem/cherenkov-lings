import React from 'react';
import { apiUrl } from '../lib/api';

const CHECKOUT_FRAME_URL = apiUrl('/embed/checkout-frame');

export const PaymentPage: React.FC = () => {
  return (
    <div className="page-container" data-testid="payment-page">
      <div className="page-header">
        <span className="badge danger">Frame Layer: frameLocator &amp; Sandboxed Gateways</span>
        <h1>Secure Payment Gateway</h1>
        <p className="page-description">
          Card details are collected inside a sandboxed iframe to simulate third-party payment
          providers. Tests must scope locators with <code>frameLocator</code> instead of reaching
          across the frame boundary naively.
        </p>
      </div>

      <div className="payment-frame-wrapper">
        <div className="frame-container-header">
          <span className="frame-tag">Cross-Origin Checkout Frame</span>
          <span className="frame-url">{CHECKOUT_FRAME_URL}</span>
        </div>
        <iframe
          name="payment-gateway"
          id="stripe-frame"
          className="payment-frame payment-iframe payment-iframe-tall"
          src={CHECKOUT_FRAME_URL}
          title="Secure Card Payment Gateway"
        />
      </div>

      <div className="card code-hint" style={{ marginTop: '16px' }}>
        <h4>Frame Automation Strategy</h4>
        <p>
          <strong>Anti-pattern:</strong> <code>page.click('#card-number')</code> fails because the
          element lives in a child browsing context, not the main frame.
        </p>
        <p>
          <strong>Fix:</strong>{' '}
          <code>
            const frame = page.frameLocator(&apos;iframe[name=&quot;payment-gateway&quot;]&apos;);
          </code>{' '}
          then query card fields inside it.
        </p>
      </div>
    </div>
  );
};
