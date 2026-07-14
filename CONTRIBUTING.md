# Contributing

## Adding a provider (Copilot, Codex, ...)

The app core is provider-agnostic — it reads every `*.json` file matching
the `UsageSnapshot` schema in the shared snapshots directory (see
[`docs/usage-snapshot-schema.md`](docs/usage-snapshot-schema.md)). To add a
new provider:

1. Write a collector script under `scripts/` that produces a
   `snapshots/<provider>.json` file matching the schema, sourced from
   whatever documented, ToS-safe mechanism that provider exposes.
2. Add a `docs/setup-<provider>.md` explaining how to wire it up.
3. Don't touch `src-tauri/src/tray.rs`, `snapshot.rs`, or the popover UI
   unless the new provider needs a genuinely new *type* of display (the
   existing percentage/countdown windows model should cover most rate-limit
   shapes).

## Rust backend

```sh
cd src-tauri
cargo check
cargo test
cargo clippy --all-targets
```

## Versioning and commit hygiene

See [`CLAUDE.md`](CLAUDE.md#versioning) for the SemVer rules this project
follows, and add a `CHANGELOG.md` entry under `[Unreleased]` for
user-visible changes.
