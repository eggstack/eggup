# Eggpack Manifest Interoperability M010 — eggup-eggpack 0.2.2 publication

Status: closed 2026-11-22; published and registry-verified from `c9a7ac0af7ea3f0a090d19fe1fffa817235f0081`; post-publication hosted-qualified; see `plans/closure/eggpack-manifest-interoperability/010-status.md`.
Repository baseline: 3be567e9839b555d7d9d5e6e43f14de4a13c901d
Source roadmap: plans/subsystems/eggpack-manifest-interoperability-roadmap.md
Affected roadmaps: verified-update-core, archive-extraction, acquisition-transport.
Long-term requirements: plans/000-long-term-specification.md; plans/001-terminology-and-domain-model.md
Applicable ADRs: ADR-0003, ADR-0004, ADR-0005.
Primary class: infrastructure.

Hard dependencies, all registry-visible and unyanked before package verification:
- eggup-core 0.2.2 (Core M022)
- eggup-archive 0.2.2 (Archive M009)
- eggup-acquisition 0.2.2 (Acquisition M023)
- eggpack-manifest 0.1.2 (separate producer schema crate)

## 1. Objective
Publish only eggup-eggpack 0.2.2 after its three exact Eggup dependencies at =0.2.2 are available. The schema dependency remains exactly eggpack-manifest 0.1.2. This release delivers the prepared consumer-side pin update; it does not republish Eggpack producer crates.

## 2. Why this milestone is ready
Eggpack M009 and schema pin M008 are closed. M022 prepared workspace 0.2.2 and exact pins; its earlier registry package failure correctly enforced the absent-dependency boundary. Before M010 packaging, exact-version crates.io reads confirmed Eggpack 0.2.2 absent, 0.2.1 unyanked, and Core/Archive/Acquisition 0.2.2 plus eggpack-manifest 0.1.2 present and unyanked. The authorized publication then completed once from `c9a7ac0af7ea3f0a090d19fe1fffa817235f0081`; readback, artifact comparison, fixture, and post-publication CI passed. No tags/releases/PRs were in scope.

## 3. Current implementation evidence
Baseline was 3be567e9839b555d7d9d5e6e43f14de4a13c901d. Eggpack production Rust source remains unchanged from M009 source ccb2bfcbbe7645541d43bb4f5fcb801339c319ed. The 0.2.2 package built from c9a7ac0af7ea3f0a090d19fe1fffa817235f0081 carries exact pins to Core, Archive, and Acquisition 0.2.2 plus eggpack-manifest 0.1.2. Those pins are now in the published release; the earlier M022 registry failure correctly enforced dependency visibility and was resolved by Core M022, Archive M009, and Acquisition M023.

## 4. Invariants that must not regress
Eggpack remains a consumer adapter: no producer authority, release selection, authenticity, install-root, or ownership policy. Published 0.2.2 exact-pins all three Eggup dependencies at =0.2.2 and eggpack-manifest at =0.1.2. The immutable 0.2.1 and 0.2.0 artifacts retain their earlier pins. Package resolution must use registry dependencies with no path/git/patch edges.

## 5. Scope
In scope: retain Unreleased exact-pin evidence; package, dry-run, publish, and verify only Eggpack 0.2.2 after all dependencies close. Out of scope: producer-side crates/schema republish; production Rust/API changes; other crate publications; tags/releases/PRs/yanks/overwrites/release CI.

## 6. Required production changes
None. Do not change src, package manifests, or dependency policy. After publication, the repository README and changelog plus plan/closure and release records were reconciled; these documentation edits do not alter the immutable 0.2.2 artifact.

## 7. Ordered work packages
1. Verify closed Core M022, Archive M009, and Acquisition M023 publication readbacks plus schema 0.1.2. 2. Recheck Eggpack 0.2.2 absent. 3. Run local gate, exact-SHA five-lane CI, strict package and dry-run. 4. Review normalized manifest/inventory. 5. Publish once and verify metadata, archive checksum, and VCS SHA. 6. Run registry-only fixture and same-SHA post-publication CI.

## 8. Failure, cancellation, restart, and contention semantics
If any exact dependency is absent, yanked, mismatched, or unverifiable, stop before Eggpack packaging. Any failed check blocks upload. On ambiguous upload, query crates.io before any retry. If artifact/checksum differs, stop; never yank/overwrite and do not claim rollback. No user installation state is touched.

## 9. Compatibility and migration
No Eggpack Rust API/runtime change. Its dependency set advances from exact 0.2.1 to exact 0.2.2 for Core/Archive/Acquisition; schema remains 0.1.2. No consumer migration beyond selecting Eggpack 0.2.2. Older Eggpack artifacts retain historical pins.

## 10. Required tests
Full local gate and Eggpack interop tests; strict cargo package and publish dry-run after dependencies; independent registry-only consumer with exact pins and no path/git/patch source; five hosted lanes at the source SHA and same-SHA post-publication rerun.

## 11. Required verification commands
./scripts/check-local.sh
 git diff --check
 cargo tree --workspace --locked (manual boundary review)
 cargo test -p eggup-eggpack --locked
 cargo package -p eggup-eggpack --locked
 cargo publish -p eggup-eggpack --locked --dry-run
 cargo check --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml --locked
 cargo tree --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml --locked
 cargo metadata --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml --locked --format-version 1

## 12. Documentation updates
After registry readback, moved the exact-pin and prerequisite notes under the Eggpack 0.2.2 changelog entry. Reconciled the crate README/changelog, root changelog, docs/releases.md, registry, Eggpack roadmap, architecture/eggpack-adapter.md, architecture/overview.md, architecture/tooling-governance.md, AGENTS.md, and publication-status skills. Recorded the immutable archive README/changelog residue. M009 and M022 history is preserved. No tag/release/PR edits.

## 13. Acceptance criteria
All three exact dependencies and schema 0.1.2 are verified available before package; normalized Eggpack manifest carries exactly the expected pins; one 0.2.2 upload matches source SHA/checksum/provenance; independent registry-only proof passes; records agree and existing tags/releases remain unchanged.

## 14. Stop conditions
Stop if any dependency is unavailable/yanked, Eggpack 0.2.2 exists, guard fails, package or hosted checks fail, source/worktree changes after qualification, or registry readback is ambiguous. Never publish Eggpack before the three dependency closures.

## 15. Closure evidence required
Record source/baseline SHAs, commit, per-lane hosted results, commands, package inventory and normalized manifest, immediate dependency registry reads, publisher/time/checksum, downloaded bytes/VCS provenance, fixture lock/tree/metadata/results, compatibility/security review, and registry/roadmap updates.

## 16. Handoff notes
This leaf closed the registry packaging blocker left by M022 after the three exact-pinned Eggup 0.2.2 dependencies were published and verified. Core M022, Archive M009, and Acquisition M023 are closed, and Eggpack M010 published the exact-pin consumer adapter. Eggpack's producer-side 0.1.2 release and `eggpack-manifest 0.1.2` remain separate and unchanged.
