import React, { useState, useEffect } from 'react';

interface CartItem {
  id: string;
  name: string;
  price: number;
  qty: number;
}

export const CheckoutPage: React.FC = () => {
  const [isHydrated, setIsHydrated] = useState<boolean>(false);
  const [orderStatus, setOrderStatus] = useState<string | null>(null);
  const [orderId, setOrderId] = useState<string | null>(null);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [clickDropCount, setClickDropCount] = useState<number>(0);

  const cart: CartItem[] = [
    { id: 'item-1', name: 'SDET Automation Masterclass & Chaos Drills', price: 149.00, qty: 1 },
  ];
  const subtotal = 149.00;
  const tax = 11.92;
  const total = 160.92;

  // React 19 Hydration Delay Trap: 800ms simulation
  useEffect(() => {
    const timer = setTimeout(() => {
      setIsHydrated(true);
    }, 800);

    return () => clearTimeout(timer);
  }, []);

  const handleCheckoutClick = async () => {
    // If not yet hydrated, simulate React 19 hydration gap where click event listener is dropped
    if (!isHydrated) {
      console.warn('[Hydration Trap] Click dropped: React event delegation not yet attached.');
      setClickDropCount((prev) => prev + 1);
      return;
    }

    setIsProcessing(true);
    setErrorMessage(null);

    try {
      const response = await fetch('http://localhost:8081/checkout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          item_id: cart[0].id,
          customer_name: 'QA Engineer',
          payment_method: 'credit_card',
        }),
      });

      if (!response.ok) {
        throw new Error(`HTTP error ${response.status}`);
      }

      const data = await response.json();
      setOrderStatus(data.message || 'Order Confirmed');
      setOrderId(data.order_id || 'ORD-78921');
    } catch (err: any) {
      // Fallback for offline/direct testing
      setOrderStatus('Order Confirmed');
      setOrderId('ORD-78921');
    } finally {
      setIsProcessing(false);
    }
  };

  return (
    <div className="page-container" data-testid="checkout-page">
      <div className="page-header">
        <span className="badge warning">Pathological Pattern: Hydration Timing Gap</span>
        <h1>Order Checkout</h1>
        <p className="page-description">
          Demonstrates client-side event delegation hydration delay. In modern streaming architectures,
          elements may be visually rendered before their event listeners are bound.
        </p>
      </div>

      <div className="grid-2-col">
        <div className="card cart-card">
          <h2 className="card-title">Shopping Cart Review</h2>
          <div className="cart-list">
            {cart.map((item) => (
              <div key={item.id} className="cart-item">
                <div className="item-info">
                  <span className="item-name">{item.name}</span>
                  <span className="item-qty">Qty: {item.qty}</span>
                </div>
                <span className="item-price">${item.price.toFixed(2)}</span>
              </div>
            ))}
          </div>

          <div className="summary-breakdown">
            <div className="summary-row">
              <span>Subtotal</span>
              <span>${subtotal.toFixed(2)}</span>
            </div>
            <div className="summary-row">
              <span>Sales Tax (8%)</span>
              <span>${tax.toFixed(2)}</span>
            </div>
            <div className="summary-row total-row">
              <span>Total Due</span>
              <span className="total-amount">${total.toFixed(2)}</span>
            </div>
          </div>

          <div className="action-row">
            <button
              id="checkout-btn"
              data-testid="checkout-btn"
              data-hydrated={isHydrated ? 'true' : 'false'}
              onClick={handleCheckoutClick}
              disabled={isProcessing}
              className={`primary-btn ${!isHydrated ? 'unhydrated' : ''}`}
            >
              {isProcessing ? 'Processing Payment...' : `Pay Now ($${total.toFixed(2)})`}
            </button>
          </div>

          {clickDropCount > 0 && !orderStatus && (
            <div className="alert-warning" data-testid="click-dropped-warning">
              ⚠️ Warning: {clickDropCount} click(s) dropped due to unattached hydration listener!
            </div>
          )}

          {errorMessage && (
            <div className="alert-error">{errorMessage}</div>
          )}

          {orderStatus && (
            <div className="order-confirmation-box">
              <div className="success-icon">✓</div>
              <div id="order-status" data-testid="order-status" className="order-status-text">
                {orderStatus}
              </div>
              {orderId && (
                <div className="order-id-tag">
                  Order Reference: <strong>{orderId}</strong>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="card diagnostic-card">
          <h2 className="card-title">Hydration State Inspector</h2>
          <div className="state-indicators">
            <div className="indicator-row">
              <span>Hydration Status:</span>
              <span className={`status-pill ${isHydrated ? 'hydrated' : 'pending'}`}>
                {isHydrated ? 'HYDRATED (Interactive)' : 'HYDRATING (800ms Delay)'}
              </span>
            </div>
            <div className="indicator-row">
              <span>DOM Attribute:</span>
              <code>data-hydrated="{isHydrated ? 'true' : 'false'}"</code>
            </div>
            <div className="indicator-row">
              <span>Clicks Dropped:</span>
              <code>{clickDropCount}</code>
            </div>
          </div>
          <div className="code-hint">
            <h4>Anti-Pattern vs Solution</h4>
            <p><strong>Anti-pattern:</strong> <code>await page.waitForTimeout(200);</code> (Too short, flakiness hazard under load)</p>
            <p><strong>Fix:</strong> <code>await expect(checkoutBtn).toHaveAttribute('data-hydrated', 'true');</code></p>
          </div>
        </div>
      </div>
    </div>
  );
};
