# AGENTS.md

Rust workspace (edition 2021, MSRV 1.89, `resolver = "2"`). `unsafe_code = "deny"`, clippy `all = warn`.

## Workspace layout

- `crates/eggup-core` — policy-neutral local mechanics: `InstallPlan → Prepared → Verified → Validated → Receipt`. Sole dep: `sha2`. Never add transport, service-manager, or Eggpack deps here.
- `crates/eggup-acquisition` — transport-neutral seam (`AcquisitionTransport`, `FixtureTransport`, `FetchLimits`). No eggup deps.
- `crates/eggup-eggfetch` — native HTTP adapter; depends only on `eggup-acquisition`.
- `crates/eggup-service` — manager-neutral lifecycle; depends only on `eggup-core`.
- `crates/eggup-eggpack` — optional leaf adapter for Eggpack ReleaseManifest v1; `publish = false`, depends on `core` + `acquisition` (pinned `=0.1.1`) + `eggpack-manifest` by git rev. Only crate allowed to touch producer types.

New crates must join `Cargo.toml` members, inherit the 5 shared keys, and set `[lints] workspace = true`.

## Verify (in this order)

Gate is `scripts/check-local.sh`:

```sh
./scripts/check-local.sh
# = cargo fmt --all -- --check
# + cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# + cargo test --workspace --all-targets --all-features --locked
# + cargo doc --workspace --no-deps --locked
# + cargo tree --workspace --locked  # review-only, no gate
```

Focused runs: `cargo test -p eggup-core`, `cargo test -p eggup-core <name>`, `cargo clippy -p <crate> --all-targets --locked -- -D warnings`.
CI (`.github/workflows/ci.yml`) adds what the script lacks: MSRV `cargo check` on 1.89.0, `cargo test` on macOS, `cargo check` on Windows. CI never publishes; releases are manual.

## Where to look

Start at `architecture/overview.md`, then the deep dive for the crate at hand:

- `eggup-core` → `architecture/core-transaction.md` + contracts in `crates/eggup-core/docs/{domain,verification,transaction}.md`
- `eggup-acquisition` → `architecture/acquisition.md`
- `eggup-eggfetch` → `architecture/eggfetch-adapter.md`
- `eggup-service` → `architecture/service-lifecycle.md`
- `eggup-eggpack` → `architecture/eggpack-adapter.md`
- workspace, CI, plans/process → `architecture/tooling-governance.md`

On-demand skills (via the `skill` tool): `planning-workflow`, `verify-workflow`
in `.opencode/skills/`. There is no `.skills/` directory; `plans/` +
`architecture/` remain the source of truth, skills only summarize them.

## Core rules (not obvious from filenames)

- Authoritative contracts: `crates/eggup-core/docs/{domain,verification,transaction}.md` + `architecture/*.md`. SHA-256 is integrity evidence only — no authenticity/signature claims, no invented timeouts, no crash-journaling claims.
- Ownership `Absent | Owned | Foreign | Unknown` is caller-proven via `OwnershipVerifier`; `Owned` is never inferred. `commit` revalidates ownership + staged digests under lock; `Foreign`/`Unknown`/flap fails closed with a receipt (`RolledBack`/`RecoveryRequired`), not `Err` — only lock contention/setup failures return `Err`.
- Core never creates live destination parents, never deletes stale locks (`MutationLock::inspect` is read-only), stages to owner-private sibling dirs (`0700`/`0600` on Unix).
- `eggup-eggpack` archives stop at extraction-required evidence; URL/install-root/ownership/permission/authenticity policy stays caller-owned.

## Planning governance

- `plans/000–003` (spec, terminology, roadmap, process) are normative — do not edit in ordinary work. Accepted ADRs in `plans/adrs/` are superseded, never rewritten.
- Work flow: subsystem roadmap → bounded `implementation/<subsystem>/NNN-*.md` plan (16-section template in `implementation/README.md`, needs SHA baseline + non-goals + failure semantics + verification commands) → implement → `closure/<subsystem>/NNN-status.md` (requirement→evidence matrix, exact commands, per-platform results) → update `plans/registry.md` + roadmap status table. Load the `planning-workflow` skill for the full checklist.
- Public API changes need rustdoc; keep crate `README.md` + `CHANGELOG.md` (`Unreleased`, with no-publication/no-migration disclaimer) current.
