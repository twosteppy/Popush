# Decisions

This log records the judgement calls made while building Popush, per the Agent
Operating Rules in the build specification: when the prompt is silent, choose the
smallest option consistent with the Prime Directives and record it here; when a
trade-off is genuinely close, state it where the decision lives.

Format: decision, reason, date.

## Architecture

- **The workspace is split into `popush-core` (pure logic) and `src-tauri` (the
  Tauri binary).**, WebKitGTK is not available in every build or CI environment,
  and the Tauri crate cannot compile without it. Putting all business logic in a
  GUI-free library crate lets the security- and correctness-critical code (the
 command escaping, config load/validate/migrate, adapter status parsing, git
  URL classification, the error taxonomy, the pipeline messages and state
  machine) compile and test on any machine with only a Rust toolchain. It also
  enforces directly: the frontend and even the IPC command layer hold no
  logic, because the logic physically lives in a crate they depend on. This is the
  single most consequential structural decision in the project. (2026-07-12)

- **`russh`, `git2`, `keyring`, and `notify` live only in `src-tauri`.**, They
  pull native libraries and, in `russh`'s case, need a live SSH server to
  exercise meaningfully. Per , their real API is verified against the
  pinned version on the Fedora target rather than trusted from memory; keeping
  them out of `popush-core` keeps the core pure and fast to test. The core still
  owns every *decision* these layers act on: command text, host-key verdicts,
  status parsing, and remote-URL classification. (2026-07-12)

## Types and IPC

- **`ts-rs` types are emitted as a single `src/types/generated.ts` by a dedicated
  generator (`popush-core/examples/generate_types.rs`), not by `#[ts(export)]`'s
  per-type files.**, The Resolved Decision mandates one `generated.ts` that CI
  checks for staleness. `ts-rs`'s default export writes one file per type, so a
  small generator that enumerates the exported types produces the required single
  artifact deterministically. CI runs the generator then `git diff --exit-code`. (2026-07-12)

- **`SshError::Timeout` carries `after_ms: u64` instead of `std::time::Duration`.**
  `Duration` does not implement `ts_rs::TS`, and it must cross the IPC boundary.
  Milliseconds as a `u64` is unambiguous and serialises cleanly. (2026-07-12)

- **Adapter status parsing is expressed as pure functions (`parse_status`,
  `resolve_status`) separate from the command-issuing layer.**, This is what
  makes the golden-file tests possible without a live server, and it keeps the
  honesty rule enforceable in a unit test: "files present, no health check"
  resolving to amber is asserted directly. (2026-07-12)

## Resolved Decisions (from the specification, restated for the record)

- **GPLv3, Copyright (c) 2026 twostep.**, The principles (accountless, no
  telemetry, free) are what a closed fork would strip first; the licence keeps
  them attached. (2026-07-12)
- **Accent colour violet `#7C6BF2`.**, Baked into the design tokens; teal
  dropped. (2026-07-12)
- **Flatpak uses XDG portals for per-repo filesystem access; no
  `--filesystem=home`.**, A security-focused tool asking for full home access
  undermines its own message. More friction, correct outcome. (2026-07-12)
- **System tray (aggregate status dot) ships in Phase 9.** (2026-07-12)
- **No `.deb`, no Snap, no auto-updater.**, Updates come from the package
  manager; an update channel would need a server and violate. (2026-07-12)

## Dependencies added beyond (requires justification)

- **`reqwest` (rustls-tls, no default features) in `src-tauri` only.**, Two
  features need an HTTP client that did not name: the static adapter's health
  check and the optional GitHub API
. `reqwest` with `rustls-tls` avoids an OpenSSL system dependency and
  composes with the existing Tokio runtime. It stays out of `popush-core`; the
  core only classifies an already-obtained HTTP status code
  (`HealthVerdict::from_http_status`) and parses already-fetched GitHub JSON. -
  2026-07-12
- **`async-trait` in `src-tauri` only.**, `russh` 0.45's `client::Handler` trait
  is declared with `#[async_trait]`, so the host-key handler impl must use it too.
  Verified against the resolved crate source. (2026-07-12)

## Verification boundary (stated honestly)

The workspace split means `popush-core` and the discrete infrastructure modules
are fully built and verified: `popush-core` compiles, passes its tests, and is
clippy/fmt clean; the `russh`, `git2`, `reqwest`+`keyring`, and
`tracing-subscriber` modules were each type-checked (and, where they hold pure
logic, unit-tested) against their real pinned APIs in isolation, because the full
`src-tauri` binary links WebKitGTK and only compiles on the Linux target.

The remaining integration point is the **SSH connection-pool lifecycle** inside
the Tauri command handlers (opening a pool on demand, caching it per server in
`AppState` behind async access, reconnect on loss) and the **live log stream**
plumbing. Every building block for these exists and is verified, `SshPool`,
`parse_known_hosts`, the adapters, the pipeline orchestrator, but their final
assembly into async Tauri commands, and the end-to-end Ship It run, are exercised
by the integration suite against the test VPS on the target rather than in
this headless environment. This is recorded rather than hidden.

## Small choices where the spec was silent

- **`RemoteCommand::render` panics on a placeholder/argument count mismatch.** -
  Such a mismatch is only reachable through a Popush bug (a template disagreeing
  with its call site), never through user input. A loud panic caught by the first
  test is safer than a silent mis-render of a security-critical command. Every
  template is exercised by a test, so the panic is unreachable in shipped paths. (2026-07-12)
- **The config loader rejects an `identity_file` value that looks like an inline
  private key** (contains `PRIVATE KEY` / `BEGIN OPENSSH`)., A guardrail for:
  a mis-paste should never silently land a secret in `config.toml`. (2026-07-12)
- **The personal-key-on-server warning uses a comment heuristic** (a key whose
  comment mentions `deploy`/`popush` is treated as a deploy key)., There is no
  perfectly reliable signal over SSH; a conservative heuristic that warns on
  anything not clearly a deploy key errs on the side of caution. -
  2026-07-12
