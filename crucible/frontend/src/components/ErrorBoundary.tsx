import React from 'react';

interface ErrorBoundaryProps {
  children: React.ReactNode;
}

interface ErrorBoundaryState {
  error: Error | null;
}

// Without this, a render-time throw anywhere in the routed page unmounts the
// whole tree to a blank <body> -- React logs "the above error occurred" and
// stops there. This turns that into a visible, recoverable message instead.
export class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error('Unhandled error in page render:', error, info.componentStack);
  }

  private handleReset = () => {
    this.setState({ error: null });
  };

  render() {
    if (this.state.error) {
      return (
        <div className="page-container">
          <div className="card" style={{ border: '1px solid var(--accent-red)' }}>
            <h2 className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span>⚠️</span> This page hit an unexpected error
            </h2>
            <p style={{ color: 'var(--text-muted)', fontSize: '13px', marginTop: '8px' }}>
              {this.state.error.message}
            </p>
            <button
              onClick={this.handleReset}
              className="secondary-btn"
              style={{ marginTop: '16px', width: 'auto', padding: '8px 18px' }}
            >
              Try again
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}
