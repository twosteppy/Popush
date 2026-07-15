import { useState } from 'react';
import { SITE } from '../lib/site';
import { ChevronDown } from './Icons';

const QA: { q: string; a: string }[] = [
  {
    q: 'Is my data private?',
    a: "Totally. Your servers, SSH keys, and config live in one file on your machine. Nothing is uploaded or phoned home, and the source is open so you can check for yourself.",
  },
  {
    q: 'Does it install anything on my server?',
    a: 'Nope, no agent or daemon. Popush drives your server over plain SSH, running the same commands you would by hand, just behind one button.',
  },
  {
    q: 'Which stacks does it support?',
    a: 'Docker Compose, systemd, pm2, and plain static sites. Point it at the folder, pick the type, and it handles build, restart, and health checks.',
  },
  {
    q: 'What does one click actually do?',
    a: 'Commit, push, pull on the server, rebuild, recreate the container with the new image, then wait for the site to answer healthy. Every step streams live, and Cancel stops it instantly.',
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
                  <span className="faq-badge" aria-hidden="true">
                    ?
                  </span>
                  <span className="faq-qt">{item.q}</span>
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
        <p className="faq-more reveal">
          Still curious?{' '}
          <a href={`${SITE.github}/issues`} rel="noreferrer">
            Ask on GitHub &rarr;
          </a>
        </p>
      </div>
    </section>
  );
}
