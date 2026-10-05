# AGENTS.md

Rust workspace (edition 2021, MSRV 1.89, `resolver = "2"`). `unsafe_code = "deny"`, clippy `all = warn`.

**This file is an index, not a second copy of the rules.** Every section below
names the `architecture/` deep dive that owns the detail. When you change code,
CI, the gate, or the plan system, follow the doc chain in
[`architecture/tooling-governance.md`](architecture/tooling-governance.md) §7.3
and the [`docs-hygiene`](.opencode/skills/docs-hygiene/SKILL.md) skill.

## Workspace layout

Full dependency rules and per-crate boundary rationale:
[`architecture/tooling-governance.md`](architecture/tooling-governance.md) §2.

- `crates/eggup-core` — policy-neutral local mechanics: `InstallPlan → Prepared → Verified → Validated → Receipt`. Deps: `sha2`, plus `self-replace` **Windows-only** (deferred deletion of a replaced running image; non-Windows graphs stay `sha2` only). Never add transport, service-manager, or Eggpack deps here.
- `crates/eggup-acquisition` — transport-neutral seam (`AcquisitionTransport`, `FixtureTransport`, `ComposedTransport`, `FetchLimits`). Zero dependencies — not even `sha2`. No eggup deps.
- `crates/eggup-eggfetch` — native HTTP adapter (`eggfetch-core` + `tokio` + `futures-util`); depends only on `eggup-acquisition`. Must not know `eggup-curl` exists.
- `crates/eggup-curl` — external-curl adapter, no embedded HTTP/TLS stack; depends only on `eggup-acquisition`. Must not know `eggup-eggfetch` exists. Transport composition lives in the seam, never in an adapter.
- `crates/eggup-service` — manager-neutral lifecycle; depends only on `eggup-core` (+ `windows-args`, + `windows-service` on Windows).
- `crates/eggup-archive` — bounded allowlisted local tar.gz/zip extraction; deps `tar`, `zip`, `flate2`, `fs_at`, `sha2`, dev-dep `eggup-core`. No transport, authenticity, or live-installation policy.
- `crates/eggup-eggpack` — optional leaf adapter for Eggpack ReleaseManifest v1; published (`=0.1.2`), depends on `core` + `archive` + `acquisition` (each pinned `=0.1.2`, local paths) + `eggpack-manifest = "=0.1.0"` from the registry + `sha2 = "0.10.9"`. Only crate allowed to touch producer types.
- `crates/eggup-transport-footprint` — non-published (`publish = false`) footprint fixtures: `curl_only` / `eggfetch_only` / `dual` binaries behind `required-features`.

New crates must join `Cargo.toml` members, inherit the 5 shared keys (`version`, `edition`, `rust-version`, `license`, `repository`), and set `[lints] workspace = true`. The 7 library crates also carry `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`; `eggup-transport-footprint` is the sole exception (binary-only, no `lib.rs`, and no `README.md`/`CHANGELOG.md` — see §3.4 of the governance deep dive).

## Verify (in this order)

Gate is `scripts/check-local.sh`; rationale, per-line purpose, and the coverage
gaps are in [`architecture/tooling-governance.md`](architecture/tooling-governance.md) §4–§5.

```sh
./scripts/check-local.sh
# = cargo fmt --all -- --check
# + cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# + cargo test --workspace --all-targets --all-features --locked
# + cargo doc --workspace --no-deps --locked
# + cargo tree --workspace --locked  # review step, not a gate
```

`cargo tree` always exits 0 on a resolvable graph. Read it: the dependency-boundary rules are enforced by **human review only**.

Focused runs: `cargo test -p eggup-core`, `cargo test -p eggup-core <name>`, `cargo clippy -p <crate> --all-targets --locked -- -D warnings`.

CI (`.github/workflows/ci.yml`) adds what the script lacks: `stable` (fmt/clippy/test/doc on Linux), `msrv` (compile-only `cargo check` on 1.89.0), `macos` (full `cargo test`), and `windows-check` — which **runs tests**, not just a check: the acquisition/archive/curl, `eggup-eggpack`, and named `eggup-service` diagnostic-safety runs, then `cargo test -p eggup-core --test current_executable` and `--test stale_lock_recovery` (Core M010/M011 self-replacement and lock-race fixtures must execute natively, not merely compile), then a full `cargo test --workspace --all-targets`. Neither `msrv` nor `windows-check` passes `--all-features`, so the feature-gated footprint binaries are built only by `stable` and `macos`. CI never publishes; releases are manual.

## Where to look

Start at [`architecture/overview.md`](architecture/overview.md) (birds-eye view, module map, cross-cutting invariants, deep-dive index), then the deep dive for the crate at hand:

- `eggup-core` → `architecture/core-transaction.md` + contracts in `crates/eggup-core/docs/{domain,verification,transaction}.md`
- `eggup-acquisition` → `architecture/acquisition.md`
- `eggup-eggfetch` → `architecture/eggfetch-adapter.md`
- `eggup-curl` → `architecture/curl-adapter.md`
- `eggup-service` → `architecture/service-lifecycle.md`
- `eggup-archive` → `architecture/archive-extraction.md`
- `eggup-eggpack` → `architecture/eggpack-adapter.md`
- `eggup-transport-footprint` → `architecture/transport-footprint.md`
- workspace, CI, plans/process, release, agent surface → `architecture/tooling-governance.md`
- consumer-facing how-to (`docs/`) → `architecture/tooling-governance.md` §7.4. `docs/` is **non-normative** and must never be the place a rule is introduced — it points at the contracts, it does not restate them.

## Skills (on-demand, via the `skill` tool)

In `.opencode/skills/`. Each summarizes a normative source; none adds policy. A
drifting skill is corrected by rewriting the skill.

- `planning-workflow` — plan → implement → closure → registry; 16-section template
- `verify-workflow` — local gate, CI lanes, coverage gaps
- `docs-hygiene` — which doc owns which fact; the doc-change chain; recurring drift classes
- `release-workflow` — manual crates.io publication, dependency ordering, exact-pin cascade

There is no `.skills/` directory; `plans/` + `architecture/` remain the source of truth.

## Core rules (not obvious from filenames)

Full statements with citations are in
[`architecture/overview.md`](architecture/overview.md) (cross-cutting invariants)
and [`architecture/tooling-governance.md`](architecture/tooling-governance.md).

- Authoritative contracts: `crates/eggup-core/docs/{domain,verification,transaction}.md` + `architecture/*.md`. SHA-256 is integrity evidence only — no authenticity/signature claims, no invented timeouts, no crash-journaling claims.
- Ownership `Absent | Owned | Foreign | Unknown` is caller-proven via `OwnershipVerifier`; `Owned` is never inferred. `commit` revalidates ownership + staged digests under lock; `Foreign`/`Unknown`/flap fails closed with a receipt (`RolledBack`/`RecoveryRequired`), not `Err` — only lock contention/setup failures return `Err`.
- `validate` requires **every** member to carry `IntegrityStatus::Verified`. A member declared `IntegrityRequirement::None` (`NotRequired`) always fails — there is no escape hatch.
- Core never creates live destination parents and never *decides* that a lock is stale. `MutationLock::acquire` never recovers; recovery is opt-in per commit via a caller-supplied `StaleLockVerifier` and displaces only the exact observed record. Staging is owner-private (`0700`/`0600` on Unix); its placement is an authority decision — ordinary plans stage beside the install root, `InstallPlan::for_current_executable` stages **inside** it so a self-update never needs write authority above the executable's directory.
- A program can replace the executable it is running through the same transaction model (`InstallPlan::for_current_executable`). The old generation is renamed aside, not deleted, so `RollBack` stays possible; on Windows a kept-installed update reports `CleanupDisposition::DeferredToProcessExit` because the old image is still mapped.
- `eggup-eggpack` archives stop at extraction-required evidence; URL/install-root/ownership/permission/authenticity policy stays caller-owned.

## Planning governance

Process detail: [`architecture/tooling-governance.md`](architecture/tooling-governance.md) §6. Full checklist: the `planning-workflow` skill.

- `plans/000–003` (spec, terminology, roadmap, process) are normative — do not edit in ordinary work. Accepted ADRs in `plans/adrs/` are superseded, never rewritten.
- Work flow: subsystem roadmap → bounded `implementation/<subsystem>/NNN-*.md` plan (16-section template in `implementation/README.md`, needs SHA baseline + non-goals + failure semantics + verification commands) → implement → `closure/<subsystem>/NNN-status.md` (requirement→evidence matrix, exact commands, per-platform results) → update `plans/registry.md` + roadmap status table.
- "medium-or-higher" is the de facto blocking-severity gate: any such issue still open forces a new corrective before the milestone closes.
- Public API changes need rustdoc; keep library-crate `README.md` + `CHANGELOG.md` (`Unreleased`, with a no-publication/no-migration disclaimer) current. The root `CHANGELOG.md` aggregates but does not replace the per-crate ones — an unpublished fix (notably a security fix in an already-published crate) must appear in both.

## Releasing

Publication is a tracked milestone, never a side effect and never automated by CI. See the `release-workflow` skill and
[`architecture/tooling-governance.md`](architecture/tooling-governance.md) §8. Note that `eggup-eggpack` pins `=0.1.2`, so a seam fix needs a separate adapter republication to reach it.
