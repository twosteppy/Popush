import { motion, useReducedMotion } from 'framer-motion';
import { Wand2, Plus, HelpCircle } from 'lucide-react';
import { Logo } from '../ui/Logo';
import { Button } from '../ui/Button';

interface EmptyStateProps {
  /** True when servers exist but none/no site is selected. */
  hasServers: boolean;
  onAddServer: () => void;
  onRunWizard: () => void;
  /** Opens the "What is Popush?" explainer. Optional so callers can omit it. */
  onOpenHelp?: () => void;
}

export function EmptyState({
  hasServers,
  onAddServer,
  onRunWizard,
  onOpenHelp,
}: EmptyStateProps) {
  const reduce = useReducedMotion();

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
        <h1 className="mt-6 font-display text-2xl font-semibold tracking-tight text-text-primary">
          Your VPS, one click away.
        </h1>
        <p className="mt-2 max-w-sm text-sm leading-relaxed text-text-secondary">
          Popush deploys your sites straight from this machine over SSH. No
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

        {onOpenHelp ? (
          <button
            type="button"
            onClick={onOpenHelp}
            className="pressable mt-5 inline-flex items-center gap-1.5 rounded-sm border border-border-strong px-3 py-1.5 text-xs text-text-secondary shadow-hard-sm hover:bg-surface-hover hover:text-text-primary"
          >
            <HelpCircle size={13} aria-hidden="true" />
            New here? See how Popush works
          </button>
        ) : null}

        <div className="mt-6 flex flex-col items-center gap-2 text-xs text-text-tertiary">
          <button
            type="button"
            onClick={onRunWizard}
            className="inline-flex items-center gap-1.5 rounded-sm px-2 py-1 hover:text-text-secondary"
          >
            <Wand2 size={13} aria-hidden="true" />
            Run the setup wizard
          </button>
        </div>
      </motion.div>
    </div>
  );
}
