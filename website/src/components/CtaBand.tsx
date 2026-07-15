import { SITE } from '../lib/site';
import { Download, GitHub } from './Icons';

export function CtaBand() {
  return (
    <section aria-label="Download Popush">
      <div className="wrap">
        <div className="cta reveal">
          <h2>Ship your next change in one click.</h2>
          <p>
            Free, open source, and yours in under a minute. Download Popush and
            give your VPS the deploy button it always needed.
          </p>
          <div className="cta-actions">
            <a className="btn btn-primary" href={SITE.releases} rel="noreferrer">
              <Download />
              Download for Linux
            </a>
            <a className="btn" href={SITE.github} rel="noreferrer">
              <GitHub />
              Star on GitHub
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
