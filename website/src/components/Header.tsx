import { useState } from 'react';
import logo from '../assets/logo.svg';
import { SITE } from '../lib/site';
import { useTheme } from '../hooks/useTheme';
import { Sun, Moon, Menu, X, Download } from './Icons';

const LINKS = [
  { href: '#features', label: 'Features' },
  { href: '#how', label: 'How it works' },
  { href: '#faq', label: 'FAQ' },
  { href: SITE.github, label: 'GitHub', ext: true },
];

export function Header() {
  const [theme, toggle] = useTheme();
  const [open, setOpen] = useState(false);

  return (
    <header className="site-header">
      <div className="wrap nav">
        <a className="brand" href="#top" aria-label="Popush home">
          <img src={logo} alt="" />
          <span>Popush</span>
        </a>

        <nav className="nav-right">
          {LINKS.slice(0, 3).map((l) => (
            <a
              key={l.label}
              className="nav-link hide-sm"
              href={l.href}
              {...(l.ext ? { rel: 'noreferrer' } : {})}
            >
              {l.label}
            </a>
          ))}
          <a className="nav-link hide-sm" href={SITE.github} rel="noreferrer">
            GitHub
          </a>

          <button
            className="icon-btn"
            type="button"
            onClick={toggle}
            aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
          >
            {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
          </button>

          <a className="btn btn-primary btn-sm hide-sm" href="#download">
            Download
          </a>

          <button
            className="menu-btn show-sm"
            type="button"
            aria-label={open ? 'Close menu' : 'Open menu'}
            aria-expanded={open}
            onClick={() => setOpen((o) => !o)}
          >
            {open ? <X size={18} /> : <Menu size={18} />}
          </button>
        </nav>
      </div>

      <div className={`mobile-menu${open ? ' open' : ''}`}>
        <div className="wrap mobile-menu-inner">
          {LINKS.map((l) => (
            <a
              key={l.label}
              href={l.href}
              onClick={() => setOpen(false)}
              {...(l.ext ? { rel: 'noreferrer' } : {})}
            >
              {l.label}
            </a>
          ))}
          <a
            className="btn btn-primary"
            href="#download"
            onClick={() => setOpen(false)}
          >
            <Download size={16} />
            Download for Linux
          </a>
        </div>
      </div>
    </header>
  );
}
