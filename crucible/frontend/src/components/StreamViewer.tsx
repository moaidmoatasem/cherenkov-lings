import React from 'react';

export const StreamViewer: React.FC = () => {
  return (
    <div className="stream-viewer" style={{ border: '1px solid #333', borderRadius: '4px', overflow: 'hidden', marginBottom: '20px' }}>
      <div className="stream-header" style={{ backgroundColor: '#222', padding: '10px', display: 'flex', alignItems: 'center', gap: '8px' }}>
        <span className="live-dot" style={{ width: '10px', height: '10px', backgroundColor: 'red', borderRadius: '50%', display: 'inline-block', animation: 'pulse 1.5s infinite' }}></span>
        <span style={{ color: '#fff', fontSize: '14px', fontWeight: 'bold' }}>Live Test Execution Stream (NoVNC/WebRTC)</span>
      </div>
      <div className="stream-content" style={{ backgroundColor: '#000', color: '#0f0', padding: '20px', minHeight: '300px', display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p>&gt;_ Mocking live device emulation stream...</p>
      </div>
    </div>
  );
};
