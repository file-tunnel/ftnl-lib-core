# File Tunnel core library agent instructions

These instructions apply to this repository and every directory beneath it.

## Repository role

- This repository owns canonical JSON Schema normalization, deterministic code generation, additive SQL bootstrap generation, and ORM-neutral query plans.
- Keep database drivers, HTTP transport, UI, credentials, capability storage, file content, and application lifecycle outside this crate.
- The declarative-migrations integration is a bounded CLI process boundary. Spawn `dpm` directly without a shell and expose only `diff`, `verify`, and `bootstrap`; never add implicit `apply` behavior.
- Generated SQL must be parameterized or additive. Never interpolate record values or emit destructive migration statements.
- The library installs no global logger or OpenTelemetry provider. Applications own observability and must use redacted, metadata-minimal events.

## Validation

- Run `nix develop --command agent-check` before completing a change.
- Keep deterministic output tests for every generator and injection/redaction tests for SQL and process boundaries.
- Never commit credentials, database URLs, generated build output, or private schemas/data.

## Git workflow

- Keep changes focused and reviewable.
- Pull and merge remote work before pushing; avoid git rebase in favor of git merge.
- Never discard unrelated or uncommitted user work.
