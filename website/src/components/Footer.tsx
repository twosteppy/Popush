import logo from '../assets/logo.svg';
import { SITE } from '../lib/site';

export function Footer() {
  return (
    <footer className="site-footer">
      <div className="wrap">
        <div className="foot-flowers" aria-hidden="true">
          ✿ ✦ ✿ ✦ ✿
        </div>
        <div className="foot-top">
          <div className="foot-brand">
            <a className="brand" href="#top" aria-label="Popush home">
              <img src={logo} alt="" />
              <span>Popush</span>
            </a>
            <p className="foot-tag">
              Your VPS, one click away. Deploy over SSH with one honest button,
              and nothing ever leaves your machine.
            </p>
          </div>

          <nav className="foot-col" aria-label="Product">
            <span className="label-mono">Product</span>
            <a href="#features">Features</a>
            <a href="#how">How it works</a>
            <a href="#download">Download</a>
            <a href="#faq">FAQ</a>
          </nav>

          <nav className="foot-col" aria-label="Project">
            <span className="label-mono">Project</span>
            <a href={SITE.github} rel="noreferrer">
              GitHub
            </a>
            <a href={SITE.releasesAll} rel="noreferrer">
              Releases
            </a>
            <a href={`${SITE.github}/issues`} rel="noreferrer">
              Issues
            </a>
            <a href="https://www.gnu.org/licenses/gpl-3.0.html" rel="noreferrer">
              License
            </a>
          </nav>
        </div>

        <div className="foot-bottom">
          <span className="foot-legal">
            made with <span className="heart">♥</span> by twostep
          </span>
          <span className="foot-legal">GPL-3.0 · popush.dev</span>
        </div>
      </div>
    </footer>
  );
}
