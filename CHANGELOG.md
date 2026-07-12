# Changelog

All notable changes to Popush are recorded here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and Popush follows
semantic versioning.

## [Unreleased]

Nothing yet.

## [1.0.0] — 2026-07-12

The first release.

### Added

- **SSH core**: a connection pool per server with keepalive and channel
  multiplexing, `ssh-agent` delegation (Popush never handles passphrases), and
  strict host-key verification (unknown hosts prompt with a fingerprint,
  mismatches refuse to connect).
- **Mandatory command escaping**: every remote command is a structured type built
  from compile-time templates plus shell-escaped arguments, never string
  formatting. Covered by an adversarial escaping corpus.
- **The command log**: a permanent, inspectable record of every remote command
  Popush ran — timestamp, server, exact command, exit code, duration.
- **Four service adapters** — Docker, systemd, pm2, and static — each reporting an
  honest status. Static sites show amber "Unknown" unless a health check is
  configured; the app never renders a green light it has not verified.
- **Local git panel**: status, staging, commit, and push via `git2` with
  agent-based credentials, with clear refusals for merge conflicts, detached
  HEAD, missing upstream, and HTTPS remotes.
- **The setup wizard**: seven preview-then-apply checks that walk SSH and GitHub
  configuration once, never overwriting an existing key, with the read-only
  deploy-key flow for private repositories.
- **Ship It**: the seven-step pipeline (Check, Commit, Push, Pull, Build, Restart,
  Verify) with skip logic, live step UI, cancellation, and manual rollback. Every
  failure names its step; there is no "Deploy failed".
- **Log viewer**: an xterm.js drawer with follow mode, pause, search, a buffer
  cap, copy, and wrap and timestamp toggles.
- **Packaging**: Flatpak (Flathub-ready, portal-based per-repo access), RPM, and
  AppImage, with a desktop file, icons, and AppStream metainfo.

### Notes

- Linux only. Fedora 44 + KDE Plasma is the first-class target.
- No telemetry, no account, no server, no payment, free forever.
