# Sample sites for the test VPS

One directory per service type, so the integration suite (§23.3) can exercise
every adapter against a real service:

- `static-site/`, a directory of files served by nginx; the static adapter's
  honest-status path (amber Unknown without a health check, green with one).
- `docker-site/`, a `compose.yaml` with a tiny web container; the Docker adapter.
- `pm2-site/`, a small Node app started with pm2; the pm2 adapter.
- `systemd-site/`, a script fronting a user service; the systemd adapter (run as
  a user unit in the container, matching the recommended setup).

Each is a git repository initialised by the harness so `git pull` in the Ship It
pipeline has something to fast-forward.
