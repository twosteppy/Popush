// WizardContainer - wires the presentational WizardView to the backend check
// runners for the currently selected server + site. It holds the per-check
// status mirror and dispatches run/fix intents through src/lib/ipc.ts.
//
// D14: it never decides what a check or fix does; it only asks the backend and
// mirrors the CheckStatus it returns.

import { useCallback, useEffect, useState } from 'react';
import type { Check, CheckStatus } from '../types/generated';
import { runWizardCheck } from '../lib/ipc';
import { WizardView } from './WizardView';
import { Logo } from '../components/ui/Logo';

const ALL_CHECKS: Check[] = [
  'local_key_exists',
  'key_in_agent',
  'key_on_github',
  'local_remote_is_ssh',
  'test_push',
  'server_can_pull',
  'server_remote_is_ssh',
];

interface WizardContainerProps {
  serverId: string | null;
  siteId: string | null;
}

export function WizardContainer({ serverId, siteId }: WizardContainerProps) {
  const [statuses, setStatuses] = useState<Partial<Record<Check, CheckStatus>>>(
    {},
  );

  const runCheck = useCallback(
    async (check: Check) => {
      if (!serverId || !siteId) return;
      setStatuses((prev) => ({ ...prev, [check]: { status: 'running' } }));
      try {
        const result = await runWizardCheck(serverId, siteId, check);
        setStatuses((prev) => ({ ...prev, [check]: result }));
      } catch {
        // Leave the prior status; outside Tauri this simply no-ops.
      }
    },
    [serverId, siteId],
  );

  // Kick off all checks once a site is available.
  useEffect(() => {
    if (!serverId || !siteId) return;
    for (const check of ALL_CHECKS) void runCheck(check);
  }, [serverId, siteId, runCheck]);

  if (!serverId || !siteId) {
    return (
      <div className="flex h-full items-center justify-center p-8 text-center">
        <div className="max-w-sm">
          <Logo size={32} markOnly />
          <h1 className="mt-4 font-display text-lg font-semibold text-text-primary">
            Setup wizard
          </h1>
          <p className="mt-2 text-sm text-text-secondary">
            Select a site first. The wizard checks the SSH and git path between
            your machine, GitHub, and the server for that site.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl">
      <WizardView
        statuses={statuses}
        onRunCheck={(c) => void runCheck(c)}
        onFix={(c) => void runCheck(c)}
        onShowCommand={() => {
          /* Command display handled inside the row; intent-only here (D14). */
        }}
      />
    </div>
  );
}
