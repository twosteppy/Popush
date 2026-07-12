// AboutView — app name, version, the privacy summary, and the twostep credit
// (D9). States plainly: no server, no account, no telemetry.

interface AboutViewProps {
  version: string;
}

export function AboutView({ version }: AboutViewProps) {
  return (
    <div className="flex flex-col gap-4 p-6">
      <div>
        <h1 className="text-2xl font-semibold text-text-primary">Popush</h1>
        <p className="font-mono text-xs text-text-tertiary">v{version}</p>
      </div>

      <p className="max-w-prose text-sm text-text-secondary">
        Popush deploys your sites straight from your machine over SSH. There is
        no Popush server, no account to create, and no telemetry. Nothing about
        your servers, keys, or deployments leaves this computer.
      </p>

      <p className="text-base font-semibold text-text-primary">
        Built by twostep.
      </p>
    </div>
  );
}
