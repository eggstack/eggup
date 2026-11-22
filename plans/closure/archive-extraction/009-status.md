# Archive Extraction M009 — Closure and Verification Record

## Status

**Status: closed 2026-11-22; published; registry-verified; hosted-qualified.** Archive M009 published only `eggup-archive 0.2.2` as the exact-pin prerequisite for Eggpack M010. The registry archive checksum matches the locally qualified package, Cargo VCS provenance identifies the qualified source SHA, the permanent registry-only consumer resolves and compiles the exact release, and all five hosted CI lanes passed on a post-publication rerun at the same source SHA.

No production source, API, extraction behavior, or runtime dependency changed. Archive 0.2.1 remains available and unyanked. The default-parallel local gate attempt observed during this release work ended with a fake-curl test child receiving signal 9 (`exit unknown`); the cause was not established, so that invocation is not claimed as passing. The complete serialized gate passed. No tag, GitHub Release, PR, yank, or other package publication was performed for M009.

## Source plan and roadmap

- Plan: `plans/implementation/archive-extraction/009-eggup-archive-0.2.2-exact-pin-publication.md`
- Roadmap: `plans/subsystems/archive-extraction-roadmap.md`
- Affected handoff: Eggpack M010; Archive 0.2.2 is now registry-visible for its exact pin.
- Registry: `plans/registry.md`
- Applicable ADRs: ADR-0002, ADR-0004, ADR-0005.

## Reviewed baseline and publication source

| Item | Value |
|---|---|
| M009 plan baseline | `3be567e9839b555d7d9d5e6e43f14de4a13c901d` |
| Publication source SHA | `4e8779ed37ca35f3f11857dbcc4aa008e386bf21` (branch `m022-workspace-0.2.2-release-readiness`) |
| Workspace source version | `0.2.2` |
| Archive source/manifest/dependency diff from Archive M008 | None. Workspace Cargo.lock reflects the shared package-version promotion to 0.2.2 and M021's unrelated eggfetch-core/eggfetch-http-connect 0.4.0 update; Archive's dependency graph is unchanged. |
| Candidate hosted CI | Run `44982971199` on the exact publication source SHA; all five lanes passed. |
| Post-publication hosted CI | [run 44982971199, attempt 2](https://github.com/eggstack/eggup/actions/runs/44982971199), same source SHA; all five lanes passed. |

The source candidate commit was clean and pushed before publication. The registry artifact's `.cargo_vcs_info.json` records the same Git SHA and `path_in_vcs = "crates/eggup-archive"`. This closure and the registry-only fixture are follow-up evidence; neither changes the immutable published archive.

## Authorization, publication, and registry readback

The user authorized the selective release sequence. Immediately before upload, the exact crates.io endpoint for Archive 0.2.2 returned 404 and Archive 0.2.1 was unyanked. `cargo publish -p eggup-archive --locked` succeeded once; Cargo reported `Published eggup-archive v0.2.2`. No retry was made.

| Registry fact | Readback |
|---|---|
| Exact version | [eggup-archive 0.2.2 crates.io API](https://crates.io/api/v1/crates/eggup-archive/0.2.2); `yanked=false` |
| Publisher and publication time | `dbowm91` (David Bowman); `2026-11-22T04:02:25.772358Z` |
| Registry checksum | `67ab697e3af41b3cd3d1ed140f821c68739ec446d2a7782fa26ca4960edc882b` |
| Local qualified package SHA-256 | `67ab697e3af41b3cd3d1ed140f821c68739ec446d2a7782fa26ca4960edc882b` |
| Cargo VCS provenance | `.cargo_vcs_info.json` records SHA `4e8779ed37ca35f3f11857dbcc4aa008e386bf21` and path `crates/eggup-archive`. |
| Previous Archive version | `0.2.1` remained unyanked at readback. |
| Published normal dependencies | `flate2 ^1.1.10` (`rust_backend`), `fs_at ^0.2.1`, `sha2 ^0.10.9`, `tar ^0.4.46`, and `zip ^9.6.1` (`deflate-flate2`); no Eggup runtime dependency. |
| Registry dependency evidence | [crates.io dependency readback](https://crates.io/api/v1/crates/eggup-archive/0.2.2/dependencies) lists exactly the five normal dependencies above. |

The downloaded registry `.crate` and `target/package/eggup-archive-0.2.2.crate` have the same SHA-256, equal to the crates.io checksum. The registry-only lockfile also records that checksum. The registry record confirms publisher, timestamp, and unyanked state; the crate archive provenance confirms the source SHA.

## Package inventory and normalized dependency surface

`cargo package -p eggup-archive --list --locked` returned exactly:

```text
.cargo_vcs_info.json
CHANGELOG.md
Cargo.lock
Cargo.toml
Cargo.toml.orig
README.md
src/lib.rs
```

The crates.io dependency endpoint and normalized published package agree on the five normal dependencies listed above. The workspace-only `eggup-core` edge remains dev-only and is stripped from the published manifest; there is no Eggup runtime edge, path/Git dependency, or patch source. No dependency or production code changed from M008.

## Requirement → evidence matrix

| Requirement | Evidence and result |
|---|---|
| Publish only Archive 0.2.2 from the qualified source | Only `eggup-archive 0.2.2` was uploaded from `4e8779ed37ca35f3f11857dbcc4aa008e386bf21`. Cargo reported success; crates.io readback verified the version. **Passed.** |
| Registry artifact and Cargo provenance agree with the qualified package | Local and downloaded registry archive SHA-256 both equal `67ab697e3af41b3cd3d1ed140f821c68739ec446d2a7782fa26ca4960edc882b`; registry `.cargo_vcs_info.json` names the publication SHA and crate path. **Passed.** |
| Published package has only the expected runtime dependency surface | Registry dependency endpoint lists exactly five normal dependencies and no Eggup crate. **Passed.** |
| An independent consumer resolves Archive 0.2.2 from crates.io | Permanent fixture `plans/closure/archive-extraction/009-package-fixture/` pins `eggup-archive = "=0.2.2"` and compiles public `ArchiveFormat` and `ArchiveLimits` values. Unlocked and locked checks pass; tree and metadata show the crates.io registry source. **Passed.** |
| Exact publication SHA passes hosted qualification, including post-publication rerun | Run `44982971199` attempt 2 completed successfully on `4e8779ed37ca35f3f11857dbcc4aa008e386bf21`. Stable, MSRV 1.89, Linux systemd, macOS, and Windows lanes all passed. **Passed.** |
| Local checks and package qualification are honestly recorded | `RUST_TEST_THREADS=1 ./scripts/check-local.sh` passed before publication; package inventory, `cargo package`, and publish dry-run passed on the candidate. The default-parallel gate's fake-curl child ended with signal 9 (`exit unknown`); cause unknown and that invocation is not described as green. **Serialized gate passed; default-parallel invocation unresolved.** |
| Compatibility, safety, and release scope stay unchanged | Archive source/API/extraction/runtime dependency surface is unchanged; no consumer migration is required. SHA-256 remains integrity evidence only, not authenticity/signature evidence. No existing version was overwritten/yanked; no tag, GitHub Release, or PR was changed. **Passed.** |

## Commands and results

| Command/check | Result |
|---|---|
| `RUST_TEST_THREADS=1 ./scripts/check-local.sh` | Passed before publication: format, Clippy, workspace tests, docs, dependency resolution/review, and documentation-truth checks. |
| Default `./scripts/check-local.sh` invocation observed during this release work | Failed: fake-curl child received signal 9 (`exit unknown`); cause was not established. Not claimed as passed. |
| `cargo package -p eggup-archive --list --locked` | Passed; seven-file inventory above. |
| `cargo package -p eggup-archive --locked` | Passed on the candidate; local package digest matches the registry digest. |
| `cargo publish -p eggup-archive --locked --dry-run` | Passed before upload. |
| `cargo publish -p eggup-archive --locked` | Passed once; Cargo reported the package published. |
| `cargo check --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml` | Passed; downloaded the registry version and generated the fixture lockfile. |
| `cargo check --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml --locked` | Passed. |
| `cargo tree --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml --locked` | Passed; resolves `eggup-archive 0.2.2` from the crates.io registry, with only its expected dependency tree. |
| `cargo metadata --manifest-path plans/closure/archive-extraction/009-package-fixture/Cargo.toml --locked --format-version 1` | Passed; metadata identifies `registry+https://github.com/rust-lang/crates.io-index` as Archive's source. |
| `gh run watch 44982971199 --repo eggstack/eggup --exit-status` | Passed after the post-publication rerun; run attempt 2 and all five lanes succeeded on the publication source SHA. |

## Compatibility, security, and handoff

- The release is a package-version promotion required by Eggpack M010's exact Archive pin. It changes no production source, public API, extraction policy, runtime dependency, or consumer migration requirement.
- The published manifest continues the established local, bounded archive-extraction boundary. There is no transport, authenticity, or live-installation policy. SHA-256 is package-integrity evidence only.
- Archive M009 is closed and satisfies Eggpack M010's Archive prerequisite. Acquisition M023 is next and must publish Acquisition before Eggfetch; Eggpack M010 remains last.
- All prior published Archive versions remain immutable and unyanked. Curl, Service, and `eggup-transport-footprint` were not published. No tags, GitHub Releases, or PRs were changed.
- Release notes and the plan, registry, Archive/Eggpack/Acquisition roadmaps, architecture release snapshot, documentation-hygiene status, and consumer release guide were reconciled after registry readback.
