import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import logo from './assets/logo.svg';
import './index.css';

// Set the favicon from the bundled logo (inlined as a data URI at build time).
const favicon = document.createElement('link');
favicon.rel = 'icon';
favicon.type = 'image/svg+xml';
favicon.href = logo;
document.head.appendChild(favicon);

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
