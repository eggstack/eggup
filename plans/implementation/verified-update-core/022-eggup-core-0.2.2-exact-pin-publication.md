# Verified Update Core M022 — eggup-core 0.2.2 exact-pin publication

Status: closed 2026-11-22; `eggup-core 0.2.2` published, registry-verified, and hosted-qualified
from `cdfe0bf0452d64c50da00a9e21c62ad2a0140bff`; see
`plans/closure/verified-update-core/022-status.md`.
Repository baseline: 3be567e9839b555d7d9d5e6e43f14de4a13c901d
Source roadmap: plans/subsystems/verified-update-core-roadmap.md
Affected roadmap: plans/subsystems/eggpack-manifest-interoperability-roadmap.md
Long-term requirements: plans/000-long-term-specification.md; plans/001-terminology-and-domain-model.md
Applicable ADRs: ADR-0001, ADR-0002, ADR-0003.
Primary class: infrastructure.

## 1. Objective
Publish only eggup-core 0.2.2 as the registry prerequisite for the Eggpack 0.2.2 candidate, whose Core edge is exact-pinned to =0.2.2. Core production source and dependency surface are unchanged from 0.2.1; this is an exact-pin package-version publication, not a new Core capability.

## 2. Why this milestone is ready
Core M016 and M021 are closed. Acquisition M022 source preparation is closed and hosted-qualified in runs 44925918931 and 44926808222. The 2026-11-21 crates.io API read reports Core 0.2.1 latest and no 0.2.2; recheck before upload. Eggpack M010 requires Core 0.2.2. The user explicitly authorized the new release; tags, GitHub Releases, and PRs remain out of scope.

## 3. Current implementation evidence
Baseline is the clean M022 branch head above. The Core source diff from publication SHA b5a989a72dc185b30cd8c0cb67b5110ed03acd86 is empty. The manifest still uses sha2 and Windows-only self-replace. M016 established the package must include docs/domain.md, docs/verification.md, and docs/transaction.md.

## 4. Invariants that must not regress
Core remains policy-neutral and transport/service-manager-free; the non-Windows runtime graph remains sha2 only. Integrity is not authenticity. Preserve the three contract docs. Existing versions and tags are immutable; no yank or overwrite.

## 5. Scope
In scope: prepare an Unreleased exact-pin note, package, publish, and verify only eggup-core 0.2.2 from one clean hosted-qualified SHA. Explicitly out of scope: production/API/runtime/dependency changes; other crate publications; tags, GitHub Releases, PRs, yanks, or release CI.

## 6. Required production changes
None. Only the Core Unreleased changelog note and plan/closure evidence change. Do not change src, dependencies, or Cargo.lock.

## 7. Ordered work packages
1. Recheck 0.2.2 absent and prior Core versions unyanked. 2. Prepare the changelog note and registry-only fixture. 3. Run local gate, package checks, and five hosted CI lanes on the exact candidate SHA; review cargo tree. 4. Inspect package inventory and dry-run. 5. Publish once and verify registry record, archive checksum, and VCS source SHA. 6. Pass the registry-only fixture before dependent publication.

## 8. Failure, cancellation, restart, and contention semantics
Any failed check stops before upload. On ambiguous upload response, query crates.io and do not retry while 0.2.2 may exist. A readback, checksum, or provenance mismatch stops dependent publications. Crates.io publication is immutable; no rollback or yank is claimed. No installation state is touched.

## 9. Compatibility and migration
No production/API/dependency change; no consumer migration. Existing Core 0.2.1 remains immutable. This version makes Eggpack's exact 0.2.2 edge resolvable.

## 10. Required tests
cargo test -p eggup-core --locked; full local gate; cargo package and publish dry-run; package inventory; independent registry-only fixture; all five hosted lanes and a same-SHA post-publication rerun after the train.

## 11. Required verification commands
./scripts/check-local.sh
 git diff --check
 cargo tree --workspace --locked (manual boundary review)
 cargo package -p eggup-core --locked
 cargo publish -p eggup-core --locked --dry-run
 cargo check --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml --locked
 cargo tree --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml --locked
 cargo metadata --manifest-path plans/closure/verified-update-core/022-package-fixture/Cargo.toml --locked --format-version 1

## 12. Documentation updates
Before upload, update the active plan/registry/roadmap. After registry readback, move the release note to 0.2.2 and update root CHANGELOG, registry, Core roadmap, architecture/tooling-governance.md §8, and architecture/overview.md. Preserve M016 and M022 history; do not edit tags/releases or CI.

## 13. Acceptance criteria
Exact source SHA passes local and all five hosted lanes; package contains all three contract docs and expected manifest; one 0.2.2 upload is registry-visible, checksum/provenance verified, and resolves from a registry-only consumer; records agree and do not claim the full Eggpack release is ready yet.

## 14. Stop conditions
Stop if 0.2.2 exists, source/worktree changes after qualification, any check fails, contract docs are missing, an unexpected dependency appears, or upload/readback is unclear. Do not publish dependents until this milestone closes.

## 15. Closure evidence required
Record baseline/publication SHAs, commits, per-lane hosted results, exact commands, package inventory and normalized manifest, registry publisher/time/checksum, archive hash and VCS provenance, fixture manifest/lock/tree/metadata/results, compatibility/security review, and registry/roadmap changes. Unrun evidence is marked not run.

## 16. Handoff notes
Next: Archive M009, then Acquisition M023. Eggpack M010 is last among exact-pin consumers. Do not publish Curl or Service solely because their workspace source version is 0.2.2.
