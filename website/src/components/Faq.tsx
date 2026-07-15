import { useState } from 'react';
import { ChevronDown } from './Icons';

const QA: { q: string; a: string }[] = [
  {
    q: 'Is my data private?',
    a: 'Yes. Your servers, SSH keys, and config live in a file on your machine. Popush never uploads, tracks, or phones anything home, and the source is open so you can verify it.',
  },
  {
    q: 'Does it install anything on my server?',
    a: 'No agent, no daemon, nothing. Popush drives your server over ordinary SSH, running the same commands you would run by hand, just behind one button.',
  },
  {
    q: 'Which stacks does it support?',
    a: 'Docker Compose, systemd, pm2, and plain static sites. Point Popush at the folder, pick the type, and it handles build, restart, and health checks for that stack.',
  },
  {
    q: 'What does one click actually do?',
    a: 'It commits and pushes your changes, pulls them on the server, rebuilds, recreates the container with the new image, and waits for the site to answer healthy. Every step streams live, and Cancel stops it instantly.',
  },
  {
    q: 'How much does it cost?',
    a: 'Nothing. Popush is free and open source under GPL-3.0. Download the AppImage or build it from source.',
  },
];

export function Faq() {
  const [open, setOpen] = useState<number | null>(0);

  return (
    <section id="faq">
      <div className="wrap">
        <div className="sec-head reveal">
          <span className="label-mono">Questions</span>
          <h2>Good to know.</h2>
        </div>
        <div className="faq reveal">
          {QA.map((item, i) => {
            const isOpen = open === i;
            return (
              <div className={`faq-item${isOpen ? ' open' : ''}`} key={item.q}>
                <button
                  type="button"
                  className="faq-q"
                  aria-expanded={isOpen}
                  onClick={() => setOpen(isOpen ? null : i)}
                >
                  {item.q}
                  <ChevronDown className="chev" />
                </button>
                <div className="faq-a">
                  <div>
                    <p>{item.a}</p>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </section>
  );
}
