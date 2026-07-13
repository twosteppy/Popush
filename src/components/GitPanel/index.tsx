import type { ChangedFile, ChangeKind, GitStatus } from '../../types/generated';

interface GitPanelProps {
  status: GitStatus | null;
  /** Set of selected (to-be-committed) file paths. */
  selected: Set<string>;
  onToggle: (path: string) => void;
  message: string;
  onMessageChange: (value: string) => void;
  /** Short sha for the clean-tree empty state. */
  sha?: string | null;
}

const CHANGE_LABEL: Record<ChangeKind, string> = {
  added: 'A',
  modified: 'M',
  deleted: 'D',
  renamed: 'R',
  untracked: '?',
};

export function GitPanel({
  status,
  selected,
  onToggle,
  message,
  onMessageChange,
  sha,
}: GitPanelProps) {
  if (!status || status.changed_files.length === 0) {
    return (
      <div className="rounded-lg border-2 border-border-strong bg-surface-raised p-4 shadow-hard-sm">
        <p className="text-sm text-text-secondary">
          Nothing to commit. Working tree is clean.
        </p>
        <p className="mt-1 font-mono text-xs text-text-tertiary">
          {status ? status.branch : 'unknown branch'}
          {sha ? ` · ${sha.slice(0, 7)}` : ''}
        </p>
      </div>
    );
  }

  return (
    <div className="rounded-lg border-2 border-border-strong bg-surface-raised p-4 shadow-hard-sm">
      <div className="mb-3 flex items-center gap-2 font-mono text-xs text-text-secondary">
        <span className="text-text-primary">{status.branch}</span>
        {status.ahead > 0 ? (
          <span aria-label={`${status.ahead} ahead`}>↑{status.ahead}</span>
        ) : null}
        {status.behind > 0 ? (
          <span aria-label={`${status.behind} behind`}>↓{status.behind}</span>
        ) : null}
      </div>

      <ul className="mb-3 flex flex-col gap-1">
        {status.changed_files.map((file: ChangedFile) => (
          <li key={file.path}>
            <label className="flex items-center gap-2 rounded-sm px-1.5 py-1 text-sm transition-colors hover:bg-surface-hover">
              <input
                type="checkbox"
                checked={selected.has(file.path)}
                onChange={() => onToggle(file.path)}
                className="h-3.5 w-3.5 accent-accent"
              />
              <span
                className="w-4 shrink-0 text-center font-mono text-xs text-text-tertiary"
                aria-label={file.change}
              >
                {CHANGE_LABEL[file.change]}
              </span>
              <span className="truncate font-mono text-xs text-text-primary">
                {file.path}
              </span>
              <span className="ml-auto shrink-0 text-xs text-text-tertiary">
                {file.staged ? 'staged' : 'unstaged'}
              </span>
            </label>
          </li>
        ))}
      </ul>

      <label className="label-mono mb-1.5 block text-[11px] font-medium text-text-secondary">
        Commit message
      </label>
      <textarea
        value={message}
        onChange={(e) => onMessageChange(e.target.value)}
        rows={3}
        placeholder="Describe this change"
        className="w-full resize-y rounded-sm border border-border-strong bg-surface-base p-2 font-mono text-xs text-text-primary placeholder:text-text-tertiary focus:border-accent focus-visible:outline focus-visible:outline-2 focus-visible:outline-accent"
      />
    </div>
  );
}
