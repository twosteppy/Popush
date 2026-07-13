// AddServerDialog - makes Popush usable without hand-editing TOML. A two-step
// Radix form: (1) the SSH connection, (2) an optional first site. On submit it
// assembles a typed ServerConfig (with slug ids derived from labels) and
// dispatches add() to the servers store, which persists via the backend and
// re-mirrors (D14: no persistence logic lives here).
//
// §20: labels sit above inputs, validation appears on blur, controls are 34px
// with visible focus rings, and a short reassurance line explains where the
// data goes and that no secrets are stored.

import { useState } from 'react';
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion';
import { Server, Globe } from 'lucide-react';
import type {
  ServerConfig,
  SiteConfig,
  ServiceKind,
} from '../../types/generated';
import { useServersStore } from '../../store/servers';
import { Dialog } from '../ui/Dialog';
import { Button } from '../ui/Button';
import { Spinner } from '../ui/Spinner';
import { Field, TextInput, SelectInput, NumberField } from '../ui/Field';
import { slugId } from '../../lib/slug';

interface AddServerDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

interface ServerForm {
  label: string;
  host: string;
  port: string;
  username: string;
  identityFile: string;
  proxyJump: string;
}

interface SiteForm {
  enabled: boolean;
  label: string;
  remotePath: string;
  serviceType: ServiceKind;
  serviceName: string;
  buildCommand: string;
  gitRemote: string;
  gitBranch: string;
  localPath: string;
  liveUrl: string;
  healthCheckUrl: string;
}

const EMPTY_SERVER: ServerForm = {
  label: '',
  host: '',
  port: '22',
  username: '',
  identityFile: '',
  proxyJump: '',
};

const EMPTY_SITE: SiteForm = {
  enabled: false,
  label: '',
  remotePath: '',
  serviceType: 'docker',
  serviceName: '',
  buildCommand: '',
  gitRemote: 'origin',
  gitBranch: 'main',
  localPath: '',
  liveUrl: '',
  healthCheckUrl: '',
};

const SERVICE_TYPES: ServiceKind[] = ['docker', 'systemd', 'pm2', 'static'];

type ServerErrors = Partial<Record<keyof ServerForm, string>>;
type SiteErrors = Partial<Record<keyof SiteForm, string>>;

function validateServer(f: ServerForm): ServerErrors {
  const e: ServerErrors = {};
  if (!f.label.trim()) e.label = 'Give this server a name.';
  if (!f.host.trim()) e.host = 'Host is required.';
  if (!f.username.trim()) e.username = 'Username is required.';
  const port = Number(f.port);
  if (!Number.isInteger(port) || port < 1 || port > 65535)
    e.port = 'Port must be between 1 and 65535.';
  return e;
}

function validateSite(f: SiteForm): SiteErrors {
  if (!f.enabled) return {};
  const e: SiteErrors = {};
  if (!f.label.trim()) e.label = 'Give this site a name.';
  if (!f.remotePath.trim()) e.remotePath = 'Remote path is required.';
  return e;
}

function nullable(v: string): string | null {
  const t = v.trim();
  return t.length ? t : null;
}

export function AddServerDialog({ open, onOpenChange }: AddServerDialogProps) {
  const add = useServersStore((s) => s.add);
  const reduce = useReducedMotion();

  const [step, setStep] = useState<0 | 1>(0);
  const [server, setServer] = useState<ServerForm>(EMPTY_SERVER);
  const [site, setSite] = useState<SiteForm>(EMPTY_SITE);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [serverErrors, setServerErrors] = useState<ServerErrors>({});
  const [siteErrors, setSiteErrors] = useState<SiteErrors>({});
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  function reset() {
    setStep(0);
    setServer(EMPTY_SERVER);
    setSite(EMPTY_SITE);
    setShowAdvanced(false);
    setServerErrors({});
    setSiteErrors({});
    setSubmitting(false);
    setSubmitError(null);
  }

  function close(next: boolean) {
    onOpenChange(next);
    if (!next) reset();
  }

  function blurServer(key: keyof ServerForm) {
    setServerErrors((prev) => ({
      ...prev,
      [key]: validateServer(server)[key],
    }));
  }
  function blurSite(key: keyof SiteForm) {
    setSiteErrors((prev) => ({ ...prev, [key]: validateSite(site)[key] }));
  }

  function goNext() {
    const errs = validateServer(server);
    setServerErrors(errs);
    if (Object.keys(errs).length === 0) setStep(1);
  }

  async function submit() {
    const sErrs = validateServer(server);
    const siteErrs = validateSite(site);
    setServerErrors(sErrs);
    setSiteErrors(siteErrs);
    if (Object.keys(sErrs).length) {
      setStep(0);
      return;
    }
    if (Object.keys(siteErrs).length) return;

    const serverId = slugId(server.label, 'server');
    const sites: SiteConfig[] = [];
    if (site.enabled) {
      sites.push({
        id: slugId(site.label, 'site'),
        label: site.label.trim(),
        remote_path: site.remotePath.trim(),
        service_type: site.serviceType,
        service_name: nullable(site.serviceName),
        web_root: null,
        build_command: nullable(site.buildCommand),
        git_remote: site.gitRemote.trim() || 'origin',
        git_branch: site.gitBranch.trim() || 'main',
        local_path: nullable(site.localPath),
        live_url: nullable(site.liveUrl),
        health_check_url: nullable(site.healthCheckUrl),
      });
    }

    const config: ServerConfig = {
      id: serverId,
      label: server.label.trim(),
      host: server.host.trim(),
      port: Number(server.port),
      username: server.username.trim(),
      identity_file: server.identityFile.trim(),
      proxy_jump: nullable(server.proxyJump),
      site: sites,
    };

    setSubmitting(true);
    setSubmitError(null);
    try {
      await add(config);
      close(false);
    } catch {
      setSubmitError(
        'Could not save the server. Check the details and try again.',
      );
      setSubmitting(false);
    }
  }

  const transition = { duration: reduce ? 0 : 0.18, ease: 'easeOut' as const };

  const footer =
    step === 0 ? (
      <>
        <Button variant="secondary" onClick={() => close(false)}>
          Cancel
        </Button>
        <Button variant="primary" onClick={goNext}>
          Continue
        </Button>
      </>
    ) : (
      <>
        <Button variant="secondary" onClick={() => setStep(0)}>
          Back
        </Button>
        <Button
          variant="primary"
          onClick={() => void submit()}
          disabled={submitting}
          disabledReason={submitting ? 'Saving…' : undefined}
        >
          {submitting ? (
            <>
              <Spinner size={14} className="text-text-inverse" />
              Saving…
            </>
          ) : (
            'Add server'
          )}
        </Button>
      </>
    );

  return (
    <Dialog
      open={open}
      onOpenChange={close}
      title="Add a server"
      size="lg"
      footer={footer}
    >
      {/* Step indicator */}
      <div className="label-mono mb-4 flex items-center gap-2 text-[10px] text-text-tertiary">
        <StepPip
          active={step === 0}
          done={step > 0}
          icon={<Server size={12} />}
        >
          Connection
        </StepPip>
        <span className="h-px w-4 bg-border-strong" aria-hidden="true" />
        <StepPip active={step === 1} done={false} icon={<Globe size={12} />}>
          First site
        </StepPip>
      </div>

      <AnimatePresence mode="wait" initial={false}>
        <motion.div
          key={step}
          initial={reduce ? false : { opacity: 0, x: 12 }}
          animate={{ opacity: 1, x: 0 }}
          exit={reduce ? undefined : { opacity: 0, x: -12 }}
          transition={transition}
        >
          {step === 0 ? (
            <div className="flex flex-col gap-4">
              <Field
                label="Name"
                htmlFor="srv-label"
                hint="A friendly label for the sidebar."
                error={serverErrors.label}
              >
                <TextInput
                  id="srv-label"
                  value={server.label}
                  invalid={Boolean(serverErrors.label)}
                  onChange={(e) =>
                    setServer({ ...server, label: e.target.value })
                  }
                  onBlur={() => blurServer('label')}
                  placeholder="Production"
                />
              </Field>

              <div className="grid grid-cols-[1fr_120px] gap-3">
                <Field
                  label="Host"
                  htmlFor="srv-host"
                  error={serverErrors.host}
                >
                  <TextInput
                    id="srv-host"
                    value={server.host}
                    invalid={Boolean(serverErrors.host)}
                    onChange={(e) =>
                      setServer({ ...server, host: e.target.value })
                    }
                    onBlur={() => blurServer('host')}
                    placeholder="203.0.113.10"
                  />
                </Field>
                <Field
                  label="Port"
                  htmlFor="srv-port"
                  error={serverErrors.port}
                >
                  <NumberField
                    id="srv-port"
                    min={1}
                    max={65535}
                    value={server.port}
                    invalid={Boolean(serverErrors.port)}
                    onValueChange={(v) => setServer({ ...server, port: v })}
                    onBlur={() => blurServer('port')}
                  />
                </Field>
              </div>

              <Field
                label="Username"
                htmlFor="srv-user"
                error={serverErrors.username}
              >
                <TextInput
                  id="srv-user"
                  value={server.username}
                  invalid={Boolean(serverErrors.username)}
                  onChange={(e) =>
                    setServer({ ...server, username: e.target.value })
                  }
                  onBlur={() => blurServer('username')}
                  placeholder="deploy"
                />
              </Field>

              <Field
                label="Identity file"
                htmlFor="srv-key"
                hint="Path to your SSH key, never the key itself."
                optional
              >
                <TextInput
                  id="srv-key"
                  value={server.identityFile}
                  onChange={(e) =>
                    setServer({ ...server, identityFile: e.target.value })
                  }
                  placeholder="~/.ssh/id_ed25519"
                />
              </Field>

              <div>
                <button
                  type="button"
                  onClick={() => setShowAdvanced((v) => !v)}
                  aria-expanded={showAdvanced}
                  className="text-xs text-text-tertiary hover:text-text-secondary"
                >
                  {showAdvanced ? '▾' : '▸'} Advanced
                </button>
                {showAdvanced ? (
                  <div className="mt-3">
                    <Field
                      label="Proxy jump"
                      htmlFor="srv-proxy"
                      hint="Connect through a bastion host, e.g. user@bastion."
                      optional
                    >
                      <TextInput
                        id="srv-proxy"
                        value={server.proxyJump}
                        onChange={(e) =>
                          setServer({ ...server, proxyJump: e.target.value })
                        }
                        placeholder="jump@bastion.example.com"
                      />
                    </Field>
                  </div>
                ) : null}
              </div>

              <p className="rounded-sm border border-border-strong bg-surface-raised px-3 py-2 text-xs text-text-tertiary">
                Saved to ~/.config/popush/config.toml. No secrets are stored,
                only the path to your key.
              </p>
            </div>
          ) : (
            <div className="flex flex-col gap-4">
              <label className="flex items-center gap-2.5 rounded-sm border border-border-strong bg-surface-raised px-3 py-2.5 text-sm text-text-primary">
                <input
                  type="checkbox"
                  checked={site.enabled}
                  onChange={(e) =>
                    setSite({ ...site, enabled: e.target.checked })
                  }
                  className="h-4 w-4 accent-accent"
                />
                Add a site to deploy now
                <span className="ml-auto text-xs text-text-tertiary">
                  optional
                </span>
              </label>

              {site.enabled ? (
                <>
                  <Field
                    label="Site name"
                    htmlFor="site-label"
                    error={siteErrors.label}
                  >
                    <TextInput
                      id="site-label"
                      value={site.label}
                      invalid={Boolean(siteErrors.label)}
                      onChange={(e) =>
                        setSite({ ...site, label: e.target.value })
                      }
                      onBlur={() => blurSite('label')}
                      placeholder="Marketing site"
                    />
                  </Field>

                  <div className="grid grid-cols-[1fr_160px] gap-3">
                    <Field
                      label="Remote path"
                      htmlFor="site-path"
                      hint="Where the app lives on the server."
                      error={siteErrors.remotePath}
                    >
                      <TextInput
                        id="site-path"
                        value={site.remotePath}
                        invalid={Boolean(siteErrors.remotePath)}
                        onChange={(e) =>
                          setSite({ ...site, remotePath: e.target.value })
                        }
                        onBlur={() => blurSite('remotePath')}
                        placeholder="/srv/app"
                      />
                    </Field>
                    <Field label="Service type" htmlFor="site-service">
                      <SelectInput
                        id="site-service"
                        value={site.serviceType}
                        onChange={(e) =>
                          setSite({
                            ...site,
                            serviceType: e.target.value as ServiceKind,
                          })
                        }
                      >
                        {SERVICE_TYPES.map((t) => (
                          <option key={t} value={t}>
                            {t}
                          </option>
                        ))}
                      </SelectInput>
                    </Field>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <Field
                      label="Service name"
                      htmlFor="site-svcname"
                      hint="Compose project, unit, or pm2 app."
                      optional
                    >
                      <TextInput
                        id="site-svcname"
                        value={site.serviceName}
                        onChange={(e) =>
                          setSite({ ...site, serviceName: e.target.value })
                        }
                        placeholder="app"
                      />
                    </Field>
                    <Field label="Build command" htmlFor="site-build" optional>
                      <TextInput
                        id="site-build"
                        value={site.buildCommand}
                        onChange={(e) =>
                          setSite({ ...site, buildCommand: e.target.value })
                        }
                        placeholder="npm run build"
                      />
                    </Field>
                  </div>

                  <div className="grid grid-cols-[1fr_160px] gap-3">
                    <Field label="Git remote" htmlFor="site-remote" optional>
                      <TextInput
                        id="site-remote"
                        value={site.gitRemote}
                        onChange={(e) =>
                          setSite({ ...site, gitRemote: e.target.value })
                        }
                      />
                    </Field>
                    <Field label="Git branch" htmlFor="site-branch" optional>
                      <TextInput
                        id="site-branch"
                        value={site.gitBranch}
                        onChange={(e) =>
                          setSite({ ...site, gitBranch: e.target.value })
                        }
                      />
                    </Field>
                  </div>

                  <Field label="Local path" htmlFor="site-local" optional>
                    <TextInput
                      id="site-local"
                      value={site.localPath}
                      onChange={(e) =>
                        setSite({ ...site, localPath: e.target.value })
                      }
                      placeholder="~/code/app"
                    />
                  </Field>

                  <div className="grid grid-cols-2 gap-3">
                    <Field label="Live URL" htmlFor="site-url" optional>
                      <TextInput
                        id="site-url"
                        value={site.liveUrl}
                        onChange={(e) =>
                          setSite({ ...site, liveUrl: e.target.value })
                        }
                        placeholder="https://example.com"
                      />
                    </Field>
                    <Field
                      label="Health check URL"
                      htmlFor="site-health"
                      optional
                    >
                      <TextInput
                        id="site-health"
                        value={site.healthCheckUrl}
                        onChange={(e) =>
                          setSite({ ...site, healthCheckUrl: e.target.value })
                        }
                        placeholder="https://example.com/health"
                      />
                    </Field>
                  </div>
                </>
              ) : (
                <p className="text-sm text-text-secondary">
                  No problem. You can add sites to this server at any time.
                </p>
              )}
            </div>
          )}
        </motion.div>
      </AnimatePresence>

      {submitError ? (
        <p className="mt-4 text-sm text-status-failed">{submitError}</p>
      ) : null}
    </Dialog>
  );
}

function StepPip({
  active,
  done,
  icon,
  children,
}: {
  active: boolean;
  done: boolean;
  icon: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <span
      className={
        active || done
          ? 'flex items-center gap-1.5 text-text-primary'
          : 'flex items-center gap-1.5'
      }
    >
      <span
        className={
          active
            ? 'inline-flex h-5 w-5 items-center justify-center rounded-full bg-accent text-text-inverse'
            : 'inline-flex h-5 w-5 items-center justify-center rounded-full border border-border-strong text-text-tertiary'
        }
      >
        {icon}
      </span>
      {children}
    </span>
  );
}
