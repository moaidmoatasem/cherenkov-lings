import React from 'react';

interface HomePageProps {
  onNavigate: (path: string) => void;
}

export const HomePage: React.FC<HomePageProps> = ({ onNavigate }) => {
  const drills = [
    {
      id: '01_hydration_timing',
      path: '/checkout',
      title: 'Drill 01: Hydration Timing Gap',
      badge: 'React 19 Trap',
      badgeClass: 'warning',
      description:
        'Tests event handler binding timing. The checkout button visualizes before event delegation is attached (800ms gap), causing naive sleep or rapid clicks to be dropped.',
      actionLabel: 'Open Checkout Page',
    },
    {
      id: '02_shadow_dom_v2',
      path: '/shadow-dom',
      title: 'Drill 02: Shadow DOM & Iframe Piercing',
      badge: 'Closed Shadow Root',
      badgeClass: 'purple',
      description:
        'Tests Web Component encapsulation piercing. Features <chaos-vault> closed shadow root containing secret tokens and an embedded cross-origin payment iframe.',
      actionLabel: 'Open Shadow DOM Page',
    },
    {
      id: '03_debounce_race_condition',
      path: '/search',
      title: 'Drill 03: Debounced Autocomplete Race',
      badge: 'Async Concurrency',
      badgeClass: 'danger',
      description:
        'Tests distributed async response ordering. Short queries have higher server latency than full queries, causing out-of-order response clobbering without synchronization.',
      actionLabel: 'Open Search Page',
    },
    {
      id: 'transfer_kafka_lag',
      path: '/transfer',
      title: 'Ledger Transfer: Kafka Lag Simulation',
      badge: 'Eventual Consistency',
      badgeClass: 'info',
      description:
        'Tests eventual consistency. Ledger transfers are submitted with dynamic X-Chaos: kafka_lag headers and require polling /balance until settled.',
      actionLabel: 'Open Transfer Page',
    },
  ];

  return (
    <div className="page-container" data-testid="home-page">
      <div className="hero-banner">
        <div className="hero-content">
          <span className="hero-badge">Cherenkov-Lings SDET Platform</span>
          <h1 className="hero-title">Micro-Crucible Target Sandbox</h1>
          <p className="hero-subtitle">
            An intentionally broken, chaos-capable testing ground for mastering modern web automation,
            flakiness eradication, and synchronization patterns.
          </p>
          <div style={{ marginTop: '16px', display: 'flex', gap: '12px' }}>
            <button
              onClick={() => onNavigate('/mission-control')}
              className="primary-btn"
              style={{ padding: '10px 20px', fontSize: '14px', fontWeight: 'bold' }}
            >
              🏆 Open Mission Control & Badges &rarr;
            </button>
            <button
              onClick={() => onNavigate('/checkout')}
              className="secondary-btn"
              style={{ padding: '10px 20px', fontSize: '14px' }}
            >
              🛒 Try Hydration Sandbox
            </button>
          </div>
        </div>
      </div>

      <div className="drills-grid">
        {drills.map((drill) => (
          <div key={drill.id} className="card drill-card" data-testid={`card-${drill.id}`}>
            <div className="card-top">
              <span className={`badge ${drill.badgeClass}`}>{drill.badge}</span>
            </div>
            <h3 className="drill-card-title">{drill.title}</h3>
            <p className="drill-card-desc">{drill.description}</p>
            <div className="drill-card-footer">
              <button
                onClick={() => onNavigate(drill.path)}
                className="secondary-btn card-action-btn"
                data-testid={`btn-goto-${drill.id}`}
              >
                {drill.actionLabel} &rarr;
              </button>
            </div>
          </div>
        ))}
      </div>

      <div className="card info-card">
        <h2 className="card-title">Sandbox Architecture & Port Allocation</h2>
        <div className="ports-table">
          <div className="port-row">
            <span className="port-tag">Frontend UI</span>
            <span className="port-num">8080</span>
            <span className="port-desc">React 18/19 SPA with pathological interaction traps</span>
          </div>
          <div className="port-row">
            <span className="port-tag">Backend API</span>
            <span className="port-num">8081</span>
            <span className="port-desc">FastAPI with dynamic X-Chaos header latency & ledger state</span>
          </div>
        </div>
      </div>
    </div>
  );
};
