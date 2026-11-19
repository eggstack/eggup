# Archive Extraction M008 — Closure and Verification Record

## Status

**Closed; registry-verified; hosted-qualified.** Exactly eggup-archive 0.2.1 was published to crates.io, and its registry checksum, publisher, dependency list, downloaded bytes, and Cargo VCS provenance agree with the qualified package. The registry-only consumer fixture builds against the exact registry release. All five hosted CI lanes passed on publication source SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c.

No production Rust source, API, extraction policy, or normal Eggup dependency changed in this release. The M007 ZIP dependency refresh is carried by the package. The prior eggup-archive 0.2.0 remains available and unyanked. The existing v0.2.1 tag and GitHub Release still identify Core's source commit b5a989a72dc185b30cd8c0cb67b5110ed03acd86; no tag was moved or created for Archive.

## Source plan and roadmap

- Plan: plans/implementation/archive-extraction/008-eggup-archive-0.2.1-publication.md
- Roadmap: plans/subsystems/archive-extraction-roadmap.md
- Registry: plans/registry.md
- Predecessor: Archive M007, closed and hosted-qualified in run 44022660351.

## Reviewed baseline and publication source

| Item | Value |
|---|---|
| Plan baseline | b5a989a72dc185b30cd8c0cb67b5110ed03acd86 |
| Publication source SHA | 5a04f68f60c8c3ad6392036fa314ed2749fa572c |
| Package version | 0.2.1 (workspace version; no version edit in M008) |
| Publication time | 2026-11-19T18:57:56.238029Z |
| Publisher | dbowm91 (David Bowman) |
| Yanked | no |
| Registry SHA-256 | b841216d7c0495885c6fe21b1aa353a4415d04139018f68f08d76e101bf61d80 |
| Local package SHA-256 | b841216d7c0495885c6fe21b1aa353a4415d04139018f68f08d76e101bf61d80 |
| Cargo provenance | .cargo_vcs_info.json records source SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c and path crates/eggup-archive |

## Published manifest and package contents

The normalized registry manifest and crates.io dependency readback agree on five normal dependencies: flate2 ^1.1.10, fs_at ^0.2.1, sha2 ^0.10.9, tar ^0.4.46, and zip ^9.6.1 with default features disabled and deflate-flate2 enabled. No Eggup crate is a normal dependency. The only workspace Eggup edge is the dev-only eggup-core path dependency; Cargo stripped it from the published manifest, whose dev-dependencies table is empty. No path, Git, or patch source is present in the published manifest.

The published .crate contains exactly:

- .cargo_vcs_info.json
- CHANGELOG.md
- Cargo.lock
- Cargo.toml
- Cargo.toml.orig
- README.md
- src/lib.rs

The archive is 34.6 KiB compressed (170.7 KiB unpacked per Cargo's upload output); the package checksum is the authoritative byte identity.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| Publish exactly eggup-archive 0.2.1 from the qualified source | crates.io direct version readback: 0.2.1, publisher dbowm91, yanked=false, created 2026-11-19T18:57:56.238029Z; .cargo_vcs_info.json names SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c. Passed. |
| Registry artifact equals the qualified package | Downloaded crates.io .crate SHA-256 equals the local package SHA-256: b841216d7c0495885c6fe21b1aa353a4415d04139018f68f08d76e101bf61d80; byte comparison passed. Passed. |
| Published dependency graph is bounded and has no Eggup runtime edge | crates.io dependency endpoint and normalized Cargo.toml list exactly the five dependencies above; no eggup-core dependency; dev-dependencies empty. Passed. |
| Package works from an independent registry-only consumer | plans/closure/archive-extraction/008-package-fixture pins eggup-archive =0.2.1, compiles public ArchiveFormat and ArchiveLimits values, and has a Cargo.lock registry source for eggup-archive. Unlocked check generated the lock; locked check, cargo tree, and cargo metadata passed. Passed. |
| Local gates and package qualification pass on publication source | ./scripts/check-local.sh, package file-list, cargo package, and cargo publish --dry-run passed on SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c. Passed. |
| All five hosted lanes pass on publication source | CI run 44591516672 on SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c: Stable, MSRV, Linux systemd, macOS, and Windows all completed with success. Passed. |
| Existing versions and tags remain immutable | Pre-upload crates.io version list showed 0.2.0, 0.1.3, and 0.1.2 unyanked; no existing version was republished or yanked. The v0.2.1 Release still targets Core SHA b5a989a72dc185b30cd8c0cb67b5110ed03acd86. Passed. |

## Commands and platform evidence

| Check | Result |
|---|---|
| ./scripts/check-local.sh | Passed on publication source SHA 5a04f68f60c8c3ad6392036fa314ed2749fa572c: formatting, Clippy, workspace tests, docs, dependency resolution/review, and documentation-truth checks. |
| cargo package -p eggup-archive --list --locked | Passed; exact seven-file inventory above. |
| cargo package -p eggup-archive --locked | Passed; packaged sources verified. |
| cargo publish --dry-run -p eggup-archive --locked | Passed; upload was aborted by Cargo's dry-run mode. |
| cargo publish -p eggup-archive --locked | Explicitly confirmed immediately before upload; exit 0; crates.io reported Published eggup-archive v0.2.1. |
| cargo check --manifest-path plans/closure/archive-extraction/008-package-fixture/Cargo.toml | Passed; downloaded and checked registry eggup-archive 0.2.1. |
| cargo tree --manifest-path plans/closure/archive-extraction/008-package-fixture/Cargo.toml --locked | Passed; eggup-archive 0.2.1 is the registry dependency, with no path/Git source. |
| cargo check --manifest-path plans/closure/archive-extraction/008-package-fixture/Cargo.toml --locked | Passed. |
| cargo metadata --manifest-path plans/closure/archive-extraction/008-package-fixture/Cargo.toml --locked --format-version 1 | Passed; metadata names crates.io as eggup-archive's source. |

| Hosted lane | Run 44591516672 |
|---|---|
| Stable checks | Passed |
| MSRV check (Rust 1.89) | Passed |
| Linux systemd service qualification | Passed |
| macOS tests | Passed |
| Windows archive, acquisition, and service tests and check | Passed |

## Compatibility, security, and failure review

- No production source, API, runtime behavior, or extraction policy changed; the archive only carries the M007 ZIP dependency refresh.
- The dependency boundary remains local archive extraction with no transport, producer, authenticity, or live-installation policy. SHA-256 remains integrity evidence only.
- The exact Eggpack =0.2.1 archive pin is now satisfiable from crates.io. Published Eggpack 0.2.0 and its immutable =0.2.0 pins are unchanged; Eggpack publication remains separately gated on all exact-pin dependencies.
- The version is immutable. No retry, overwrite, yank, or tag move occurred. No medium-or-higher finding was introduced by M008.

## Documentation and registry disposition

The M008 plan, Archive roadmap, registry, Archive changelog, root release changelog, docs/releases.md, architecture release snapshot, and the existing v0.2.1 GitHub Release notes were reconciled after registry readback. The GitHub Release note was extended to identify Archive 0.2.1 separately while preserving Core's tag and original release source. The release plan remained limited to Archive. Acquisition M019 subsequently published `eggup-acquisition 0.2.1` from the same qualified source SHA; both exact Eggpack pins are now registry-visible, and Eggpack M009 is the next publication milestone.
