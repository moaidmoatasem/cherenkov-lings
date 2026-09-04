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
    {
      id: 'catalog_response_stubbing',
      path: '/products',
      title: 'Product Catalog',
      badge: 'Response Stubbing',
      badgeClass: 'info',
      description:
        'A paginated catalog backed by GET /products. Automation drills intercept this route to stub responses and verify rendering decoupled from backend availability.',
      actionLabel: 'Open Catalog Page',
    },
    {
      id: 'operations_dashboard',
      path: '/dashboard',
      title: 'Operations Dashboard',
      badge: 'Visual Regression',
      badgeClass: 'purple',
      description:
        'A deterministic layout with intentionally volatile widgets (live clock, session id, chaos indicator). Visual drills must mask dynamic regions and apply pixel tolerance.',
      actionLabel: 'Open Dashboard Page',
    },
    {
      id: 'user_profile_isolation',
      path: '/profile',
      title: 'User Profile',
      badge: 'Storage Isolation',
      badgeClass: 'warning',
      description:
        'Browser-storage-backed profile state. Parallel workers must isolate cookies and localStorage per context to avoid cross-test pollution.',
      actionLabel: 'Open Profile Page',
    },
    {
      id: 'payment_frame_boundary',
      path: '/payment',
      title: 'Secure Payment Gateway',
      badge: 'Iframe Boundary',
      badgeClass: 'danger',
      description:
        'Card details are collected inside a sandboxed iframe to simulate a third-party payment provider. Tests must scope locators with frameLocator instead of reaching across the frame boundary.',
      actionLabel: 'Open Payment Page',
    },
    {
      id: 'mobile_biometric_fallback',
      path: '/mobile-test',
      title: 'Cherenkov Bank (Mobile)',
      badge: 'Mobile Flows',
      badgeClass: 'info',
      description:
        'A mobile banking flow: biometric check falls back to a PIN, a deep link lands on a specific account balance, and a long product list scrolls -- mobile automation patterns without a device farm.',
      actionLabel: 'Open Mobile Test Page',
    },
    {
      id: 'code_review_mentor',
      path: '/code-review',
      title: 'Code Review & Senior QA Mentor',
      badge: 'AST Review Engine',
      badgeClass: 'purple',
      description:
        'Runs test automation code against the backend AST review engine, surfaces anti-patterns with Socratic critiques, and can apply a suggested fix back into the editor.',
      actionLabel: 'Open Code Review',
    },
    {
      id: 'pipeline_builder',
      path: '/pipeline-builder',
      title: 'CI/CD Pipeline Simulator & Workflow Builder',
      badge: 'CI/CD Simulator',
      badgeClass: 'warning',
      description:
        'Design a parallel test-matrix GitHub Actions workflow, validate it against enterprise SDET rules, and run it against the real pipeline backend.',
      actionLabel: 'Open Pipeline Builder',
    },
    {
      id: 'allure_triage',
      path: '/allure-triage',
      title: 'Enterprise Allure Reports & Triage Station',
      badge: 'Root-Cause Triage',
      badgeClass: 'danger',
      description:
        'Correlate failing tests against proxy logs, tell a product defect from flaky infrastructure, and submit a root-cause hypothesis for the backend to grade.',
      actionLabel: 'Open Allure Triage',
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
