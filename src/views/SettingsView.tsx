// SettingsView — theme selector, poll interval, and ONE dismissible,
// never-repeated suggestion about the optional GitHub PAT (Phase 10). A single
// line, not a nag.
//
// D14: emits preference-change intents; the backend persists and owns config.

import { useState } from 'react';
import type { Theme } from '../types/generated';

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
    <div className="flex flex-col gap-6 p-6">
      <h1 className="text-lg font-semibold text-text-primary">Settings</h1>

      <section className="flex flex-col gap-2">
        <label
          htmlFor="theme-select"
          className="text-sm font-medium text-text-secondary"
        >
          Theme
        </label>
        <select
          id="theme-select"
          value={theme}
          onChange={(e) => onThemeChange(e.target.value as Theme)}
          className="w-48 rounded-md border border-border-strong bg-surface-raised px-2 py-1.5 text-sm text-text-primary"
        >
          {THEMES.map((t) => (
            <option key={t} value={t}>
              {t[0]?.toUpperCase()}
              {t.slice(1)}
            </option>
          ))}
        </select>
      </section>

      <section className="flex flex-col gap-2">
        <label
          htmlFor="poll-interval"
          className="text-sm font-medium text-text-secondary"
        >
          Poll interval (seconds, 0 disables)
        </label>
        <input
          id="poll-interval"
          type="number"
          min={0}
          value={pollIntervalSeconds}
          onChange={(e) => onPollIntervalChange(Number(e.target.value))}
          className="w-48 rounded-md border border-border-strong bg-surface-raised px-2 py-1.5 text-sm text-text-primary"
        />
      </section>

      {showPat ? (
        <p className="flex items-center gap-3 rounded-md border border-border-subtle bg-surface-raised px-3 py-2 text-sm text-text-secondary">
          <span>
            Optional: add a GitHub personal access token to check keys against
            GitHub during the setup wizard.
          </span>
          <button
            type="button"
            onClick={() => {
              setShowPat(false);
              onDismissPatSuggestion();
            }}
            className="ml-auto shrink-0 text-xs text-text-tertiary hover:text-text-secondary"
          >
            Dismiss
          </button>
        </p>
      ) : null}
    </div>
  );
}
