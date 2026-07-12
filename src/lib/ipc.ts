// Thin, typed wrappers around the Tauri IPC surface.
//
// D14: the frontend contains NO business logic. These wrappers only marshal
// intents to the backend and marshal state back. They do not decide what a
// command does; they name a backend command and pass typed arguments.
//
// §6.3: the backend is authoritative. Everything returned here is a snapshot
// the UI mirrors; it is never the source of truth.

import type {
  Capabilities,
  CommandLogEntry,
  Fix,
  FixPreview,
  GitStatus,
  PipelineState,
  ServerConfig,
  SiteConfig,
  SiteStatus,
  Check,
  CheckStatus,
} from '../types/generated';

/**
 * True when running inside the Tauri shell. Outside Tauri (dev server, tests)
 * we return empty/mock data so nothing crashes.
 */
function inTauri(): boolean {
  return (
    typeof window !== 'undefined' &&
    typeof (window as { __TAURI__?: unknown }).__TAURI__ !== 'undefined'
  );
}

async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (!inTauri()) {
    throw new Error(`IPC command "${command}" is unavailable outside Tauri.`);
  }
  const { invoke: tauriInvoke } = await import('@tauri-apps/api/core');
  return tauriInvoke<T>(command, args);
}

/** Subscribe to a backend event. Returns an unsubscribe function. */
export async function listen<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<() => void> {
  if (!inTauri()) {
    return () => {};
  }
  const { listen: tauriListen } = await import('@tauri-apps/api/event');
  const unlisten = await tauriListen<T>(event, (e) => handler(e.payload));
  return unlisten;
}

export async function listServers(): Promise<ServerConfig[]> {
  if (!inTauri()) return [];
  return invoke<ServerConfig[]>('list_servers');
}

export async function listSites(serverId: string): Promise<SiteConfig[]> {
  if (!inTauri()) return [];
  return invoke<SiteConfig[]>('list_sites', { serverId });
}

export async function getSiteStatus(
  serverId: string,
  siteId: string,
): Promise<SiteStatus | null> {
  if (!inTauri()) return null;
  return invoke<SiteStatus>('get_site_status', { serverId, siteId });
}

export async function startDeploy(
  serverId: string,
  siteId: string,
  commitMessage: string | null,
): Promise<void> {
  return invoke<void>('start_deploy', { serverId, siteId, commitMessage });
}

export async function cancelPipeline(pipelineId: string): Promise<void> {
  return invoke<void>('cancel_pipeline', { pipelineId });
}

export async function gitStatus(
  serverId: string,
  siteId: string,
): Promise<GitStatus | null> {
  if (!inTauri()) return null;
  return invoke<GitStatus>('git_status', { serverId, siteId });
}

export async function commandLog(): Promise<CommandLogEntry[]> {
  if (!inTauri()) return [];
  return invoke<CommandLogEntry[]>('command_log');
}

export async function appCredit(): Promise<string> {
  if (!inTauri()) return 'twostep';
  return invoke<string>('app_credit');
}

export async function runWizardCheck(
  serverId: string,
  siteId: string,
  check: Check,
): Promise<CheckStatus> {
  return invoke<CheckStatus>('run_wizard_check', { serverId, siteId, check });
}

export async function applyWizardFix(
  serverId: string,
  siteId: string,
  fix: Fix,
): Promise<FixPreview> {
  return invoke<FixPreview>('apply_wizard_fix', { serverId, siteId, fix });
}

export type { Capabilities, PipelineState };
