// Thin, typed wrappers around the Tauri IPC surface. Each names a backend
// command and passes typed arguments; everything returned is a snapshot the UI
// mirrors, never the source of truth.

import type {
  Capabilities,
  CommandLogEntry,
  Config,
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

/** Persist a new server to config.toml. The backend owns the file. */
export async function addServer(server: ServerConfig): Promise<void> {
  return invoke<void>('add_server', { server });
}

/** Remove a server from config.toml by id. */
export async function removeServer(serverId: string): Promise<void> {
  return invoke<void>('remove_server', { serverId });
}

/** Read the whole authoritative config snapshot. */
export async function getConfig(): Promise<Config | null> {
  if (!inTauri()) return null;
  return invoke<Config>('get_config');
}

/** Absolute path to config.toml, e.g. for the "open your config" affordance. */
export async function configFilePath(): Promise<string | null> {
  if (!inTauri()) return null;
  return invoke<string>('config_file_path');
}

/**
 * Reveal a path in the user's default handler via the tauri-opener plugin.
 * No-ops outside Tauri. Used to open config.toml from the onboarding screen.
 */
export async function openPath(path: string): Promise<void> {
  if (!inTauri()) return;
  return invoke<void>('plugin:opener|open_path', { path });
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
