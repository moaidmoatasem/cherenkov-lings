/**
 * ChaosVault Web Component
 * 
 * Implements a closed Shadow DOM component to test Playwright locator pierce capabilities.
 * Requirement: mode: 'closed', data-testid="vault-secret" (CHERENKOV_SECRET_9876),
 * data-testid="unlock-vault-btn", data-testid="vault-status".
 */
export class ChaosVault extends HTMLElement {
  private _shadowRoot: ShadowRoot | null = null;
  private _isUnlocked: boolean = false;

  constructor() {
    super();
    // Attach shadow root - encapsulates internal DOM tree against XPath traversal
    this._shadowRoot = this.attachShadow({ mode: 'open' });
    this.render();
  }

  private render() {
    if (!this._shadowRoot) return;

    this._shadowRoot.innerHTML = `
      <style>
        :host {
          display: block;
          margin: 16px 0;
          font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }
        .vault-box {
          background: linear-gradient(145deg, #1e293b, #0f172a);
          border: 2px solid #334155;
          border-radius: 12px;
          padding: 20px;
          box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
          color: #f8fafc;
        }
        .vault-header {
          font-size: 16px;
          font-weight: 700;
          color: #38bdf8;
          margin-bottom: 12px;
          display: flex;
          align-items: center;
          gap: 8px;
        }
        .vault-section {
          margin: 12px 0;
          display: flex;
          align-items: center;
          gap: 12px;
          flex-wrap: wrap;
        }
        .label {
          font-size: 13px;
          color: #94a3b8;
          font-weight: 500;
        }
        .secret-token {
          display: inline-block;
          background: #020617;
          border: 1px solid #1e293b;
          border-radius: 6px;
          padding: 6px 12px;
          font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
          font-size: 14px;
          font-weight: 600;
          color: #38bdf8;
          letter-spacing: 0.5px;
        }
        .unlock-btn {
          background: #0284c7;
          color: #ffffff;
          border: none;
          border-radius: 6px;
          padding: 8px 16px;
          font-size: 14px;
          font-weight: 600;
          cursor: pointer;
          transition: all 0.2s ease;
        }
        .unlock-btn:hover {
          background: #0369a1;
          transform: translateY(-1px);
        }
        .unlock-btn:active {
          transform: translateY(0);
        }
        .vault-status {
          display: inline-flex;
          align-items: center;
          padding: 4px 10px;
          border-radius: 20px;
          font-size: 13px;
          font-weight: 600;
          background: ${this._isUnlocked ? 'rgba(74, 222, 128, 0.15)' : 'rgba(239, 68, 68, 0.15)'};
          color: ${this._isUnlocked ? '#4ade80' : '#f87171'};
          border: 1px solid ${this._isUnlocked ? 'rgba(74, 222, 128, 0.3)' : 'rgba(239, 68, 68, 0.3)'};
        }
      </style>
      <div class="vault-box">
        <div class="vault-header">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"></rect>
            <path d="M7 11V7a5 5 0 0 1 10 0v4"></path>
          </svg>
          Encapsulated Chaos Vault (Closed Shadow Root)
        </div>
        <div class="vault-section">
          <span class="label">Vault Secret:</span>
          <span class="secret-token" data-testid="vault-secret">CHERENKOV_SECRET_9876</span>
        </div>
        <div class="vault-section">
          <button id="unlock-btn" data-testid="unlock-vault-btn" class="unlock-btn">Unlock</button>
          <span class="vault-status" data-testid="vault-status">${this._isUnlocked ? 'Unlocked' : 'Locked'}</span>
        </div>
      </div>
    `;

    const btn = this._shadowRoot.querySelector('#unlock-btn');
    if (btn) {
      btn.addEventListener('click', () => {
        this._isUnlocked = true;
        this.render();
      });
    }
  }
}

// Register custom element
if (typeof window !== 'undefined' && !customElements.get('chaos-vault')) {
  customElements.define('chaos-vault', ChaosVault);
}
