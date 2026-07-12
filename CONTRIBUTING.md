# Contributing to Popush

Thank you for wanting to help. Popush is a small, deliberate tool. Contributions
that keep it that way are the most welcome kind.

## The shape of the codebase

Popush is a Cargo workspace with a hard rule about where logic lives:

- **`popush-core`** holds all business logic and links no GUI libraries. It must
  stay headless-testable. If your change involves *what a command does*, *how
  status is parsed*, *how an error reads*, or *how the pipeline behaves*, it
  belongs here, with tests.
- **`src-tauri`** is the Tauri binary. It is glue: IPC commands, event emission,
  and socket I/O. It contains no business logic.
- **`src/`** is the React frontend. It renders state and dispatches intents. It
  does not know what command restarts a Docker container.

The single most security-critical file is
[`popush-core/src/ssh/command.rs`](popush-core/src/ssh/command.rs). Any change
there must come with adversarial tests, and must never introduce string
formatting of remote commands.

## Before you open a pull request

Run the full local gate:

```
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test -p popush-core
cargo run -p popush-core --example generate_types   # regenerate types if you changed a shared type
pnpm lint
pnpm test
```

If you changed a type that crosses the IPC boundary, regenerate
`src/types/generated.ts` with the command above and commit it. CI fails if it is
stale. Do not edit that file by hand.

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`,
`fix:`, `docs:`, `refactor:`, `test:`, `chore:`. Keep commits small and scoped.

## Dependencies

The dependency tree is kept small on purpose. Adding a crate or npm package that
is not already in use requires an entry in
[`docs/DECISIONS.md`](docs/DECISIONS.md) justifying it. No analytics, telemetry,
or crash-reporting SDK may ever enter the tree.

## Comments

Comments explain *why*, not *what*. Every public Rust item gets a doc comment.

## Code of conduct

Be plain, honest, and calm. That is the brand voice, and it is also how we talk
to each other.
