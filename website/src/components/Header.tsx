import logo from '../assets/logo.svg';
import { SITE } from '../lib/site';
import { useTheme } from '../hooks/useTheme';
import { Sun, Moon } from './Icons';

export function Header() {
  const [theme, toggle] = useTheme();

  return (
    <header className="site-header">
      <div className="wrap nav">
        <a className="brand" href="#top" aria-label="Popush home">
          <img src={logo} alt="" />
          <span>Popush</span>
        </a>
        <nav className="nav-right">
          <a className="nav-link hide-sm" href="#features">
            Features
          </a>
          <a className="nav-link hide-sm" href="#how">
            How it works
          </a>
          <a className="nav-link" href={SITE.github} rel="noreferrer">
            GitHub
          </a>
          <button
            className="icon-btn"
            type="button"
            onClick={toggle}
            aria-label={`Switch to ${theme === 'dark' ? 'light' : 'dark'} theme`}
          >
            {theme === 'dark' ? <Sun /> : <Moon />}
          </button>
          <a className="btn btn-primary btn-sm" href="#download">
            Download
          </a>
        </nav>
      </div>
    </header>
  );
}
