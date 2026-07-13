// The "What is Popush?" explainer: a one-line intro, the three words the rest
// of the UI leans on (Server, Site, Ship It), the privacy stance, and a
// three-step start.

import {
  Server,
  Globe,
  Rocket,
  ShieldCheck,
  KeyRound,
  Plus,
  ListChecks,
} from 'lucide-react';
import type { ReactNode } from 'react';
import { Logo } from '../components/ui/Logo';
import { Button } from '../components/ui/Button';

interface HelpViewProps {
  /** Opens the Add Server dialog from the getting-started step. */
  onAddServer: () => void;
}

export function HelpView({ onAddServer }: HelpViewProps) {
  return (
    <div className="mx-auto flex max-w-3xl flex-col gap-8 p-6">
      <header className="flex items-start gap-3">
        <Logo size={34} markOnly />
        <div>
          <h1 className="font-display text-2xl font-semibold tracking-tight text-text-primary">
            What is Popush?
          </h1>
          <p className="mt-1.5 max-w-prose text-sm leading-relaxed text-text-secondary">
            Popush deploys your websites to servers you already own, over SSH.
            Nothing to host, no cloud dashboard, no account.
          </p>
        </div>
      </header>

      <Section title="The key ideas" icon={<ListChecks size={15} />}>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-3">
          <IdeaCard icon={<Server size={16} />} term="Server">
            A machine you rent that Popush connects to over SSH.
          </IdeaCard>
          <IdeaCard icon={<Globe size={16} />} term="Site">
            One app on a server. A server can hold several.
          </IdeaCard>
          <IdeaCard icon={<Rocket size={16} />} term="Ship It">
            One click to commit, push, build, restart, and verify.
          </IdeaCard>
        </div>
      </Section>

      <Section title="Your data stays yours" icon={<ShieldCheck size={15} />}>
        <p className="max-w-prose text-sm leading-relaxed text-text-secondary">
          No account and no sign-in. No secrets are stored: Popush keeps the
          path to your SSH key, never the key itself. Nothing leaves this
          machine.
        </p>
      </Section>

      <Section title="Getting started" icon={<KeyRound size={15} />}>
        <ol className="flex flex-col gap-3">
          <Step
            n={1}
            title="Add a server"
            body="Give Popush the host, username, and the path to your SSH key."
          />
          <Step
            n={2}
            title="Add a site"
            body="Point it at a folder on that server and its git branch."
          />
          <Step
            n={3}
            title="Press Ship It"
            body="Watch the pipeline run step by step in the log drawer."
          />
        </ol>

        <div className="mt-5">
          <Button variant="primary" onClick={onAddServer}>
            <Plus size={14} aria-hidden="true" />
            Add your first server
          </Button>
        </div>
      </Section>
    </div>
  );
}

function Section({
  title,
  icon,
  children,
}: {
  title: string;
  icon: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="flex flex-col gap-3">
      <h2 className="label-mono flex items-center gap-2 text-[11px] font-semibold text-text-tertiary">
        <span className="text-text-secondary">{icon}</span>
        {title}
      </h2>
      {children}
    </section>
  );
}

function IdeaCard({
  icon,
  term,
  children,
}: {
  icon: ReactNode;
  term: string;
  children: ReactNode;
}) {
  return (
    <div className="lift-card rounded-lg border-2 border-border-strong bg-surface-raised p-4 shadow-hard-sm">
      <h3 className="flex items-center gap-2 font-display text-sm font-semibold text-text-primary">
        <span className="text-accent">{icon}</span>
        {term}
      </h3>
      <p className="mt-1.5 text-sm leading-relaxed text-text-secondary">
        {children}
      </p>
    </div>
  );
}

function Step({ n, title, body }: { n: number; title: string; body: string }) {
  return (
    <li className="flex items-start gap-3 rounded-sm border border-border-strong bg-surface-raised p-3 shadow-hard-sm">
      <span className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-sm border border-accent bg-accent font-display text-xs font-semibold text-text-inverse">
        {n}
      </span>
      <div>
        <h3 className="text-sm font-semibold text-text-primary">{title}</h3>
        <p className="mt-0.5 text-sm leading-relaxed text-text-secondary">
          {body}
        </p>
      </div>
    </li>
  );
}
