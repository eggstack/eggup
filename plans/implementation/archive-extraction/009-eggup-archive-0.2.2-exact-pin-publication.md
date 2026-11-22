# Archive Extraction M009 — eggup-archive 0.2.2 exact-pin publication

Status: active; Core M022 is published and registry-verified; Archive M009 is preparing its exact-SHA candidate.
Repository baseline: 3be567e9839b555d7d9d5e6e43f14de4a13c901d
Source roadmap: plans/subsystems/archive-extraction-roadmap.md
Affected roadmap: plans/subsystems/eggpack-manifest-interoperability-roadmap.md
Long-term requirements: plans/000-long-term-specification.md; plans/001-terminology-and-domain-model.md
Applicable ADRs: ADR-0002, ADR-0004, ADR-0005.
Primary class: infrastructure.

## 1. Objective
Publish only eggup-archive 0.2.2 as the registry prerequisite for Eggpack 0.2.2, whose Archive edge is exact-pinned to =0.2.2. Production source and runtime dependency surface are unchanged from Archive 0.2.1.

## 2. Why this milestone is ready
Archive M008 and Core M022 are closed. Core 0.2.2 is published and unyanked; its closure commit `abdae619a4b00ac0f20ee9b7918c30f1fbca14c3` passed all five hosted lanes in run `44982351142`. The exact Archive 0.2.2 endpoint returned 404 on 2026-11-22, and Archive 0.2.1 is unyanked; recheck immediately before upload. The user authorized this selective publication. No tag, GitHub Release, or PR action is in scope.

## 3. Current implementation evidence
Baseline is 3be567e9839b555d7d9d5e6e43f14de4a13c901d. Archive src is unchanged from M008 source SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c. The manifest retains five external normal dependencies; eggup-core is dev-only and must not appear in the normalized published dependency manifest.

## 4. Invariants that must not regress
Extraction stays local, bounded, allowlisted, regular-file-only, no-clobber, and independent of authenticity or release-selection policy. Keep SHA-256 as integrity evidence only. Preserve the five normal dependencies and no Eggup runtime edge. Existing versions are immutable.

## 5. Scope
In scope: Unreleased exact-pin note, package, publish, and registry verification for only eggup-archive 0.2.2. Out of scope: Rust/API/extraction/dependency changes; other crate publications; tags, GitHub Releases, PRs, yanks, overwrites, or release CI.

## 6. Required production changes
None. Only changelog and plan/closure evidence change. Keep src, manifest dependencies, and Cargo.lock unchanged; package normalization must strip the dev-only Core edge.

## 7. Ordered work packages
1. Confirm Archive 0.2.2 absent and prior versions unyanked. 2. Prepare Unreleased note and registry-only fixture. 3. Run local gate, package checks, and five-lane hosted CI on exact SHA. 4. Review package inventory and normalized manifest. 5. Publish once and read back metadata, checksum, and VCS SHA. 6. Pass registry-only fixture before dependent releases.

## 8. Failure, cancellation, restart, and contention semantics
Any failed check or unexpected manifest edge stops before upload. For ambiguous upload, query crates.io and do not retry while 0.2.2 may exist. Checksum/provenance mismatch stops the train. Never yank or overwrite. No local installation state is mutated.

## 9. Compatibility and migration
No production/API/extraction/runtime dependency change from 0.2.1; no consumer migration. The artifact exists only to satisfy Eggpack's exact pin.

## 10. Required tests
cargo test -p eggup-archive --locked; full local gate; package/dry-run and inventory; normalized manifest review; registry-only fixture; all five hosted lanes and post-publication same-SHA rerun.

## 11. Required verification commands
./scripts/check-local.sh
 git diff --check
 cargo tree --workspace --locked (manual boundary review)
 cargo package -p eggup-archive --locked
 cargo publish -p eggup-archive --locked --dry-run
 cargo check --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml --locked
 cargo tree --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml --locked
 cargo metadata --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml --locked --format-version 1

## 12. Documentation updates
Update plan/active registry and roadmap before upload. After readback, update Archive and root changelogs, registry, roadmap, architecture/tooling-governance.md §8 and architecture/overview.md. Preserve M008/M022 history; no tags/releases/PRs.

## 13. Acceptance criteria
Exact source SHA passes local and five hosted lanes; package manifest contains only expected normal dependencies and no Eggup edge; one 0.2.2 upload is verified by checksum/provenance and a registry-only consumer; records agree.

## 14. Stop conditions
Stop if 0.2.2 exists, source changes after qualification, any lane/check fails, package contents/dependencies differ, the dev edge leaks, or registry readback is unclear. Do not start dependent publication before closure.

## 15. Closure evidence required
Record baseline/source SHAs, commits, CI lane results, exact commands, package inventory/normalized manifest, crates.io publisher/time/checksum, local archive hash and provenance, fixture resolution, compatibility/security review, and registry/roadmap updates. Mark unrun evidence not run.

## 16. Handoff notes
Acquisition M023 follows Archive; Eggpack M010 is last. Curl and Service use caret edges and require no republication for this exact-pin transition.
