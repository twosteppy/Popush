import { SITE } from '../lib/site';
import { detectOS } from '../lib/os';
import { Download as DownloadIcon } from './Icons';
import { InstallCommand } from './CopyButton';

export function Download() {
  const os = detectOS();
  const isWindows = os === 'windows';
  const osName =
    os === 'windows' ? 'Windows' : os === 'mac' ? 'macOS' : 'Linux';
  const winBtn = (
    <a
      className={`btn dl-btn ${isWindows ? 'btn-primary' : ''}`}
      href={SITE.download.windows}
      rel="noreferrer"
    >
      <DownloadIcon />
      Download for Windows
    </a>
  );
  const linuxBtn = (
    <a
      className={`btn dl-btn ${isWindows ? '' : 'btn-primary'}`}
      href={SITE.download.linux}
      rel="noreferrer"
    >
      <DownloadIcon />
      Download for Linux
    </a>
  );
  return (
    <section id="download">
      <div className="wrap">
        <div className="dl-box reveal">
          <div>
            <span className="label-mono">Get Popush</span>
            <h2>Download and run. No account, no setup.</h2>
            <p className="lead">
              Grab the installer for your OS and launch. On Linux you can also
              let the one-line installer drop an icon in your app menu and on
              your desktop for you.
            </p>
            <span className="dl-detect">
              <span className="dot" />
              {os === 'mac' ? (
                <>
                  We detected <b>macOS</b>. Pick a build below.
                </>
              ) : (
                <>
                  We detected <b>{osName}</b> and featured it below.
                </>
              )}
            </span>
            <div className="dl-actions">
              {isWindows ? (
                <>
                  {winBtn}
                  {linuxBtn}
                </>
              ) : (
                <>
                  {linuxBtn}
                  {winBtn}
                </>
              )}
              <a className="btn" href={SITE.github} rel="noreferrer">
                View source
              </a>
            </div>
            <p className="dl-meta">
              Windows 10/11 (x64) or Linux x86-64 (needs{' '}
              <code>WebKitGTK 4.1</code> and <code>GTK 3</code>).
              <br />
              Verified against <code>SHA256SUMS</code> on every release.
            </p>
          </div>
          <div className="dl-side">
            <span className="label-mono">Or, one line on Linux</span>
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
