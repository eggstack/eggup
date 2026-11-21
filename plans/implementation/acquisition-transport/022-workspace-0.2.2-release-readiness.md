# Acquisition Transport Milestone 022 — Workspace 0.2.2 release-readiness preparation

Status: conditionally closed 2026-11-21; local source-preparation qualification passed; hosted CI and registry publication qualification remain separate gates.

Repository baseline: 6371a7131f0f5eb42b9d8a151d7872f5a33e7ee1

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md (classification only)

Primary class: infrastructure

## 1. Objective

Prepare the source workspace for the next patch version by changing the shared Eggup package version from 0.2.1 to 0.2.2, updating Eggpack's source-only exact pins, and reconciling release-facing docs with current source and crates.io state. This is release preparation only: do not publish, yank, tag, create or edit a GitHub Release, commit, or push.

## 2. Why this milestone is ready

- M021 is closed and hosted-qualified on its final source commit; its Eggfetch dependency candidate is eggfetch-core 0.4.0 and it deliberately left Eggup package versions unchanged.
- The workspace declares one shared version in Cargo.toml and all eight members inherit it. The user requested a 0.0.1 increment, so the source target is 0.2.2.
- Direct crates.io API reads on 2026-11-21 confirm latest published versions remain Core/Archive/Acquisition/Eggfetch/Eggpack 0.2.1 and Curl/Service 0.2.0. No 0.2.2 release is being made here.
- The workspace-source Eggpack exact-pin cascade is understood: its three local Eggup edges must match the bumped workspace version, while published Eggpack 0.2.1 (and immutable 0.2.0) remain unchanged.

## 3. Current implementation evidence

- Reviewed, clean repository baseline: 6371a7131f0f5eb42b9d8a151d7872f5a33e7ee1.
- Cargo.toml sets workspace.package.version to 0.2.1; every workspace member inherits it.
- crates/eggup-eggpack/Cargo.toml exact-pins Core, Archive, and Acquisition at =0.2.1; every other in-workspace Eggup edge uses a caret requirement.
- The current Eggfetch workspace source uses eggfetch-core 0.4.0, while the published eggup-eggfetch 0.2.1 archive uses 0.3.0.
- Baseline ./scripts/check-docs-truth.sh passes, but audit found current-source claims still at 0.2.1, stale Eggpack publication claims at 0.2.0, and release docs still omitting M021's unpublished 0.4.0 candidate.

## 4. Invariants that must not regress

- All eight workspace package manifests continue to inherit one shared version; only its value advances from 0.2.1 to 0.2.2.
- Keep Eggpack's three source exact pins aligned to =0.2.2 and keep eggpack-manifest =0.1.2.
- Preserve all runtime APIs, dependency boundaries, Rust MSRV, and Eggfetch's four explicit features with default-features = false.
- Published versions, checksums, provenance, tags, and GitHub Releases are immutable historical facts; do not rewrite them as 0.2.2.
- No package publication, yank, tag creation/move, GitHub Release edit, commit, or push is in scope.

## 5. Scope

### In scope

- Bump the workspace shared version and refresh Cargo.lock's local package records.
- Update Eggpack's source exact pins.
- Correct source-version and latest-published claims in the root index, changelogs, docs, architecture, and release-workflow skill.
- Correct the stale published Eggpack 0.2.0 claims in the source-facing README/architecture surfaces to 0.2.1, while retaining all immutable 0.2.0 history.
- Register M022 as active and record evidence honestly.

### Explicitly out of scope

- Publishing any crate or deciding the final package set/order for a later release milestone.
- Editing historical plans/closures to retrofit the 0.2.2 source state.
- Changing Rust production source, public APIs, behavior, external dependencies, or the Eggfetch feature set.
- Creating/moving tags, changing GitHub Releases, or pushing/committing this candidate.

## 6. Required production changes

No Rust production-source change is expected. Set workspace.package.version to 0.2.2, update the three workspace Eggpack exact pins to =0.2.2, and resolve only the corresponding workspace package-version changes in Cargo.lock. Stop if Cargo resolution requires an unrelated external dependency update.

## 7. Ordered work packages

1. Register this bounded M022 plan as active in the acquisition roadmap and registry.
2. Bump Cargo.toml and the three Eggpack pins; refresh and review Cargo.lock.
3. Reconcile AGENTS.md, root and crate changelogs, Eggpack README, docs/releases.md, the current-source architecture claims, and release-workflow skill. Preserve published package history and tag/release statements.
4. Run the docs-truth gate, full local gate, manual dependency-tree review, and a registry-based package check for the Eggfetch candidate. Confirm the Eggpack exact-pin package check is blocked only by the not-yet-published 0.2.2 dependency set; do not work around this with path or git dependencies.
5. Record local evidence and keep hosted qualification/publication explicitly separate; neither is authorized by this plan.

## 8. Failure, cancellation, restart, and contention semantics

This is a metadata/documentation change and performs no live installation or registry mutation. If dependency resolution changes unrelated versions, the documentation-truth gate fails, a current package-status fact conflicts with crates.io, or a package check fails for a reason other than the expected Eggpack exact-pin availability, stop and investigate. Do not bypass registry resolution or publish to clear a package failure.

## 9. Compatibility and migration

The requested source bump is 0.2.1 → 0.2.2, a patch increment; it does not itself claim a public API or runtime change. All eight workspace members share the source version, including the publish = false fixture crate. Published Eggup crates remain at their independently verified registry versions until separate publication milestones. The source Eggpack pins change to =0.2.2; the published Eggpack 0.2.1 and 0.2.0 artifacts retain their existing exact pins. The Eggfetch source candidate's eggfetch-core 0.4.0 dependency is not yet in the published eggup-eggfetch 0.2.1 artifact.

## 10. Required tests

- Documentation truth and the complete local verification gate.
- Workspace all-target/all-feature compile/tests and manual dependency-boundary review through the gate's cargo tree output.
- cargo package -p eggup-eggfetch --locked --allow-dirty to check the source adapter against registry dependencies where available.
- cargo package -p eggup-eggpack --locked --allow-dirty may remain blocked until its exact-pinned Eggup 0.2.2 dependencies are on crates.io; record that registry prerequisite and do not infer package readiness from the workspace path graph.
- No production Rust code or runtime-behavior test changes are in scope. The acquisition publication-coupling guard may distinguish a dependent's matching, explicitly recorded exact-pin candidate in its `Unreleased` section from a registry-qualified package; it must keep rejecting unrecorded or mismatched pins. The full workspace test suite remains required as a regression check.

## 11. Required verification commands

- ./scripts/check-docs-truth.sh
- ./scripts/check-local.sh
- cargo package -p eggup-eggfetch --locked --allow-dirty
- cargo package -p eggup-eggpack --locked --allow-dirty (record registry-only exact-pin blocker if dependencies remain unpublished)
- cargo tree --workspace --locked (review output manually)
- git diff --check
- Direct crates.io API read for current published versions before finalizing release documentation.

The local gate's cargo tree exits 0 on any resolvable graph; boundary compliance requires human review. Hosted CI is not run from an unpushed working tree.

## 12. Documentation updates

Update the workspace-source version in architecture/tooling-governance.md §2.2/§2.4/§8, architecture/overview.md, architecture/acquisition.md, architecture/eggpack-adapter.md, architecture/transport-footprint.md, AGENTS.md, and docs/releases.md. Keep the user-facing release snapshot's latest-published table unchanged after rechecking crates.io, but update its checked date and add the M021 0.4.0 unpublished source note. Update crates/eggup-eggpack/README.md and its Unreleased changelog for the =0.2.2 source pins; clarify root and eggfetch Unreleased changelogs. Update .opencode/skills/release-workflow/SKILL.md as a summary of the new exact-pin asymmetry. Do not rewrite historical publication plans or closures.

## 13. Acceptance criteria

- Cargo.toml declares workspace version 0.2.2; all eight members inherit it; Cargo.lock resolves all workspace members at 0.2.2.
- Eggpack source pins Core, Archive, and Acquisition at =0.2.2 and retains eggpack-manifest =0.1.2.
- Source-vs-published claims agree across docs, architecture, README, changelogs, and skill; crates.io published-version facts are verified as of 2026-11-21.
- Documentation-truth and the full local gate pass; the cargo tree is manually reviewed.
- No registry artifact, tag, GitHub Release, commit, or push is changed.

## 14. Stop conditions

Stop if the shared version is not 0.2.1 at the reviewed baseline, if the user-requested increment would collide with a published version, if crates.io state differs from the verified snapshot, if the source exact-pin invariant cannot be preserved, if unrelated Cargo.lock dependency versions move, or if any required local gate fails. Do not publish or push.

## 15. Closure evidence required

Record the exact reviewed SHA, manifest/lockfile diff, changed documentation surfaces, crates.io version snapshot, docs-truth and local-gate results, package-check outcomes, and manual dependency-tree review. State hosted CI as not run for the unpushed M022 worktree; the M021 run does not qualify this candidate. State that publication, tags, and GitHub Releases were not changed. Keep the milestone active if hosted qualification is required before closure.

## 16. Handoff notes

This plan prepares source version 0.2.2 only. A subsequent publication has to be its own authorized plan, decide the release set, package in dependency order, qualify a registry-only fixture, and keep eggup-service publication behind explicit maintainer authorization. Eggpack's exact pins mean its package/registry-only qualification must follow the availability of the pinned 0.2.2 dependencies. The 0.2.2 source candidate is not a crates.io release.
