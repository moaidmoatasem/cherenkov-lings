import React, { useState, useEffect } from 'react';
import { Navbar } from './components/Navbar';
import { HomePage } from './pages/HomePage';
import { CheckoutPage } from './pages/CheckoutPage';
import { TransferPage } from './pages/TransferPage';
import { SearchPage } from './pages/SearchPage';
import { ShadowDomPage } from './pages/ShadowDomPage';
import { CatalogPage } from './pages/CatalogPage';
import { DashboardPage } from './pages/DashboardPage';
import { ProfilePage } from './pages/ProfilePage';
import { PaymentPage } from './pages/PaymentPage';
import { MissionControlPage } from './pages/MissionControlPage';
import { MobileTestPage } from './pages/MobileTestPage';
import { CodeReviewPage } from './pages/CodeReviewPage';
import { PipelineBuilderPage } from './pages/PipelineBuilderPage';
import { AllureTriagePage } from './pages/AllureTriagePage';
import { LearnApp } from './learn/LearnApp';

export const App: React.FC = () => {
  const [currentPath, setCurrentPath] = useState<string>(() => {
    return window.location.pathname || '/';
  });

  useEffect(() => {
    const handlePopState = () => {
      setCurrentPath(window.location.pathname || '/');
    };

    window.addEventListener('popstate', handlePopState);
    return () => window.removeEventListener('popstate', handlePopState);
  }, []);

  const handleNavigate = (path: string) => {
    window.history.pushState({}, '', path);
    setCurrentPath(path);
    window.scrollTo(0, 0);
  };

  const renderPage = () => {
    switch (currentPath) {
      case '/mission-control':
        return <MissionControlPage />;
      case '/code-review':
        return <CodeReviewPage />;
      case '/pipeline-builder':
        return <PipelineBuilderPage />;
      case '/allure-triage':
        return <AllureTriagePage />;
      case '/checkout':
        return <CheckoutPage />;
      case '/transfer':
        return <TransferPage />;
      case '/search':
        return <SearchPage />;
      case '/shadow-dom':
        return <ShadowDomPage />;
      case '/products':
        return <CatalogPage />;
      case '/dashboard':
        return <DashboardPage />;
      case '/profile':
        return <ProfilePage />;
      case '/payment':
        return <PaymentPage />;
      case '/mobile-test':
        return <MobileTestPage />;
      case '/':
      default:
        return <HomePage onNavigate={handleNavigate} />;
    }
  };

  // The learning environment brings its own full-height shell, so it renders
  // outside the sandbox chrome rather than inside the navbar/footer layout.
  if (currentPath === '/learn') {
    return <LearnApp onExit={() => handleNavigate('/')} />;
  }

  return (
    <div className="app-layout">
      <Navbar currentPath={currentPath} onNavigate={handleNavigate} />
      <main className="main-content">{renderPage()}</main>
      <footer className="site-footer">
        <div className="footer-content">
          <span>Cherenkov-Lings Crucible Sandbox &copy; 2026</span>
          <span className="footer-divider">•</span>
          <span>FastAPI (8081) & React (8080)</span>
          <span className="footer-divider">•</span>
          <span>Zero Cloud • Zero Flakiness</span>
        </div>
      </footer>
    </div>
  );
};
