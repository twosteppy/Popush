import { useState } from 'react';
import { Palette, Timer, Github, X } from 'lucide-react';
import type { Theme } from '../types/generated';
import { Field, SelectInput, NumberField } from '../components/ui/Field';

interface SettingsViewProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  pollIntervalSeconds: number;
  onPollIntervalChange: (seconds: number) => void;
  /** Whether the one-time PAT suggestion has already been dismissed. */
  patSuggestionDismissed: boolean;
  onDismissPatSuggestion: () => void;
}

const THEMES: Theme[] = ['system', 'dark', 'light'];

function titleCase(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1);
}

export function SettingsView({
  theme,
  onThemeChange,
  pollIntervalSeconds,
  onPollIntervalChange,
  patSuggestionDismissed,
  onDismissPatSuggestion,
}: SettingsViewProps) {
  const [showPat, setShowPat] = useState(!patSuggestionDismissed);

  return (
    <div className="mx-auto flex max-w-2xl flex-col gap-6 p-6">
      <header>
        <h1 className="font-display text-xl font-semibold tracking-tight text-text-primary">
          Settings
        </h1>
        <p className="mt-1 text-sm text-text-secondary">
          Preferences are stored locally in your config file.
        </p>
      </header>

      <Card icon={<Palette size={15} />} title="Appearance">
        <Field
          label="Theme"
          htmlFor="theme-select"
          hint="Follow the system, or pin a look."
        >
          <SelectInput
            id="theme-select"
            value={theme}
            onChange={(e) => onThemeChange(e.target.value as Theme)}
            className="max-w-xs"
          >
            {THEMES.map((t) => (
              <option key={t} value={t}>
                {titleCase(t)}
              </option>
            ))}
          </SelectInput>
        </Field>
      </Card>

      <Card icon={<Timer size={15} />} title="Polling">
        <Field
          label="Status poll interval"
          htmlFor="poll-interval"
          hint="Seconds between background status checks. 0 disables polling."
        >
          <NumberField
            id="poll-interval"
            min={0}
            value={pollIntervalSeconds}
            onValueChange={(v) => onPollIntervalChange(Number(v))}
            className="max-w-xs"
          />
        </Field>
      </Card>

      <Card icon={<Github size={15} />} title="GitHub">
        {showPat ? (
          <div className="flex items-start gap-3 rounded-md border border-border-subtle bg-surface-base px-3 py-2.5">
            <p className="text-sm text-text-secondary">
              Optional: add a GitHub personal access token so the setup wizard
              can verify your keys against GitHub. Popush works fine without it.
            </p>
            <button
              type="button"
              aria-label="Dismiss suggestion"
              onClick={() => {
                setShowPat(false);
                onDismissPatSuggestion();
              }}
              className="ml-auto shrink-0 rounded p-1 text-text-tertiary transition-colors hover:bg-surface-hover hover:text-text-secondary"
            >
              <X size={14} aria-hidden="true" />
            </button>
          </div>
        ) : (
          <p className="text-sm text-text-tertiary">
            No token configured. Popush uses your local SSH keys to deploy.
          </p>
        )}
      </Card>
    </div>
  );
}

function Card({
  icon,
  title,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <section className="lift-card rounded-lg border-2 border-border-strong bg-surface-raised p-5 shadow-hard-sm">
      <h2 className="label-mono mb-4 flex items-center gap-2 text-xs font-semibold text-text-primary">
        <span className="text-text-tertiary">{icon}</span>
        {title}
      </h2>
      {children}
    </section>
  );
}
