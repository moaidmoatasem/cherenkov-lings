import React from 'react';

/**
 * Placeholder for an embedded NoVNC / WebRTC device stream.
 *
 * Styling lives in index.css alongside the rest of the dashboard rather than
 * inline: the inline version hardcoded its own palette and referenced a
 * `pulse` keyframe that was never defined, so the "live" indicator sat static.
 */
export const StreamViewer: React.FC = () => {
  return (
    <div className="stream-viewer" data-testid="stream-viewer">
      <div className="stream-header">
        <span className="live-dot" aria-hidden="true" />
        <span className="stream-title">Live Test Execution Stream (NoVNC/WebRTC)</span>
        <span className="stream-badge" role="status" aria-live="polite">
          LIVE
        </span>
      </div>
      <div className="stream-content">
        <p>&gt;_ Mocking live device emulation stream...</p>
      </div>
    </div>
  );
};
