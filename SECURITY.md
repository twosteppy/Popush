# Security

Popush exists to protect two things: your private SSH keys and your servers.

## Reporting a vulnerability

Email `security@popush.dev` with a description of the issue and, if you can, a
proof of concept. Please do not open a public issue for a security problem until
it has been addressed. You will get an acknowledgement, and a fix or an
explanation, as quickly as we can manage.

## Supported versions

The latest released version is supported. Popush has no auto-updater by design;
updates come from your package manager.

## What Popush defends against

| Threat | Defence |
|---|---|
| A malicious or corrupted `config.toml` turning into remote code execution | Every interpolated value in a remote command is shell-escaped, and commands are built from compile-time templates, never string concatenation. A malicious `remote_path` cannot execute arbitrary commands. This is the most security-critical code in the project (`popush-core/src/ssh/command.rs`) and is covered by an adversarial escaping test corpus. |
| A man-in-the-middle on an SSH connection | Strict host-key verification. Unknown hosts prompt with a fingerprint and are never auto-accepted. A changed host key is a hard stop and cannot be dismissed with one click. |
| Popush itself exfiltrating a key | Popush never reads your private keys. Config holds key **paths**; authentication is delegated to `ssh-agent`. The code is open and auditable. |
| A compromised VPS gaining write access to your GitHub account | Deploy keys are read-only and scoped to a single repository. The setup wizard warns if it finds a personal key on a server. |
| Credentials leaking into logs | The logging layer has a redaction filter: key material, token patterns, and `Authorization` headers are replaced before anything is written. Logs are local-only and never transmitted. |

## What Popush does not defend against, and says so

| Threat | Why not |
|---|---|
| A compromised local machine | An attacker with code execution on your desktop already has your agent and your keys. Nothing at this layer can help. |
| A malicious `build_command` | You configure your own build commands. Popush is a tool that runs the commands you asked it to run. See "The honest weakness" in the README. |
| A supply-chain attack on a dependency | Mitigated by `cargo-audit` and `cargo-deny` in CI and by a deliberately small dependency tree. Mitigated, not eliminated — stated honestly. |

## The honest weakness

> Popush runs commands on your servers. If someone can modify your Popush config
> file, they can make Popush run their commands on your servers. Shell escaping
> prevents injection through *values*, but a `build_command` is, by definition, a
> command you asked Popush to run. Protect your config file the way you protect
> your `~/.ssh` directory.

## No secrets on disk

No file Popush writes contains a private key, a passphrase, or a token. The
optional GitHub token lives in the system keyring only. `config.toml` is
guaranteed secret-free, and you are invited to verify that in a text editor.
