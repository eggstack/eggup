# Acquisition Transport Milestone 020 — eggup-eggfetch 0.2.1 publication with eggfetch-core 0.3.0

Status: closed 2026-11-20; published eggup-eggfetch 0.2.1 from qualified source 2a33cfe4712dfb88a823614ee6f6a439ecdf792e; registry SHA-256 2ca53436f6f12ca3ae0a9d194e81a6af6bf04fadf2d90b49d866359ab9eab2dc; hosted run 44698883796 passed all five lanes; closure: plans/closure/acquisition-transport/020-status.md

Repository baseline: bf63aaefab0bcb84c134e831455c065df0dd4c76

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md (classification only; no producer-side work)

Primary class: infrastructure

## 1. Objective

Update the native HTTP adapter to the latest registry release eggfetch-core 0.3.0, preserve the adapter's explicit four-feature policy, and publish eggup-eggfetch 0.2.1 from one hosted-qualified source commit. The current workspace source already declares 0.2.1; crates.io lists only 0.2.0, so this is the requested +0.0.1 publication, not a 0.2.2 source bump.

## 2. Why this milestone is ready

- Acquisition M019 is closed; eggup-acquisition 0.2.1 is available for this crate's ^0.2 requirement.
- crates.io reports eggfetch-core 0.3.0 as latest, non-yanked, Rust 1.89, published 2026-10-31. Its http1, tls-rustls, tls-native-roots, and proxy features remain available.
- crates.io reports eggup-eggfetch 0.2.0 as latest and unyanked; 0.2.1 is absent. Workspace source is already 0.2.1.
- The code explicitly reports upstream 0.2.0 and a test pins that report. The dependency upgrade is testable before upload.
- Current reverse-dependency metadata lists eggsact 1.2.7 (^0.1.0), stegoeggo-cli 0.5.0 (^0.1.0), cargo-cleanme 0.2.15 (^0.1.2), and eggpass 0.1.2 (^0.2.0). Re-read after publication for the requested final list.

## 3. Current implementation evidence

- Baseline bf63aaefab0bcb84c134e831455c065df0dd4c76 has a clean worktree.
- The adapter manifest currently requires eggfetch-core = 0.2.0 with four explicit features; eggup-acquisition is a ^0.2 path dependency.
- The source reports eggfetch-core 0.2.0 in EggfetchTransport::eggfetch_version() and in its expected-value test.
- The workspace currently resolves eggfetch-core 0.2.2. The upstream 0.3.0 minor line is a major pre-1.0 dependency update and must be tested against adapter behavior and packaged sources.
- Workspace package version is 0.2.1; no workspace version change is in scope.

## 4. Invariants that must not regress

- Preserve the public adapter API, acquisition seam, bounded I/O, redaction, timeout and redirect policy, and fail-closed behavior.
- Keep exactly the four current Eggfetch features; do not enable cookies, retry, JSON, compression, or other protocols.
- eggup-eggfetch must not depend on eggup-curl; core and acquisition remain transport-neutral.
- Publish only eggup-eggfetch 0.2.1; do not republish, overwrite, or yank any version or publish the footprint crate.
- Do not move or create tags. The existing annotated v0.2.1 tag and GitHub Release target remain at Core's source commit; append release notes only.
- CI never publishes; upload is manual and only after exact-SHA qualification.

## 5. Scope

### In scope

- Change the eggfetch-core requirement to 0.3.0 using Cargo's normal caret semantics, retaining the four explicit features.
- Make only the internal API adaptation needed for compatibility; preserve Eggup's public contract and safety behavior.
- Update the upstream-version diagnostic and test, lockfile, documentation, changelogs, plan/closure, roadmap, registry, and existing release notes.
- Qualify eggup-eggfetch 0.2.1, prove the package against registry dependencies, publish once, and record artifact provenance and checksum.
- Re-query crates.io reverse dependencies after publication and list package versions and declared requirements.

### Explicitly out of scope

- Publishing eggup-curl, eggup-service, eggup-acquisition, eggup-eggpack, or any other crate.
- Updating downstream consumers such as eggsact, stegoeggo-cli, cargo-cleanme, or eggpass.
- Changes to Eggup public APIs, authentication/signature policy, release selection, destinations, or transport composition.
- Changing the workspace package version, tags, or CI publication configuration.

## 6. Required production changes

Update the dependency version and preserve its feature list. Bring the reported eggfetch-core version and its test expectation to 0.3.0. Make the smallest internal adaptation needed, if any; stop if preservation of the adapter contract cannot be established. Regenerate Cargo.lock from the qualified registry graph. Add an Unreleased changelog entry without claiming publication prematurely.

## 7. Ordered work packages

1. Register M020 in this plan, the acquisition roadmap, and plans/registry.md before production edits.
2. Update the dependency, lockfile, diagnostic, tests, and dependency-ownership documentation; inspect the resolved tree.
3. Run focused adapter tests/clippy, the full local gate, package inventory, package verification, and publish dry-run.
4. Build a standalone fixture that resolves eggup-eggfetch = 0.2.1 and eggfetch-core 0.3.0 solely from crates.io; inspect lockfile, tree, and metadata for path or Git edges.
5. Push the exact publication candidate and require all five hosted CI lanes green on its SHA. Do not upload if a required lane is missing or fails.
6. Reconfirm 0.2.1 absent, 0.2.0 unyanked, eggfetch-core 0.3.0 available, and publisher authority; publish eggup-eggfetch 0.2.1 once.
7. Read back version, publisher, yanked state, dependencies, downloaded archive checksum, and .cargo_vcs_info.json source SHA; verify the fixture against the registry artifact.
8. Close M020, update published-state docs and changelogs, append to the existing v0.2.1 GitHub Release without moving its target, and re-query crates.io reverse dependencies.

## 8. Failure, cancellation, restart, and contention semantics

This publication adds no runtime external-process behavior or installed-software mutation. Any compile, test, package, fixture, or CI failure blocks upload. If upstream API changes require expanding the public API or weakening bounded/fail-closed behavior, stop and report the blocker. crates.io publication is irreversible at the version level: never retry an ambiguous upload without checking registry state first; never overwrite or yank a version. Preserve unrelated worktree changes and stop if the target version is no longer absent.

## 9. Compatibility and migration

The intended Eggup version change is from the latest published 0.2.0 to 0.2.1; source already inherits workspace 0.2.1. The upstream dependency requirement moves from ^0.2.0 to ^0.3.0, selecting a distinct pre-1.0 minor line. The Eggup API is intended to remain unchanged; eggfetch-core is an implementation detail. Existing lockfiles remain on their prior version until updated; a fresh resolution of this package selects a 0.3.x version. Report reverse dependencies with their actual constraints; do not imply that ^0.1 consumers can use Eggup 0.2.1.

## 10. Required tests

- Full adapter tests for policy, redaction, streaming, timeout, and status behavior against resolved eggfetch-core 0.3.0.
- Full workspace test/clippy/doc gate with all features.
- Package verification against published eggup-acquisition 0.2.1, not a path edge.
- Independent registry-only consumer compile, exact package version, resolved tree, and metadata inspection.
- Hosted Stable, MSRV, service-systemd, macOS, and Windows-check lanes on the exact publication SHA.

## 11. Required verification commands

- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
- cargo test --workspace --all-targets --all-features --locked
- cargo doc --workspace --no-deps --locked
- cargo tree --workspace --locked (human boundary review)
- ./scripts/check-docs-truth.sh
- cargo test -p eggup-eggfetch --locked
- cargo clippy -p eggup-eggfetch --all-targets --locked -- -D warnings
- cargo package -p eggup-eggfetch --list --locked
- cargo package -p eggup-eggfetch --locked
- cargo publish --dry-run -p eggup-eggfetch --locked
- cargo check --manifest-path plans/closure/acquisition-transport/020-package-fixture/Cargo.toml --locked
- cargo tree --manifest-path plans/closure/acquisition-transport/020-package-fixture/Cargo.toml --locked
- cargo metadata --manifest-path plans/closure/acquisition-transport/020-package-fixture/Cargo.toml --locked --format-version 1

## 12. Documentation updates

Update the adapter architecture's current dependency version (not historical 0.2.0 behavior notes), root overview where the current adapter graph is described, tooling-governance publication table, docs/releases.md, consumer guidance only if needed, crate/root changelogs, acquisition roadmap, registry, and release-workflow/docs-hygiene summaries that state current publication status. Keep historical compatibility notes intact and identify them as historical. Append GitHub release notes only after registry verification.

## 13. Acceptance criteria

- The manifest requests eggfetch-core 0.3.0 and retains the four intended features; lockfile/build resolve 0.3.0.
- All required local and hosted checks pass; no platform lane is inferred.
- Package and dry-run pass; the registry-only fixture has no path, Git, or patch edges and compiles.
- Exactly eggup-eggfetch 0.2.1 is published from the qualified SHA; registry checksum matches local and downloaded artifacts, provenance matches, and the version is not yanked.
- Existing v0.2.1 tag and release target remain unchanged; no other crate is published.
- Closure, registry, roadmap, changelogs, and docs agree with crates.io; reverse dependencies are based on post-publication registry data.

## 14. Stop conditions

Stop before upload if upstream 0.3.0 is unavailable/yanked or lacks required features; if its API prevents preserving the adapter contract; if workspace, package, fixture, or hosted checks fail; if eggup-eggfetch 0.2.1 unexpectedly exists; if publisher authority cannot be confirmed; or if any artifact/provenance mismatch appears. Do not guess past an ambiguous registry response.

## 15. Closure evidence required

Record baseline and exact publication SHA; local gate and package/dry-run results; all five hosted CI results; package file list; registry version/dependency/publisher/yanked readback; matching SHA-256 for local, registry, and downloaded artifact; .cargo_vcs_info.json; registry-only fixture check/tree/metadata; tag/release-target check; post-publish reverse dependencies and requirements; docs/roadmap/registry reconciliation; unresolved findings and per-platform evidence.

## 16. Handoff notes

The requested package version is the next published patch, 0.2.1, because crates.io currently has 0.2.0 while repository source already has workspace version 0.2.1. Do not create source version 0.2.2 for this task. This milestone requires one publication only; public reverse dependencies with ^0.1 requirements are visibility evidence, not a request to update or publish those consumers.
