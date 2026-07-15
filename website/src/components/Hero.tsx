import { SITE } from '../lib/site';
import { Download, GitHub, Check } from './Icons';
import { InstallCommand } from './CopyButton';
import { Terminal } from './Terminal';

export function Hero() {
  return (
    <section className="hero">
      <div className="wrap hero-grid">
        <div className="reveal">
          <h1>
            Your VPS,
            <br />
            <span className="grad">one click</span> away.
          </h1>
          <p className="hero-sub">
            A native desktop app that ships your sites over SSH. See honest live
            status, then commit, push, build, and restart with one button, all
            streamed live.
          </p>
          <div className="hero-cta">
            <a className="btn btn-primary" href="#download">
              <Download />
              Download for Linux
            </a>
            <a className="btn" href={SITE.github} rel="noreferrer">
              <GitHub />
              Star on GitHub
            </a>
          </div>
          <InstallCommand
            className="hero-cmd"
            display={SITE.installDisplay}
            command={SITE.installCommand}
          />
          <p className="hero-note">
            <Check strokeWidth={2.5} />
            Free, open source, Linux .AppImage. Nothing leaves your machine.
          </p>
        </div>
        <div className="reveal" style={{ transitionDelay: '90ms' }}>
          <Terminal />
        </div>
      </div>
    </section>
  );
}
