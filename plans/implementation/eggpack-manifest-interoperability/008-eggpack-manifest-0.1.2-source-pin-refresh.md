# Eggpack Manifest Interoperability Milestone 008 — eggpack-manifest 0.1.2 source-pin refresh

Status: active

Repository baseline: 47b202b5f35fe8dd05d5bd9c2b456b08688f3b4d

Source roadmap:
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md

Primary class: polish

## 1. Objective

Advance the unpublished workspace-source eggup-eggpack adapter's exact registry
dependency from eggpack-manifest 0.1.1 to the latest published compatible
release, 0.1.2, and qualify the resulting source candidate. Update only the
matching Cargo.lock entry, the exact-pin test expectation, and current
documentation/planning records.

This is a separate Eggpack M008 from Verified Update Core M016. M016 remains
Core-only: it does not own or authorize this dependency change, and its
publication scope is not widened. M008 is stacked on the M016 candidate branch
for review isolation. It does not publish, tag, create a release, or merge any
package or PR.

## 2. Why this milestone is ready

- The crates.io API reports eggpack-manifest 0.1.2 published on 2026-11-18,
  non-yanked, Rust 1.89, checksum
  4bbbe8d12026adf7c8e8ddaecbed0c6ac403a5621d4dd1ac9949f8daf8149a09.
  cargo search and cargo info eggpack-manifest@0.1.2 both identify it as the
  latest release at preflight.
- The published 0.1.2 src/lib.rs production section (before #[cfg(test)]) is
  byte-for-byte identical to 0.1.1. Its upstream test-only changes do not
  change the consumer schema, API, parser, or runtime behavior.
- The M016 candidate currently pins 0.1.1 in the adapter manifest, lockfile,
  and interoperability test. The adapter is the only workspace crate allowed
  to depend on this producer schema.
- The published eggup-eggpack 0.2.0 artifact is immutable and remains outside
  this change. No separate Eggup package publication is authorized.

## 3. Current implementation evidence

- The reviewed baseline is the exact M016 candidate commit above; its branch
  and PR #3 remain unchanged by this milestone.
- Workspace-source eggup-eggpack is 0.2.1 and uses exact local Eggup pins
  =0.2.1; its external producer pin is =0.1.1.
- Cargo.lock resolves eggpack-manifest 0.1.1 from crates.io with checksum
  352c3e327159615b763298e911e239f1decc4a2c9e28c28be8ddd28e50515e37.
- tests/interoperability.rs asserts the exact external registry pin. The
  production adapter source and public API are not in scope.
- M016's initial source baseline used =0.1.1; this M008 pin advance is a
  separately planned adapter dependency refresh, not a change to M016's Core
  publication requirements.

## 4. Invariants that must not regress

- Only eggup-eggpack may depend on eggpack-manifest; eggup-core,
  eggup-acquisition, and eggup-service remain Eggpack-independent.
- The producer dependency stays exact-pinned and registry-only; introduce no
  path, Git, branch, tag, or VCS source edge.
- The new lock resolution is eggpack-manifest 0.1.2; unrelated registry
  dependency versions remain unchanged.
- No production Rust source, public API, adapter behavior, fixture semantics,
  crate version, runtime dependency boundary, or publication status changes.
- Published eggup-eggpack 0.2.0 and its =0.1.0 producer pin remain immutable;
  this source-tree refresh is not a republication.
- M016 stays Core-only. No cargo publish, tag, GitHub Release, or merge is
  part of M008.

## 5. Scope

### In scope

- Update the workspace-source exact registry pin to =0.1.2 and regenerate
  only the corresponding Cargo.lock package entry/checksum.
- Update the adapter's exact-pin test constant and explanatory comment.
- Update current-facing dependency claims, the adapter's Unreleased changelog,
  the M016 plan's baseline note, and Eggpack roadmap/registry status.
- Run adapter-focused checks, documentation-truth checks, the full local gate,
  and all five hosted CI lanes on this branch.
- Open a separate M008 PR against the M016 candidate branch so the two reviews
  remain independently scoped.

### Explicitly out of scope

- Any production Rust source, schema, parser, API, behavior, or fixture-content
  change.
- Any change to eggup-core source, API, dependency graph, package contents, or
  M016 publication scope.
- Publishing eggup-eggpack, eggpack-manifest, or any other crate; changing
  package versions; creating tags or releases; merging either PR.
- Refreshing any other dependency or permitting broad/unrelated lockfile
  resolution changes.
- Editing immutable historical plans or closure evidence to rewrite what was
  true at their recorded baselines.

## 6. Required production changes

1. Set crates/eggup-eggpack/Cargo.toml to eggpack-manifest = "=0.1.2".
2. Update the exact-pin constant in tests/interoperability.rs and keep the
   assertion registry-only.
3. Regenerate the lockfile with the exact 0.1.2 checksum. Stop if the lockfile
   changes any unrelated registry package.
4. Update the adapter README/changelog and current architecture/index statements
   to identify the source candidate's =0.1.2 edge. Preserve the published
   eggup-eggpack 0.2.0 history at =0.1.0.
5. Reconcile the active M016 plan to say its own baseline used =0.1.1 and that
   M008 owns the separate update; do not broaden M016's non-goals or authority.

## 7. Ordered work packages

1. Freeze evidence: reconfirm crates.io latest/yanked/MSRV metadata, compare the
   published 0.1.1/0.1.2 production source sections, and record the baseline SHA.
2. Register M008: add this plan and mark it active in the Eggpack roadmap and
   plans/registry.md before the dependency edit.
3. Refresh the edge: change the manifest pin and test constant, update the
   lockfile narrowly, then inspect git diff and cargo tree.
4. Reconcile docs: update current-facing docs, the adapter Unreleased note, and
   the M016 baseline note; leave historical closures unchanged.
5. Qualify: run the exact commands in §11, inspect the dependency tree, commit
   and push the separate branch, open the stacked PR, and wait for all hosted
   CI lanes.
6. Close: record command results and each hosted lane in the M008 closure, mark
   the roadmap/registry entry closed, and leave both PRs unmerged.

## 8. Failure, cancellation, restart, and contention semantics

This is a source-manifest and documentation change. It does not mutate an
installed application or invoke a release upload. Cargo may update Cargo.lock
and build artifacts under target/; the lockfile change is intentional only for
eggpack-manifest. If a command fails, preserve its output and the worktree; do
not restore or overwrite unrelated changes. If registry metadata or the latest
version changes before qualification, repeat preflight and stop for review
rather than silently chasing a newer dependency. If any required check fails
outside this bounded pin/test/docs scope, stop and record it instead of
changing production behavior under M008.

## 9. Compatibility and migration

The 0.1.2 production source is byte-for-byte identical to 0.1.1, so the
adapter's compiled API and runtime behavior do not change and consumers need no
migration. The test-only upstream source changes are not part of this crate's
runtime surface. The published eggup-eggpack 0.2.0 artifact still resolves its
own immutable eggpack-manifest =0.1.0 dependency; M008 changes only the
unpublished workspace source used by the candidate branch. Any future
eggup-eggpack publication requires its own explicit plan and authorization.

## 10. Required tests

- The exact-pin test passes with =0.1.2 and rejects VCS/path edges.
- Eggpack projection, archive-handoff, caller-destination, and digest-reuse
  integration tests pass against the 0.1.2 registry package.
- Clippy and the complete workspace tests/docs pass with the refreshed lockfile.
- Dependency-tree inspection proves only the adapter has a producer-schema
  edge and that the resolved version is 0.1.2.
- Package publication and tag tests are not applicable and must not be run.

## 11. Required verification commands

Run in this order and record exact outcomes:

1. cargo test -p eggup-eggpack --locked
2. cargo clippy -p eggup-eggpack --all-targets --locked -- -D warnings
3. cargo tree --workspace --locked -i eggpack-manifest — manually review the
   exact version and dependency boundary.
4. ./scripts/check-docs-truth.sh
5. git diff --check
6. ./scripts/check-local.sh
7. Hosted CI on the exact pushed M008 head: stable, msrv, service-systemd,
   macos, and windows-check, all green.

## 12. Documentation updates

- crates/eggup-eggpack/README.md and CHANGELOG.md (Unreleased).
- AGENTS.md, architecture/overview.md,
  architecture/tooling-governance.md, and architecture/eggpack-adapter.md.
- The M016 implementation plan's baseline reference, without changing its
  Core-only scope.
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md and
  plans/registry.md, including the M008 plan/closure link.
- Check docs/ for affected current claims; it remains non-normative and should
  not be changed unless a matching claim actually exists.

## 13. Acceptance criteria

- The M008 source manifest and lockfile use exactly eggpack-manifest 0.1.2
  from crates.io; no unrelated lock versions or source edges change.
- Eggup production source/API/behavior is unchanged; the adapter exact-pin test
  and all required local checks pass.
- The full hosted CI matrix is green on the pushed M008 head.
- Plan, closure, roadmap, registry, changelog, and current-facing docs agree on
  the 0.1.2 workspace pin and the unchanged 0.2.0 published artifact.
- M008 is a separate PR based on the M016 candidate branch; PR #3's branch,
  diff, and Core-only publication scope are unchanged.
- No crate is published, tagged, released, or merged.

## 14. Stop conditions

Stop if 0.1.2 is yanked, no longer latest before the pin is finalized, or has a
Rust requirement above 1.89; if its production source differs from the reviewed
0.1.1 production source; if adapter tests reveal an API/behavior incompatibility;
if the lockfile changes unrelated packages; if a dependency boundary is
violated; or if a required local/hosted check fails and cannot be corrected
without out-of-scope source changes. Do not publish, tag, release, or merge as a
way to resolve a stop condition.

## 15. Closure evidence required

The M008 closure must record: source plan and roadmap; reviewed baseline; the
crates.io metadata, checksum, yanked/MSRV state, and source-comparison result;
implementation commit and PR URL/head/base; manifest, lockfile, and tree
boundary evidence; exact command outputs/results; full local-gate result; each
hosted CI lane separately; compatibility/security review; documentation and
registry updates; and an explicit statement that no package was published,
tagged, released, or merged.

## 16. Handoff notes

M008 is a narrow workspace-source dependency refresh, not a new adapter release.
The separate PR is stacked on release/0.2.1-preparation; it depends on the M016
branch and must be retargeted or rebased after that base is integrated. Keep
Core M016's upload authorization and publication gates independent. The
published eggup-eggpack 0.2.0 package and eggpack-manifest 0.1.0 edge are not
altered by this source update.
