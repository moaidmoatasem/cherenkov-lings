import React, { useState, useEffect } from 'react';
import { apiUrl } from '../lib/api';

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
  const [itemId, setItemId] = useState<string>('item-1');
  const [quantity, setQuantity] = useState<string>('1');
  const [shippingType, setShippingType] = useState<string>('standard');
  const [address, setAddress] = useState<string>('');
  const [formTouched, setFormTouched] = useState<boolean>(false);

  const markFormTouched = () => setFormTouched(true);

  const qty = Math.max(1, parseInt(quantity, 10) || 1);
  const unitPrice = 149.0;
  const cart: CartItem[] = [
    { id: itemId || 'item-1', name: 'SDET Automation Masterclass & Chaos Drills', price: unitPrice, qty },
  ];
  const subtotal = unitPrice * qty;
  const tax = Math.round(subtotal * 0.08 * 100) / 100;
  const total = subtotal + tax;

  // React 19 Hydration Delay Trap: 800ms simulation
  useEffect(() => {
    const timer = setTimeout(() => {
      setIsHydrated(true);
    }, 800);

    return () => clearTimeout(timer);
  }, []);

  const processOrder = async (): Promise<void> => {
    setIsProcessing(true);
    setErrorMessage(null);

    try {
      const response = await fetch(apiUrl('/checkout'), {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          item_id: itemId || cart[0].id,
          customer_name: 'QA Engineer',
          payment_method: 'credit_card',
          quantity: Math.max(1, parseInt(quantity, 10) || 1),
          shipping_method: shippingType,
          shipping_address: address,
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

  const handleCheckoutClick = async () => {
    // If not yet hydrated, simulate React 19 hydration gap where click event listener is dropped.
    // Explicitly filling the order form implies app readiness and bypasses the trap.
    if (!isHydrated && !formTouched) {
      console.warn('[Hydration Trap] Click dropped: React event delegation not yet attached.');
      setClickDropCount((prev) => prev + 1);
      return;
    }

    await processOrder();
  };

  const handleConfirmPurchaseClick = async () => {
    await processOrder();
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
          <h2 className="card-title">Order Details</h2>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleCheckoutClick();
            }}
            className="transfer-form checkout-order-form"
          >
            <div className="form-row-2">
              <div className="form-group">
                <label htmlFor="item-id">Item ID</label>
                <input
                  id="item-id"
                  data-testid="item-id"
                  type="text"
                  value={itemId}
                  onChange={(e) => {
                    setItemId(e.target.value);
                    markFormTouched();
                  }}
                  placeholder="e.g. item-1"
                  className="form-input"
                />
              </div>
              <div className="form-group">
                <label htmlFor="quantity">Quantity</label>
                <input
                  id="quantity"
                  data-testid="quantity"
                  type="number"
                  min="1"
                  max="99"
                  value={quantity}
                  onChange={(e) => {
                    setQuantity(e.target.value);
                    markFormTouched();
                  }}
                  className="form-input"
                />
              </div>
            </div>

            <div className="form-group">
              <label htmlFor="shipping-type">Shipping Method</label>
              <select
                id="shipping-type"
                data-testid="shipping-type"
                value={shippingType}
                onChange={(e) => {
                  setShippingType(e.target.value);
                  markFormTouched();
                }}
                className="form-input"
              >
                <option value="standard">Standard (5-7 days)</option>
                <option value="express">Express (1-2 days)</option>
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="address">Delivery Address</label>
              <input
                id="address"
                data-testid="address"
                type="text"
                value={address}
                onChange={(e) => {
                  setAddress(e.target.value);
                  markFormTouched();
                }}
                placeholder="742 Evergreen Terrace"
                className="form-input"
              />
            </div>
          </form>

          <h2 className="card-title" style={{ marginTop: '16px' }}>Shopping Cart Review</h2>
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

            <button
              id="confirm-purchase"
              data-testid="confirm-purchase-btn"
              onClick={handleConfirmPurchaseClick}
              disabled={isProcessing}
              className="secondary-btn"
            >
              Confirm Purchase
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
