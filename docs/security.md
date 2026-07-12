# Security model

Popush protects your private SSH keys and your servers. This document describes
how.

## Posture

| Control | Implementation |
|---|---|
| Private keys never read | Config holds key **paths**. Authentication is delegated to `ssh-agent`. The only exception is an unencrypted key you explicitly configured. |
| Passphrases never handled | There is no passphrase input field anywhere in the app. |
| Tokens never in config | The optional GitHub token lives in the system keyring only. `config.toml` is guaranteed secret-free. |
| No shell injection | Compile-time command templates plus `shell-escape` on every argument. Adversarially tested. |
| Host-key verification | Strict. Unknown hosts prompt with a fingerprint. A mismatch refuses to connect and warns loudly. |
| No auto-update | Updates come from the package manager. There is no update channel to attack. |
| Dependency auditing | `cargo-audit` and `cargo-deny` run in CI; any advisory fails the build. |
| No telemetry SDKs | Enforced by `cargo-deny` policy and review. |

## Command escaping

The most security-critical code in Popush is
`popush-core/src/ssh/command.rs`. A `RemoteCommand` is not a string: it is a
compile-time template with `{}` placeholders plus a vector of arguments, each
shell-escaped at construction. A value like `; rm -rf /` arriving through a
`remote_path` cannot break out of its argument position — it is one inert word.
An adversarial test corpus injects known attack strings into every argument
position and asserts each is neutralised, and that the command log shows exactly
what was sent.

## Host keys

On first connection Popush reads `~/.ssh/known_hosts`. A known, matching key
connects. An unknown host is presented with its fingerprint for you to verify —
never auto-accepted. A known host whose key has **changed** is refused: that is
the signature of a man-in-the-middle, and the warning is not dismissible with a
single click.

## Deploy keys

When a VPS needs to `git pull` from a private repository, the right answer is a
**deploy key**: an SSH key that lives on the server and is registered with that
one repository, read-only. A compromised VPS then has read access to a single
repo, not write access to your whole GitHub account — strictly better than a
personal access token on the server. The setup wizard walks this flow, and
actively warns if it finds a **personal** private key on a server, which it must
never place there.

## The honest weakness

> Popush runs commands on your servers. If someone can modify your Popush config
> file, they can make Popush run their commands on your servers. Shell escaping
> prevents injection through *values*, but a `build_command` is, by definition, a
> command you asked Popush to run. Protect your config file the way you protect
> your `~/.ssh` directory.
