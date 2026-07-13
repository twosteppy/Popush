// HelpView - the "What is Popush?" explainer. Plain, friendly onboarding for
// people who open the app and are not sure what it does. It defines the few
// words the rest of the UI leans on (Server, Site, Ship It, the setup wizard),
// states the privacy stance, and lays out a 3-step getting-started path.
//
// D14: presentation only. The single CTA dispatches an "add server" intent.

import {
  Server,
  Globe,
  Rocket,
  Wand2,
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
            Popush is a desktop app for running and deploying your websites on
            your own servers over SSH. You point it at a machine you already
            rent, and it commits, ships, and restarts your site for you. There
            is nothing to host and no dashboard in the cloud.
          </p>
        </div>
      </header>

      <Section title="The key ideas" icon={<ListChecks size={15} />}>
        <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          <IdeaCard icon={<Server size={16} />} term="Server">
            One of your machines, usually a VPS. It is the computer Popush
            connects to over SSH to do the actual work.
          </IdeaCard>
          <IdeaCard icon={<Globe size={16} />} term="Site">
            A single app or website living on a server. A server can hold
            several sites, each with its own path, branch, and service.
          </IdeaCard>
          <IdeaCard icon={<Rocket size={16} />} term="Ship It">
            The one-click deploy. Popush runs commit, push, pull, build,
            restart, and verify in order, and streams each step live.
          </IdeaCard>
          <IdeaCard icon={<Wand2 size={16} />} term="Setup wizard">
            A one-time helper that checks the SSH and GitHub path between your
            machine, GitHub, and the server, and offers to fix what is broken.
          </IdeaCard>
        </div>
      </Section>

      <Section title="Your data stays yours" icon={<ShieldCheck size={15} />}>
        <ul className="flex flex-col gap-2 text-sm text-text-secondary">
          <Point>
            No account and no sign-in. Popush runs entirely on this computer.
          </Point>
          <Point>
            No secrets are stored. Popush keeps the path to your SSH key, never
            the key itself, and never a password.
          </Point>
          <Point>
            No telemetry. Nothing about your servers, keys, or deployments
            leaves your machine.
          </Point>
        </ul>
      </Section>

      <Section title="Getting started" icon={<KeyRound size={15} />}>
        <ol className="flex flex-col gap-3">
          <Step
            n={1}
            title="Create or load an SSH key"
            body="If you already deploy over SSH you are set. If not, the setup wizard can create a key and register it with GitHub for you."
          />
          <Step
            n={2}
            title="Add a server"
            body="Give Popush the host, username, and the path to your key. Nothing is tested against the network until you ask it to."
          />
          <Step
            n={3}
            title="Add a site and Ship It"
            body="Point a site at a folder on that server, then press Ship It. Watch the pipeline run, step by step, in the log drawer."
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
    <div className="rounded-lg border-2 border-border-strong bg-surface-raised p-4 shadow-hard-sm">
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

function Point({ children }: { children: ReactNode }) {
  return (
    <li className="flex items-start gap-2">
      <span
        aria-hidden="true"
        className="mt-1.5 inline-block h-1.5 w-1.5 shrink-0 bg-accent"
      />
      <span className="leading-relaxed">{children}</span>
    </li>
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
