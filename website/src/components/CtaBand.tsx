import { SITE } from '../lib/site';
import { detectOS } from '../lib/os';
import { Download, GitHub } from './Icons';
import { StarBurst } from './StarBurst';

export function CtaBand() {
  const isWindows = detectOS() === 'windows';
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
            <a
              className="btn btn-primary dl-btn"
              href={isWindows ? SITE.download.windows : SITE.download.linux}
              rel="noreferrer"
            >
              <Download />
              {isWindows ? 'Download for Windows' : 'Download for Linux'}
            </a>
            <a className="btn starbtn" href={SITE.github} rel="noreferrer">
              <GitHub size={16} />
              Star on GitHub
              <StarBurst />
            </a>
          </div>
        </div>
      </div>
    </section>
  );
}
