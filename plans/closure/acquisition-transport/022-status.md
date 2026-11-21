# Acquisition Transport M022 Closure — Workspace 0.2.2 release-readiness preparation

Status: closed 2026-11-21 — source-preparation qualification passed locally and in hosted CI on the exact candidate SHA; Eggpack registry packaging remains a separate publication-order gate.

Source plan: plans/implementation/acquisition-transport/022-workspace-0.2.2-release-readiness.md
Source roadmap: plans/subsystems/acquisition-transport-roadmap.md

Reviewed repository baseline (HEAD before M022 changes): 6371a7131f0f5eb42b9d8a151d7872f5a33e7ee1.

Implementation commit: 74aadfe36ef0d3742fadf2d0e84b6a0aeb8e018a on branch m022-workspace-0.2.2-release-readiness, pushed under explicit user authorization after the plan was drafted. No PR was opened. Hosted run [44925918931](https://github.com/eggstack/eggup/actions/runs/44925918931) passed all five lanes on this exact SHA.

## Executive finding

The shared source version is 0.2.2 for all eight workspace members, including the non-published footprint fixture. Eggpack's three source exact pins now match =0.2.2; eggpack-manifest =0.1.2 and Eggfetch's default-features = false plus its four explicit features are preserved. Current-source and release-facing documentation distinguishes the 0.2.2 source candidate from published registry releases.

The local macOS verification gate passes, and hosted CI run [44925918931](https://github.com/eggstack/eggup/actions/runs/44925918931) passed Stable, MSRV, Linux systemd, macOS, and Windows on the exact candidate SHA. eggup-eggfetch 0.2.2 packages and verifies against registry dependencies. eggup-eggpack 0.2.2 cannot yet be prepared for registry upload because crates.io does not have the exact-pinned Eggup 0.2.2 dependencies; this remains a release-order gate, not a reason to bypass registry resolution. No crate was published or yanked, and no tag or GitHub Release was created, moved, or edited; only the authorized source branch was pushed.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Bump the shared source version across all eight members | Cargo.toml, all eight member manifests, Cargo.lock; cargo check --workspace --all-targets --all-features and the full gate resolve/build the members as 0.2.2 | Pass |
| Keep Eggpack's source pins aligned and producer pin unchanged | crates/eggup-eggpack/Cargo.toml: Core/Archive/Acquisition =0.2.2; eggpack-manifest =0.1.2; its changelog records the staged registry prerequisite | Pass |
| Preserve Eggfetch features and dependency boundaries | crates/eggup-eggfetch/Cargo.toml; all-feature cargo tree manually reviewed | Pass; defaults disabled and four explicit features retained |
| Reconcile documentation with source and published state | AGENTS.md, changelogs, Eggpack README, docs/releases.md, architecture, release-workflow skill, M022 plan; direct crates.io API snapshot dated 2026-11-21 | Pass |
| Keep local checks green | ./scripts/check-local.sh, ./scripts/check-docs-truth.sh, git diff --check | Pass on macOS |
| Check registry packaging | Eggfetch package command passes; Eggpack package command fails because registry has no eggup-acquisition 0.2.2 matching the exact pin (available candidates include 0.2.1, 0.2.0, 0.1.3) | Eggfetch pass; Eggpack expected external prerequisite remains |
| Qualify the exact candidate in hosted CI | [GitHub Actions run 44925918931](https://github.com/eggstack/eggup/actions/runs/44925918931), exact SHA 74aadfe36ef0d3742fadf2d0e84b6a0aeb8e018a; all five lanes passed | Pass |
| Preserve registry/tag/release state | No publish/yank/tag/GitHub Release action taken | Pass |

## Production implementation evidence

No production Rust source, public API, runtime behavior, external dependency, MSRV, or Eggfetch feature set changed. The source version and Eggpack exact pins changed as planned. The acquisition publication-coupling test guard now accepts a dependent's exact pin only when it equals the seam's current workspace package version, the dependent inherits its version from the workspace, and its Unreleased changelog records the same exact requirement. Registry package checks remain separate and still fail when a required published dependency is unavailable.

Cargo.lock changes are limited to the eight workspace package records moving from 0.2.1 to 0.2.2; unrelated registry dependency versions did not move. The all-feature dependency review found: Acquisition has no dependencies; Core has no transport/service/Eggpack edge (the macOS graph contains sha2); Curl depends on Acquisition only; Eggfetch depends on Acquisition and the external Eggfetch stack, not Curl; Eggpack depends on Core, Archive, Acquisition, and eggpack-manifest; Service depends on Core; and the footprint fixture composes its optional adapters. No prohibited cross-edge was found.

## Exact commands and results

| Command | Result |
|---|---|
| ./scripts/check-docs-truth.sh | Passed. Assertion 7 was skipped because no baseline was supplied; one ambiguous published-version claim was noted and skipped, not failed. |
| ./scripts/check-local.sh | Passed on macOS: fmt, clippy, all-target/all-feature workspace tests, docs, Cargo tree resolution, and docs-truth. Cargo emitted existing non-failing redundant package.homepage warnings. |
| cargo test -p eggup-acquisition --test publication_coupling --locked | Passed: 3 tests, 0 failures. |
| cargo package -p eggup-eggfetch --locked | Initial attempt stopped because Cargo refuses to package this intentionally dirty tree without --allow-dirty. |
| cargo package -p eggup-eggfetch --locked --allow-dirty | Passed: packaged 8 files and verified source against registry dependencies. |
| cargo package -p eggup-eggpack --locked --allow-dirty | Expected failure: crates.io has no eggup-acquisition 0.2.2 matching the exact pin; package qualification must follow availability of the pinned dependency set. |
| cargo tree --workspace --locked --all-features | Passed and manually reviewed for crate boundaries; the local gate's cargo tree --workspace --locked also passed. |
| git diff --check | Passed. |
| cargo check --workspace --all-targets --all-features | Passed earlier in M022 after the version/lockfile update. |

## Platform and hosted CI evidence

| Platform/lane | Evidence |
|---|---|
| macOS local | Full local gate passed on the current macOS workstation. |
| Linux stable | Hosted run 44925918931 passed. |
| Rust 1.89 MSRV | Hosted run 44925918931 passed. |
| Linux systemd | Hosted run 44925918931 passed. |
| macOS hosted | Hosted run 44925918931 passed. |
| Windows | Hosted run 44925918931 passed, including the live SCM fixture and full workspace suite. |

The successful M022 run [44925918931](https://github.com/eggstack/eggup/actions/runs/44925918931) qualifies the exact candidate SHA 74aadfe36ef0d3742fadf2d0e84b6a0aeb8e018a; the earlier M021 run is separate evidence.

## Invariant review

- Published Eggup versions remain Core/Archive/Acquisition/Eggfetch/Eggpack 0.2.1 and Curl/Service 0.2.0 per the 2026-11-21 crates.io snapshot; the footprint crate remains publish = false.
- Published Eggpack 0.2.1 retains its =0.2.1 Eggup pins; immutable 0.2.0 retains its original pins. Only the workspace-source candidate uses =0.2.2.
- Eggfetch retains default-features = false and its four explicit features.
- No dependency boundary, public API, release artifact, tag, GitHub Release, or package registry state changed.

## Failure, rollback, and recovery review

This is a source metadata, test-guard, and documentation change. It performs no installation, live filesystem promotion, or registry mutation. The authorized commit and branch push changed remote source state solely to obtain exact-SHA hosted qualification; no runtime rollback/recovery path applies. The first local gate attempt exposed the guard's outdated assumption that every source dependent must accept the last published seam version; the guard was narrowly updated to require explicit Unreleased evidence for the matching exact-pin candidate, then the focused test and full gate passed.

## Compatibility and migration review

The shared version moves by a patch increment, 0.2.1 → 0.2.2; no API or runtime change is claimed and no consumer migration is required by this source-only metadata update. Registry consumers continue to receive their existing immutable versions. Eggpack's registry package qualification is blocked until its three exact-pinned Eggup 0.2.2 dependencies are available. The future package set/order is a separate authorized milestone; any eggup-service publication still requires explicit maintainer authorization.

## Security review

No runtime code or authority boundary changed. The additional acquisition test guard is compile-time source inspection of embedded manifest/changelog strings; it adds no dependency, network access, filesystem discovery, or secret handling.

## Documentation and operations evidence

Updated current-source claims in architecture/tooling-governance.md, architecture/overview.md, architecture/acquisition.md, architecture/eggpack-adapter.md, and architecture/transport-footprint.md; release snapshots, changelogs, Eggpack README, root index, release-workflow skill, plan, roadmap, and registry distinguish source 0.2.2 from published versions. Historical publication evidence and tags were not rewritten.

## Remaining publication gates (outside M022 source-preparation closure)

- Eggpack's exact-pinned registry package check must be rerun after all required Eggup 0.2.2 dependencies are published; no path/git workaround is accepted.
- Publication of eggup-service remains behind explicit maintainer authorization; the release set/order requires its own plan.

No medium-or-higher M022 code finding remains open. The release gates above are explicitly not represented as passed.

## Roadmap disposition

M022 is closed as a locally and hosted-qualified source-preparation milestone. This closure does not certify publication readiness: Eggpack's registry package check remains blocked until its exact-pinned Eggup 0.2.2 dependencies are available, and any resulting crate publication requires its own authorized release plan.

## Registry updates

The acquisition-transport roadmap and plans/registry.md record M022 as closed and hosted-qualified and point to this record. No registry artifact, tag, or GitHub Release changed; the candidate branch is pushed at 74aadfe36ef0d3742fadf2d0e84b6a0aeb8e018a.
