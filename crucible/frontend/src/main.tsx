import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import './components/ChaosVault';
import './index.css';

const rootElement = document.getElementById('root') || document.getElementById('app');

if (rootElement) {
  ReactDOM.createRoot(rootElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
} else {
  console.error('Failed to find root DOM element (#root or #app)');
}
