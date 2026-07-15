import type { MouseEvent, ReactNode } from 'react';
import { Clock, Rocket, Stack, Lock } from './Icons';

interface Feature {
  icon: ReactNode;
  title: string;
  body: string;
}

const FEATURES: Feature[] = [
  {
    icon: <Clock />,
    title: 'Honest status',
    body: "Green when your site actually answers over HTTPS, amber when Popush isn't sure. It shows the truth a visitor sees, never an optimistic guess.",
  },
  {
    icon: <Rocket />,
    title: 'One button, whole deploy',
    body: 'Commit, push, pull, build, restart, verify. Every step streams its logs live, waits for the site to come healthy, and cancels the moment you say so.',
  },
  {
    icon: <Stack />,
    title: 'Your stack, your rules',
    body: 'Docker Compose, systemd, pm2, or a plain static site. Popush drives them over ordinary SSH, with nothing extra to install on the server.',
  },
  {
    icon: <Lock />,
    title: 'Local and private',
    body: 'Your servers, keys, and config live in a file on your machine. Nothing is uploaded, tracked, or phoned home, ever. Verify it; the source is open.',
  },
];

function track(e: MouseEvent<HTMLElement>) {
  const el = e.currentTarget;
  const r = el.getBoundingClientRect();
  el.style.setProperty('--mx', `${e.clientX - r.left}px`);
  el.style.setProperty('--my', `${e.clientY - r.top}px`);
}

export function Features() {
  return (
    <section id="features">
      <div className="wrap">
        <div className="sec-head reveal">
          <span className="label-mono">Why Popush</span>
          <h2>The deploy button your server always needed.</h2>
          <p>
            No web dashboard, no agent to install on the box, no yaml to
            babysit. Just SSH, your git repo, and one honest button.
          </p>
        </div>
        <div className="features">
          {FEATURES.map((f, i) => (
            <article
              className="card reveal"
              key={f.title}
              style={{ transitionDelay: `${i * 70}ms` }}
              onMouseMove={track}
            >
              <div className="kico">{f.icon}</div>
              <h3>{f.title}</h3>
              <p>{f.body}</p>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
