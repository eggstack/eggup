# Eggpack Manifest Interoperability M009 — eggup-eggpack 0.2.1 publication

Status: active

Repository baseline: 5a04f68f60c8c3ad6392036fa314ed2749fa572c

Source roadmap:
- plans/subsystems/eggpack-manifest-interoperability-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md
- plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md
- plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md

Primary class: infrastructure

Hard dependencies — all must be registry-visible before packaging:

| Dependency | Requirement |
|---|---|
| eggup-core 0.2.1 | Published, registry-visible, and not yanked. |
| eggup-archive 0.2.1 | Published, registry-visible, and not yanked. |
| eggup-acquisition 0.2.1 | Published, registry-visible, and not yanked; the exact seam pin in this adapter must resolve. |
| eggpack-manifest 0.1.2 | Exact producer-schema pin already recorded by Eggpack M008; confirm it remains registry-visible and not yanked. |

## 1. Objective

Publish exactly eggup-eggpack 0.2.1 to crates.io from one clean, recorded source commit after all package, local-gate, and hosted-CI checks pass. Then verify the registry artifact and prove from a separate registry-only consumer that its exact pins resolve Core, Archive, and Acquisition 0.2.1.

This publication completes the selective 0.2.1 dependency chain. The package is the only remaining publishable Eggup crate whose immutable registry manifest exact-pins the earlier 0.2.0 set. The release carries the already-prepared =0.2.1 Eggup pins and Eggpack M008's eggpack-manifest 0.1.2 source pin to consumers; it does not change the adapter's Rust implementation or public API.

## 2. Why this milestone is ready

- The crates.io public API reports eggup-eggpack 0.2.0 as the latest existing release, non-yanked and published by dbowm91; 0.2.1 is absent. Do not overwrite or yank any existing version.
- The exact prerequisite set is available: Core M016 published eggup-core 0.2.1; Archive M008 and Acquisition M019 published eggup-archive 0.2.1 and eggup-acquisition 0.2.1 from 5a04f68. M019's closure records registry visibility and all-five-lane hosted run 44591516672.
- Eggpack M008 closed on hosted run 44348100277 and qualified the workspace-source exact eggpack-manifest 0.1.2 pin. Its production schema source is byte-identical to 0.1.1, and the pin is already on this baseline.
- The workspace package version is 0.2.1; eggup-eggpack/Cargo.toml already exact-pins Core, Archive, and Acquisition at =0.2.1. No manifest or lockfile edit is needed for this publication.
- Published eggup-curl 0.2.0 and eggup-eggfetch 0.2.0 use ^0.2 acquisition requirements; published eggup-service 0.2.0 uses ^0.2 for Core. They accept the 0.2.1 releases and are not republished by M009. eggup-transport-footprint is publish = false and is never published.
- No eggup-eggpack/src/** production change is required or authorized. The package changelog must be cut truthfully for the 0.2.1 candidate before packaging; the resulting publication source commit must be clean and recorded in the closure.

## 3. Current implementation evidence

- Reviewed baseline and current checkout HEAD: 5a04f68f60c8c3ad6392036fa314ed2749fa572c. The local tree contains Archive M008 and Acquisition M019 release-record changes; M009 must preserve them.
- crates/eggup-eggpack/Cargo.toml inherits workspace version 0.2.1; normal dependencies are eggup-core, eggup-archive, and eggup-acquisition at exact version 0.2.1, plus eggpack-manifest at exact version 0.1.2. The paths on the three workspace dependencies are development conveniences; Cargo's packaged manifest must contain registry requirements without local paths.
- The package's development ZIP dependency is 9.6.1; dev-dependencies are not published as runtime dependencies. M008's source refresh changed no adapter production Rust code or API.
- Registry metadata for immutable eggup-eggpack 0.2.0 still reports the three exact =0.2.0 Eggup dependencies and eggpack-manifest =0.1.0. This release must leave those bytes unchanged.
- The uploaded 0.2.1 package's .cargo_vcs_info.json must identify the clean M009 publication source commit, not a dirty-tree build or an unrelated tag.

## 4. Invariants that must not regress

- Publish one package only: eggup-eggpack 0.2.1.
- Every in-workspace package dependency must be registry-visible at the exact version in the published manifest. Never bypass failed resolution with a patch, local path, Git source, or another package source.
- Preserve the adapter as an optional leaf. Only eggup-eggpack may depend on eggpack-manifest; lower layers remain producer-independent.
- Leave Rust production source, public API, behavior, dependency declarations, and Cargo.lock unchanged in M009. Only release documentation/planning records may change.
- Existing crates.io versions are immutable. Do not overwrite, yank, or retry an uncertain upload without first reading registry state.
- Do not create or move any Git tag. Preserve the existing annotated v0.2.1 tag at Core's publication source commit b5a989a72dc185b30cd8c0cb67b5110ed03acd86.
- Preserve the existing GitHub Release v0.2.1 target. The previously authorized release-note extension may append Eggpack publication evidence; do not create a new release or change its target.
- Every crates.io upload requires its own immediate, exact maintainer confirmation. Prior Archive and Acquisition confirmations do not authorize this upload.

## 5. Scope

### In scope

- Publish eggup-eggpack 0.2.1 after exact source, registry, package, local, and hosted qualification.
- Cut crates/eggup-eggpack/CHANGELOG.md for the 0.2.1 release candidate before packaging, without claiming that the upload has already completed.
- Register M009 in this roadmap and plans/registry.md; after upload, write the closure and reconcile current release summaries.
- Verify crates.io metadata, normalized dependencies, publisher, checksum, downloaded bytes, and Cargo provenance.
- Prove the published exact-pin behavior with a separate registry-only consumer fixture.
- Extend the existing v0.2.1 GitHub Release notes with the Eggpack publication evidence, preserving the Core release target and existing tag.
- Commit and push prepared and completed release records to origin/main as authorized.

### Explicitly out of scope

- Republishing eggup-curl, eggup-eggfetch, or eggup-service; their published ^0.2 requirements already accept the 0.2.1 dependencies.
- Publishing eggup-transport-footprint (publish = false).
- Any source/API/behavior change, package manifest or lockfile change, dependency refresh, version bump, or new consumer policy.
- Re-publishing or changing eggup-core, eggup-archive, eggup-acquisition, or eggpack-manifest.
- Creating, moving, or deleting tags; yanking or replacing registry versions; creating a new GitHub Release; or merging an unrelated branch/PR.
- Adding release automation or CI publication credentials.

## 6. Required production changes

None. M009 is a publication and evidence milestone; the source candidate already contains the version and exact pins. Do not modify crates/eggup-eggpack/src/**, its Cargo dependency declarations, or Cargo.lock.

The only crate-package content edit is the 0.2.1 changelog cut, which must be included before package qualification and publication so the uploaded archive is the same artifact that is verified. The release entry must describe the exact dependency refresh and unchanged adapter API without making an inaccurate claim about whether crates.io has already accepted the upload.

## 7. Ordered work packages

1. Recheck registry preflight. Confirm eggup-eggpack 0.2.1 is absent; existing 0.2.0 is present and not yanked; dbowm91 remains the authorized publisher; and Core, Archive, Acquisition 0.2.1 plus eggpack-manifest 0.1.2 are registry-visible and not yanked.
2. Register M009 and prepare the package. Add this plan, active roadmap/registry entries, and a concise 0.2.1 crate changelog section. Preserve the M019 records already in the working tree. Do not change production source, package manifests, or the lockfile.
3. Qualify locally before the release-preparation commit. Run focused Eggpack tests/clippy, inspect the dependency tree, then run documentation-truth, diff, and the complete local gate. Do not run Cargo packaging while the crate changelog is uncommitted; Cargo correctly refuses that state.
4. Establish a clean publication source. Commit and push the prepared release candidate and pending release records to origin/main as authorized. Record the resulting full SHA; require a clean worktree and no change to crate source/manifest/lockfile. On this clean commit, run cargo package --list, cargo package, and cargo publish --dry-run; inspect the normalized package manifest for registry-only exact pins and no unintended runtime dependency. Wait for all five hosted CI lanes on that exact pushed SHA; do not infer hosted results from the prior 44591516672 run.
5. Stop for the separate upload authorization. After all pre-upload gates pass, ask for explicit confirmation naming eggup-eggpack 0.2.1, crates.io, and the exact clean publication source SHA. If confirmation is absent or declined, do not upload. If confirmed, the next operation is the single cargo publish -p eggup-eggpack --locked invocation; do not insert another action between confirmation and upload.
6. Verify the immutable registry artifact. Read back version, publisher, yanked state, checksum, normalized dependency requirements, and dependency list. Download the .crate, compare SHA-256 and bytes with the locally packaged candidate, and inspect .cargo_vcs_info.json for the exact publication SHA and package path.
7. Prove the consumer graph. Add a minimal fixture under plans/closure/eggpack-manifest-interoperability/009-package-fixture/ that pins eggup-eggpack to exact version 0.2.1 from crates.io. Require a committed locked lockfile with registry sources only and Core/Archive/Acquisition 0.2.1. Run fixture cargo check, cargo tree, and cargo metadata with --locked.
8. Close the milestone. Record source SHA, registry checksum, publisher and API evidence, artifact/provenance comparison, fixture result, exact local/hosted command outcomes, and tag/release target checks. Update the registry, roadmap, crate/root changelogs, docs/releases.md, architecture release records, and applicable agent-facing summaries. Append Eggpack evidence to the existing GitHub Release notes without changing its target or tag. Commit/push closure records to origin/main.

## 8. Failure, cancellation, restart, and contention semantics

Qualification commands may update Cargo build artifacts under target/; the only expected tracked preparation change inside the crate is its changelog. No install, live filesystem mutation, or application transaction occurs.

The crates.io upload is irreversible because a published version cannot be overwritten. A denied or declined confirmation leaves crates.io unchanged. If cargo publish reports an ambiguous network/result failure, first read crates.io for 0.2.1 and compare publisher/checksum before deciding what happened; never blindly repeat the upload. Do not yank a version as rollback. If registry metadata, a prerequisite, package bytes, CI, or a local gate fails or changes, stop, preserve evidence and worktree state, and do not publish or widen scope to repair it.

## 9. Compatibility and migration

The adapter's own production source, public signatures, and behavior are unchanged from the qualified source candidate. The published 0.2.0 package remains immutable with exact Eggup pins at 0.2.0 and producer pin 0.1.0. A consumer choosing eggup-eggpack 0.2.1 receives exact Eggup dependencies at 0.2.1 and eggpack-manifest 0.1.2; this is the intended consumer-visible dependency refresh. No adapter API migration is required. Verify the registry-only fixture rather than inferring the result from the in-tree path graph.

The release makes no authenticity claims; SHA-256 values are integrity and artifact-identity evidence only. Release selection, URLs, installation roots, ownership, and permissions remain caller-owned.

## 10. Required tests

- All existing Eggpack unit and integration tests pass with locked workspace dependencies.
- The package succeeds against registry dependencies, not only the workspace path graph.
- The full local gate passes on the prepared candidate.
- All five hosted lanes pass on the exact pushed publication source SHA.
- After upload, the registry-only fixture resolves eggup-eggpack 0.2.1 and all three exact Eggup dependencies at 0.2.1, with no path or Git source edge.
- Registry metadata, normalized manifest, downloaded checksum/bytes, and Cargo provenance match the package built from the publication source.

## 11. Required verification commands

Run these before requesting upload confirmation. Do not package while the crate changelog is uncommitted; Cargo's dirty-package guard is intentional.

Before the clean release-preparation commit:

1. cargo fmt --all -- --check
2. cargo test -p eggup-eggpack --locked
3. cargo clippy -p eggup-eggpack --all-targets --locked -- -D warnings
4. cargo tree --workspace --locked — manually inspect Eggpack as a leaf, exact in-workspace pins, and producer-only dependency boundary.
5. ./scripts/check-docs-truth.sh
6. git diff --check
7. ./scripts/check-local.sh

After commit/push, on the clean exact publication-source SHA:

8. cargo package -p eggup-eggpack --list --locked
9. cargo package -p eggup-eggpack --locked
10. cargo publish --dry-run -p eggup-eggpack --locked
11. Hosted run: Stable, MSRV, service-systemd, macOS, and windows-check all green on that exact pushed SHA.
12. gh release view v0.2.1 --repo eggstack/eggup --json tagName,targetCommitish,url,body and git ls-remote origin refs/tags/v0.2.1 'refs/tags/v0.2.1^{}' — verify existing target/tag before and after; do not change either.

After upload, run with the committed fixture:

13. cargo check --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml --locked
14. cargo tree --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml --locked
15. cargo metadata --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml --locked --format-version 1
16. Read crates.io version/dependency APIs, download the .crate, compare local and registry bytes/checksums, and inspect .cargo_vcs_info.json; record exact results in the closure.
17. Rerun ./scripts/check-docs-truth.sh and git diff --check after closure/doc updates; run ./scripts/check-local.sh if any additional change affects the gate's scope.

## 12. Documentation updates

- Cut crates/eggup-eggpack/CHANGELOG.md to a 0.2.1 section before package creation; preserve a truthful Unreleased section and do not claim an upload until it exists.
- After registry confirmation, update CHANGELOG.md, docs/releases.md, architecture/overview.md, and architecture/tooling-governance.md to distinguish the three previously published 0.2.1 packages from Eggpack's new publication and the three remaining 0.2.0 crates.
- Reconcile the Eggpack roadmap and plans/registry.md; add M009 closure and registry-only fixture evidence.
- Update applicable .opencode/skills/ publication-state summaries if they state the now-superseded latest version or exact-pin status. Do not duplicate new policy in AGENTS.md or non-normative docs/.
- Extend only the existing GitHub Release v0.2.1 notes with publication facts; keep its Core target and existing annotated tag untouched.

## 13. Acceptance criteria

- eggup-eggpack 0.2.1 is present on crates.io, not yanked, and public metadata identifies the authorized publisher and exact dependency requirements.
- The registry download is byte-identical to the package built from the clean M009 publication source; checksum and .cargo_vcs_info.json agree with the closure.
- The registry-only consumer fixture locks Core, Archive, and Acquisition 0.2.1 from crates.io and passes checks without path/Git overrides.
- Focused package checks, full local gate, docs-truth checks, diff checks, and every hosted lane pass as recorded. No platform result is inferred.
- M009 closure, roadmap, registry, changelogs, and current release summaries agree on the exact publication set; Curl, Eggfetch, Service, and transport-footprint were not published.
- The existing v0.2.1 tag and GitHub Release target remain Core's original source commit; only existing release notes are extended.

## 14. Stop conditions

Stop without uploading if eggup-eggpack 0.2.1 already exists; any exact prerequisite is absent or yanked; the current package manifest/pins differ from this plan; the package is not built from a clean recorded source SHA; a package, focused check, docs gate, local gate, or hosted lane fails; the normalized package has a path/Git dependency or unintended runtime edge; registry readback is ambiguous; the existing tag/release target differs from the recorded Core target; or separate exact confirmation is not obtained. Do not publish other packages, change versions, move tags, or relax pins to work around a stop.

## 15. Closure evidence required

The M009 closure must include the reviewed baseline and exact publication source SHA; implementation/release-record commits; registry preflight and post-publication metadata; publisher, yanked state, checksum, normalized dependency list, downloaded bytes, and .cargo_vcs_info.json; the registry-only fixture lockfile and exact command outputs; the requirement-to-evidence matrix; focused and full local command results; all five hosted lane results; invariant, failure/recovery, compatibility, migration, and security reviews; changelog/roadmap/registry/release-note evidence; unresolved findings with severity; and explicit verification that the pre-existing tag and release target were not changed.

## 16. Handoff notes

M009 is the final known 0.2.1 publication needed to align the Eggup exact-pin chain. Do not publish Curl, Eggfetch, Service, or transport-footprint for cosmetic version uniformity: their dependency policy does not require it. This plan authorizes preparation and qualification, not the crates.io upload itself. The upload requires a separate, immediate confirmation naming package, version, destination, and the clean publication source SHA.
