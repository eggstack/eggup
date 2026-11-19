# Acquisition Transport Milestone 019 — eggup-acquisition 0.2.1 publication

Status: closed — see `plans/closure/acquisition-transport/019-status.md`.

Repository baseline: b5a989a72dc185b30cd8c0cb67b5110ed03acd86

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Affected roadmaps:
- plans/subsystems/archive-extraction-roadmap.md
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md

Primary class: infrastructure

Hard dependencies:
- Acquisition M018 is closed and hosted-qualified in run 44155438351.
- The workspace package version is 0.2.1 and eggup-acquisition inherits it. No package version edit is part of M019.
- The current Eggpack source candidate exact-pins eggup-acquisition at =0.2.1. The published Eggpack 0.2.0 pin remains =0.2.0, so this release is required before Eggpack 0.2.1 can be published.
- crates.io must report eggup-acquisition 0.2.1 absent immediately before upload.

## 1. Objective

Publish only eggup-acquisition 0.2.1 to crates.io as the seam prerequisite for the Eggpack 0.2.1 exact-pin update. Publish after Archive M008 and before Eggpack M009. The current package candidate adds no runtime dependency and has no production source change from acquisition 0.2.0; the versioned registry package is required because Eggpack 0.2.1 pins the seam exactly.

## 2. Why this milestone is ready

- Acquisition M018 is closed and hosted-qualified; the complete current source baseline is the Core 0.2.1 publication source.
- crates.io currently lists eggup-acquisition 0.2.0 as latest; 0.2.1 is absent at plan authoring.
- The workspace candidate sets every crate version to 0.2.1. eggup-acquisition retains an empty dependencies table.
- The workspace eggup-eggpack candidate pins eggup-acquisition =0.2.1, while the published eggpack 0.2.0 pins =0.2.0.
- The published curl and eggfetch 0.2.0 manifests require eggup-acquisition ^0.2, which already accepts 0.2.1. They are not stale exact-pin dependents and do not need republication for this dependency transition.

## 3. Current implementation evidence

- Repository baseline: b5a989a72dc185b30cd8c0cb67b5110ed03acd86.
- Acquisition M018 plan/closure: plans/implementation/acquisition-transport/018-native-fake-curl-test-helper.md and plans/closure/acquisition-transport/018-status.md.
- crates/eggup-acquisition/Cargo.toml inherits workspace version and retains zero dependencies.
- The source diff from the 0.2.0 train baseline has no acquisition Rust or manifest dependency change; the package-version change comes from the shared workspace version and current release records.
- Crates.io registry reads show current curl and eggfetch requirements are ^0.2; Eggpack alone exact-pins this seam at =0.2.0.

## 4. Invariants that must not regress

- eggup-acquisition remains the transport-neutral zero-dependency seam; do not add transport, service, archive, or producer dependencies.
- No release-selection, destination, authenticity, or execution policy is added.
- The AcquisitionError contract, cancellation/bounds semantics, diagnostic limits, and adapter-facing hidden functions remain unchanged.
- Do not move the existing v0.2.1 tag or alter any existing crate version.
- No package is published from CI; this is a manual, plan-gated publication.

## 5. Scope

### In scope

- Publish exactly eggup-acquisition 0.2.1 from a single exact hosted-qualified source commit.
- Confirm the package remains dependency-free and prove registry-only consumption with an external fixture.
- Record source SHA, registry checksum, publisher, package contents, and registry readback in the M019 closure.
- Reconcile the acquisition roadmap, registry, crate changelog, version guide, architecture release table, and existing v0.2.1 release notes after the upload.

### Explicitly out of scope

- Any production Rust/API change or new dependency.
- Publication of eggup-curl or eggup-eggfetch: their current published ^0.2 seam requirements already accept acquisition 0.2.1 and no stale exact edge was found.
- Publication of eggup-archive or eggup-eggpack under M019; Archive M008 and Eggpack M009 own those releases.
- Publication of eggup-service, eggup-core, or eggup-transport-footprint.
- Moving tags, yanking or overwriting versions, or enabling release automation.

## 6. Required production changes

No production Rust, manifest dependency, API, or behavior change is required. Prepare the crate's 0.2.1 changelog entry to state that the seam still has zero dependencies and no source/API behavior change; publication exists to satisfy Eggpack's exact =0.2.1 dependency. Do not claim the registry publication until readback succeeds.

## 7. Ordered work packages

1. Reconfirm source SHA, clean package inputs, current crates.io version absence, prior release non-yanked state, and publisher authority.
2. Register M019 and mark it ready in the acquisition roadmap and plans/registry.md.
3. Run the local gate, inspect cargo tree, package file list, cargo package, and publish dry-run.
4. Require all five hosted CI lanes green on the exact publication source SHA.
5. Immediately before upload, obtain confirmation for eggup-acquisition 0.2.1. Publish once; on ambiguous response, check crates.io before any retry.
6. Read back version, publisher, non-yanked status, dependency list, downloaded archive checksum, and .cargo_vcs_info.json source SHA.
7. Build an external registry-only fixture depending on eggup-acquisition =0.2.1 and inspect its lockfile for registry-only sources.
8. Close M019 before drafting the downstream Eggpack M009 plan; then update records and extend the existing v0.2.1 release notes without moving the tag.

## 8. Failure, cancellation, restart, and contention semantics

This package publication does not mutate an installed application or start an external process at runtime. A build, package, fixture, or CI failure blocks publication. Never add a dependency merely to make the package appear substantive. crates.io publication is irreversible at the version level: do not republish, overwrite, or yank an existing version; if an upload response is ambiguous, use registry readback before deciding whether another action is safe. If an unexpected version or publisher appears, stop and report it. Preserve unrelated worktree changes.

## 9. Compatibility and migration

There is no source/API/runtime change and no migration required for direct users of acquisition 0.2.0. The seam remains zero-dependency. A fresh Cargo resolution of curl, eggfetch, or service can accept 0.2.1 through their ^0.2 requirements; existing lockfiles may retain 0.2.0 until updated. This version is also required by the next Eggpack exact-pin release, which deliberately selects a coherent 0.2.1 set.

## 10. Required tests

- Run full workspace tests, clippy, docs, and the exact dependency-tree review on the publication source.
- Confirm the package's normal dependency list is empty and package contents include the expected README, changelog, and source while excluding tests/benches as configured.
- After registry upload, compile a separate fixture requiring exactly eggup-acquisition 0.2.1 and ensure the resulting lock graph contains no path, Git, or patch source.

## 11. Required verification commands

Run in order and record exact outcomes:
1. cargo fmt --all -- --check
2. cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
3. cargo test --workspace --all-targets --all-features --locked
4. cargo doc --workspace --no-deps --locked
5. cargo tree --workspace --locked, with manual review of dependency boundaries
6. ./scripts/check-docs-truth.sh
7. git diff --check
8. cargo package -p eggup-acquisition --list --locked
9. cargo package -p eggup-acquisition --locked
10. cargo publish --dry-run -p eggup-acquisition --locked
11. Hosted CI on the exact source SHA: stable, msrv, service-systemd, macos, and windows-check
12. After upload, cargo check --manifest-path plans/closure/acquisition-transport/019-package-fixture/Cargo.toml --locked and cargo tree --manifest-path plans/closure/acquisition-transport/019-package-fixture/Cargo.toml --locked
13. After upload, read back crates.io metadata and verify downloaded .crate checksum and .cargo_vcs_info.json source SHA.
14. cargo publish -p eggup-acquisition --locked is an externally visible final action and must not run until the exact package/version is confirmed.

## 12. Documentation updates

- crates/eggup-acquisition/CHANGELOG.md: 0.2.1 exact-pin availability with no source/API/dependency change.
- plans/implementation/acquisition-transport/019-eggup-acquisition-0.2.1-publication.md and plans/closure/acquisition-transport/019-status.md.
- plans/subsystems/acquisition-transport-roadmap.md and plans/registry.md.
- After publication: root CHANGELOG.md, docs/releases.md, architecture/tooling-governance.md and overview.md, and the existing GitHub Release v0.2.1 notes.
- Preserve the immutable 0.2.0 record and explain why curl/eggfetch do not need republication.

## 13. Acceptance criteria

- Exactly eggup-acquisition 0.2.1 is published for M019, while 0.2.0 stays available and not yanked.
- Registry dependency list remains empty; no forbidden source edge appears.
- Local package checks and all five hosted CI lanes pass on the exact source SHA.
- External registry-only fixture resolves and builds the exact 0.2.1 release.
- Checksum, registry publisher/version, and .cargo_vcs_info.json agree with the qualified package/source.
- M019 closure, roadmap, registry, changelog, docs, and release notes agree; no existing tag moves and no other crate is published under this milestone.

## 14. Stop conditions

Stop before upload if 0.2.1 is already present, publication authority is absent, the package is not dependency-free, the package or any required hosted lane fails, the source SHA is not the qualified commit, the registry response cannot be reconciled, or the user has not confirmed the exact package/version. Do not start Eggpack M009 until M019 is closed and registry-visible.

## 15. Closure evidence required

Record final status; source plan and roadmap; exact reviewed and published SHA; commit/PR details; registry version, publisher, yanked status, checksum, normalized manifest and package file list; registry-only fixture and lockfile; exact local and hosted command outcomes; platform-specific evidence; compatibility, invariant, security, failure, rollback, and recovery reviews; unresolved findings with severity; and registry/roadmap/changelog/docs/release-note updates. Mark any unrun platform not run.

## 16. Handoff notes

M019 is a dependency-promotion publication, not a code fix. The exact consumer edge is Eggpack's current-source =0.2.1 pin. Published curl and eggfetch use ^0.2 and do not require republication; service's ^0.2 core edge likewise accepts Core 0.2.1. Publish Archive M008 and Acquisition M019 before authoring the downstream Eggpack M009 plan.
