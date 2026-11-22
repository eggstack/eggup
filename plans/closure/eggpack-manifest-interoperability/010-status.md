# Eggpack Manifest Interoperability M010 — Closure and Verification Record

## Executive finding and status

**Status: closed 2026-11-22; published; registry-verified; hosted-qualified.** M010 published only eggup-eggpack 0.2.2. The crates.io checksum, the downloaded artifact, and the clean local package candidate agree; Cargo provenance identifies the qualified source commit and package path. The normalized manifest exact-pins Core, Archive, and Acquisition at 0.2.2 and eggpack-manifest at 0.1.2. An independent locked registry-only consumer resolves the full graph, and post-publication hosted CI passed all five lanes on the exact publication SHA.

This was a package-only promotion: no Rust source, public API, runtime behavior, or dependency policy changed. The source README and changelog have been reconciled in this repository after publication, but the immutable 0.2.2 archive retains their pre-publication text; this low-severity documentation residual is recorded below. No Curl, Service, or transport-footprint package was published. No existing version was overwritten or yanked, and no tag, GitHub Release, or PR was changed.

## Source plan, roadmap, and reviewed baseline

- Plan: plans/implementation/eggpack-manifest-interoperability/010-eggup-eggpack-0.2.2-publication.md
- Roadmap: plans/subsystems/eggpack-manifest-interoperability-roadmap.md
- Registry: plans/registry.md
- Reviewed repository baseline: 3be567e9839b555d7d9d5e6e43f14de4a13c901d
- Publication source SHA: c9a7ac0af7ea3f0a090d19fe1fffa817235f0081
- Branch: m022-workspace-0.2.2-release-readiness
- Affected dependency closures: Core M022, Archive M009, Acquisition M023
- Implementation commits / PRs: no production implementation commit or PR was needed; the prepared package was promoted from the qualified source SHA. The closure and documentation reconciliation are repository records after publication.
- Authorization: the user authorized all actions needed for this release before the publication sequence.

## Registry prerequisites and publication readback

Before packaging, exact-version registry reads confirmed Eggpack 0.2.2 absent, Eggpack 0.2.1 present and unyanked, and all four exact prerequisites present and unyanked:

| Prerequisite | Registry state | Checksum |
|---|---|---|
| eggup-core 0.2.2 | present; yanked=false | a0b0dba6035ec7d403ddf0298f1021a5bd544764f9bb4e3011cb31b2dacd614b |
| eggup-archive 0.2.2 | present; yanked=false | 67ab697e3af41b3cd3d1ed140f821c68739ec446d2a7782fa26ca4960edc882b |
| eggup-acquisition 0.2.2 | present; yanked=false | 4000935b1d4d384eaba066c9742e70f91d5cb3dd12618ac3aa79d654e2aab865 |
| eggpack-manifest 0.1.2 | present; yanked=false | 4bbbe8d12026adf7c8e8ddaecbed0c6ac403a5621d4dd1ac9949f8daf8149a09 |

Eggpack 0.2.2 was uploaded once from the authorized source. Exact-version crates.io readback reports:

| Registry fact | Readback |
|---|---|
| Package | eggup-eggpack 0.2.2; present; yanked=false |
| Publisher / published at | dbowm91 / 2026-11-22T05:27:11.823686Z |
| Registry SHA-256 | f24df78cf088e665dd69db1b319d7168007175ebc1ec5d2df76a069e31242e49 |
| Local qualified package SHA-256 | Same as registry checksum |
| Downloaded registry archive SHA-256 | Same as registry checksum; direct byte comparison passed |
| Cargo provenance | .cargo_vcs_info.json records source SHA c9a7ac0af7ea3f0a090d19fe1fffa817235f0081 and path crates/eggup-eggpack |
| Prior package | Eggpack 0.2.1 remains present and unyanked with its original exact pins |

The normalized manifest and dependency endpoint report exactly four normal dependencies:

| Dependency | Requirement |
|---|---|
| eggup-core | =0.2.2 |
| eggup-archive | =0.2.2 |
| eggup-acquisition | =0.2.2 |
| eggpack-manifest | =0.1.2 |

No path, Git, patch, or unintended runtime dependency appears in the published manifest.

## Package inventory

The qualified package contains 20 files:

- .cargo_vcs_info.json, CHANGELOG.md, Cargo.lock, Cargo.toml, Cargo.toml.orig, README.md, and src/lib.rs;
- tests/archive_handoff.rs, tests/caller_destinations.rs, tests/digest_reuse.rs, and tests/interoperability.rs;
- tests/fixtures/README.md, archive-manifest.json, bundle-manifest.json, direct-manifest.json, projection-archive.json, projection-bundle.json, projection-direct.json, unknown-schema.json, and wrong-target.json.

The package inventory was listed and package verification plus publish dry-run passed during clean-source qualification. After the repository README and changelog were revised as post-publication documentation, a later inventory-only command without --allow-dirty correctly refused the dirty tree; cargo package --list --locked --allow-dirty then confirmed the same 20 paths. No package or upload was attempted by that inventory-only retry.

## Registry-only consumer fixture

Fixture: plans/closure/eggpack-manifest-interoperability/010-package-fixture/.

The standalone fixture depends on eggup-eggpack 0.2.2 and owns its lockfile. Its locked metadata contains 80 package records: the fixture root plus 79 registry packages. All 79 dependencies use the crates.io registry source; no path- or Git-sourced package appears. The graph resolves eggup-eggpack, eggup-core, eggup-archive, and eggup-acquisition at 0.2.2, and eggpack-manifest at 0.1.2.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| All exact prerequisites available before packaging | Exact-version crates.io reads recorded above showed Core, Archive, Acquisition 0.2.2 and eggpack-manifest 0.1.2 present and unyanked; Eggpack 0.2.2 was absent and 0.2.1 unyanked before upload. Passed. |
| Publish only Eggpack 0.2.2 once from the qualified source | Cargo reported a successful single upload from c9a7ac0af7ea3f0a090d19fe1fffa817235f0081. Registry readback reports publisher, timestamp, version, and yanked=false. Passed. |
| Artifact identity and provenance match | Registry checksum, local candidate checksum, and downloaded archive checksum agree; direct byte comparison passed; Cargo provenance records the publication SHA and crate path. Passed. |
| Normalized dependency set is exact and registry-resolvable | The registry manifest and dependency endpoint agree on three Eggup =0.2.2 pins plus eggpack-manifest =0.1.2; no path/Git/patch source. Passed. |
| Consumers can resolve the publication without workspace overrides | Locked standalone check, tree, and metadata passed; 80 packages total, 79 registry packages, no non-registry dependency source. Passed. |
| Local and hosted qualification pass | Serialized local gate passed on the publication SHA; post-publication GitHub Actions run 44988995059 attempt 2 passed all five lanes on that same SHA. Passed. |
| Scope and immutable state are preserved | No source/API/runtime change, no overwrite or yank, no Curl/Service/footprint or producer-schema publication, and no tag, GitHub Release, or PR change. Passed. |

## Commands and platform evidence

| Check | Result |
|---|---|
| RUST_TEST_THREADS=1 ./scripts/check-local.sh | Passed on macOS against publication source SHA c9a7ac0af7ea3f0a090d19fe1fffa817235f0081 before upload. This includes the workspace tests, docs truth check, and cargo tree resolution/review. |
| cargo package -p eggup-eggpack --list --locked | Passed during clean-source qualification; 20-file inventory above. The post-publication dirty-tree retry is described above. |
| cargo package -p eggup-eggpack --locked | Passed during clean-source qualification. |
| cargo publish -p eggup-eggpack --locked --dry-run | Passed during clean-source qualification; dry-run did not upload. |
| cargo publish -p eggup-eggpack --locked | Passed once; registry readback resolved the publication outcome. |
| Registry API reads and downloaded archive comparison | Passed; version/dependency facts, checksum, byte equality, and .cargo_vcs_info.json provenance agree. |
| cargo fmt --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml -- --check | Passed. |
| cargo check --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml | Passed during fixture qualification. |
| cargo check --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml --locked | Passed in the final fixture check. |
| cargo tree --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml --locked | Passed; reviewed exact 0.2.2 Eggup packages and schema 0.1.2. |
| cargo metadata --manifest-path plans/closure/eggpack-manifest-interoperability/010-package-fixture/Cargo.toml --locked --format-version 1 | Passed; 80 packages total, 79 registry packages, no non-registry dependencies. |
| GitHub Actions run 44988995059, attempt 2 | Passed; head SHA c9a7ac0af7ea3f0a090d19fe1fffa817235f0081; all five lanes succeeded: Stable, MSRV 1.89, Linux systemd, macOS, and Windows. |
| ./scripts/check-docs-truth.sh and git diff --check | Passed after the release documentation and this closure record were added; note 7 was skipped because no --baseline was supplied. |

No platform result is inferred: native local evidence is macOS, and the hosted run reports Linux stable, Linux MSRV, Linux systemd, macOS, and Windows separately.

## Production, compatibility, failure, and security review

- M010 changed no production Rust source, public API, runtime behavior, package dependency policy, or workspace lockfile. It promoted the prepared exact-pin package after all named dependencies became registry-visible.
- The adapter remains an optional consumer-side leaf. It does not select releases, assert authenticity, choose install roots, or authorize ownership. The producer schema dependency remains exactly 0.1.2.
- The immutable Eggpack 0.2.1 and 0.2.0 artifacts retain their historical exact pins. No prior version was overwritten or yanked.
- The upload succeeded unambiguously and was read back before further release work; no retry or rollback was needed. No rollback of registry publication is claimed.
- SHA-256 values in this record establish package integrity and byte identity only; they make no authenticity or signature claim. The registry-only fixture performs no network request or installation operation.

## Documentation, operations, and residual finding

After registry readback, the repository Eggpack README and changelog, root CHANGELOG.md, docs/releases.md, architecture summaries and governance, AGENTS.md, docs-hygiene and release-workflow skills, plans/registry.md, the Eggpack roadmap, and this plan/closure were reconciled. M009 and M022 history is preserved. No tags, GitHub Releases, or PRs were edited.

**Low severity — immutable release documentation.** The package archive is byte-identical to the candidate built from publication SHA c9a7ac0af7ea3f0a090d19fe1fffa817235f0081. That commit's README said the 0.2.2 source update was not yet a publication, and its CHANGELOG left M010 under Unreleased. The repository copies are now corrected, but the already-published archive cannot be edited in place. This is documentation-only and does not affect API, runtime behavior, or dependency resolution; changing the registry artifact would require a later Eggpack version. No later publication is authorized or part of M010.

No M010-specific unresolved finding remains at medium severity or higher. The low-severity artifact documentation residual is recorded in architecture/tooling-governance.md under Known doc/code drift and on docs/releases.md. Unrelated open findings remain tracked in plans/registry.md.

## Roadmap and registry disposition

M010 is closed in the Eggpack roadmap and the authoritative Dependency-ready implementation work table in plans/registry.md. The exact-pin publication chain is complete: Core M022, Archive M009, Acquisition M023, then Eggpack M010. The registry row records the publication source, checksum, fixture evidence, and hosted run. Curl, Service, and eggup-transport-footprint remain outside the selective release; eggup-transport-footprint remains non-publishable.
