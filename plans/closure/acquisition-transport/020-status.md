# Acquisition Transport M020 — Closure and Verification Record

## Status

**Closed; registry-verified; hosted-qualified.** Exactly eggup-eggfetch 0.2.1 was published from the hosted-qualified source commit. The registry version is not yanked, its archive checksum matches both the local package and the downloaded crates.io archive, Cargo provenance records the publication SHA, and the standalone registry-only consumer compiles against eggfetch-core 0.3.0.

Only eggup-eggfetch 0.2.1 was published. No other crate was published or yanked, and no tag was created or moved.

## Source plan, roadmap, and registry

- Plan: plans/implementation/acquisition-transport/020-eggup-eggfetch-0.2.1-publication.md
- Roadmap: plans/subsystems/acquisition-transport-roadmap.md
- Registry: plans/registry.md
- Predecessor: Acquisition M019, closed and published 2026-11-19.

## Baseline and publication source

| Item | Value |
|---|---|
| Plan baseline | bf63aaefab0bcb84c134e831455c065df0dd4c76 |
| Publication source SHA | 2a33cfe4712dfb88a823614ee6f6a439ecdf792e |
| Package | eggup-eggfetch 0.2.1 |
| Registry publication time | 2026-11-20T08:01:16.635232Z |
| Publisher | dbowm91 (David Bowman) |
| Yanked | no |
| Registry SHA-256 | 2ca53436f6f12ca3ae0a9d194e81a6af6bf04fadf2d90b49d866359ab9eab2dc |
| Locally packaged SHA-256 | 2ca53436f6f12ca3ae0a9d194e81a6af6bf04fadf2d90b49d866359ab9eab2dc |
| Downloaded crates.io archive SHA-256 | 2ca53436f6f12ca3ae0a9d194e81a6af6bf04fadf2d90b49d866359ab9eab2dc |
| Cargo provenance | .cargo_vcs_info.json records SHA 2a33cfe4712dfb88a823614ee6f6a439ecdf792e and path crates/eggup-eggfetch |
| Artifact size | 24,509 bytes (registry metadata) |

The package inventory contains eight files: .cargo_vcs_info.json, CHANGELOG.md, Cargo.lock, Cargo.toml, Cargo.toml.orig, README.md, examples/eggfetch_fetch.rs, and src/lib.rs.

## Published dependency metadata

The crates.io dependency endpoint reports the following normal dependencies:

| Dependency | Requirement | Enabled features |
|---|---|---|
| eggfetch-core | ^0.3.0 | http1, tls-rustls, tls-native-roots, proxy; default features disabled |
| eggup-acquisition | ^0.2 | defaults |
| futures-util | ^0.3 | alloc; default features disabled |
| tokio | ^1 | rt, time, fs, io-util, net; default features disabled |

The package also records dev-dependencies on tokio ^1 (rt/time/net/macros/io-util/fs) and url ^2. The published graph contains no Curl adapter edge. The registry metadata reports edition 2021 and Rust 1.89.

## Hosted and local qualification

The exact publication SHA 2a33cfe4712dfb88a823614ee6f6a439ecdf792e was pushed to origin/main before publication. GitHub Actions run [44698883796](https://github.com/eggstack/eggup/actions/runs/44698883796) completed successfully on that exact SHA in all five required lanes:

- Stable checks
- Rust 1.89 MSRV check
- Linux systemd service qualification
- macOS tests
- Windows archive, acquisition, and service tests/check

On the publication source, the focused adapter tests and Clippy, ./scripts/check-local.sh, ./scripts/check-docs-truth.sh, package inventory, package verification, and cargo publish --dry-run -p eggup-eggfetch --locked passed. The dry-run stopped before upload; the one registry publication was the explicit cargo publish -p eggup-eggfetch --locked after all five hosted lanes passed.

## Registry-only consumer fixture

Fixture: plans/closure/acquisition-transport/020-package-fixture. Its only declared dependency is eggup-eggfetch = "=0.2.1", and it constructs the public transport/config types without making a network request.

- Unlocked check generated the standalone lockfile and compiled the downloaded registry package; locked check passed.
- cargo tree resolved eggup-eggfetch 0.2.1, eggup-acquisition 0.2.1, and eggfetch-core 0.3.0.
- cargo metadata reports 110 registry packages and no non-registry dependency packages; the fixture itself is the sole local workspace root. No path, Git, or patch dependency edge was found.
- The lockfile checksums for eggup-eggfetch 0.2.1 and eggup-acquisition 0.2.1 match crates.io metadata; the selected eggfetch-core package is exactly 0.3.0.

## Post-publication reverse dependencies

The crates.io reverse-dependencies endpoint reported four published dependents when queried after publication:

| Dependent crate | Version | Declared requirement |
|---|---:|---|
| eggsact | 1.2.7 | ^0.1.0 |
| cargo-cleanme | 0.2.15 | ^0.1.2 |
| stegoeggo-cli | 0.5.0 | ^0.1.0 |
| eggpass | 0.1.2 | ^0.2.0 |

Only the eggpass ^0.2.0 range accepts Eggup Eggfetch 0.2.1. The three ^0.1.x dependents do not accept it; no downstream crate was changed or republished by M020. Registry source: https://crates.io/api/v1/crates/eggup-eggfetch/reverse_dependencies

## Existing tag and GitHub Release

The annotated v0.2.1 tag still peels to Core source SHA b5a989a72dc185b30cd8c0cb67b5110ed03acd86. The existing GitHub Release v0.2.1 retains that same target; M020 publication notes were appended to its body only. No tag was created or moved.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| Upgrade only the adapter's upstream HTTP dependency | Published dependency endpoint and fixture lockfile resolve eggfetch-core 0.3.0; four existing HTTP features remain enabled. Passed. |
| Publish exactly eggup-eggfetch 0.2.1 from the qualified SHA | crates.io readback: version 0.2.1, publisher dbowm91, yanked=false, source SHA in Cargo provenance equals the hosted-qualified SHA. Passed. |
| Published archive matches the qualified package | Registry checksum, local archive, and downloaded archive all equal 2ca53436f6f12ca3ae0a9d194e81a6af6bf04fadf2d90b49d866359ab9eab2dc. Passed. |
| Independent consumer resolves from crates.io | Fixture's exact pin, locked check, tree, and metadata confirm registry-only resolution with no non-registry dependency edges. Passed. |
| All required hosted lanes pass on publication SHA | Run [44698883796](https://github.com/eggstack/eggup/actions/runs/44698883796), exact head 2a33cfe4712dfb88a823614ee6f6a439ecdf792e: all five lanes green. Passed. |
| Existing tag/release target and other package versions remain untouched | Tag and release still target Core's b5a989a source; only the existing Release notes were extended. No other crate was published. Passed. |
| Report reverse dependencies from post-publication registry state | Four dependents and their declared requirements recorded above from crates.io. Passed. |

## Commands and platform evidence

| Check | Result |
|---|---|
| cargo test -p eggup-eggfetch --locked | Passed on publication source. |
| cargo clippy -p eggup-eggfetch --all-targets --locked -- -D warnings | Passed on publication source. |
| ./scripts/check-local.sh | Passed on publication source. |
| ./scripts/check-docs-truth.sh | Passed on publication source; post-publication docs were then reconciled and rechecked. |
| cargo package -p eggup-eggfetch --list --locked | Passed; the eight files listed above. |
| cargo package -p eggup-eggfetch --locked | Passed; packaged sources verified. |
| cargo publish --dry-run -p eggup-eggfetch --locked | Passed; no upload occurred during dry-run. |
| cargo check --manifest-path plans/closure/acquisition-transport/020-package-fixture/Cargo.toml | Passed; downloaded and compiled registry Eggfetch 0.2.1. |
| Same fixture check with --locked | Passed. |
| Fixture cargo tree and cargo metadata --format-version 1 | Passed; dependency graph contains registry packages only (apart from the fixture root). |
| GitHub Actions 44698883796 | Passed on exact publication SHA; Stable, MSRV, Linux systemd, macOS, and Windows all green. |
| GitHub Release and v0.2.1 target check | Passed; both still point to b5a989a72dc185b30cd8c0cb67b5110ed03acd86. |

## Closure

No unresolved M020 findings or platform claims remain. The post-publication source/docs closure changes do not alter the qualified crate source or published artifact.
