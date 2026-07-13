// A vertical checklist of the seven setup checks with pass/fail/running/n-a
// icons, plain-English names, expandable failed rows showing the exact command,
// a "Fix it" button, and a "Show me the command instead" link.

import { useState } from 'react';
import { Check as CheckIcon, X, Loader2, MinusCircle } from 'lucide-react';
import type { Check, CheckStatus } from '../types/generated';
import { Button } from '../components/ui/Button';

// Plain-English names for each check, ordered as the enum.
const CHECK_ORDER: Check[] = [
  'local_key_exists',
  'key_in_agent',
  'key_on_github',
  'local_remote_is_ssh',
  'test_push',
  'server_can_pull',
  'server_remote_is_ssh',
];

const CHECK_NAME: Record<Check, string> = {
  local_key_exists: 'You have an SSH key on this computer',
  key_in_agent: 'Your SSH key is loaded in the agent',
  key_on_github: 'Your key is registered with GitHub',
  local_remote_is_ssh: 'This repo uses an SSH remote',
  test_push: 'A test push to GitHub succeeds',
  server_can_pull: 'The server can pull from GitHub',
  server_remote_is_ssh: 'The server repo uses an SSH remote',
};

interface WizardViewProps {
  statuses: Partial<Record<Check, CheckStatus>>;
  onRunCheck: (check: Check) => void;
  /** Preview then apply a fix for a failed check. */
  onFix: (check: Check) => void;
  /** Show the exact command for a failed check instead of auto-fixing. */
  onShowCommand: (check: Check) => void;
}

export function WizardView({
  statuses,
  onFix,
  onShowCommand,
}: WizardViewProps) {
  return (
    <div className="flex flex-col gap-2 p-6">
      <h1 className="mb-2 font-display text-lg font-semibold text-text-primary">
        Setup checks
      </h1>
      <ol className="flex flex-col gap-1">
        {CHECK_ORDER.map((check) => (
          <WizardRow
            key={check}
            check={check}
            status={statuses[check]}
            onFix={() => onFix(check)}
            onShowCommand={() => onShowCommand(check)}
          />
        ))}
      </ol>
    </div>
  );
}

function WizardRow({
  check,
  status,
  onFix,
  onShowCommand,
}: {
  check: Check;
  status: CheckStatus | undefined;
  onFix: () => void;
  onShowCommand: () => void;
}) {
  const failed = status?.status === 'fail';
  const [expanded, setExpanded] = useState(failed);

  return (
    <li className="lift-card rounded-sm border border-border-strong bg-surface-raised px-3 py-2 shadow-hard-sm">
      <div className="flex items-center gap-2">
        <RowIcon status={status} />
        <span className="text-sm text-text-primary">{CHECK_NAME[check]}</span>
        {failed ? (
          <button
            type="button"
            onClick={() => setExpanded((e) => !e)}
            className="ml-auto text-xs text-text-tertiary hover:text-text-secondary"
            aria-expanded={expanded}
          >
            {expanded ? 'Hide' : 'Details'}
          </button>
        ) : null}
      </div>

      {failed && expanded && status?.status === 'fail' ? (
        <div className="mt-2 pl-6">
          <p className="mb-2 text-xs text-text-secondary">
            {status.what_is_wrong}
          </p>
          <div className="flex items-center gap-3">
            <Button variant="primary" onClick={onFix}>
              Fix it
            </Button>
            <button
              type="button"
              onClick={onShowCommand}
              className="text-xs text-accent hover:text-accent-hover"
            >
              Show me the command instead
            </button>
          </div>
        </div>
      ) : null}

      {status?.status === 'not_applicable' ? (
        <p className="mt-1 pl-6 text-xs text-text-tertiary">{status.why}</p>
      ) : null}
    </li>
  );
}

function RowIcon({ status }: { status: CheckStatus | undefined }) {
  const common = 'shrink-0';
  switch (status?.status) {
    case 'pass':
      return (
        <CheckIcon
          size={14}
          className={`${common} text-status-running`}
          aria-label="Passed"
        />
      );
    case 'fail':
      return (
        <X
          size={14}
          className={`${common} text-status-failed`}
          aria-label="Failed"
        />
      );
    case 'running':
      return (
        <Loader2
          size={14}
          className={`${common} text-status-working motion-safe:animate-spin`}
          aria-label="Running"
        />
      );
    case 'not_applicable':
      return (
        <MinusCircle
          size={14}
          className={`${common} text-text-tertiary`}
          aria-label="Not applicable"
        />
      );
    default:
      return (
        <MinusCircle
          size={14}
          className={`${common} text-text-tertiary`}
          aria-label="Not run yet"
        />
      );
  }
}
