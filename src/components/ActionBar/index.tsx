import { Rocket, RotateCw, Square, Play, ScrollText } from 'lucide-react';
import type { Capabilities } from '../../types/generated';
import { Button } from '../ui/Button';

interface ActionBarProps {
  capabilities: Capabilities;
  /** True while a pipeline is running; disables mutating actions. */
  busy?: boolean;
  /** Whether the site is currently online; decides Stop vs Start. */
  online?: boolean;
  /** True while a log fetch is in flight. */
  logsBusy?: boolean;
  onShipIt: () => void;
  onRestart: () => void;
  onStop: () => void;
  onStart: () => void;
  onLogs: () => void;
}

export function ActionBar({
  capabilities,
  busy = false,
  online = false,
  logsBusy = false,
  onShipIt,
  onRestart,
  onStop,
  onStart,
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
        online ? (
          <Button
            variant="destructive"
            onClick={onStop}
            disabled={busy}
            disabledReason={busyReason}
          >
            <Square size={14} aria-hidden="true" />
            Stop
          </Button>
        ) : (
          <Button
            variant="secondary"
            onClick={onStart}
            disabled={busy}
            disabledReason={busyReason}
          >
            <Play size={14} aria-hidden="true" />
            Start
          </Button>
        )
      ) : null}

      {capabilities.has_logs ? (
        <Button variant="secondary" onClick={onLogs} disabled={logsBusy}>
          <ScrollText size={14} aria-hidden="true" />
          {logsBusy ? 'Loading…' : 'Logs'}
        </Button>
      ) : null}
    </div>
  );
}
