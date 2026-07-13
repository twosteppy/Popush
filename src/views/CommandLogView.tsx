// The "what did this app do to my server" record: a read-only, chronological
// list of every command Popush sent, with the exact command text shown
// monospaced.

import { useEffect, useState } from 'react';
import type { CommandLogEntry } from '../types/generated';
import { commandLog } from '../lib/ipc';

function formatDuration(ms: bigint): string {
  const seconds = Number(ms) / 1000;
  if (seconds < 1) return `${Number(ms)}ms`;
  return `${seconds.toFixed(1)}s`;
}

export function CommandLogView() {
  const [entries, setEntries] = useState<CommandLogEntry[]>([]);

  useEffect(() => {
    let cancelled = false;
    void commandLog().then((log) => {
      if (!cancelled) setEntries(log);
    });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <div className="flex flex-col gap-4 p-6">
      <div>
        <h1 className="font-display text-lg font-semibold text-text-primary">
          Command log
        </h1>
        <p className="mt-1 max-w-prose text-sm text-text-secondary">
          Every command Popush has run on your servers, exactly as sent.
        </p>
      </div>

      {entries.length === 0 ? (
        <p className="text-sm text-text-secondary">
          No commands have been run yet.
        </p>
      ) : (
        <div className="overflow-x-auto rounded-lg border-2 border-border-strong bg-surface-raised shadow-hard-sm">
          <table className="w-full text-left text-xs">
            <thead className="label-mono text-[10px] text-text-tertiary">
              <tr className="border-b border-border-subtle">
                <th className="px-3 py-2 font-medium">Time</th>
                <th className="px-3 py-2 font-medium">Server</th>
                <th className="px-3 py-2 font-medium">Command</th>
                <th className="px-3 py-2 font-medium">Exit</th>
                <th className="px-3 py-2 font-medium">Duration</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry, i) => (
                <tr
                  key={`${entry.timestamp}-${i}`}
                  className="border-b border-border-subtle transition-colors last:border-0 hover:bg-surface-hover"
                >
                  <td className="whitespace-nowrap px-3 py-2 font-mono text-text-tertiary">
                    {entry.timestamp}
                  </td>
                  <td className="whitespace-nowrap px-3 py-2 text-text-secondary">
                    {entry.server}
                  </td>
                  <td className="px-3 py-2 font-mono text-text-primary">
                    {entry.command}
                  </td>
                  <td className="whitespace-nowrap px-3 py-2 font-mono">
                    <span
                      className={
                        entry.exit_code === 0
                          ? 'text-status-running'
                          : entry.exit_code === null
                            ? 'text-text-tertiary'
                            : 'text-status-failed'
                      }
                    >
                      {entry.exit_code === null ? '-' : entry.exit_code}
                    </span>
                  </td>
                  <td className="whitespace-nowrap px-3 py-2 font-mono text-text-tertiary">
                    {formatDuration(entry.duration_ms)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
