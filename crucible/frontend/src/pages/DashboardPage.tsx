import React, { useState, useEffect } from 'react';

const CHAOS_STATUSES = [
  'Nominal',
  'Kafka Lag Detected',
  'Hydration Draining',
  'Chaos Injected',
];

export const DashboardPage: React.FC = () => {
  const [clock, setClock] = useState<string>('--:--:--');
  const [sessionId] = useState<string>(() =>
    Array.from({ length: 12 }, () => Math.floor(Math.random() * 16).toString(16)).join('')
  );
  const [chaosStatus, setChaosStatus] = useState<string>(CHAOS_STATUSES[0]);

  useEffect(() => {
    const formatClock = () => {
      const now = new Date();
      setClock(
        [now.getUTCHours(), now.getUTCMinutes(), now.getUTCSeconds()]
          .map((unit) => String(unit).padStart(2, '0'))
          .join(':')
      );
    };
    formatClock();
    const clockTimer = window.setInterval(formatClock, 1000);
    const chaosTimer = window.setInterval(() => {
      setChaosStatus(
        CHAOS_STATUSES[Math.floor(Math.random() * CHAOS_STATUSES.length)]
      );
    }, 2000);
    return () => {
      window.clearInterval(clockTimer);
      window.clearInterval(chaosTimer);
    };
  }, []);

  const services = [
    { name: 'checkout-api', region: 'us-east-1', uptime: '99.98%', latency: '112ms' },
    { name: 'ledger-worker', region: 'eu-west-2', uptime: '99.91%', latency: '834ms' },
    { name: 'search-suggest', region: 'us-west-1', uptime: '99.99%', latency: '45ms' },
    { name: 'auth-gateway', region: 'global', uptime: '100.00%', latency: '38ms' },
  ];

  const stats = [
    { label: 'Requests (24h)', value: '1,284,553' },
    { label: 'Error Budget Remaining', value: '87%' },
    { label: 'Active Chaos Experiments', value: '3' },
  ];

  return (
    <div className="page-container" data-testid="dashboard-page">
      <div className="page-header">
        <span className="badge purple">Visual Layer: Masked Baselines &amp; Diff Tolerance</span>
        <h1>Operations Dashboard</h1>
        <p className="page-description">
          A deterministic layout with intentionally volatile widgets (live clock, session id,
          chaos indicator). Visual drills must mask dynamic regions and apply pixel tolerance.
        </p>
      </div>

      <div className="dashboard-volatile-bar">
        <span className="volatile-widget" data-testid="live-clock">
          UTC Clock: {clock}
        </span>
        <span className="volatile-widget" data-testid="session-id">
          Session: {sessionId}
        </span>
        <span
          className={`status-pill volatile-widget ${
            chaosStatus === 'Nominal' ? 'hydrated' : 'pending'
          }`}
          data-testid="chaos-status"
        >
          {chaosStatus}
        </span>
      </div>

      <div className="dashboard-stats">
        {stats.map((stat) => (
          <div key={stat.label} className="card stat-card">
            <span className="stat-value">{stat.value}</span>
            <span className="stat-label">{stat.label}</span>
          </div>
        ))}
      </div>

      <div className="card dashboard-table-card">
        <h2 className="card-title">Service Health</h2>
        <table className="dashboard-table" data-testid="service-table">
          <thead>
            <tr>
              <th>Service</th>
              <th>Region</th>
              <th>Uptime (30d)</th>
              <th>P95 Latency</th>
            </tr>
          </thead>
          <tbody>
            {services.map((svc) => (
              <tr key={svc.name}>
                <td>{svc.name}</td>
                <td>{svc.region}</td>
                <td>{svc.uptime}</td>
                <td>{svc.latency}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="card code-hint">
        <h4>Visual Testing Strategy</h4>
        <p>
          <strong>Anti-pattern:</strong> Raw pixel diff fails on every run because the clock,
          session badge, and chaos pill mutate continuously.
        </p>
        <p>
          <strong>Fix:</strong> Pass <code>mask: [page.getByTestId('live-clock'), ...]</code> and{' '}
          <code>maxDiffPixelRatio</code> to <code>toHaveScreenshot()</code>.
        </p>
      </div>
    </div>
  );
};
