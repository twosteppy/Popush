import logo from '../assets/logo.svg';
import { SITE } from '../lib/site';

export function Footer() {
  return (
    <footer className="site-footer">
      <div className="wrap foot">
        <div className="brand">
          <img src={logo} alt="" />
          <span style={{ fontFamily: 'var(--font-mono)' }}>Popush</span>
        </div>
        <nav className="foot-links">
          <a href="#features">Features</a>
          <a href="#download">Download</a>
          <a href={SITE.github} rel="noreferrer">
            GitHub
          </a>
          <a href={SITE.releasesAll} rel="noreferrer">
            Releases
          </a>
        </nav>
        <span className="foot-legal">Built by twostep · GPL-3.0</span>
      </div>
    </footer>
  );
}
