import { Check, Dash, Circle, Spinner, Rocket, ArrowUpRight } from './Icons';

type StepState = 'ok' | 'skipped' | 'running' | 'pending';

interface Step {
  name: string;
  state: StepState;
  meta?: string;
}

const STEPS: Step[] = [
  { name: 'Check', state: 'ok', meta: 'server reachable' },
  { name: 'Commit', state: 'skipped', meta: 'skipped' },
  { name: 'Push', state: 'skipped', meta: 'nothing to push' },
  { name: 'Pull', state: 'ok', meta: 'Fast-forward' },
  { name: 'Build', state: 'ok', meta: 'Build succeeded' },
  { name: 'Restart', state: 'running', meta: 'up -d' },
  { name: 'Verify', state: 'pending' },
];

function StepIcon({ state }: { state: StepState }) {
  switch (state) {
    case 'ok':
      return <Check style={{ color: 'var(--status-running)' }} />;
    case 'skipped':
      return <Dash style={{ color: 'var(--text-tertiary)' }} />;
    case 'running':
      return <Spinner className="spin" />;
    case 'pending':
      return <Circle style={{ color: 'var(--text-tertiary)' }} />;
  }
}

/** A faithful, static recreation of the app's Ship It pipeline. */
export function PipelineDemo() {
  return (
    <div className="window" aria-hidden="true">
      <div className="window-bar">
        <div className="dots">
          <i />
          <i />
          <i />
        </div>
        <span className="title">Popush — Pook Review</span>
      </div>
      <div className="pipe">
        <div className="pipe-head">
          <span className="label-mono">Ship It pipeline</span>
        </div>
        {STEPS.map((s) => (
          <div
            key={s.name}
            className={`step ${s.state === 'running' ? 'active' : ''} ${
              s.state === 'skipped' || s.state === 'pending' ? 'pending' : ''
            }`.trim()}
          >
            <span className="ico">
              <StepIcon state={s.state} />
            </span>
            <span className="name">{s.name}</span>
            {s.meta ? <span className="meta">{s.meta}</span> : null}
          </div>
        ))}
        <div className="shipped">
          <span className="rk">
            <Rocket />
          </span>
          <div>
            <h4>Shipped and live</h4>
            <p>Every step passed and the site is responding.</p>
          </div>
          <span className="visit">
            Visit site <ArrowUpRight />
          </span>
        </div>
      </div>
    </div>
  );
}
