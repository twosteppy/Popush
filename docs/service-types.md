# Service types

A site's `service_type` selects the adapter — the module that knows how to check,
start, stop, restart, and tail logs for that kind of service. The adapter also
declares its **capabilities**, which decide which buttons appear. A button that an
adapter does not support is not rendered, rather than rendered and failing.

## docker (recommended)

| Operation | Command |
|---|---|
| Status | `cd <path> && docker compose ps --format json` |
| Start | `cd <path> && docker compose up -d` |
| Stop | `cd <path> && docker compose down` |
| Restart | `cd <path> && docker compose restart` |
| Logs | `cd <path> && docker compose logs -f --tail=200` |

Status is mapped from each container's state and health: all running and healthy
is **Running**; any container exited non-zero is **Failed**, naming the container;
all exited cleanly is **Stopped**; a mixed set is **Failed**, describing which
container is down. Docker is the recommended setup, and status is reliable.

## systemd

| Operation | Command |
|---|---|
| Status | `systemctl show <unit> --property=ActiveState,SubState,ActiveEnterTimestamp` |
| Start / Stop / Restart | `sudo systemctl start\|stop\|restart <unit>` |
| Logs | `journalctl -u <unit> -f -n 200` |

### The sudo problem

System units need root, and Popush must never prompt for a password. The
recommended answer is to run your service as a **user unit**, which needs no
sudo:

```
systemctl --user start my-service
```

Where a system unit is genuinely required, the setup wizard detects that sudo is
needed and **generates** a sudoers snippet for you to install by hand. Popush
never edits sudoers itself.

```
# /etc/sudoers.d/popush
deploy ALL=(root) NOPASSWD: /usr/bin/systemctl start my-service.service
deploy ALL=(root) NOPASSWD: /usr/bin/systemctl stop my-service.service
deploy ALL=(root) NOPASSWD: /usr/bin/systemctl restart my-service.service
```

## pm2

| Operation | Command |
|---|---|
| Status | `pm2 jlist` |
| Start / Stop / Restart | `pm2 start\|stop\|restart <app>` |
| Logs | `pm2 logs <app> --lines 200` |

Status comes from the app's `pm2_env.status`: `online` is Running, `stopped` is
Stopped, `errored` is Failed. If pm2 has no app by that name, the status is an
honest Unknown, not a guess.

## static

A static site served by nginx has no process of its own. Its status is whether
files exist and are being served.

| Operation | Behaviour |
|---|---|
| Status | `test -d <root> && ls -1 <root> \| head -1`, plus an HTTP `HEAD` to `health_check_url` if configured |
| Start / Stop / Restart | Unsupported. These buttons are hidden. |
| Logs | The nginx log for the vhost, if configured; otherwise unsupported. |

**Honest status:** without a `health_check_url`, a static site shows an amber
"Unknown" dot, because "the folder exists" is not the same as "the site is being
served". Only a passing health check earns green. This is deliberate: a green
light that only means the directory is present would be worse than an honest
amber one.
