import React from 'react';
import { apiUrl } from '../lib/api';

interface PaymentFrameProps {
  src?: string;
}

export const PaymentFrame: React.FC<PaymentFrameProps> = ({
  src = apiUrl('/embed/payment-frame'),
}) => {
  return (
    <div className="payment-frame-wrapper">
      <div className="frame-container-header">
        <span className="frame-tag">Cross-Origin Sandbox Frame</span>
        <span className="frame-url">{src}</span>
      </div>
      <iframe
        id="payment-gateway-frame"
        data-testid="payment-frame"
        src={src}
        title="Crucible Secure Payment Gateway"
        className="payment-iframe"
      />
    </div>
  );
};
