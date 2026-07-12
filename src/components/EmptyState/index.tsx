// EmptyState — the first-run onboarding hero (highest-priority). When there
// are no servers it presents the Popush mark, a tagline, one honest line about
// what the app does, and a large primary CTA that opens the Add Server dialog.
// Secondary muted affordances let power users open the raw config.toml or run
// the setup wizard.
//
// D14: no logic here — it dispatches intents (open dialog, open the config
// file via the ipc opener wrapper).

import { motion, useReducedMotion } from 'framer-motion';
import { FolderOpen, Wand2, Plus } from 'lucide-react';
import { Logo } from '../ui/Logo';
import { Button } from '../ui/Button';
import { configFilePath, openPath } from '../../lib/ipc';

interface EmptyStateProps {
  /** True when servers exist but none/no site is selected. */
  hasServers: boolean;
  onAddServer: () => void;
  onRunWizard: () => void;
}

export function EmptyState({
  hasServers,
  onAddServer,
  onRunWizard,
}: EmptyStateProps) {
  const reduce = useReducedMotion();

  async function openConfig() {
    const path = await configFilePath();
    if (path) await openPath(path);
  }

  if (hasServers) {
    return (
      <div className="flex h-full items-center justify-center p-8 text-center">
        <div className="max-w-sm">
          <p className="text-sm text-text-secondary">
            Select a site from the sidebar to see its status, git changes, and
            the Ship It pipeline.
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full items-center justify-center p-8">
      <motion.div
        initial={reduce ? false : { opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.25, ease: 'easeOut' }}
        className="flex w-full max-w-md flex-col items-center text-center"
      >
        <Logo size={44} />
        <h1 className="mt-6 text-2xl font-semibold tracking-tight text-text-primary">
          Your VPS, one click away.
        </h1>
        <p className="mt-2 max-w-sm text-sm leading-relaxed text-text-secondary">
          Popush deploys your sites straight from this machine over SSH — no
          account, no Popush server, nothing leaves your computer.
        </p>

        <Button
          variant="primary"
          onClick={onAddServer}
          className="mt-7 h-[40px] px-5 text-[15px]"
        >
          <Plus size={16} aria-hidden="true" />
          Add your first server
        </Button>

        <div className="mt-6 flex flex-col items-center gap-2 text-xs text-text-tertiary">
          <button
            type="button"
            onClick={() => void openConfig()}
            className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 hover:text-text-secondary"
          >
            <FolderOpen size={13} aria-hidden="true" />
            Prefer TOML? Open your config file
          </button>
          <button
            type="button"
            onClick={onRunWizard}
            className="inline-flex items-center gap-1.5 rounded-md px-2 py-1 hover:text-text-secondary"
          >
            <Wand2 size={13} aria-hidden="true" />
            Run the setup wizard
          </button>
        </div>
      </motion.div>
    </div>
  );
}
