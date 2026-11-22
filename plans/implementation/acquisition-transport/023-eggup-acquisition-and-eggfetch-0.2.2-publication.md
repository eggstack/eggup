# Acquisition Transport M023 — eggup-acquisition and eggup-eggfetch 0.2.2 publications

Status: active; publication authorized 2026-11-21, uploads not yet performed.
Repository baseline: 3be567e9839b555d7d9d5e6e43f14de4a13c901d
Source roadmap: plans/subsystems/acquisition-transport-roadmap.md
Affected roadmaps: archive-extraction and eggpack-manifest-interoperability.
Long-term requirements: plans/000-long-term-specification.md; plans/001-terminology-and-domain-model.md
Applicable ADRs: ADR-0001, ADR-0003, ADR-0004 (classification only).
Primary class: infrastructure.

## 1. Objective
Publish eggup-acquisition 0.2.2 first as the zero-dependency seam required by Eggpack's =0.2.2 exact pin, then publish eggup-eggfetch 0.2.2 to deliver the already-qualified eggfetch-core 0.4.0 dependency update. This is a selective two-package milestone, not a seven-crate train.

## 2. Why this milestone is ready
Acquisition M019 and Eggfetch M020 are closed. M021's Eggfetch 0.4.0 workspace dependency update passed run 44879536834. M022 source preparation passed runs 44925918931 and 44926808222. The 2026-11-21 registry reads show Acquisition and Eggfetch 0.2.1 latest and no 0.2.2; recheck both before upload. Eggpack requires Acquisition 0.2.2. The current user explicitly authorized publication. Core M022 and Archive M009 are closed; Archive 0.2.2 has a verified registry readback and satisfies the Archive prerequisite. No tags/releases/PRs are in scope.

## 3. Current implementation evidence
Baseline is 3be567e9839b555d7d9d5e6e43f14de4a13c901d. Acquisition src is unchanged from 0.2.1 and its dependency table remains empty. Eggfetch carries M021's 0.4.0 dependency and diagnostic update; two source lines changed, with no public API/policy change and the same four explicit features. The M022 coupling guard requires Eggpack's exact pin to remain in its Unreleased changelog until its publication. Published Curl 0.2.0 and Service 0.2.0 use caret requirements and are not republished.

## 4. Invariants that must not regress
Acquisition remains zero-dependency and transport-neutral. Eggfetch must not depend on Curl; preserve default-features=false and the four features http1, tls-rustls, tls-native-roots, proxy. No public API, policy, or new dependency change. Existing versions are immutable.

## 5. Scope
In scope: prepare Acquisition's Unreleased pin-only note; publish Acquisition then Eggfetch 0.2.2; verify packages, registry metadata, checksums, provenance, and a registry-only graph including older Curl/Service caret consumers. Out of scope: extra production changes; Curl, Service, Eggpack, Core, Archive, transport-footprint publications; tags/releases/PRs/yanks/CI publication.

## 6. Required production changes
None beyond M021's already-qualified Eggfetch dependency/diagnostic update. No new production edits. Only changelogs and planning/closure evidence change.

## 7. Ordered work packages
1. Recheck both 0.2.2 versions absent, prior releases unyanked, and eggfetch-core 0.4.0 available. 2. Keep Eggpack's pin evidence in Unreleased; add Acquisition's Unreleased note and fixture. 3. Run full local gate and five-lane exact-SHA hosted CI. 4. Package/dry-run/publish Acquisition, then verify. 5. Only after Acquisition is visible, package/dry-run/publish Eggfetch and verify. 6. Run registry-only compatibility fixture with both legacy caret consumers.

## 8. Failure, cancellation, restart, and contention semantics
Any failed gate blocks upload. If Acquisition publishes but Eggfetch fails, record partial state; never yank Acquisition or claim full completion. Query crates.io after ambiguous responses and do not retry while a version may exist. Any checksum/provenance mismatch stops later releases. No rollback is claimed.

## 9. Compatibility and migration
Acquisition 0.2.2 has no source/API/runtime/dependency change. Eggfetch 0.2.2 carries the M021 implementation dependency update from eggfetch-core 0.3.0 to 0.4.0 while preserving API, policy, and features. No migration is required. Curl/Service 0.2.0 caret edges should resolve new compatible 0.2.x dependencies; prove this in the fixture.

## 10. Required tests
cargo test -p eggup-acquisition --locked; cargo test -p eggup-eggfetch --locked; full local gate; package/dry-run in order; registry-only resolution of Acquisition/Eggfetch 0.2.2, eggfetch-core 0.4.0, and Curl/Service 0.2.0; all five hosted lanes and same-SHA post-publication rerun.

## 11. Required verification commands
./scripts/check-local.sh
 git diff --check
 cargo tree --workspace --locked (manual boundary review)
 cargo package -p eggup-acquisition --locked
 cargo publish -p eggup-acquisition --locked --dry-run
 cargo package -p eggup-eggfetch --locked
 cargo publish -p eggup-eggfetch --locked --dry-run
 cargo check --manifest-path plans/closure/acquisition-transport/023-package-fixture/Cargo.toml --locked
 cargo tree --manifest-path plans/closure/acquisition-transport/023-package-fixture/Cargo.toml --locked
 cargo metadata --manifest-path plans/closure/acquisition-transport/023-package-fixture/Cargo.toml --locked --format-version 1

## 12. Documentation updates
Update active plan, registry, and affected roadmaps before upload. After readback move notes under 0.2.2, update root changelog, registry, Acquisition roadmap, architecture/tooling-governance.md §8, and architecture/overview.md. Preserve M019-M022 historical records. No tags/releases/PRs.

## 13. Acceptance criteria
Each package publishes once from exact hosted-qualified SHA in dependency order; Acquisition's published dependency set is empty; Eggfetch has only expected registry edges and four features; checksum/provenance match; fixture has no path/git/patch source and proves legacy caret compatibility; records agree.

## 14. Stop conditions
Stop if any version exists, eggfetch-core 0.4.0 is unavailable, pins/features/graph differ, a check/lane fails, the worktree changes, or upload verification is unclear. Never publish Eggfetch before Acquisition is visible. A partial seam publication is recorded as partial.

## 15. Closure evidence required
Record exact SHAs/commits, each hosted lane, all package commands/inventories/manifests, registry publisher/time/checksum and VCS provenance, fixture lock/tree/metadata/results, compatibility/security review, order, and registry/roadmap updates. Unrun platforms are not run.

## 16. Handoff notes
Core M022 and Archive M009 are closed and precede this milestone. Eggpack M010 is the exact-pin leaf and may proceed only after this milestone is closed. Do not publish Curl or Service for test/comment-only deltas.
