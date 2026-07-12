// SiteView — composes SiteCard + ActionBar + GitPanel + Pipeline for the
// selected site.
//
// D14: this view wires stores to presentational components and dispatches
// intents through src/lib/ipc.ts. It contains no deployment logic.

import { useEffect, useMemo, useState } from 'react';
import type { Capabilities, SiteConfig, ServiceKind } from '../types/generated';
import { useSitesStore } from '../store/sites';
import { usePipelineStore } from '../store/pipeline';
import { SiteCard } from '../components/SiteCard';
import { ActionBar } from '../components/ActionBar';
import { GitPanel } from '../components/GitPanel';
import { Pipeline } from '../components/Pipeline';
import { Button } from '../components/ui/Button';
import { startDeploy, cancelPipeline } from '../lib/ipc';

// Capabilities are reported by the backend adapter. Until the real value is
// wired through an IPC call, derive a conservative default from service_type
// purely for rendering; the backend remains authoritative (§6.3).
function defaultCapabilities(kind: ServiceKind): Capabilities {
  switch (kind) {
    case 'static':
      return {
        can_start_stop: false,
        can_restart: false,
        has_logs: false,
        status_is_reliable: false,
      };
    default:
      return {
        can_start_stop: true,
        can_restart: true,
        has_logs: true,
        status_is_reliable: true,
      };
  }
}

interface SiteViewProps {
  serverId: string;
  site: SiteConfig;
  capabilities?: Capabilities;
}

export function SiteView({ serverId, site, capabilities }: SiteViewProps) {
  const status = useSitesStore((s) => s.statusBySite[site.id]);
  const gitStatus = useSitesStore((s) => s.gitBySite[site.id]);
  const refreshGit = useSitesStore((s) => s.refreshGit);

  const pipelineId = usePipelineStore((s) => s.pipelineId);
  const steps = usePipelineStore((s) => s.steps);
  const finished = usePipelineStore((s) => s.finished);
  const begin = usePipelineStore((s) => s.begin);
  const setDrawerOpen = usePipelineStore((s) => s.setDrawerOpen);

  const [selectedFiles, setSelectedFiles] = useState<Set<string>>(new Set());
  const [message, setMessage] = useState('');

  const caps = capabilities ?? defaultCapabilities(site.service_type);
  const busy = useMemo(
    () => steps.length > 0 && !finished,
    [steps.length, finished],
  );

  // Mirror the working-tree state for the selected site (§6.3).
  useEffect(() => {
    void refreshGit(serverId, site.id);
  }, [serverId, site.id, refreshGit]);

  function toggleFile(path: string) {
    setSelectedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  async function shipIt() {
    begin(site.id);
    setDrawerOpen(true);
    await startDeploy(serverId, site.id, message.trim() || null);
  }

  return (
    <div className="flex flex-col gap-4 p-6">
      <SiteCard site={site} status={status} selected />

      <ActionBar
        capabilities={caps}
        busy={busy}
        onShipIt={() => void shipIt()}
        onRestart={() => void startDeploy(serverId, site.id, null)}
        onStop={() => {
          /* Stop dispatched via ipc by the parent action; intent only (D14). */
        }}
        onLogs={() => setDrawerOpen(true)}
      />

      <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
        <GitPanel
          status={gitStatus ?? null}
          selected={selectedFiles}
          onToggle={toggleFile}
          message={message}
          onMessageChange={setMessage}
        />
        <div className="rounded-lg border border-border-subtle bg-surface-raised p-4">
          <div className="mb-2 flex items-center justify-between">
            <h2 className="text-sm font-semibold text-text-secondary">
              Pipeline
            </h2>
            {busy && pipelineId ? (
              <Button
                variant="secondary"
                onClick={() => void cancelPipeline(pipelineId)}
              >
                Cancel
              </Button>
            ) : null}
          </div>
          <Pipeline />
        </div>
      </div>
    </div>
  );
}
