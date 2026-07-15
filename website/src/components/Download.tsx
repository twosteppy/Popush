import { SITE } from '../lib/site';
import { Download as DownloadIcon } from './Icons';
import { InstallCommand } from './CopyButton';

export function Download() {
  return (
    <section id="download">
      <div className="wrap">
        <div className="dl-box reveal">
          <div>
            <span className="label-mono">Get Popush</span>
            <h2>Download and run. No account, no setup.</h2>
            <p className="lead">
              Grab the AppImage, make it executable, and launch. Or let the
              one-line installer drop an icon in your app menu and on your
              desktop for you.
            </p>
            <div className="dl-actions">
              <a className="btn btn-primary" href={SITE.releases} rel="noreferrer">
                <DownloadIcon />
                Download .AppImage
              </a>
              <a className="btn" href={SITE.github} rel="noreferrer">
                View source
              </a>
            </div>
            <p className="dl-meta">
              Linux, x86-64, needs <code>WebKitGTK 4.1</code> and{' '}
              <code>GTK 3</code>.
              <br />
              Verified against <code>SHA256SUMS</code> on every release.
            </p>
          </div>
          <div className="dl-side">
            <span className="label-mono">Or, one line in a terminal</span>
            <InstallCommand
              display={SITE.installDisplay}
              command={SITE.installCommand}
            />
            <p className="dl-alt">
              Downloads the AppImage, checks its hash, and adds a launcher plus
              desktop icon. Prefer to build it yourself?{' '}
              <a href={SITE.readme} rel="noreferrer">
                Clone and run <code>install.sh</code> &rarr;
              </a>
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
