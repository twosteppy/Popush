import { useEffect, useMemo, useState } from 'react';
import type { Capabilities, SiteConfig, ServiceKind } from '../types/generated';
import { useSitesStore } from '../store/sites';
import { usePipelineStore } from '../store/pipeline';
import { SiteCard } from '../components/SiteCard';
import { ActionBar } from '../components/ActionBar';
import { GitPanel } from '../components/GitPanel';
import { Pipeline } from '../components/Pipeline';
import { Button } from '../components/ui/Button';
import {
  startDeploy,
  cancelPipeline,
  siteAction,
  setSshPassword,
} from '../lib/ipc';

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
  const refreshStatus = useSitesStore((s) => s.refreshStatus);

  const [actionError, setActionError] = useState<string | null>(null);
  const [actionBusy, setActionBusy] = useState(false);
  const [password, setPassword] = useState('');
  const [lastAction, setLastAction] = useState<
    'start' | 'stop' | 'restart' | null
  >(null);

  // Offer the password box whenever the failure smells like SSH auth.
  const needsAuth = actionError
    ? /passphrase|agent|authentication|refused/i.test(actionError)
    : false;

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

  useEffect(() => {
    void refreshGit(serverId, site.id);
    void refreshStatus(serverId, site.id);
  }, [serverId, site.id, refreshGit, refreshStatus]);

  function describeError(e: unknown): string {
    if (typeof e === 'string') return e;
    if (e && typeof e === 'object') {
      const o = e as { message?: string };
      if (o.message) return o.message;
      try {
        return JSON.stringify(e);
      } catch {
        /* ignore */
      }
    }
    return 'Action failed.';
  }

  async function submitPassword() {
    if (!password) return;
    setActionBusy(true);
    try {
      await setSshPassword(serverId, password);
      setPassword('');
      setActionError(null);
    } catch (e) {
      setActionError(describeError(e));
    } finally {
      setActionBusy(false);
    }
    if (lastAction) await runAction(lastAction);
  }

  async function runAction(action: 'start' | 'stop' | 'restart') {
    setLastAction(action);
    setActionBusy(true);
    setActionError(null);
    try {
      await siteAction(site.id, action);
      await refreshStatus(serverId, site.id);
      // A service can take a few seconds to answer again after a restart, so
      // check once more when the dust settles.
      window.setTimeout(() => void refreshStatus(serverId, site.id), 5000);
    } catch (e) {
      setActionError(describeError(e));
    } finally {
      setActionBusy(false);
    }
  }

  function toggleFile(path: string) {
    setSelectedFiles((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  }

  async function shipIt() {
    setActionError(null);
    begin(site.id);
    setDrawerOpen(true);
    try {
      await startDeploy(serverId, site.id, message.trim() || null);
    } catch (e) {
      setActionError(describeError(e));
    }
  }

  return (
    <div className="mx-auto flex max-w-4xl flex-col gap-5 p-6">
      <SiteCard site={site} status={status} selected />

      <ActionBar
        capabilities={caps}
        busy={busy || actionBusy}
        onShipIt={() => void shipIt()}
        onRestart={() => void runAction('restart')}
        onStop={() => void runAction('stop')}
        onLogs={() => setDrawerOpen(true)}
      />
      {actionError ? (
        <div className="flex flex-col gap-2 rounded-sm border border-status-failed bg-status-failed/10 px-3 py-2">
          <p className="text-sm text-status-failed">{actionError}</p>
          {needsAuth ? (
            <div className="flex flex-col gap-1.5">
              <div className="flex items-center gap-2">
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') void submitPassword();
                  }}
                  placeholder="SSH password for this server"
                  aria-label="SSH password"
                  className="h-8 min-w-0 flex-1 rounded-sm border-2 border-border-strong bg-surface-base px-2 font-mono text-sm text-text-primary outline-none focus:border-accent"
                />
                <Button
                  variant="secondary"
                  onClick={() => void submitPassword()}
                  disabled={actionBusy || !password}
                  className="h-8 shrink-0 px-3 text-xs"
                >
                  Connect
                </Button>
              </div>
              <p className="text-xs text-text-tertiary">
                Kept in memory for this session only. Never saved to disk.
              </p>
            </div>
          ) : null}
        </div>
      ) : null}

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
