import React, { useState } from 'react';

export const ProfilePage: React.FC = () => {
  const resolveInitialName = (): string => {
    const saved = window.localStorage.getItem('profile_username');
    if (saved) {
      return saved;
    }
    const authToken = window.localStorage.getItem('auth_token') || '';
    if (authToken.startsWith('token_')) {
      const derived = authToken.replace('token_', '').replace(/[_\d]+$/g, '');
      if (derived) {
        return derived.charAt(0).toUpperCase() + derived.slice(1);
      }
    }
    return 'Guest User';
  };

  const [username, setUsername] = useState<string>('');
  const [displayName, setDisplayName] = useState<string>(resolveInitialName);
  const [saveConfirmation, setSaveConfirmation] = useState<string | null>(null);

  const handleSaveProfile = () => {
    const trimmed = username.trim();
    if (!trimmed) {
      setSaveConfirmation('Please enter a username before saving.');
      return;
    }
    window.localStorage.setItem('profile_username', trimmed);
    setDisplayName(trimmed);
    setSaveConfirmation(`Profile saved for ${trimmed}.`);
  };

  return (
    <div className="page-container" data-testid="profile-page">
      <div className="page-header">
        <span className="badge info">State Layer: Per-Worker Context Isolation</span>
        <h1>User Profile</h1>
        <p className="page-description">
          Demonstrates browser-storage-backed profile state. Parallel workers must isolate
          cookies and localStorage per context to avoid cross-test pollution.
        </p>
      </div>

      <div className="grid-2-col">
        <div className="card">
          <h2 className="card-title">Edit Profile</h2>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              handleSaveProfile();
            }}
            className="transfer-form"
          >
            <div className="form-group">
              <label htmlFor="username">Username</label>
              <input
                id="username"
                data-testid="username-input"
                type="text"
                value={username}
                onChange={(e) => setUsername(e.target.value)}
                placeholder="e.g. Alice"
                className="form-input"
              />
            </div>

            <div className="form-actions">
              <button
                id="save-profile"
                data-testid="save-profile-btn"
                type="submit"
                className="primary-btn"
              >
                Save Profile
              </button>
            </div>
          </form>

          {saveConfirmation && (
            <div className={saveConfirmation.startsWith('Profile saved') ? 'alert-success' : 'alert-warning'} data-testid="save-confirmation">
              {saveConfirmation}
            </div>
          )}
        </div>

        <div className="card diagnostic-card">
          <h2 className="card-title">Live Profile State</h2>
          <div className="state-indicators">
            <div className="indicator-row">
              <span>Display Name:</span>
              <strong id="display-name" data-testid="display-name" className="display-name-value">
                {displayName}
              </strong>
            </div>
            <div className="indicator-row">
              <span>Auth Token (localStorage):</span>
              <code>{window.localStorage.getItem('auth_token') || 'none'}</code>
            </div>
            <div className="indicator-row">
              <span>Session Cookie:</span>
              <code>
                {document.cookie.includes('session_id') ? 'present' : 'absent'}
              </code>
            </div>
          </div>
          <div className="code-hint">
            <h4>Isolation Strategy</h4>
            <p>
              <strong>Anti-pattern:</strong> A single shared browser context lets worker A&apos;s
              localStorage clobber worker B&apos;s mid-flight assertions.
            </p>
            <p>
              <strong>Fix:</strong> Create one <code>browser.newContext(&#123; storageState &#125;)</code>{' '}
              per parallel worker with unique session cookies and tokens.
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
