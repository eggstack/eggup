# Eggpack Manifest Interoperability M008 — Closure and Verification Record

Status: **closed** — the source-pin refresh is qualified; every acceptance criterion has evidence and no unresolved M008 findings remain. This closure does not authorize publication or merge.

Source plan: plans/implementation/eggpack-manifest-interoperability/008-eggpack-manifest-0.1.2-source-pin-refresh.md

Source roadmap: plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Reviewed repository baseline: 47b202b5f35fe8dd05d5bd9c2b456b08688f3b4d, the Core M016 candidate on release/0.2.1-preparation.

Implementation commit: 31af43f024525fef543cabdeb44586c408a239a3.

Pull request: https://github.com/eggstack/eggup/pull/4, based on release/0.2.1-preparation. Implementation qualification head: 31af43f024525fef543cabdeb44586c408a239a3.

Hosted qualification: push run 44347961501 and pull-request run 44348100277 both tested that exact head. All five lanes passed in each run; the pull-request run is the primary PR evidence.

## Executive finding

The unpublished workspace-source eggup-eggpack 0.2.1 candidate now pins eggpack-manifest exactly to 0.1.2 from crates.io. Cargo.lock changes only that package version and checksum, and the exact-pin test asserts 0.1.2 while continuing to reject Git/VCS edges. The dependency tree contains one producer-schema edge, from eggup-eggpack only. No production Rust source, API, behavior, crate version, or publication status changed. The immutable published eggup-eggpack 0.2.0 artifact retains its original eggpack-manifest 0.1.0 edge.

Crates.io preflight identified eggpack-manifest 0.1.2 as latest, non-yanked, published 2026-11-18, with Rust 1.89 and checksum 4bbbe8d12026adf7c8e8ddaecbed0c6ac403a5621d4dd1ac9949f8daf8149a09. The published 0.1.2 production src/lib.rs section is byte-identical to 0.1.1; upstream changes are test-only.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Use the latest compatible published producer schema with an exact registry pin | Crates.io preflight and cargo info identified 0.1.2 as latest, non-yanked, Rust 1.89; checksum recorded above; manifest requires exactly 0.1.2 | pass |
| Keep the lockfile refresh narrow and registry-only | Cargo.lock changes only eggpack-manifest 0.1.1 to 0.1.2 and its checksum; source remains the crates.io registry | pass |
| Preserve adapter behavior and public API | Published 0.1.2 production source is byte-identical to 0.1.1; git diff from baseline to implementation under crates/eggup-eggpack/src is empty | pass |
| Preserve the producer boundary | cargo tree resolves 0.1.2 only beneath eggup-eggpack 0.2.1; the test asserts the exact registry pin and absence of VCS edges | pass |
| Qualify adapter tests, lint, docs, and workspace | Focused tests, clippy, documentation-truth, diff check, and scripts/check-local.sh all passed | pass |
| Qualify hosted CI on the pushed implementation head | Runs 44347961501 and 44348100277 both used 31af43f; Stable, MSRV, Linux systemd, macOS, and Windows all passed | pass |
| Keep M016 Core-only and the published adapter immutable | M016 plan assigns the separate edge refresh to M008; published eggup-eggpack 0.2.0 remains on producer version 0.1.0 | pass |
| Avoid release-side effects | No cargo publish, tag, GitHub release, merge, or adapter republication was performed | pass |

## Production implementation evidence

The implementation changes are limited to crates/eggup-eggpack/Cargo.toml, Cargo.lock, and tests/interoperability.rs. The manifest edge requires exactly 0.1.2; Cargo.lock records the crates.io source and checksum above; the test constant and comment identify the current qualified schema. The diff under crates/eggup-eggpack/src is empty between the reviewed baseline and implementation commit. Remaining implementation-commit changes are current documentation and planning records.

The inverse dependency tree is eggpack-manifest 0.1.2 beneath eggup-eggpack 0.2.1 only. The published eggup-eggpack 0.2.0 package remains immutable and is not rebuilt or uploaded by M008. No package version changed.

## Exact commands and results

1. cargo search eggpack-manifest --limit 10 && cargo info eggpack-manifest — identified 0.1.2 as latest at preflight.
2. cargo info eggpack-manifest@0.1.2 --verbose — reports Rust 1.89 and compatible published metadata.
3. cargo update -p eggpack-manifest --precise 0.1.2 — success; only eggpack-manifest updated.
4. cargo test -p eggup-eggpack --locked — 7 unit, 13 archive handoff, 15 caller destinations, 8 digest reuse, and 28 interoperability tests passed; total 71 passed, 0 failed.
5. cargo clippy -p eggup-eggpack --all-targets --locked -- -D warnings — passed.
6. cargo tree --workspace --locked -i eggpack-manifest — resolved 0.1.2 under eggup-eggpack only.
7. ./scripts/check-docs-truth.sh — passed. Note 7 skipped because no --baseline was supplied; this is the gate's documented no-baseline behavior, not a failed check.
8. git diff --check — passed.
9. ./scripts/check-local.sh — exit code 0; formatting, workspace clippy, all-target/all-feature tests, docs, dependency resolution, and docs-truth passed.
10. Hosted run 44347961501 (push) — Stable, MSRV, Linux systemd service qualification, macOS, and Windows all success.
11. Hosted run 44348100277 (pull_request) — Stable, MSRV, Linux systemd service qualification, macOS, and Windows all success.
12. git diff --exit-code 47b202b5f35fe8dd05d5bd9c2b456b08688f3b4d 31af43f024525fef543cabdeb44586c408a239a3 -- crates/eggup-eggpack/src — empty, exit code 0.
13. git diff --numstat 47b202b5f35fe8dd05d5bd9c2b456b08688f3b4d 31af43f024525fef543cabdeb44586c408a239a3 -- Cargo.lock — 2 insertions and 2 deletions, the single package version/checksum update.
14. ./scripts/check-local.sh on the closure-doc tree initially exited 101 in eggup-curl: artifact_temp_is_owner_private_and_no_clobber returned a Transport error for its example.com fixture. No source was changed.
15. cargo test -p eggup-curl --locked artifact_temp_is_owner_private_and_no_clobber -- --exact — exit 0 but ran 0 tests because the exact filter omitted the tests:: module prefix; this attempt did not validate the fixture.
16. cargo test -p eggup-curl --locked artifact_temp_is_owner_private_and_no_clobber — passed the targeted test (1 passed, 0 failed).
17. ./scripts/check-local.sh rerun after the targeted test passed — exit code 0; no production source change was made.
18. ./scripts/check-docs-truth.sh && git diff --check && ./scripts/check-local.sh after the closure SHA correction — documentation-truth and diff checks passed; the gate exited 101 on eggup-curl test build_curl_args_passes_sub_second_deadlines_to_fake_curl with an exit-unknown Transport error.
19. cargo test -p eggup-curl --locked build_curl_args_passes_sub_second_deadlines_to_fake_curl — passed the targeted test (1 passed, 0 failed).
20. ./scripts/check-local.sh final rerun — exit code 0; the transient failure did not recur and no production source change was made.

Local qualification ran on macOS 26.6.2 arm64. No Linux or Windows result is inferred from local execution; those platforms are reported only from hosted CI.

## Per-platform hosted results

| Platform | Evidence | Result |
|---|---|---|
| Linux | Stable checks, MSRV check on Rust 1.89, and Linux systemd service qualification in runs 44347961501 and 44348100277 | pass |
| macOS | macOS tests in both runs; native adapter tests and full local gate also passed on macOS arm64 | pass |
| Windows | Windows archive, acquisition, and service tests and check in both runs | pass |

## Invariant review

1. Only eggup-eggpack depends on eggpack-manifest; lower Eggup crates remain producer-independent.
2. The producer edge is exact and registry-only; the lockfile checksum matches the reviewed 0.1.2 registry release.
3. The adapter's production source, public API, and behavior are unchanged.
4. The published eggup-eggpack 0.2.0 artifact and its 0.1.0 producer pin remain untouched.
5. M016 remains Core-only; M008 is separately planned and reviewed.
6. SHA-256 remains integrity evidence only; this refresh makes no authenticity or provenance claim.
7. No package publication, tag, release, or merge is part of this closure.

## Failure, rollback, and recovery review

This is a manifest, lockfile, test-assertion, and documentation change. It performs no installation or live filesystem mutation and has no application rollback path. The dependency refresh is reversible in source control; no published artifact needs recovery because none was published. All required local and hosted checks passed. No failure or recovery action was required.

## Compatibility and migration review

The producer 0.1.2 production source is byte-identical to 0.1.1, so no Eggup adapter API or runtime behavior changes and consumers require no migration. Upstream test-only changes do not affect the adapter runtime surface. The published eggup-eggpack 0.2.0 package remains immutable with its original 0.1.0 producer requirement; M008 affects only the unpublished workspace source candidate.

## Security review

The pin is exact and the lockfile checksum matches the reviewed registry release. The adapter is the only workspace crate with a producer-schema edge. No new runtime dependency family, transport, filesystem authority, authenticity claim, or credential path was introduced. No unresolved M008 security or correctness findings remain.

## Documentation and operations evidence

The adapter README and Unreleased changelog distinguish workspace source from the immutable published package. AGENTS.md and architecture dependency records state the 0.1.2 workspace pin. The M016 plan records its historical 0.1.1 baseline and points to M008 without widening Core scope. The M008 plan, Eggpack roadmap, and registry point to this closure and agree on the closed status. No consumer-facing how-to required a change.

No package was published; no tag or GitHub release was created; neither PR #4 nor PR #3 was merged. PR #3 remains on its original branch and scope.

## Unresolved findings

None specific to M008. The documentation-truth note 7 skip is recorded above and is not a failed acceptance criterion.

## Roadmap disposition and registry updates

Eggpack M008 is closed and hosted-qualified. The Eggpack roadmap records M001–M008 closed and links this closure. The registry subsystem summary records the 2026-11-18 completion and run 44348100277; its milestone-status table links both the source plan and this closure. No follow-on Eggpack milestone is made dependency-ready by M008.
