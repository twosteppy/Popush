import { useState, useEffect } from 'react';
import logo from '../assets/logo.svg';
import { SITE } from '../lib/site';
import { Menu, X, Download } from './Icons';

const LINKS = [
  { href: '#features', label: 'Features' },
  { href: '#how', label: 'How it works' },
  { href: '#faq', label: 'FAQ' },
  { href: SITE.github, label: 'GitHub', ext: true },
];

export function Header() {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    document.body.style.overflow = open ? 'hidden' : '';
    return () => {
      document.body.style.overflow = '';
    };
  }, [open]);

  return (
    <>
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
    </header>

    <div className={`mobile-menu${open ? ' open' : ''}`}>
      <a
        className="brand mobile-menu-brand"
        href="#top"
        aria-label="Popush home"
        onClick={() => setOpen(false)}
      >
        <img src={logo} alt="" />
        <span>Popush</span>
      </a>
      <button
        className="mobile-menu-close"
        type="button"
        aria-label="Close menu"
        onClick={() => setOpen(false)}
      >
        <X size={20} />
      </button>
      <div className="mobile-menu-inner">
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
    </>
  );
}
