import React from 'react';

interface NavbarProps {
  currentPath: string;
  onNavigate: (path: string) => void;
}

export const Navbar: React.FC<NavbarProps> = ({ currentPath, onNavigate }) => {
  const navItems = [
    { path: '/', label: 'Overview', icon: '⚡' },
    { path: '/mission-control', label: 'Mission Control (XP & Drills)', icon: '🏆' },
    { path: '/checkout', label: 'Checkout (Hydration)', icon: '🛒' },
    { path: '/transfer', label: 'Transfer (Kafka Lag)', icon: '💸' },
    { path: '/search', label: 'Search (Race Condition)', icon: '🔍' },
    { path: '/shadow-dom', label: 'Shadow DOM & Iframe', icon: '🛡️' },
    { path: '/products', label: 'Catalog (Stubbing)', icon: '📦' },
    { path: '/dashboard', label: 'Dashboard (Visual)', icon: '📊' },
    { path: '/payment', label: 'Payment (frameLocator)', icon: '💳' },
    { path: '/profile', label: 'Profile (Isolation)', icon: '👤' },
    { path: '/mobile-test', label: 'Mobile Test', icon: '📱' },
  ];

  return (
    <header className="site-header">
      <div className="header-brand">
        <a
          href="/"
          onClick={(e) => {
            e.preventDefault();
            onNavigate('/');
          }}
          className="brand-logo"
        >
          <span className="brand-icon">⚛</span>
          <div className="brand-text">
            <span className="brand-title">Micro-Crucible</span>
            <span className="brand-subtitle">Cherenkov Chaos Sandbox</span>
          </div>
        </a>
      </div>

      <nav className="header-nav">
        {navItems.map((item) => (
          <a
            key={item.path}
            href={item.path}
            onClick={(e) => {
              e.preventDefault();
              onNavigate(item.path);
            }}
            className={`nav-link ${currentPath === item.path ? 'active' : ''}`}
            data-testid={`nav-${item.path.replace('/', '') || 'home'}`}
          >
            <span className="nav-icon">{item.icon}</span>
            <span>{item.label}</span>
          </a>
        ))}
      </nav>

      <div className="header-status">
        <span className="status-indicator live"></span>
        <span className="status-text">Port 8080 : LIVE</span>
      </div>
    </header>
  );
};
