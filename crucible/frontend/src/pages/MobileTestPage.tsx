import React, { useState, useEffect } from 'react';

export const MobileTestPage: React.FC = () => {
  const [screen, setScreen] = useState<'home' | 'balance' | 'products' | 'alerts'>('home');
  const [biometricState, setBiometricState] = useState<'idle' | 'checking' | 'unavailable' | 'pin' | 'authenticated'>('idle');
  const [deepLinkAccount, setDeepLinkAccount] = useState<string | null>(null);
  const [, setProductScroll] = useState(false);

  useEffect(() => {
    const hash = window.location.hash;
    if (hash.startsWith('#account=')) {
      const accountId = hash.substring('#account='.length);
      setDeepLinkAccount(accountId);
      setScreen('balance');
    }
  }, []);

  const handleBiometric = () => {
    setBiometricState('checking');
    setTimeout(() => {
      setBiometricState('unavailable');
      setTimeout(() => {
        setBiometricState('pin');
      }, 800);
    }, 1000);
  };

  const handlePinSubmit = () => {
    setBiometricState('authenticated');
  };

  const products = Array.from({ length: 20 }, (_, i) => ({
    id: `item-${i + 1}`,
    name: `Product ${i + 1}`,
    price: (Math.random() * 100).toFixed(2),
  }));

  return (
    <div style={{ padding: '20px', fontFamily: 'sans-serif', maxWidth: '400px', margin: '0 auto' }}>
      <h1>Cherenkov Bank</h1>

      {biometricState !== 'idle' && biometricState !== 'authenticated' && (
        <div style={{ padding: '10px', margin: '10px 0', border: '1px solid #ccc', borderRadius: '4px' }}>
          {biometricState === 'checking' && <p>Checking biometric availability...</p>}
          {biometricState === 'unavailable' && <p>Biometric unavailable</p>}
          {biometricState === 'pin' && (
            <div>
              <p>Enter PIN:</p>
              <input type="password" id="pin-input" data-testid="pin-input" style={{ padding: '8px', width: '100%', marginBottom: '8px' }} />
              <button id="pin-submit" data-testid="pin-submit" onClick={handlePinSubmit} style={{ padding: '8px 16px', backgroundColor: '#007bff', color: 'white', border: 'none', borderRadius: '4px' }}>
                Submit PIN
              </button>
            </div>
          )}
        </div>
      )}

      {biometricState === 'authenticated' && (
        <div style={{ padding: '10px', margin: '10px 0', backgroundColor: '#d4edda', borderRadius: '4px' }}>
          <p id="welcome-message" data-testid="welcome-message">Welcome, SDET Engineer</p>
        </div>
      )}

      {screen === 'home' && (
        <div>
          <button id="login-biometric" data-testid="login-biometric" onClick={handleBiometric}
            style={{ display: 'block', width: '100%', padding: '12px', marginBottom: '10px', backgroundColor: '#28a745', color: 'white', border: 'none', borderRadius: '4px', fontSize: '16px' }}>
            Login with Biometric
          </button>
          <button id="view-balance" data-testid="view-balance" onClick={() => setScreen('balance')}
            style={{ display: 'block', width: '100%', padding: '12px', marginBottom: '10px', backgroundColor: '#007bff', color: 'white', border: 'none', borderRadius: '4px', fontSize: '16px' }}>
            View Balance
          </button>
          <button id="view-products" data-testid="view-products" onClick={() => { setScreen('products'); setProductScroll(true); }}
            style={{ display: 'block', width: '100%', padding: '12px', marginBottom: '10px', backgroundColor: '#17a2b8', color: 'white', border: 'none', borderRadius: '4px', fontSize: '16px' }}>
            View Products
          </button>
          <button id="view-alerts" data-testid="view-alerts" onClick={() => setScreen('alerts')}
            style={{ display: 'block', width: '100%', padding: '12px', marginBottom: '10px', backgroundColor: '#ffc107', color: 'black', border: 'none', borderRadius: '4px', fontSize: '16px' }}>
            View Alerts
          </button>
        </div>
      )}

      {screen === 'balance' && (
        <div>
          <button onClick={() => setScreen('home')} style={{ marginBottom: '10px', padding: '8px' }}>← Back</button>
          <h2 id="account-summary" data-testid="account-summary">Account Summary</h2>
          {deepLinkAccount && <p>Account: {deepLinkAccount}</p>}
          <p id="account-balance" data-testid="account-balance" style={{ fontSize: '24px', fontWeight: 'bold' }}>
            Account Balance: USD 1000
          </p>
        </div>
      )}

      {screen === 'products' && (
        <div>
          <button onClick={() => setScreen('home')} style={{ marginBottom: '10px', padding: '8px' }}>← Back</button>
          <h2>Products</h2>
          <div style={{ maxHeight: '300px', overflowY: 'auto', border: '1px solid #ccc', borderRadius: '4px' }} data-testid="product-list">
            {products.map((p) => (
              <div key={p.id} style={{ padding: '10px', borderBottom: '1px solid #eee', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                <span>{p.name} - ${p.price}</span>
              </div>
            ))}
            <div style={{ padding: '15px', textAlign: 'center', backgroundColor: '#f8f9fa' }}>
              <button id="btn-checkout" data-testid="btn-checkout"
                onClick={() => { setScreen('home'); alert('Order Confirmed'); }}
                style={{ padding: '10px 24px', backgroundColor: '#28a745', color: 'white', border: 'none', borderRadius: '4px', fontSize: '16px' }}>
                Checkout
              </button>
            </div>
          </div>
        </div>
      )}

      {screen === 'alerts' && (
        <div>
          <button onClick={() => setScreen('home')} style={{ marginBottom: '10px', padding: '8px' }}>← Back</button>
          <h2 id="system-alerts" data-testid="system-alerts">System Alerts</h2>
          <ul>
            <li>Your balance is low</li>
            <li>New login detected from unknown device</li>
            <li>Payment of $45.00 is pending</li>
          </ul>
        </div>
      )}
    </div>
  );
};
