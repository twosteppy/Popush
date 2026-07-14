import { useEffect, useState } from 'react';
import type { SiteConfig, ServiceKind } from '../../types/generated';
import { useSitesStore } from '../../store/sites';
import { isSafeHttpUrl } from '../../lib/url';
import { addSite } from '../../lib/ipc';
import { slugId } from '../../lib/slug';
import { Dialog } from '../ui/Dialog';
import { Button } from '../ui/Button';
import { Spinner } from '../ui/Spinner';
import { Field, TextInput, SelectInput } from '../ui/Field';

interface AddSiteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  serverId: string | null;
  /** When set, the dialog edits this site in place instead of adding one. */
  editSite?: SiteConfig | null;
}

/** Fill the form from an existing site so it can be edited. */
function formFromSite(s: SiteConfig): SiteForm {
  return {
    label: s.label,
    remotePath: String(s.remote_path ?? ''),
    serviceType: s.service_type,
    serviceName: s.service_name ?? '',
    buildCommand: s.build_command ?? '',
    gitRemote: s.git_remote,
    gitBranch: s.git_branch,
    localPath: s.local_path ? String(s.local_path) : '',
    liveUrl: s.live_url ?? '',
    healthCheckUrl: s.health_check_url ?? '',
  };
}

interface SiteForm {
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

const EMPTY: SiteForm = {
  label: '',
  remotePath: '',
  // Most sites people add are plain websites served by a web server, so start
  // there. Docker/systemd/pm2 are a dropdown away when the site really is one.
  serviceType: 'static',
  serviceName: '',
  buildCommand: '',
  gitRemote: 'origin',
  gitBranch: 'main',
  localPath: '',
  liveUrl: '',
  healthCheckUrl: '',
};

const SERVICE_TYPES: ServiceKind[] = ['docker', 'systemd', 'pm2', 'static'];
type SiteErrors = Partial<Record<keyof SiteForm, string>>;

function validate(f: SiteForm): SiteErrors {
  const e: SiteErrors = {};
  if (!f.label.trim()) e.label = 'Give this site a name.';
  if (!f.remotePath.trim()) e.remotePath = 'Remote path is required.';
  if (f.serviceType !== 'static' && !f.serviceName.trim())
    e.serviceName = `A ${f.serviceType} site needs a service name.`;
  if (f.liveUrl.trim() && !isSafeHttpUrl(f.liveUrl.trim()))
    e.liveUrl = 'Must be an http:// or https:// URL.';
  if (f.healthCheckUrl.trim() && !isSafeHttpUrl(f.healthCheckUrl.trim()))
    e.healthCheckUrl = 'Must be an http:// or https:// URL.';
  return e;
}

function nullable(v: string): string | null {
  const t = v.trim();
  return t.length ? t : null;
}

export function AddSiteDialog({
  open,
  onOpenChange,
  serverId,
  editSite,
}: AddSiteDialogProps) {
  const refreshSites = useSitesStore((s) => s.refreshSites);
  const [site, setSite] = useState<SiteForm>(EMPTY);
  const [errors, setErrors] = useState<SiteErrors>({});
  const [submitting, setSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  const isEdit = Boolean(editSite);

  // Prefill from the site being edited each time the dialog opens.
  useEffect(() => {
    if (open) {
      setSite(editSite ? formFromSite(editSite) : EMPTY);
      setErrors({});
      setSubmitError(null);
    }
  }, [open, editSite]);

  function close(next: boolean) {
    onOpenChange(next);
    if (!next) {
      setSite(EMPTY);
      setErrors({});
      setSubmitting(false);
      setSubmitError(null);
    }
  }

  function blur(key: keyof SiteForm) {
    setErrors((prev) => ({ ...prev, [key]: validate(site)[key] }));
  }

  async function submit() {
    if (!serverId) return;
    const errs = validate(site);
    setErrors(errs);
    if (Object.keys(errs).length) return;

    const isStatic = site.serviceType === 'static';
    const config: SiteConfig = {
      // Keep the same id when editing so the backend updates in place.
      id: editSite?.id ?? slugId(site.label, 'site'),
      label: site.label.trim(),
      remote_path: site.remotePath.trim(),
      service_type: site.serviceType,
      // For a static site the web root is the folder nginx serves, which is the
      // remote path; other types carry a service name instead.
      service_name: isStatic ? null : nullable(site.serviceName),
      web_root: isStatic ? site.remotePath.trim() : null,
      build_command: nullable(site.buildCommand),
      git_remote: site.gitRemote.trim() || 'origin',
      git_branch: site.gitBranch.trim() || 'main',
      local_path: nullable(site.localPath),
      live_url: nullable(site.liveUrl),
      health_check_url: nullable(site.healthCheckUrl),
    };

    setSubmitting(true);
    setSubmitError(null);
    try {
      await addSite(serverId, config);
      await refreshSites(serverId);
      close(false);
    } catch {
      setSubmitError(
        'Could not save the site. Check the details and try again.',
      );
      setSubmitting(false);
    }
  }

  const footer = (
    <>
      <Button variant="secondary" onClick={() => close(false)}>
        Cancel
      </Button>
      <Button
        variant="primary"
        onClick={() => void submit()}
        disabled={submitting}
      >
        {submitting ? (
          <>
            <Spinner size={14} className="text-text-inverse" />
            Saving…
          </>
        ) : isEdit ? (
          'Save changes'
        ) : (
          'Add site'
        )}
      </Button>
    </>
  );

  return (
    <Dialog
      open={open}
      onOpenChange={close}
      title={isEdit ? 'Edit site' : 'Add a site'}
      size="lg"
      footer={footer}
    >
      <div className="flex flex-col gap-4">
        <Field label="Site name" htmlFor="as-label" error={errors.label}>
          <TextInput
            id="as-label"
            value={site.label}
            invalid={Boolean(errors.label)}
            onChange={(e) => setSite({ ...site, label: e.target.value })}
            onBlur={() => blur('label')}
            placeholder="Marketing site"
          />
        </Field>

        <div className="grid grid-cols-[1fr_160px] gap-3">
          <Field
            label="Remote path"
            htmlFor="as-path"
            hint="The folder this site lives in on the server."
            error={errors.remotePath}
          >
            <TextInput
              id="as-path"
              value={site.remotePath}
              invalid={Boolean(errors.remotePath)}
              onChange={(e) => setSite({ ...site, remotePath: e.target.value })}
              onBlur={() => blur('remotePath')}
              placeholder="/srv/app"
            />
          </Field>
          <Field label="Service type" htmlFor="as-service">
            <SelectInput
              id="as-service"
              value={site.serviceType}
              onChange={(e) =>
                setSite({ ...site, serviceType: e.target.value as ServiceKind })
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

        {site.serviceType !== 'static' ? (
          <div className="grid grid-cols-2 gap-3">
            <Field
              label="Service name"
              htmlFor="as-svcname"
              hint="Compose project, unit, or pm2 app."
              error={errors.serviceName}
            >
              <TextInput
                id="as-svcname"
                value={site.serviceName}
                invalid={Boolean(errors.serviceName)}
                onChange={(e) =>
                  setSite({ ...site, serviceName: e.target.value })
                }
                onBlur={() => blur('serviceName')}
                placeholder="app"
              />
            </Field>
            <Field label="Build command" htmlFor="as-build" optional>
              <TextInput
                id="as-build"
                value={site.buildCommand}
                onChange={(e) =>
                  setSite({ ...site, buildCommand: e.target.value })
                }
                placeholder="npm run build"
              />
            </Field>
          </div>
        ) : (
          <Field label="Build command" htmlFor="as-build" optional>
            <TextInput
              id="as-build"
              value={site.buildCommand}
              onChange={(e) =>
                setSite({ ...site, buildCommand: e.target.value })
              }
              placeholder="hugo --minify"
            />
          </Field>
        )}

        <div className="grid grid-cols-2 gap-3">
          <Field
            label="Live URL"
            htmlFor="as-url"
            optional
            error={errors.liveUrl}
          >
            <TextInput
              id="as-url"
              value={site.liveUrl}
              invalid={Boolean(errors.liveUrl)}
              onChange={(e) => setSite({ ...site, liveUrl: e.target.value })}
              onBlur={() => blur('liveUrl')}
              placeholder="https://example.com"
            />
          </Field>
          <Field
            label="Health check URL"
            htmlFor="as-health"
            optional
            error={errors.healthCheckUrl}
          >
            <TextInput
              id="as-health"
              value={site.healthCheckUrl}
              invalid={Boolean(errors.healthCheckUrl)}
              onChange={(e) =>
                setSite({ ...site, healthCheckUrl: e.target.value })
              }
              onBlur={() => blur('healthCheckUrl')}
              placeholder="https://example.com/health"
            />
          </Field>
        </div>

        <p className="rounded-sm border border-border-strong bg-surface-raised px-3 py-2 text-xs text-text-tertiary">
          Saved to ~/.config/popush/config.toml on this machine only. Nothing is
          uploaded anywhere.
        </p>

        {submitError ? (
          <p className="text-sm text-status-failed">{submitError}</p>
        ) : null}
      </div>
    </Dialog>
  );
}
