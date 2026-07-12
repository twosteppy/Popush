# Troubleshooting

Every Popush error names what happened, what it means for you, and what to do
next. This page collects the common ones.

## "The key … has a passphrase and is not loaded in your SSH agent."

Popush does not handle passphrases, by design. Load the key into your agent and
try again:

```
ssh-add ~/.ssh/id_ed25519
```

## "SSH_AUTH_SOCK is not set; no ssh-agent is running."

There is no agent for Popush to delegate to. Start one, then add your key:

```
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_ed25519
```

On KDE Plasma you can also let KDE Wallet manage your keys, which keeps them
loaded across sessions.

## An unknown host asks you to verify a fingerprint

This is expected the first time you connect to a server. Confirm the fingerprint
matches your server (compare it against what your provider shows, or run
`ssh-keygen -lf` on the server's public key) before accepting. Popush never
auto-accepts.

## "The host key for … has changed."

Popush will not connect. A changed host key can mean someone is intercepting the
connection. If you changed the server yourself (a rebuild, a new host), remove the
old entry from `~/.ssh/known_hosts` by hand and reconnect. Otherwise, do not
connect, and investigate.

## "This repository uses an HTTPS remote."

GitHub removed password authentication for HTTPS in 2021, so an HTTPS push needs
a token, and Popush does not collect tokens. Run the setup wizard; it offers to
convert the remote to SSH, which removes the problem entirely. The change is shown
before it is applied and is fully reversible.

## "Cannot reach …"

The server did not respond. Check it is up and reachable from your machine (try
`ssh` to it directly). Popush marks that server's sites as Unknown and offers a
Retry rather than silently looping.

## "Build failed with exit code …"

The build ran on the server and returned non-zero. The code is on the server but
has not been deployed; your site is still running the previous version. Read the
build output shown with the error, fix the build, and Ship It again. If you want
to return the server to the exact commit it was on before, the pipeline shows the
previous SHA and the exact rollback command.

## The git panel looks stale

Popush watches your `local_path` and refreshes within about half a second of a
file change, ignoring `.git/`, `node_modules/`, `target/`, `dist/`, `.next/`, and
anything in `.gitignore`. If it still looks stale, confirm `local_path` points at
the right clone.
