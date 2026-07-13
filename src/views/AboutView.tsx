import { Logo } from '../components/ui/Logo';

interface AboutViewProps {
  version: string;
}

export function AboutView({ version }: AboutViewProps) {
  return (
    <div className="flex flex-col gap-4 p-6">
      <div className="flex items-center gap-3">
        <Logo size={30} markOnly />
        <div>
          <h1 className="font-display text-2xl font-semibold text-text-primary">
            Popush
          </h1>
          <p className="font-mono text-xs text-text-tertiary">v{version}</p>
        </div>
      </div>

      <p className="max-w-prose text-sm text-text-secondary">
        Popush deploys your sites straight from your machine over SSH. There is
        no Popush server, no account to create, and no telemetry. Nothing about
        your servers, keys, or deployments leaves this computer.
      </p>

      <p className="label-mono text-[11px] text-text-tertiary">
        By twostep · GPLv3
      </p>
    </div>
  );
}
