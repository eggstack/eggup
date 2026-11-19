# Archive Extraction Milestone 008 — eggup-archive 0.2.1 publication

Status: closed — see `plans/closure/archive-extraction/008-status.md`.

Repository baseline: b5a989a72dc185b30cd8c0cb67b5110ed03acd86

Source roadmap:
- plans/subsystems/archive-extraction-roadmap.md

Affected roadmaps:
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md
- plans/subsystems/acquisition-transport-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md
- plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md

Primary class: infrastructure

Hard dependencies:
- Archive M007 is closed and hosted-qualified in run 44022660351. It refreshed the ZIP dependency to 9.6.1 and retained the explicit bounded extraction implementation.
- The workspace package version is 0.2.1 and eggup-archive inherits it. The package version is already present in the reviewed source baseline; this milestone authorizes publication, not a version edit.
- The current eggup-eggpack 0.2.1 source candidate exact-pins eggup-archive at =0.2.1. The published eggpack 0.2.0 still pins =0.2.0, so a new archive version is required before the leaf can be published.
- crates.io must still report eggup-archive 0.2.1 absent immediately before upload.

## 1. Objective

Publish only eggup-archive 0.2.1 to crates.io, after the existing eggup-core 0.2.1 release and before the dependent eggup-eggpack 0.2.1 release. This makes the qualified archive dependency-refresh candidate available to the new Eggpack exact pin.

The package carries the workspace dependency refresh from Archive M007, principally the ZIP requirement update to 9.6.1. There is no production Rust source change from the 0.2.0 release source, no public API change, and no change to extraction policy. The published 0.2.0 archive is immutable and remains unchanged.

## 2. Why this milestone is ready

- Archive M007 is closed, hosted-qualified in run 44022660351, and its local gate, Rust 1.89 check, advisory check, and archive/Eggpack package checks passed.
- Current crates.io metadata reports eggup-archive 0.2.0 as latest; the exact 0.2.1 version is absent at plan authoring.
- The reviewed workspace source is version 0.2.1. Its archive manifest selects zip 9.6.1 with default features disabled and the existing deflate-flate2 feature.
- Eggpack M008 is closed and the candidate Eggpack manifest pins archive exactly to 0.2.1. The publication is required to satisfy that registry edge.
- The archive package has no normal Eggup dependency. Its eggup-core path edge is dev-only and is removed by packaging.

## 3. Current implementation evidence

- Repository baseline: b5a989a72dc185b30cd8c0cb67b5110ed03acd86, the published Core 0.2.1 source.
- Archive M007 plan/closure: plans/implementation/archive-extraction/007-workspace-dependency-security-refresh.md and plans/closure/archive-extraction/007-status.md.
- crates/eggup-archive/Cargo.toml sets version.workspace = true, selects zip 9.6.1, and retains default-features = false with deflate-flate2.
- Relative to the 0.2.0 train source, the archive candidate changes its manifest, README, and changelog; its production Rust source is unchanged.
- The packaged 0.2.0 archive manifest has no eggup-core edge; the current dev-only path dependency is stripped during packaging.
- The workspace Eggpack candidate exact-pins eggup-archive =0.2.1; no release of curl, eggfetch, or service is needed to satisfy this archive edge.

## 4. Invariants that must not regress

- Archive extraction remains local, bounded, allowlisted, regular-file-only, no-clobber, and independent of authenticity or release-selection policy.
- Do not use ZIP convenience extraction APIs or alter handle-relative authority, member identity, size, digest, cleanup, or ownership behavior.
- SHA-256 remains integrity evidence only; make no authenticity or signature claim.
- eggup-archive has no normal dependency on any Eggup crate; eggup-core remains a dev-dependency only and is not in the published manifest.
- Rust 1.89 remains the package MSRV; no dependency is sourced from Git, a local path, or a patch override in the published package.
- Do not move the existing v0.2.1 tag, which denotes the earlier Core publication source.

## 5. Scope

### In scope

- Publish exactly eggup-archive 0.2.1 from one exact, hosted-qualified source commit.
- Reconfirm registry version availability and publisher authority; qualify the package and a registry-only consumer fixture.
- Record the checksum, source SHA, package contents, and registry readback in the M008 closure.
- Extend the existing v0.2.1 GitHub Release notes after the upload; do not create or move a version tag.
- Reconcile this roadmap, plans/registry.md, the crate changelog, docs/releases.md, architecture release table, and root release changelog after registry readback.

### Explicitly out of scope

- Any production Rust code or public API change, manifest version change, or dependency-boundary change.
- Publishing eggup-acquisition, eggup-eggpack, eggup-curl, eggup-eggfetch, eggup-service, eggup-core, or eggup-transport-footprint under this milestone. Acquisition and Eggpack have separate milestones; curl, eggfetch, and service use compatible published ^0.2 ranges and have no stale exact Eggup pin.
- Moving v0.2.1 or altering any prior tag. Do not yank or overwrite an existing package version.
- Creating release automation or changing CI permissions.

## 6. Required production changes

No production source or API change is required. The package candidate already inherits version 0.2.1 and carries the qualified dependency refresh. Prepare the 0.2.1 changelog entry so it accurately describes the ZIP dependency update, the unchanged API and extraction behavior, and the publication's place in Eggpack's exact-pin train. Do not claim registry publication until it has been read back successfully.

## 7. Ordered work packages

1. Reconfirm the publication source SHA, clean package inputs, registry version absence, non-yanked prior release, and crates.io publisher identity.
2. Register this plan and mark Archive M008 ready in the archive roadmap and registry before release qualification.
3. Run the local gate and verify the dependency graph and package contents; run cargo package and cargo publish --dry-run for eggup-archive 0.2.1.
4. Run hosted CI on the exact source SHA and require all five lanes green.
5. Immediately before upload, obtain confirmation for publishing the exact package/version. Publish eggup-archive 0.2.1 once; if the result is ambiguous, query crates.io before any retry.
6. Read back the registry version, publisher, non-yanked status, dependency list, downloaded archive checksum, and .cargo_vcs_info.json source SHA.
7. Create and run a registry-only archive consumer fixture. Only after M008 is closed and the archive version is visible may the downstream Eggpack publication plan be authored.
8. Update release records and extend the existing v0.2.1 release notes without moving its tag.

## 8. Failure, cancellation, restart, and contention semantics

This is a build/package publication milestone; it does not run the updater or mutate an installation. Cargo resolution, package verification, or CI failure blocks publication and must not be worked around by relaxing archive bounds or dependency features. A crates.io upload is irreversible at the version level: never retry an ambiguous publish until registry readback proves the version absent, never overwrite or yank an existing version, and stop if an unexpected version or publisher appears. If the upload is rejected, correct the local publication claim and record the failure; do not advance to Eggpack. Preserve unrelated local changes.

## 9. Compatibility and migration

The package API and runtime behavior are unchanged from eggup-archive 0.2.0. The package carries the ZIP 9.6.1 dependency refresh qualified by M007; no consumer API migration is required. Existing locks may continue to select 0.2.0 until updated. The archive package contains no normal Eggup dependency, and no claim should say that archive 0.2.1 upgrades Core. The exact consumer edge motivating this release is Eggpack 0.2.1's pin to archive =0.2.1.

## 10. Required tests

- Run the full workspace tests and clippy with all targets/features and the locked graph.
- Preserve archive hostile-entry, traversal, symlink/special-file, duplicate, size, digest, no-clobber, and cleanup coverage on the hosted platform matrix.
- Confirm the archive package manifest has no eggup-core dependency and contains zip 9.6.1 with the intended features.
- Build an external fixture depending only on eggup-archive =0.2.1 after publication; its lockfile must have registry sources only and compile a public archive type.

## 11. Required verification commands

Run in order and record exact results:
1. cargo fmt --all -- --check
2. cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
3. cargo test --workspace --all-targets --all-features --locked
4. cargo doc --workspace --no-deps --locked
5. cargo tree --workspace --locked, with manual review of dependency boundaries
6. ./scripts/check-docs-truth.sh
7. git diff --check
8. cargo package -p eggup-archive --list --locked
9. cargo package -p eggup-archive --locked
10. cargo publish --dry-run -p eggup-archive --locked
11. Hosted CI on the exact source SHA: stable, msrv, service-systemd, macos, and windows-check
12. After upload, cargo check --manifest-path plans/closure/archive-extraction/008-package-fixture/Cargo.toml --locked and cargo tree --manifest-path plans/closure/archive-extraction/008-package-fixture/Cargo.toml --locked
13. After upload, download and verify the .crate checksum and .cargo_vcs_info.json SHA; read back crates.io version and dependencies.
14. cargo publish -p eggup-archive --locked is an externally visible final action and must not run until the exact package/version is confirmed.

## 12. Documentation updates

- crates/eggup-archive/CHANGELOG.md: 0.2.1 dependency refresh and compatibility statement.
- plans/implementation/archive-extraction/008-eggup-archive-0.2.1-publication.md and plans/closure/archive-extraction/008-status.md.
- plans/subsystems/archive-extraction-roadmap.md and plans/registry.md.
- After publication: CHANGELOG.md, docs/releases.md, architecture/tooling-governance.md and overview.md, and the existing GitHub Release v0.2.1 notes.
- Preserve M007's historical statement that it did not authorize publication; M008 is the distinct publication milestone.

## 13. Acceptance criteria

- The exact published version is eggup-archive 0.2.1, and the prior 0.2.0 remains unchanged and not yanked.
- Its published manifest has the intended external dependency graph and no Eggup runtime dependency or path/Git/patch edge.
- Local and all five hosted checks pass on the exact source SHA; registry-only fixture resolves the package and has no non-registry package source.
- Registry checksum and .cargo_vcs_info.json match the locally qualified package/source.
- M008 closure, roadmap, registry, changelog, consumer docs, and release notes agree on what was published; v0.2.1 is not moved.
- No package outside this plan is published.

## 14. Stop conditions

Stop before upload if the version already exists, the package source SHA differs from the hosted-qualified SHA, the archive has a normal Eggup dependency, package verification fails, any required lane fails, the package contains an unexpected path/source edge, the user has not confirmed the exact upload, or the registry readback is inconsistent. Do not proceed to Eggpack if Archive M008 lacks complete registry evidence.

## 15. Closure evidence required

Record the final status; exact plan, roadmap, and source SHA; implementation and documentation commit(s); registry version/publisher/yanked state; downloaded archive SHA-256 and .cargo_vcs_info.json; package file list and normalized dependency manifest; registry-only fixture source and lockfile; local command results; each hosted lane; compatibility, migration, security, invariant, and recovery reviews; any residual findings with severity; and the registry/roadmap/changelog/release-note reconciliation. Report Linux, macOS, and Windows evidence separately and mark any unrun lane not run.

## 16. Handoff notes

This is the first publication in the narrowed exact-pin train after the already-published Core 0.2.1. Acquisition 0.2.1 is the other prerequisite; Eggpack 0.2.1 is the leaf and must wait until both prerequisite versions are on crates.io. Do not republish curl, eggfetch, or service solely for version-number lockstep: their published ^0.2 requirements accept 0.2.1.
