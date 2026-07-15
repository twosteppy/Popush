import type { ReactNode } from 'react';
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
    body: "Green when your site actually answers over HTTPS, amber when Popush isn't sure. It shows the truth a visitor sees — never an optimistic guess.",
  },
  {
    icon: <Rocket />,
    title: 'One button, whole deploy',
    body: 'Commit → push → pull → build → restart → verify. Every step streams its logs live, waits for the site to come healthy, and cancels the moment you say so.',
  },
  {
    icon: <Stack />,
    title: 'Your stack, your rules',
    body: 'Docker Compose, systemd, pm2, or a plain static site. Popush drives them over ordinary SSH — there is nothing extra to install on the server.',
  },
  {
    icon: <Lock />,
    title: 'Local & private',
    body: 'Your servers, keys, and config live in a file on your machine. Nothing is uploaded, tracked, or phoned home — ever. Verify it; the source is open.',
  },
];

export function Features() {
  return (
    <section id="features">
      <div className="wrap">
        <div className="sec-head">
          <span className="label-mono">Why Popush</span>
          <h2>The deploy button your server always needed.</h2>
          <p>
            No web dashboard, no agent to install on the box, no yaml to
            babysit. Just SSH, your git repo, and one honest button.
          </p>
        </div>
        <div className="features">
          {FEATURES.map((f) => (
            <article className="card" key={f.title}>
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
