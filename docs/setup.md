# Setup

Popush reads its configuration from `~/.config/popush/config.toml`. The file is
plain TOML, safe to edit by hand, and contains no secrets: SSH keys are
referenced by **path**, never copied into the file.

## Before you start

Make sure your SSH key is loaded into your agent. Popush does not handle
passphrases; it asks the agent.

```
ssh-add ~/.ssh/id_ed25519
```

If `SSH_AUTH_SOCK` is not set, start an agent first:

```
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

## Adding a server

A server is a VPS Popush connects to over SSH. Add a `[[server]]` table:

```toml
[[server]]
id = "vps-main"                     # stable id, referenced internally
label = "Main VPS"                  # shown in the sidebar
host = "203.0.113.10"
port = 22
username = "deploy"
identity_file = "~/.ssh/id_ed25519" # PATH ONLY. Never the key itself.
# proxy_jump = "bastion.example.com" # optional jump host
```

## Adding a site

A site is one website or service on a server. Add `[[server.site]]` tables
nested under the server:

```toml
  [[server.site]]
  id = "sterling-defence"
  label = "Sterling Defence"
  remote_path = "/srv/sterling-defence"
  service_type = "docker"                  # docker | systemd | pm2 | static
  service_name = "sterling-defence"        # compose project, unit, or pm2 app name
  build_command = "pnpm install --frozen-lockfile && pnpm build"
  git_remote = "origin"
  git_branch = "main"
  local_path = "~/dev/sterling-defence"    # local clone, for the git panel
  live_url = "https://sterlingdefence.co.uk"
  health_check_url = "https://sterlingdefence.co.uk/api/health"  # optional
```

## What each field means

| Field | Meaning |
|---|---|
| `remote_path` | The directory on the server where the site lives. Popush runs adapter commands here. |
| `service_type` | Selects the adapter. See [service-types.md](service-types.md). |
| `service_name` | The compose project, systemd unit, or pm2 app name. Not needed for static sites. |
| `build_command` | Run on the server during the Build step of Ship It. Optional. |
| `git_remote` / `git_branch` | Which remote and branch the pipeline pushes and pulls. |
| `local_path` | Your local clone, so the git panel can show changes. |
| `live_url` | Shown in the UI; opened when you click the site's URL. |
| `health_check_url` | If set, Popush does an HTTP check to earn a real status. For static sites this is what turns amber "Unknown" into a verified green. |

The loader validates the file on read and rejects a malformed config with a
message that names the field and the problem. If the file does not exist yet,
that is fine — first launch shows the empty state and an "Add a server" button.
