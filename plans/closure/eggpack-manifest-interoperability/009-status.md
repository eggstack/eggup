# Eggpack Manifest Interoperability M009 — Closure and Verification Record

## Status

**Closed; published; registry-verified; hosted-qualified.** Exactly eggup-eggpack 0.2.1 was published to crates.io after the required separate confirmation. The registry package is byte-identical to the candidate built from the clean publication source; Cargo provenance identifies that source commit; and a separate locked consumer proves the exact dependency pins resolve from crates.io.

M009 changed no production Rust, public API, behavior, package manifest, or workspace lockfile. It published only eggup-eggpack 0.2.1. Curl, Eggfetch, Service, and the non-publishable transport-footprint package were not published.

## Source plan and roadmap

- Plan: plans/implementation/eggpack-manifest-interoperability/009-eggup-eggpack-0.2.1-publication.md
- Roadmap: plans/subsystems/eggpack-manifest-interoperability-roadmap.md
- Registry: plans/registry.md
- Reviewed baseline: 5a04f68f60c8c3ad6392036fa314ed2749fa572c
- Release-preparation commit: 2377a949e21645913c7eec23be5960a3ff8cb1bb
- Cleanup commit: ccb2bfcbbe7645541d43bb4f5fcb801339c319ed (removed accidentally committed fixture build artifacts without rewriting history)

## Publication source and registry readback

| Item | Value |
|---|---|
| Publication source SHA | ccb2bfcbbe7645541d43bb4f5fcb801339c319ed |
| Source tree at packaging | Clean; package built from this pushed commit |
| Package version | eggup-eggpack 0.2.1 |
| Published at | 2026-11-19T21:20:01.740049Z |
| Publisher | dbowm91 (David Bowman) |
| Yanked | no |
| Registry SHA-256 | 492271d54d9529fcef74bcf354466c1971e102c261e7ceac696ea3746fded795 |
| Local candidate SHA-256 | 492271d54d9529fcef74bcf354466c1971e102c261e7ceac696ea3746fded795 |
| Downloaded artifact SHA-256 | 492271d54d9529fcef74bcf354466c1971e102c261e7ceac696ea3746fded795 |
| Compressed registry artifact size | 39,726 bytes |
| Cargo provenance | .cargo_vcs_info.json records SHA ccb2bfcbbe7645541d43bb4f5fcb801339c319ed and path crates/eggup-eggpack |
| Registry publisher authorization | cargo owner --list reported dbowm91 (David Bowman); explicit exact-package upload confirmation was obtained immediately before the single upload |

The crates.io version endpoint reports 0.2.1 present, unyanked, and published by dbowm91. The prior 0.2.0 version remains present and unyanked, with its original exact Eggup requirements at =0.2.0 and producer pin at =0.1.0. No version was overwritten or yanked.

## Published manifest and package contents

The normalized packaged Cargo.toml and crates.io dependency endpoint agree on four normal dependencies: eggpack-manifest =0.1.2 and eggup-acquisition, eggup-archive, and eggup-core each =0.2.1. There are no path or Git edges and no unintended runtime dependency. The registry endpoint also lists the package's six test-only development dependencies: flate2, serde, serde_json, sha2, tar, and zip.

The package inventory contains exactly 20 files:

- .cargo_vcs_info.json, CHANGELOG.md, Cargo.lock, Cargo.toml, Cargo.toml.orig, README.md, and src/lib.rs;
- tests/archive_handoff.rs, tests/caller_destinations.rs, tests/digest_reuse.rs, and tests/interoperability.rs;
- tests/fixtures/README.md, archive-manifest.json, bundle-manifest.json, direct-manifest.json, projection-archive.json, projection-bundle.json, projection-direct.json, unknown-schema.json, and wrong-target.json.

The downloaded crates.io archive has the same SHA-256 as the locally packaged candidate; cmp also confirmed byte-for-byte identity. Extracting .cargo_vcs_info.json from the downloaded archive reports the exact publication SHA and package path shown above.

## Registry-only consumer fixture

Fixture: plans/closure/eggpack-manifest-interoperability/009-package-fixture/.

Its standalone manifest declares only eggup-eggpack =0.2.1 and owns its own lockfile. The locked Cargo metadata contains 80 package records: the local fixture plus registry packages. All five relevant packages are from registry+https://github.com/rust-lang/crates.io-index: eggup-eggpack 0.2.1, eggup-core 0.2.1, eggup-archive 0.2.1, eggup-acquisition 0.2.1, and eggpack-manifest 0.1.2. No Git or path dependency edge appears in the resolved package graph.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| Publish only eggup-eggpack 0.2.1 from the qualified clean source | cargo publish -p eggup-eggpack --locked exited 0 after the exact confirmation; Cargo reported Uploaded and Published version 0.2.1. The publication source was ccb2bfcbbe7645541d43bb4f5fcb801339c319ed. Passed. |
| Registry artifact matches the qualified candidate | crates.io checksum, local package SHA-256, and downloaded artifact SHA-256 all equal 492271d54d9529fcef74bcf354466c1971e102c261e7ceac696ea3746fded795; cmp passed. Cargo provenance matches source SHA and package path. Passed. |
| Exact normal dependency requirements are registry-resolvable | Normalized manifest and crates.io API report eggpack-manifest =0.1.2 and eggup-core/archive/acquisition =0.2.1. No path/Git source. Passed. |
| Separate registry-only consumer resolves the complete exact-pin chain | Fixture cargo check, locked check, cargo tree, and cargo metadata passed; all non-fixture packages resolve from crates.io, including the three Eggup 0.2.1 prerequisites. Passed. |
| Local and hosted qualification pass | Focused tests/clippy and the full local gate passed on the prepared candidate; package checks passed on the exact publication SHA. GitHub Actions run 44610409556 passed all five hosted jobs on that exact SHA. Passed. |
| Existing releases and tags remain unchanged | Registry readback leaves eggup-eggpack 0.2.0 unyanked and unchanged. The existing annotated v0.2.1 tag still peels to b5a989a72dc185b30cd8c0cb67b5110ed03acd86; the GitHub Release target remains that Core commit. Only the existing release body was appended. Passed. |

## Commands and platform evidence

| Check | Result |
|---|---|
| cargo test -p eggup-eggpack --locked | Passed; the focused targets reported 71 tests across the suite. |
| cargo clippy -p eggup-eggpack --all-targets --locked -- -D warnings | Passed. |
| cargo tree -p eggup-eggpack --locked | Passed; manually reviewed as an optional leaf with exact workspace pins and the producer dependency confined to Eggpack. |
| ./scripts/check-docs-truth.sh and git diff --check | Passed during release preparation; both rerun after closure edits. |
| ./scripts/check-local.sh | Passed on the prepared release candidate before the cleanup-only commit; the cleanup removed generated target artifacts only. |
| cargo package -p eggup-eggpack --list --locked | Passed on the clean source SHA; 20-file inventory recorded above. |
| cargo package -p eggup-eggpack --locked | Passed; Cargo verified packaged sources. Non-failing warning: package.homepage duplicates package.repository. |
| cargo publish --dry-run -p eggup-eggpack --locked | Passed; Cargo aborted the upload because dry-run was enabled. |
| cargo owner --list eggup-eggpack | Passed; publisher identity dbowm91 (David Bowman) is listed. |
| cargo publish -p eggup-eggpack --locked | Explicitly confirmed immediately before upload; one invocation exited 0 and Cargo reported Published eggup-eggpack v0.2.1. |
| crates.io version/dependency readback | Passed; 0.2.1 exists, publisher dbowm91, yanked=false, exact normal dependency set as above; 0.2.0 remains present and unyanked with its old pins. |
| Download, shasum -a 256, cmp, and tar -xzOf for .cargo_vcs_info.json | Passed; downloaded artifact byte-identical, checksum and provenance agree. |
| cargo check --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml | Passed; Cargo downloaded and compiled eggup-eggpack 0.2.1 and its registry dependencies. |
| cargo check --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml --locked | Passed. |
| cargo tree --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml --locked | Passed; graph uses the exact Eggpack and prerequisite versions. |
| cargo metadata --manifest-path plans/closure/eggpack-manifest-interoperability/009-package-fixture/Cargo.toml --locked --format-version 1 | Passed; all dependency package sources are crates.io registry sources; the fixture itself is the sole local package. |
| gh release view v0.2.1 --repo eggstack/eggup --json tagName,targetCommitish,url,body | Passed after appending M009 evidence; targetCommitish remains b5a989a72dc185b30cd8c0cb67b5110ed03acd86. |
| git ls-remote origin refs/tags/v0.2.1 'refs/tags/v0.2.1^{}' | Passed; annotated tag object 3e6ecae9158f834b66a56cd7a9d8776f22330652 still peels to b5a989a72dc185b30cd8c0cb67b5110ed03acd86. |

Hosted run: https://github.com/eggstack/eggup/actions/runs/44610409556, head SHA ccb2bfcbbe7645541d43bb4f5fcb801339c319ed; all five jobs succeeded: Stable checks, MSRV check, Linux systemd service qualification, macOS tests, and Windows archive/acquisition/service checks.

## Invariant, compatibility, failure, and security review

- No Rust production source, public API, runtime behavior, workspace dependency declaration, or workspace Cargo.lock changed. The exact pins were already present on the source candidate; M009 promoted that candidate as one registry package.
- The published Eggpack adapter remains an optional leaf. Only eggup-eggpack depends on eggpack-manifest. The consumer fixture proves the exact dependency graph without workspace path overrides.
- Published eggup-curl, eggup-eggfetch, and eggup-service 0.2.0 use compatible ^0.2 requirements and were not republished. eggup-transport-footprint remains publish=false and was not published.
- eggup-eggpack 0.2.0 remains immutable with its original exact =0.2.0 Eggup pins and eggpack-manifest =0.1.0. Consumers choosing 0.2.1 receive the new exact 0.2.1 Eggup set and eggpack-manifest 0.1.2. The adapter API and behavior are unchanged; no migration is required.
- Publication is irreversible at the version level. The authorized upload ran once and succeeded; registry readback and artifact comparison resolved the outcome. No retry, overwrite, yank, tag movement, or release-target change occurred.
- SHA-256 is package-integrity and artifact-identity evidence only; no authenticity or signature claim is made. No credentials or secrets were added to logs or release files.
- No M009-specific unresolved finding remains at medium severity or higher. Unrelated open findings remain tracked in plans/registry.md.

## Documentation and registry disposition

M009 is closed in this record, its implementation plan, the Eggpack roadmap, and the authoritative milestone table in plans/registry.md. The Eggpack and root changelogs, docs/releases.md, architecture overview and governance release records, and applicable publication-status skills now include the verified 0.2.1 release. The existing GitHub Release v0.2.1 body records Eggpack publication evidence while its Core target and the annotated v0.2.1 tag remain unchanged.
