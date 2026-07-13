import { useEffect, useState } from 'react';
import { Dialog } from '../ui/Dialog';
import { Button } from '../ui/Button';
import { Spinner } from '../ui/Spinner';
import { TextInput } from '../ui/Field';

interface ConfirmDeleteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  /** "server" or "site", used in the copy. */
  kind: 'server' | 'site';
  /** The exact label the user must type to confirm. */
  name: string;
  /** Extra line describing what else gets removed (e.g. a server's sites). */
  consequence?: string;
  onConfirm: () => Promise<void>;
}

export function ConfirmDeleteDialog({
  open,
  onOpenChange,
  kind,
  name,
  consequence,
  onConfirm,
}: ConfirmDeleteDialogProps) {
  const [typed, setTyped] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setTyped('');
      setBusy(false);
      setError(null);
    }
  }, [open]);

  const matches = typed.trim() === name;

  async function confirm() {
    if (!matches) return;
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
      onOpenChange(false);
    } catch {
      setError('Could not remove it. Please try again.');
      setBusy(false);
    }
  }

  const footer = (
    <>
      <Button variant="secondary" onClick={() => onOpenChange(false)}>
        Cancel
      </Button>
      <Button
        variant="destructive"
        onClick={() => void confirm()}
        disabled={!matches || busy}
      >
        {busy ? (
          <>
            <Spinner size={14} />
            Removing…
          </>
        ) : (
          `Delete ${kind}`
        )}
      </Button>
    </>
  );

  return (
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={`Delete this ${kind}?`}
      size="md"
      footer={footer}
    >
      <div className="flex flex-col gap-3">
        <p className="text-sm text-text-secondary">
          This removes{' '}
          <span className="font-semibold text-text-primary">{name}</span> from
          your local config. It does not touch anything on the server.
          {consequence ? ` ${consequence}` : ''}
        </p>
        <label className="flex flex-col gap-1.5 text-xs text-text-tertiary">
          Type <span className="font-mono text-text-secondary">{name}</span> to
          confirm
          <TextInput
            value={typed}
            onChange={(e) => setTyped(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') void confirm();
            }}
            placeholder={name}
            autoFocus
          />
        </label>
        {error ? <p className="text-sm text-status-failed">{error}</p> : null}
      </div>
    </Dialog>
  );
}
