// Ship It (primary), Restart, Stop, and Logs.
//
// Capability-driven: buttons the adapter does not support are not rendered at
// all, rather than disabled-and-failing. Buttons that render but are momentarily
// unavailable are disabled and carry a tooltip explaining why.

import { Rocket, RotateCw, Square, ScrollText } from 'lucide-react';
import type { Capabilities } from '../../types/generated';
import { Button } from '../ui/Button';

interface ActionBarProps {
  capabilities: Capabilities;
  /** True while a pipeline is running; disables mutating actions. */
  busy?: boolean;
  onShipIt: () => void;
  onRestart: () => void;
  onStop: () => void;
  onLogs: () => void;
}

export function ActionBar({
  capabilities,
  busy = false,
  onShipIt,
  onRestart,
  onStop,
  onLogs,
}: ActionBarProps) {
  const busyReason = busy ? 'A deployment is in progress.' : undefined;

  return (
    <div className="flex items-center gap-2">
      <Button
        variant="primary"
        onClick={onShipIt}
        disabled={busy}
        disabledReason={busyReason}
      >
        <Rocket size={14} aria-hidden="true" />
        Ship It
      </Button>

      {capabilities.can_restart ? (
        <Button
          variant="secondary"
          onClick={onRestart}
          disabled={busy}
          disabledReason={busyReason}
        >
          <RotateCw size={14} aria-hidden="true" />
          Restart
        </Button>
      ) : null}

      {capabilities.can_start_stop ? (
        <Button
          variant="destructive"
          onClick={onStop}
          disabled={busy}
          disabledReason={busyReason}
        >
          <Square size={14} aria-hidden="true" />
          Stop
        </Button>
      ) : null}

      {capabilities.has_logs ? (
        <Button variant="secondary" onClick={onLogs}>
          <ScrollText size={14} aria-hidden="true" />
          Logs
        </Button>
      ) : null}
    </div>
  );
}
