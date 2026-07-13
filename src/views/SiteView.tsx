// Composes SiteCard, ActionBar, GitPanel, and Pipeline for the selected site.
// Wires stores to the presentational components and dispatches intents through
// src/lib/ipc.ts.

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
// for rendering.
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

  // Mirror the working-tree state for the selected site.
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
    <div className="mx-auto flex max-w-4xl flex-col gap-5 p-6">
      <SiteCard site={site} status={status} selected />

      <ActionBar
        capabilities={caps}
        busy={busy}
        onShipIt={() => void shipIt()}
        onRestart={() => void startDeploy(serverId, site.id, null)}
        onStop={() => {
          /* Stop is dispatched via ipc by the parent action. */
        }}
        onLogs={() => setDrawerOpen(true)}
      />

      <div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
        <section className="flex flex-col gap-2">
          <h2 className="label-mono px-1 text-[11px] font-semibold text-text-tertiary">
            Changes
          </h2>
          <GitPanel
            status={gitStatus ?? null}
            selected={selectedFiles}
            onToggle={toggleFile}
            message={message}
            onMessageChange={setMessage}
          />
        </section>
        <section className="flex flex-col gap-2">
          <div className="flex items-center justify-between px-1">
            <h2 className="label-mono text-[11px] font-semibold text-text-tertiary">
              Ship It pipeline
            </h2>
            {busy && pipelineId ? (
              <Button
                variant="secondary"
                onClick={() => void cancelPipeline(pipelineId)}
                className="h-7 px-2.5 text-xs"
              >
                Cancel
              </Button>
            ) : null}
          </div>
          <div className="rounded-lg border-2 border-border-strong bg-surface-raised p-4 shadow-hard-sm">
            {steps.length === 0 ? (
              <p className="text-sm text-text-tertiary">
                Press <span className="text-text-secondary">Ship It</span> to
                commit, push, build, and restart. Each step streams live.
              </p>
            ) : (
              <Pipeline />
            )}
          </div>
        </section>
      </div>
    </div>
  );
}
