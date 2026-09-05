import React, { useEffect, useState } from 'react';
import { apiUrl } from '../lib/api';

interface NavbarProps {
  currentPath: string;
  onNavigate: (path: string) => void;
}

// The badge used to hardcode "Port 8080 : LIVE" no matter what backend this
// build actually points at, and no matter whether it answered. Derive the
// port from the same VITE_API_BASE apiUrl() uses, and poll the real /health
// endpoint so "LIVE" means something.
const backendPort = (() => {
  try {
    return new URL(apiUrl('/')).port || '80';
  } catch {
    return '?';
  }
})();

export const Navbar: React.FC<NavbarProps> = ({ currentPath, onNavigate }) => {
  const [isLive, setIsLive] = useState(false);

  useEffect(() => {
    let cancelled = false;
    const checkHealth = () => {
      fetch(apiUrl('/health'))
        .then((res) => {
          if (!cancelled) setIsLive(res.ok);
        })
        .catch(() => {
          if (!cancelled) setIsLive(false);
        });
    };
    checkHealth();
    const interval = window.setInterval(checkHealth, 15000);
    return () => {
      cancelled = true;
      window.clearInterval(interval);
    };
  }, []);

  const navItems = [
    // '/' serves the Learn environment, so the sandbox overview lives at
    // /sandbox. Pointing this entry at '/' made it a duplicate of the Learn
    // link and left HomePage with no way in.
    { path: '/sandbox', label: 'Overview', icon: '⚡' },
    { path: '/learn', label: 'Learn', icon: '📖' },
    { path: '/mission-control', label: 'Mission Control (XP & Drills)', icon: '🏆' },
    { path: '/code-review', label: 'Code Review', icon: '🧐' },
    { path: '/pipeline-builder', label: 'CI/CD Builder', icon: '🚀' },
    { path: '/allure-triage', label: 'Allure & Triage', icon: '📊' },
    { path: '/checkout', label: 'Checkout (Hydration)', icon: '🛒' },
    { path: '/transfer', label: 'Transfer (Kafka Lag)', icon: '💸' },
    { path: '/search', label: 'Search (Race Condition)', icon: '🔍' },
    { path: '/shadow-dom', label: 'Shadow DOM & Iframe', icon: '🛡️' },
    { path: '/products', label: 'Catalog (Stubbing)', icon: '📦' },
    { path: '/dashboard', label: 'Dashboard (Visual)', icon: '📈' },
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
        <span className={`status-indicator ${isLive ? 'live' : 'offline'}`}></span>
        <span className="status-text">Port {backendPort} : {isLive ? 'LIVE' : 'OFFLINE'}</span>
      </div>
    </header>
  );
};
