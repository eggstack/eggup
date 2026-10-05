# Service Lifecycle Milestone 009 — eggup-service 0.1.2 publication

Status: ready

Repository baseline: `db5b3121f17f92029a47c389e68a464bc4478b27`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`

Primary class: infrastructure / release operations

Hard dependencies:

- Service M001-M008 are closed with green Stable/MSRV/macOS/Windows evidence.
- `eggup-service 0.1.0` and `0.1.1` are already owned on crates.io.
- `eggup-core 0.1.2` is published and satisfies `eggup-core = ^0.1.0`.

Release-order dependency created by this milestone:

- Service M009 SHOULD close before Acquisition M010 changes the shared workspace version from 0.1.2 to 0.1.3. Acquisition M010 is release-order blocked on M009 unless deliberately replanned to skip service 0.1.2.

## 1. Objective

Publish the already-qualified current `eggup-service 0.1.2` package to crates.io without changing service runtime behavior, so downstream consumers can use M005/M006 lifecycle/disposition semantics from a registry dependency.

This is the missing service publication gate for the Gregg migration line. It is not the Gregg migration.

## 2. Readiness and dependencies

M009 is dependency-ready now. The service source contains closed M001-M008 behavior, while the crate changelog explicitly states that 0.1.2 has never been published. The root workspace remains version 0.1.2 at this planning baseline.

Because Acquisition M010 plans a workspace-wide 0.1.3 bump, service 0.1.2 should publish first if that version is to represent the already-qualified substrate.

If 0.1.2 already exists unexpectedly or ownership is ambiguous, stop before changing release state.

## 3. Current implementation evidence

Current unpublished service functionality includes systemd/launchd/cron/Windows SCM adapters; M005 transaction/lifecycle composition with `KeepInstalled | RollBack`; M006 managed-running/managed-stopped/direct-running/stopped/foreign-preserved disposition/revalidation; M007 UTF-8-safe bounded diagnostics; M008 deterministic deadline proof; and post-closure bug-audit fixes already documented under Unreleased.

The packaged crate must prove its `eggup-core = ^0.1.0` dependency resolves from crates.io rather than the workspace path.

## 4. Invariants that must not regress

- zero service runtime/source change for publication;
- no Gregg-specific policy/constants;
- no automatic elevation;
- foreign/unknown registrations remain non-destructible;
- stopped-stays-stopped and M006 exact executable/config semantics unchanged;
- Rust 1.89 remains MSRV;
- manual publication only;
- do not move/recreate existing `v0.1.2`, which already denotes the core/archive publication source;
- release notes explicitly state service 0.1.2 was published later from a different source commit.

## 5. Scope and non-scope

### In scope

Registry version/ownership preflight; changelog/release-note cut; exact package/publish dry-run from clean source; full hosted matrix; manual service publish; registry-only external compile/smoke; planning/docs closure.

### Explicitly out of scope

Service source edits; Core M010/M011 implementation; Acquisition M010 bump/publication; changing core dependency policy; publishing any other crate; moving `v0.1.2`; creating another GitHub release named 0.1.2; consumer changes.

## 6. Required release changes and publication procedure

### 6.1 Preflight

Confirm exact service 0.1.2 is absent, publish authority is valid, core 0.1.2 is registry-visible, workspace version is still 0.1.2, no service source diff is required, and the release-prep tree is clean.

### 6.2 Changelog cut

Convert `crates/eggup-service/CHANGELOG.md` Unreleased entries into `0.1.2 — <date>`, remove wording saying no publication milestone authorizes it, and preserve historical closure evidence.

Update root `CHANGELOG.md` and GitHub Release 0.1.2 notes additively to say service 0.1.2 was published later from a different commit. Never imply the existing tag moved.

### 6.3 Qualify exact package

Run full workspace gates plus:

```text
cargo package -p eggup-service --locked
cargo package -p eggup-service --list --locked
cargo publish -p eggup-service --dry-run --locked
```

Inspect the normalized packaged manifest/dependency graph.

Use an external registry-only fixture with `eggup-service = "=0.1.2"`, no path/Git overrides. Compile representative service types and run deterministic ownership/disposition tests where platform-appropriate without mutating real host managers.

### 6.4 Publish and verify

From the exact clean release-prep commit:

```text
cargo publish -p eggup-service --locked
```

Wait for exact registry visibility, record checksum/metadata, then rerun the registry-only fixture. Publish no other 0.1.2 crate.

## 7. Ordered work packages

1. Registry/ownership/version preflight.
2. Cut service/root changelog and release-note text.
3. Commit release-prep docs only.
4. Run full local/hosted qualification.
5. Package/dry-run and inspect contents/dependencies.
6. Pre-publication external fixture against registry dependencies.
7. Publish service 0.1.2 manually.
8. Verify registry and post-publication fixture.
9. Write closure; update service/consumer roadmaps and registry.
10. Unblock Acquisition M010 release order and mark Gregg service prerequisite satisfied.

## 8. Failure, cancellation, restart, and contention semantics

No runtime semantics change.

Before registry acceptance every step is reversible. After acceptance 0.1.2 is immutable. If a defect is found later, plan a new patch; do not overwrite/yank reflexively.

If registry visibility lags, verify exact state before retrying. Never assume timeout means upload failure.

## 9. Compatibility and migration

Published 0.1.0/0.1.1 remain valid. 0.1.2 exposes already-qualified corrective behavior; existing consumers are not migrated automatically.

The crate remains on compatible `eggup-core ^0.1.0`. Gregg M004 should use an immutable registry service version once this closes.

## 10. Required tests

Full service suite, full Stable workspace, MSRV check/tests per CI, macOS service tests, Windows SCM tests, package/dry-run, registry-only external fixture, dependency tree proving registry core, and proof no service `src/` diff exists in release-prep.

## 11. Required verification commands

```text
git status --porcelain
git rev-parse HEAD
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo package -p eggup-service --locked
cargo package -p eggup-service --list --locked
cargo publish -p eggup-service --dry-run --locked
git diff --check
```

After authorization: `cargo publish -p eggup-service --locked`.

## 12. Documentation updates

Update service changelog, root changelog, append-only GitHub Release 0.1.2 notes, `docs/crates.md`, `architecture/service-lifecycle.md`, service roadmap, consumer roadmap, registry, and new `plans/closure/service-lifecycle/009-status.md`.

## 13. Acceptance criteria

M009 closes only when exact absence/ownership was proven; no service source behavior changed; release-prep is locally/hosted green; package and dry-run pass; only service 0.1.2 is published; registry-only fixture passes; registry metadata/source commit are recorded; existing v0.1.2 tag is unchanged; release notes record later service publication; Acquisition M010 becomes ready; and Gregg service substrate is registry-consumable.

## 14. Stop conditions

Stop if service 0.1.2 exists unexpectedly, any service source edit is needed, packaged dependencies resolve path/Git, hosted native qualification is not green, publishing requires moving v0.1.2, or the workspace has already moved to 0.1.3.

If the workspace is 0.1.3, write a replacement service 0.1.3 publication plan rather than backdating version metadata.

## 15. Closure evidence required

Record release-prep SHA, registry preflight, package file list/checksum/size, dry-run/publish results, hosted run id, registry-only fixture manifest/lock/test, proof tag was not moved, release-note append, list of crates intentionally not published, Acquisition M010/Gregg dependency reconciliation, and unresolved findings.

## 16. Handoff notes

Publish this before the workspace-wide 0.1.3 bump.

The point is to make the already-qualified M005/M006 substrate consumable, not to use publication as an excuse for another service refactor.
