# Popush

**Built by twostep.**

*Your VPS, one click away.*

Popush is a native Linux desktop deployment client. It connects to your VPS
instances over SSH using your existing keys, shows every hosted site with an
honest live status, and turns deploy, restart, stop, start, logs, commit, and
push into buttons. Its headline feature, **Ship It**, chains uncommitted local
code to a verified-live site in one action, showing every step along the way.

The point is simple: go from "I changed a file" to "the change is live on my
site" without opening a terminal, and understand exactly what happened at every
step.

Linux only. Fedora 44 with KDE Plasma is the first-class target.

## Install (one command)

```sh
git clone https://github.com/twosteppy/popush.git
cd popush
bash install.sh
```

That is the whole thing. `install.sh` installs the build prerequisites, builds
Popush, and adds it to your application launcher **and** puts a double-clickable
icon on your Desktop — so you never have to remember a command to open it again.
Search your launcher for "Popush", or double-click the icon on your Desktop.

It never touches your `~/.ssh` or your servers; it only builds and installs the
app. To remove the desktop integration later, run `bash uninstall.sh` (your
config is left untouched).

First run: if you do not have an SSH key yet, create one with
`ssh-keygen -t ed25519`, then click **Add your first server** — no config files,
no terminal.

## Screenshots

![Main window](docs/screenshots/main.png)

![The Ship It pipeline](docs/screenshots/ship-it.png)

![The setup wizard](docs/screenshots/wizard.png)

## Privacy

> **Popush collects nothing.**
>
> There is no telemetry. No analytics. No crash reporting. No usage statistics.
> No update check. No phone-home of any kind.
>
> There is no Popush server. There is no Popush account. There is no Popush
> database. Popush does not know you exist.
>
> The only network traffic Popush generates is:
> - SSH connections to servers **you** configured
> - Git operations to remotes **you** configured
> - Optionally, and only if you explicitly enable it, read-only requests to
>   `api.github.com` using a token **you** provided
>
> Everything Popush knows about you lives in `~/.config/popush/config.toml`, on
> your machine. Open it. Read it. It is plain text and it contains no secrets.
>
> Fonts are bundled with the application, not fetched from a CDN, because a CDN
> request is a network call to a third party.
>
> This is not a policy. It is an architecture. There is nothing to collect
> because there is nowhere to send it.

There is nothing to breach, nothing to leak, nothing to subpoena, and nothing to
shut down.

## No server

Popush has no backend. There is no account system, no user table, no database,
and no cloud sync. First launch goes straight to the app. Your configuration
lives on your machine as human-editable TOML, and the app is not the sole source
of truth: you can edit `~/.config/popush/config.toml` by hand at any time.

## Security boundary: passphrases and ssh-agent

Popush never handles your SSH key passphrase. It delegates authentication to
`ssh-agent` through `SSH_AUTH_SOCK`. If a key has a passphrase and is not loaded
in the agent, Popush shows you the exact command to run:

```
ssh-add ~/.ssh/id_ed25519
```

rather than prompting for the passphrase itself. The moment Popush accepted a
passphrase, it would become a credential-handling application and enter the trust
path for your private key. Delegating to the agent keeps it out of that path.
Whether KDE Wallet, gnome-keyring, or a plain `ssh-agent` sits behind the socket
is not Popush's concern; it talks only to `SSH_AUTH_SOCK`.

## The honest weakness

> Popush runs commands on your servers. If someone can modify your Popush config
> file, they can make Popush run their commands on your servers. Shell escaping
> prevents injection through *values*, but a `build_command` is, by definition, a
> command you asked Popush to run. Protect your config file the way you protect
> your `~/.ssh` directory.

## Building from source

Popush is a Cargo workspace with two crates:

- **`popush-core`** — all business logic (SSH command construction, config,
  service adapters, git URL handling, the error taxonomy, the pipeline). It links
  no GUI libraries, so it builds and tests anywhere:

  ```
  cargo test -p popush-core
  ```

- **`src-tauri`** — the Tauri v2 binary. It is the presentation and IPC shell and
  contains no business logic. It links WebKitGTK and only builds on Linux. On
  Fedora you need the WebKitGTK 4.1 development libraries and the usual Tauri
  Linux dependencies:

  ```
  sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel librsvg2-devel
  pnpm install --ignore-scripts
  pnpm tauri dev     # or: pnpm tauri build
  ```

  `--ignore-scripts` is used because esbuild's optional postinstall is not needed
  (its platform binary ships prebuilt) and some strict pnpm configurations treat a
  skipped build script as a fatal error. The repo sets `verifyDepsBeforeRun: false`
  so `pnpm tauri dev/build` does not re-trigger that install.

The frontend is built with Vite and pnpm:

```
pnpm install
pnpm build
```

The TypeScript types the frontend uses are generated from Rust with `ts-rs` and
must not be edited by hand:

```
cargo run -p popush-core --example generate_types
```

Target platform: Fedora 44 + KDE Plasma. Linux only — there are no Windows or
macOS builds by design.

## Licence

Popush is released under the **GNU General Public License v3.0**. See
[LICENSE](LICENSE).

The principles behind Popush — accountless, no telemetry, free forever — are
exactly what a closed fork would strip first. GPLv3 is what keeps them attached
to the name. If you fork Popush, those principles come with it.

Built by twostep.
