# Eggpack Manifest Interoperability M007 — Closure and Verification Record

Status: **closed** — `eggup-eggpack 0.2.0` is on crates.io, not yanked, byte-identical
to the artifact built from the publication source commit, and the exact-pin re-pin is
proven live on the registry from outside this repository. The milestone's own scope is
complete: every requirement in the plan's §§3, 4, and 8 is either evidenced below or
recorded as a partial with the reason. One **low**-severity qualification gap remains
open (F-2), and it is an evidence-completeness gap in the registry-only fixture, not a
defect in the published artifact.

Source plan: `plans/implementation/eggpack-manifest-interoperability/007-eggup-eggpack-0.2.0-publication.md`

Source roadmap: `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`

Reviewed repository baseline: `b61260857fdb3904a091c503b1dff05251ec0bff`
(short `b612608`, branch `plans/corrective-batch-2026-10-23`) — the **publication source
commit**. The plan's own baseline was `b75f73b`; the delta between them is the workspace
version bump, the adapter's three re-pins, and the release-prep material, and
`git diff b75f73b b612608 -- crates/eggup-eggpack/src` is **empty** (G12).

Publication source commit: `b61260857fdb3904a091c503b1dff05251ec0bff`. The published
`.cargo_vcs_info.json` records exactly this SHA, `path_in_vcs` `crates/eggup-eggpack`,
and no dirty-tree marker. The published `src/lib.rs` is byte-identical to the repository
file at this commit (`sha256 efc896c8ea56aaf7a089fec40ad6dd37ec7e1d204ada69c6067a55a5508e02df`
on both sides). The byte-identity holds **file by file**: `src/lib.rs`, `README.md`,
`CHANGELOG.md`, `Cargo.toml.orig`, and the four integration test files each hash
identically to their `b612608` counterparts, and the packaged `Cargo.toml` is cargo's
normalized rewrite of that same source manifest.

Hosted qualification run: **`40079833945`** — `stable`, `msrv`, `macos`,
`windows-check`, `service-systemd`, all five lanes green. Per-lane results and the
first-attempt/re-run history are in the shared evidence set
(`/tmp/eggup-release/closure-evidence.md`), which this record cites rather than
restates.

Implementation commits: the publication train's release-prep commits are `afe6201`
(the five `0.2.0` publication plans), `2161f36` (the workspace version bump to `0.2.0`,
the requirement widenings, the `eggup-eggpack` re-pin, the eight changelog cuts, the
documentation-truth gate generalisation, and the M016 publication-coupling guard
re-point), `b612608` (the train-order exception recorded in the core plan's §9.1), and
`2d6e089` (restoring the strict packaged-service step in CI, the follow-on named by
registry finding 16). No commit in this milestone changed a line under
`crates/eggup-eggpack/src/`; the only adapter change in the `0.1.3` → `0.2.0` train is
the manifest re-pin plus the M006 content that M006 itself introduced.

## Executive finding

`eggup-eggpack 0.2.0` is published to crates.io from the single publication source
commit `b612608`, and the artifact on the registry is byte-identical to the artifact
built from that commit: both the locally packaged `target/package/eggup-eggpack-0.2.0.crate`
and the copy in the local cargo registry cache hash to
`33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6` at 38981 bytes, which
is the checksum crates.io reports for `0.2.0`. The version is present and not yanked;
`0.1.2` and `0.1.3` are unchanged and not yanked; nothing was re-published or withdrawn.
The adapter's **own public API is unchanged** — the published `src/lib.rs` differs from
the published `0.1.3` source in exactly two hunks — the `hash_file` import and the
delegated digest, with an added `# Errors` rustdoc section in the second — and the
public item list is identical. What the release
changes is the **published dependency requirements**: the three Eggup exact pins move
from `=0.1.3` to `=0.2.0`, and that re-pin is the milestone's whole consumer-facing
mechanism. The registry-only fixture at `/tmp/eggup-registry-fixtures/eggpack/`, a fresh
crate outside this repository with no `path`, `git`, or `patch` source anywhere, depends
**only** on `eggup-eggpack = "=0.2.0"` and its generated lockfile resolves
`eggup-acquisition 0.2.0`, `eggup-archive 0.2.0`, `eggup-core 0.2.0`, and
`eggup-eggpack 0.2.0` — exactly one copy of each, every one from
`source = "registry+https://github.com/rust-lang/crates.io-index"`, with no `0.1.3`
anywhere in the graph. That transitive resolution is the proof the milestone exists to
produce: the re-pin reached the registry, so a consumer of `eggup-eggpack 0.2.0` receives
the fixed `eggup-core`, the fixed `eggup-archive`, and the retyped seam rather than the
`0.1.3` set. **One source break arrives through this publication, and it arrives through
the pin rather than through the adapter's own API**: the seam's `0.2.0` replaces
`AcquisitionError::Io(String)` with `AcquisitionError::Io { operation, source }` and
removes `Clone`, `PartialEq`, and `Eq`, and the adapter re-exposes the seam's request and
limits types through `PlannedAcquisition`, so a consumer of `eggup-eggpack 0.2.0`
inherits that break transitively. A consumer must not adopt `0.2.0` believing the adapter
release is source-compatible with `0.1.3`.

## Publication record

| Field | Value |
|---|---|
| Registry version | `eggup-eggpack 0.2.0` |
| Checksum (crates.io) | `33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6` |
| Locally built `.crate` sha256 | identical — `33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6` |
| Registry-cached `.crate` sha256 | identical — `33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6` |
| Crate size | 38981 bytes, locally packaged and as reported by crates.io |
| Yanked | no |
| Published by | `dbowm91` (David Bowman), the same identity that published `0.1.2` and `0.1.3` |
| Created on crates.io | `2026-10-24T13:51:33.906205Z` |
| Rust version | `1.89` |
| License | `MIT` |
| Tag | `v0.2.0`, annotated, object `aae1d2d1d033e33d3f8857cad43f42684cc88e98`, target commit `b61260857fdb3904a091c503b1dff05251ec0bff` |
| GitHub release | `0.2.0`, tag `v0.2.0`, not a draft, not a prerelease, published `2026-10-24T13:54:03Z` |
| Earlier versions | `0.1.3` (checksum `952e88b87ffd7b88c7db246e52169542abff9b01771f1086c22e86b4739619c5`, created `2026-10-06`) and `0.1.2` (checksum `9dbfdfb74b4faa88d6badaa6f24e71dada3c7322dd0e9877cebe8b0168b13fef`, created `2026-10-03`) — both present, both not yanked, both unchanged |

## Requirement → evidence matrix

Result vocabulary: **pass** — evidenced; **partial** — the requirement is evidenced by
another source than the one the plan named, and the substitution is stated; **not
applicable** — the requirement does not bind this milestone.

### §8 acceptance criteria

| # | Acceptance criterion | Evidence | Result |
|---|---|---|---|
| AC1 | `eggup-eggpack 0.2.0` is present on crates.io, not yanked, with a recorded checksum and version metadata | crates.io public API read: `0.2.0`, `yanked: false`, `cksum 33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6`, `crate_size 38981`, `published_by dbowm91`, `created_at 2026-10-24T13:51:33.906205Z`. The locally packaged and registry-cached artifacts hash to the same value | pass |
| AC2 | `0.1.2` and `0.1.3` are unchanged and not yanked; no version was re-published or yanked | Same API read: `0.1.3` `yanked: false` `cksum 952e88b8…19c5`, `0.1.2` `yanked: false` `cksum 9dbfdfb7…b13fef`; their checksums match the ones recorded before this milestone in `plans/registry.md` and in the subsystem roadmap's M004 entry. No upload other than `0.2.0` | pass |
| AC3 | The published package's dependency requirements are exactly `eggup-core = "=0.2.0"`, `eggup-archive = "=0.2.0"`, `eggup-acquisition = "=0.2.0"`, `eggpack-manifest = "=0.1.0"`, with no path, git, or patch source | Packaged normalized manifest: `[dependencies.eggup-core] version = "=0.2.0"`, `[dependencies.eggup-archive] version = "=0.2.0"`, `[dependencies.eggup-acquisition] version = "=0.2.0"`, `[dependencies.eggpack-manifest] version = "=0.1.0"`. No `path`, `git`, `rev`, `branch`, or `tag` key in any dependency entry. The in-tree `path = "../eggup-*"` development keys were rewritten to registry-only requirements by packaging | pass |
| AC4 | The published package's public API is exactly the surface of §2.3, with `AdapterError`'s variant set unchanged | *Public API boundary* below — every item verified against the published `src/lib.rs`, which is byte-identical to the repository file at `b612608`. `AdapterError` (`lib.rs:19-38`) is `#[derive(Debug)]` + `#[non_exhaustive]` with the same eight variants: `InvalidManifest`, `TargetNotFound`, `MapMismatch(&'static str)`, `CallerLimitTooSmall(String)`, `InvalidAcquiredFile(String)`, `ArchiveExtractionRequired`, `UnsupportedArchiveFormat`, `Eggup(String)` | pass |
| AC5 | The published package's runtime dependency set is the four entries of §2.5; `sha2` is dev-only and absent from the packaged runtime requirements | The packaged manifest's `[dependencies]` table carries exactly the four entries — `sha2` is **absent** from it. Precision, stated because the plan's §2.5 shorthand says the dev-dependencies are "stripped": cargo's normalized manifest **retains** a `[dev-dependencies]` table (`flate2`, `serde`, `serde_json`, `sha2`, `tar`, `zip`), because dev-dependencies are part of a crate's own manifest. What is stripped is the dev-dependency's place in the **published dependency surface**: the packaged `Cargo.lock` and the registry-only fixture's lockfile both list `eggup-eggpack 0.2.0`'s dependencies as exactly `eggpack-manifest`, `eggup-acquisition`, `eggup-archive`, `eggup-core`, and a downstream consumer resolves none of the six dev-dependencies | pass |
| AC6 | The package file list is recorded and matches §2.6 | *Package inventory* below. The artifact ships `src/lib.rs`, `README.md`, `CHANGELOG.md`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json`, the four integration test files, and `tests/fixtures/` with its eight JSON documents and README. The crate manifest declares **no `exclude` key**, and the crate has no `benches/` directory, so nothing is excluded from the package | pass |
| AC7 | `validate_acquired_archive` delegates to `eggup_core::hash_file`, the symlink and exact-size guards precede the digest call, and all three failure modes still report `AdapterError::InvalidAcquiredFile` | Published source: `hash_file` imported at `lib.rs:8-11`; `symlink_metadata` guard at `:513-514`; type and size guard at `:515-517`; the digest call at `:522-523`; the comparison at `:524-526`. The digest step's open and read failures map through `.map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?` at `:523`, and the mismatch returns the same variant at `:525`. The published-source diff against `0.1.3` shows the inline loop replaced, the guard untouched and still above the call. Fixture evidence: `tests/digest_reuse.rs` (8 tests) plus the whole adapter suite — 71 passed, 0 failed | pass |
| AC8 | The hosted matrix (`stable`, `msrv`, `macos`, `windows-check`) is green on the publication source commit, recorded per lane with the run id | Hosted run `40079833945`; per-lane table in *Per-platform results*. The adapter's tests execute in the `stable`, `macos`, and `windows-check` lanes; the `msrv` lane is a `cargo check`, so it compiles but does not execute the adapter's tests | pass |
| AC9 | The pre-upload registry-only fixture resolves exactly one `eggup-core`, one `eggup-archive`, and one `eggup-acquisition`, each at `0.2.0` from crates.io, with no path, git, or patch source anywhere in its lockfile | The pre-upload fixture instance is **not retained** in the evidence set; the surviving fixture is the post-upload one. The same assertion is satisfied by the surviving fixture's lockfile, which resolves exactly one copy of each at `0.2.0` from the registry with a single source line — see R-1 to R-4 | partial — see F-2 |
| AC10 | The pre-upload fixture independently confirms that the published `0.1.3` resolves its three dependencies at `0.1.3` and carries the direct `sha2` runtime dependency, and that the §2.3 adapter API compiles and behaves unchanged against the published `0.1.3` | Verified directly from the published `0.1.3` artifact instead: its packaged manifest declares `[dependencies.sha2] version = "0.10.9"` and pins `=0.1.3` on all three Eggup crates, and its `src/lib.rs` carries the inline `sha2` loop. The §2.3 API is identical in both published artifacts — the file-list and source diffs below | partial — see F-2 |
| AC11 | The post-upload registry-only fixture resolves exactly one `eggup-core`, one `eggup-archive`, and one `eggup-acquisition`, each at `0.2.0` from crates.io, with no path, git, or patch source anywhere in its lockfile | `/tmp/eggup-registry-fixtures/eggpack/Cargo.lock`: `eggup-acquisition 0.2.0`, `eggup-archive 0.2.0`, `eggup-core 0.2.0`, `eggup-eggpack 0.2.0`, each with `source = "registry+https://github.com/rust-lang/crates.io-index"` and a recorded `checksum`. Exactly one distinct source line in the whole lockfile; no `path =`, `git =`, or `patch` entry; the fixture's manifest has no `[patch]` section and no `.cargo/config.toml` | pass |
| AC12 | The post-upload fixture's projection, archive-plan/bound-member round trip, delegated-digest, and seam-typing assertions all pass | The surviving fixture carries **two** tests — `published_eggpack_maps_archive_names_to_formats` and `published_eggpack_bounds_the_manifest_size` — and neither exercises the projection, the archive round trip, the delegated digest, or the seam typing. Each of those four is covered by another source: the projection and archive round trip by the adapter's own shipped test files against the byte-identical published source; the delegated digest by the published-source diff plus `tests/digest_reuse.rs`; the seam typing by the published `eggup-acquisition 0.2.0` artifact's `AcquisitionError::Io { operation, source }` under `#[derive(Debug)]` only. The fixture's own two tests pass | partial — see F-2 |
| AC13 | `v0.2.0` exists, is annotated, and points at the exact publication source commit; `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved | `git cat-file -t v0.2.0` → `tag`; the tag object `aae1d2d1d033e33d3f8857cad43f42684cc88e98` names `object b61260857fdb3904a091c503b1dff05251ec0bff`, `type commit`, and the tag message names the train and this crate's checksum. `v0.1.1` → `881c95f`, `v0.1.2` → `e8e07eb`, `v0.1.3` → `bd43683` — the same commits the plan's §5 records | pass |
| AC14 | The GitHub release for `v0.2.0` covers the seven-crate train and references the crates.io release of `eggup-eggpack 0.2.0` | `gh release view v0.2.0`: name `0.2.0`, tag `v0.2.0`, draft `false`, prerelease `false`, published `2026-10-24T13:54:03Z`. Its body carries a per-package table naming all seven crates with their registry checksums — including `eggup-eggpack 0.2.0` at `33dc1da4…14f1f6` — the publication order, the "why 0.2.0 and not 0.1.4" rationale, and the three source breaks a consumer must plan for | pass |
| AC15 | The consumer-unblock reconciliation is recorded, including the transitive `eggup-acquisition 0.2.0` source break and the `AcquisitionError` typing change it carries | *Compatibility and migration review* below, and the lockfile evidence in R-1 to R-4 | pass |
| AC16 | The changelog's publication wording is either true because the upload was accepted, or corrected by its owner to record the unpublished state | `crates/eggup-eggpack/CHANGELOG.md` § `0.2.0` asserts "Published to crates.io (`plans/closure/eggpack-manifest-interoperability/007-status.md`)" and § `Unreleased` states that "`0.2.0` is the published baseline on crates.io". Both statements are true: the upload was accepted at `2026-10-24T13:51:33Z`. The published artifact's section heading reads `## 0.2.0 — 2026-10-23` while the repository's copy now reads `## 0.2.0 — 2026-10-24` — a one-line, permanent divergence between immutable registry bytes and the repository — see F-4 | pass |

### §3 pre-publication gates

| # | Requirement | Evidence | Result |
|---|---|---|---|
| G1 | Registry preflight: `0.2.0` absent before the upload; `0.1.2` and `0.1.3` present and unyanked; nothing re-published or yanked | The plan's §1 records the preflight read: `eggup-eggpack` held `0.1.2` and `0.1.3`, neither yanked, and `0.2.0` absent. After the upload the API read shows the same two older versions with unchanged checksums and `yanked: false`, plus the new `0.2.0` | pass |
| G2 | Publish authority: the identity that published `0.1.2` and `0.1.3` holds the authority for `0.2.0` | crates.io reports `published_by.login = "dbowm91"` for `0.2.0`, the same identity named in the plan's G2 | pass |
| G3 | Version identity: the publication source commit carries the workspace bump to `0.2.0` and the adapter inherits it | `Cargo.toml` at `b612608`: `[workspace.package] version = "0.2.0"`; `crates/eggup-eggpack/Cargo.toml:3` `version.workspace = true`; the packaged manifest's `version = "0.2.0"`. The `git diff b75f73b b612608 -- Cargo.toml` shows the `0.1.3` → `0.2.0` bump as the sole change to the root manifest | pass |
| G4 | Exact-pin identity: the packaged manifest carries the three `=0.2.0` pins and `eggpack-manifest = "=0.1.0"`, with no path, git, or patch source | Packaged normalized manifest, quoted under AC3 | pass |
| G5 | Dependency-order gate: `eggup-core 0.2.0`, `eggup-archive 0.2.0`, and `eggup-acquisition 0.2.0` are all registry-visible before the upload | The shared evidence set's registry-state table records all three at `0.2.0` with no yank; `cargo package -p eggup-eggpack --locked` resolved the three exact pins from the registry, which is only possible once they are visible. The packaged `Cargo.lock` carries a registry `checksum` for each | pass |
| G6 | Public API surface: the §2.3 items are present and unchanged in the packaged source, and `AdapterError`'s variant set is unchanged | *Public API boundary* below | pass |
| G7 | Dependency surface: the runtime dependency set is exactly the four entries of §2.5; `sha2` is absent from `[dependencies]` and present in `[dev-dependencies]` | The packaged manifest's `[dependencies]` table has exactly four entries and no `sha2`; `[dev-dependencies.sha2] version = "0.10"` is present; the published dependency surface carries no dev-dependency — see the precision note under AC5 | pass |
| G8 | Package contents: the package file list is recorded and checked against the actual `.crate` payload | *Package inventory* below, read from the published artifact's tarball directory, not from an expectation | pass |
| G9 | Digest-reuse content: `validate_acquired_archive` calls `eggup_core::hash_file`; the symlink and exact-size guards precede the digest call; open, read, and mismatch failures all map to `AdapterError::InvalidAcquiredFile` | Published-source line evidence under AC7, plus the published-source diff against `0.1.3` | pass |
| G10 | Hosted matrix green on the publication source commit | Hosted run `40079833945`, all five lanes green — see *Per-platform results* | pass |
| G11 | Changelog truthfulness | See AC16. The wording stands because the upload was accepted, and the date header has since been corrected by the changelog's owner (F-4) | pass |
| G12 | No unclassified source delta: no file under `crates/eggup-eggpack/src/**` differs from the qualified baseline | `git diff b75f73b b612608 -- crates/eggup-eggpack/src` → **empty** (exit 0, no output). The only adapter change in the train is the manifest re-pin: `git diff b75f73b b612608 -- crates/eggup-eggpack/Cargo.toml` is three changed lines, `=0.1.3` → `=0.2.0` on the three Eggup edges | pass |
| G13 | Maintainer authorization: the upload happened only on explicit maintainer authorization naming `eggup-eggpack 0.2.0`, after every pre-upload gate held | The upload was maintainer-authorized and is on the registry; the registry-only fixture was validated before publication against the local sources in a scratch copy, which is recorded in the shared evidence set. The pre-upload fixture **instance** was not retained, so its exact assertion set is not re-verifiable from the surviving evidence — see F-2 | pass, with F-2 |
| G14 | Train completion: the six preceding crates are published and registry-visible at `0.2.0`, and the `v0.2.0` tag and release cover the whole train | The shared evidence set's registry-state table records all seven crates at `0.2.0`, none yanked; the `v0.2.0` tag message enumerates all seven with their checksums; the GitHub release body carries the same seven-package table | pass |

### §4 registry-only proof

Fixture location: `/tmp/eggup-registry-fixtures/eggpack/` — a crate outside this
repository, with its own `[workspace]` table, `publish = false`, and **no**
`path`, `git`, or `patch` source in its manifest, its workspace table, or any
`.cargo/config.toml` (there is no `.cargo` directory). Its manifest declares one
dependency and one only:

```toml
[dependencies]
eggup-eggpack = "=0.2.0"
```

That single direct requirement is what makes the transitive resolution observable: cargo
permits more than one version of the same crate in one graph, so if the adapter's pin
were still `=0.1.3` the lockfile would carry a second copy of each dependency rather
than the `0.2.0` one.

| # | Required assertion | Evidence | Result |
|---|---|---|---|
| P-1 | The train's front resolves at `0.2.0`: exactly one `eggup-core`, one `eggup-archive`, and one `eggup-acquisition`, each at `0.2.0` from crates.io, with no path, git, or patch source and no `[patch]` section | The surviving post-upload fixture's lockfile carries exactly one entry of each, all at `0.2.0`, all with `source = "registry+https://github.com/rust-lang/crates.io-index"` and a recorded `checksum`. The pre-upload instance is not retained — see F-2 | partial — see F-2 |
| P-2 | The published `0.1.3` carries the defect this milestone removes: it resolves its three dependencies at `0.1.3` and declares a direct `sha2` runtime dependency | Read from the published `0.1.3` artifact itself: the packaged manifest declares `[dependencies.sha2] version = "0.10.9"` and `[dependencies.eggup-core] version = "=0.1.3"` (and the same for archive and acquisition); its `src/lib.rs` carries the inline `sha2` streaming loop | partial — asserted against the published artifact rather than against the pre-upload fixture; see F-2 |
| P-3 | The adapter's public API is unchanged: the fixture compiles against the published adapter using exactly the §2.3 items and exercises the manifest projection and the `archive_plan_for`/`bind_archive_members` round trip | The published `0.1.3` and `0.2.0` `src/lib.rs` files differ in exactly two hunks — the `hash_file` import and the delegated digest, with the added `# Errors` rustdoc section in the second — and the public item list is identical in both. The fixture's two surviving tests exercise `archive_format_for_name` and `MAX_MANIFEST_BYTES` from the published `0.2.0` | partial — the API identity is proven by the published-source diff, the projection and round-trip exercises are not in the surviving fixture; see F-2 |
| R-1 | The post-upload lockfile contains exactly one `eggup-core` node, at `0.2.0`, from crates.io | `/tmp/eggup-registry-fixtures/eggpack/Cargo.lock`: `name = "eggup-core"`, `version = "0.2.0"`, `source = "registry+https://github.com/rust-lang/crates.io-index"`, `checksum = "2c86a0ebc94c75465855f661cd3aae91447f53fdb38e6f277092b155dbeb23b8"`. One entry only; no `0.1.3` copy anywhere in the lockfile | pass |
| R-2 | The post-upload lockfile contains exactly one `eggup-archive` node, at `0.2.0`, from crates.io | Same lockfile: `eggup-archive 0.2.0`, registry source, `checksum = "ce35edd5d2246f91a0b98a7eddbecc25ad5e944c1e92193887e6490e11ff57ad"`. One entry only | pass |
| R-3 | The post-upload lockfile contains exactly one `eggup-acquisition` node, at `0.2.0`, from crates.io | Same lockfile: `eggup-acquisition 0.2.0`, registry source, `checksum = "2ef74b4dce2b9f861b2759a45a5bc96377d8273892b04d45843c2f9348299fc3"`. One entry only | pass |
| R-4 | No entry anywhere in the lockfile carries a path, git, or patch source, and no `[patch]` section exists | The lockfile's distinct `source =` values are exactly one line — `source = "registry+https://github.com/rust-lang/crates.io-index"` — and it contains no `path =`, `git =`, `patch`, or `0.1.3` token. The fixture's manifest has no `[patch]` section and there is no `.cargo/config.toml` in the fixture tree | pass |
| R-5 | Behavioural: manifest projection — build a `ReleaseManifest`, project it with `project_json`, and assert the exact member set, archive size, and digest bytes; assert `install_ids` and `default_destinations` agree | Not asserted by the surviving fixture. Covered in-tree against the byte-identical published source by `tests/interoperability.rs` (28 tests) and `tests/caller_destinations.rs` (15 tests), green on every platform lane of run `40079833945` | partial — see F-2 |
| R-6 | Behavioural: archive plan and bound-member round trip — `archive_format_for_name` → `archive_plan_for` → extract → persist → `into_bound_sources` → `into_members` → `bind_archive_members` → `core_plan_for_archive_with_destinations`, with the bound member count equal to the declared count and a projection mismatch failing closed | Not asserted by the surviving fixture. Covered in-tree against the byte-identical published source by `tests/archive_handoff.rs` (13 tests), green on every platform lane | partial — see F-2 |
| R-7 | Behavioural: delegated digest — `validate_acquired_archive` succeeds on a correctly authored archive and fails with `AdapterError::InvalidAcquiredFile` after a one-byte tamper, exercising the published `eggup-core 0.2.0`'s `hash_file` **through** the adapter's exact pin | Not asserted by the surviving fixture. The same code path is covered in-tree by `tests/digest_reuse.rs` (8 tests) against the byte-identical published source, and the delegation itself is proven by the published-source diff (`lib.rs:522-523`) | partial — see F-2 |
| R-8 | Behavioural: seam typing — the seam's retyped error is observable through the adapter's re-exposed seam types, so the fixture compiles against the published `eggup-acquisition 0.2.0` and not against a stale `0.1.3` | Not asserted by the surviving fixture. Verified from the published `eggup-acquisition 0.2.0` artifact instead: `pub enum AcquisitionError` is `#[derive(Debug)]` + `#[non_exhaustive]` with the variant `Io { operation, source }` and the rustdoc recording that `Clone`, `PartialEq`, and `Eq` are deliberately absent. The published adapter re-exposes the seam's types at `lib.rs:8` (`use eggup_acquisition::{AcquisitionRequest, FetchLimits}`) and `lib.rs:310-323` (`PlannedAcquisition { request, limits, .. }`) | partial — see F-2 |

**Count.** 41 requirement→evidence rows: 16 acceptance criteria, 14 pre-publication
gates, 11 registry-only proof assertions. 31 pass, 10 partial. Every partial row is
covered by an alternative source that is stated in the row, and every partial row is
covered by the same finding (F-2).

## Production implementation evidence

### The published source delta, `0.1.3` → `0.2.0`

The published `0.1.3` was built from `bd4368369aaf0ae1de2ce213fc16689281bdd664`; the
published `0.2.0` from `b61260857fdb3904a091c503b1dff05251ec0bff`. Diffing the two
published `src/lib.rs` files gives exactly two hunks: one that adds `hash_file` to the
`eggup_core` import list, and one that adds the `# Errors` rustdoc section to
`validate_acquired_archive` and replaces its inline digest loop with the delegated
call. The whole behavioural delta:

```diff
@@ -6,8 +6,8 @@
 use eggup_acquisition::{AcquisitionRequest, FetchLimits};
 use eggup_archive::{ArchiveFormat, ArchiveLimits, ArchiveMember, ArchivePlan, BoundMember};
 use eggup_core::{
-    ArtifactMember, ArtifactSet, BoundSources, InstallPlan, IntegrityRequirement, MemberId,
-    PermissionsIntent, ProductId, ReleaseId,
+    hash_file, ArtifactMember, ArtifactSet, BoundSources, InstallPlan, IntegrityRequirement,
+    MemberId, PermissionsIntent, ProductId, ReleaseId,
 };
@@ -478,12 +478,26 @@
 pub fn validate_acquired_archive(
     projection: &ManifestProjection,
     acquired_archive_path: &Path,
 ) -> Result<(), AdapterError> {
-    use sha2::{Digest, Sha256};
-    use std::io::Read;
     …
     if !meta.file_type().is_file() || meta.len() != exact_size {
         return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
     }
-    let mut file = fs::File::open(acquired_archive_path)
+    // The guard above runs first: `hash_file` follows symlinks when it opens
+    // the path, so this is the only link rejection on the path. The digest
+    // step's own failures collapse into the same variant and payload as the
+    // mismatch below, preserving the adapter's published error surface.
+    let digest: [u8; 32] = hash_file(acquired_archive_path)
         .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
-    let mut hasher = Sha256::new();
-    let mut chunk = [0u8; 16 * 1024];
-    loop {
-        let read = file
-            .read(&mut chunk)
-            .map_err(|_| AdapterError::InvalidAcquiredFile(artifact_name.clone()))?;
-        if read == 0 {
-            break;
-        }
-        hasher.update(&chunk[..read]);
-    }
-    let digest: [u8; 32] = hasher.finalize().into();
     if digest != sha256 {
         return Err(AdapterError::InvalidAcquiredFile(artifact_name.clone()));
     }
```

Four facts hold, and each is read off the published bytes rather than asserted:

1. **The guard is untouched and still above the digest call.** `symlink_metadata` at
   `lib.rs:513-514`, the type and size guard at `:515-517`, the digest at `:522-523`.
   The guard lines are context in the diff, not changed lines.
2. **The error mapping is applied, not bypassed.** The digest step's open and read
   failures are consumed by the `.map_err` closure and reported as
   `AdapterError::InvalidAcquiredFile(artifact_name.clone())` — the same variant and
   payload the digest mismatch reports at `:525`. `AdapterError` implements no
   `From<eggup_core::Error>`, so there is no path by which the core error could fall
   through into another variant.
3. **The comparison stays in the adapter.** `hash_file` returns a digest; the comparison
   against the manifest digest is still the adapter's own, at `:524-526`.
4. **The digest value is unchanged.** Both routines are SHA-256 over the same byte
   stream; the chunk size changed (the adapter's 16 KiB → core's 64 KiB) and SHA-256
   over a byte stream is chunk-independent.

The file-list delta between the two published packages is a single added file:

```text
> ./tests/digest_reuse.rs
```

The `README.md` delta is one paragraph, recording the runtime dependency set, the
delegated digest, and the dev-dependency-only status of `sha2`.

### The re-pin

The adapter's manifest changed in exactly three lines:

```diff
 [dependencies]
-eggup-core = { version = "=0.1.3", path = "../eggup-core" }
-eggup-archive = { version = "=0.1.3", path = "../eggup-archive" }
-eggup-acquisition = { version = "=0.1.3", path = "../eggup-acquisition" }
+eggup-core = { version = "=0.2.0", path = "../eggup-core" }
+eggup-archive = { version = "=0.2.0", path = "../eggup-archive" }
+eggup-acquisition = { version = "=0.2.0", path = "../eggup-acquisition" }
 eggpack-manifest = "=0.1.0"
```

`eggpack-manifest` stays at `=0.1.0`. It is an external crate, it is not republished
here, and its version does not change.

The crate's own test suite asserts the pin invariant without hardcoding a literal: the
`adapter_pins_registry_eggpack_manifest_and_no_producer_crates` fixture derives
`= {env!("CARGO_PKG_VERSION")}` and requires `eggup-core`, `eggup-archive`, and
`eggup-acquisition` to carry it, so "pinned to *our* version" survives the next
workspace bump. That fixture is green on every platform lane of run `40079833945`.

### Package inventory

`tar -tzf target/package/eggup-eggpack-0.2.0.crate` — the published payload, read from
the artifact, not from an expectation:

```text
eggup-eggpack-0.2.0/.cargo_vcs_info.json
eggup-eggpack-0.2.0/CHANGELOG.md
eggup-eggpack-0.2.0/Cargo.lock
eggup-eggpack-0.2.0/Cargo.toml
eggup-eggpack-0.2.0/Cargo.toml.orig
eggup-eggpack-0.2.0/README.md
eggup-eggpack-0.2.0/src/lib.rs
eggup-eggpack-0.2.0/tests/archive_handoff.rs
eggup-eggpack-0.2.0/tests/caller_destinations.rs
eggup-eggpack-0.2.0/tests/digest_reuse.rs
eggup-eggpack-0.2.0/tests/interoperability.rs
eggup-eggpack-0.2.0/tests/fixtures/README.md
eggup-eggpack-0.2.0/tests/fixtures/archive-manifest.json
eggup-eggpack-0.2.0/tests/fixtures/bundle-manifest.json
eggup-eggpack-0.2.0/tests/fixtures/direct-manifest.json
eggup-eggpack-0.2.0/tests/fixtures/projection-archive.json
eggup-eggpack-0.2.0/tests/fixtures/projection-bundle.json
eggup-eggpack-0.2.0/tests/fixtures/projection-direct.json
eggup-eggpack-0.2.0/tests/fixtures/unknown-schema.json
eggup-eggpack-0.2.0/tests/fixtures/wrong-target.json
```

**Correction to a premise, stated plainly.** The crate's manifest declares **no
`exclude` key**, and the crate has **no `benches/` directory** — there is no
`crates/eggup-eggpack/benches/` in the tree. Nothing is excluded from the package: the
four integration test files and the whole `tests/fixtures/` tree are shipped. This
matches the source plan's §2.6, which says the package ships the `tests/` tree and
warns that no packaging claim may state that the integration tests or the fixtures are
excluded. The published `0.1.3` shipped three of the four test files; `0.2.0` adds
`tests/digest_reuse.rs`.

The normalized manifest's dependency tables, as shipped:

```toml
[dependencies.eggpack-manifest]
version = "=0.1.0"

[dependencies.eggup-acquisition]
version = "=0.2.0"

[dependencies.eggup-archive]
version = "=0.2.0"

[dependencies.eggup-core]
version = "=0.2.0"
```

`Cargo.toml.orig` — the pre-normalization manifest — retains the in-tree
`path = "../eggup-*"` development keys. Packaging rewrote them to registry-only
requirements, which is exactly the rewrite the registry-only proof checks. `Cargo.toml.orig`
hashes identically to `crates/eggup-eggpack/Cargo.toml` at `b612608`.

### Public API boundary

Verified against the published `src/lib.rs`, which is byte-identical to the repository
file at `b612608`:

| Item | Line | Notes |
|---|---|---|
| `AdapterError` | `lib.rs:19-38` | `#[derive(Debug)]`, `#[non_exhaustive]`, eight variants, unchanged from `0.1.3` |
| `MAX_MANIFEST_BYTES` | `lib.rs:61` | constant |
| `ArtifactRequirement` | `lib.rs:65` | public fields `artifact_name`, `exact_size`, `sha256`, `member_id`, `destination` |
| `ArchiveMemberRequirement` | `lib.rs:80` | public fields `source`, `install`, `exact_size`, `sha256` |
| `ManifestProjection` | `lib.rs:93` | `Installable` and `Archive` variants |
| `PlannedAcquisition` | `lib.rs:310-323` | public fields `artifact_name`, `request`, `limits`, `exact_size`, `sha256`, `member_id` |
| `project` | `lib.rs:326` | free function |
| `project_json` | `lib.rs:409` | free function |
| `install_ids` | `lib.rs:422` | free function |
| `archive_format_for_name` | `lib.rs:458` | free function |
| `validate_acquired_archive` | `lib.rs:497` | free function |
| `archive_plan_for` | `lib.rs:540` | free function |
| `core_plan_for_archive_with_destinations` | `lib.rs:589` | free function |
| `core_plan_for_archive` | `lib.rs:656` | free function |
| `bind_archive_members` | `lib.rs:687` | free function |

Inherent methods on `ManifestProjection`, unchanged and still part of the published
surface: `default_destinations` (`lib.rs:147`), `bind_requests` (`:165`),
`materialize_artifact_set_with_destinations` (`:238`), `materialize_artifact_set`
(`:298`).

The adapter's own public API did not move. What moved is what the adapter's
`PlannedAcquisition` **carries**: `request: AcquisitionRequest` and
`limits: FetchLimits` are the seam's types, imported at `lib.rs:6`, so the seam's
retyping is inherited transitively by every consumer of the adapter.

### Boundary (ADR-0004)

Unchanged, and it did not move in this release. The adapter still stops at
extraction-required evidence: it selects no release, verifies no authenticity, and
authorizes no filesystem destination. SHA-256 remains integrity evidence only. The
crate's own `adapter_source_claims_no_producer_or_service_authority` fixture asserts
that production source contains none of the producer or service authority tokens, and
it is green on every platform lane.

## Exact commands run, with results

Commands are recorded in the order they were run. The commands below are the ones run
**for this closure record**; the train-wide qualification commands and their outputs
are in the shared evidence set, which this record cites rather than restates.

```text
$ git log --oneline -12 && git rev-parse HEAD && git status --short && git branch --show-current
2d6e089 ci: restore the strict packaged-service verification
b612608 plans(core): record the train-order exception for the first crate
2161f36 release: bump workspace to 0.2.0 and cut the 0.2.0 changelogs
afe6201 plans: add the five 0.2.0 publication plans
b75f73b docs(core): close the M018 hosted-qualification finding in the closure record
…
HEAD = 2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219
branch = plans/corrective-batch-2026-10-23
```

```text
$ git diff b75f73b b612608 -- crates/eggup-eggpack/src
(empty — exit 0, no output)          # G12
$ git diff b75f73b b612608 -- crates/eggup-eggpack/Cargo.toml
-eggup-core = { version = "=0.1.3", path = "../eggup-core" }
-eggup-archive = { version = "=0.1.3", path = "../eggup-archive" }
-eggup-acquisition = { version = "=0.1.3", path = "../eggup-acquisition" }
+eggup-core = { version = "=0.2.0", path = "../eggup-core" }
+eggup-archive = { version = "=0.2.0", path = "../eggup-archive" }
+eggup-acquisition = { version = "=0.2.0", path = "../eggup-acquisition" }
$ git diff b75f73b b612608 -- Cargo.toml
-version = "0.1.3"
+version = "0.2.0"
```

```text
$ tar -tzf target/package/eggup-eggpack-0.2.0.crate
(21 entries — the package inventory quoted above)

$ tar -xzOf target/package/eggup-eggpack-0.2.0.crate eggup-eggpack-0.2.0/.cargo_vcs_info.json
{
  "git": {
    "sha1": "b61260857fdb3904a091c503b1dff05251ec0bff"
  },
  "path_in_vcs": "crates/eggup-eggpack"
}

$ shasum -a 256 target/package/eggup-eggpack-0.2.0.crate
33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6  target/package/eggup-eggpack-0.2.0.crate
$ stat -f '%z bytes' target/package/eggup-eggpack-0.2.0.crate
38981 bytes
$ shasum -a 256 ~/.cargo/registry/cache/index.crates.io-1949cf8c6b5b557f/eggup-eggpack-0.2.0.crate
33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6
```

```text
$ curl -fsSL -A "eggup-release-check/1.0" https://index.crates.io/eg/gp/eggup-eggpack
curl: (56) The requested URL returned error: 404
```

The sparse index read returns **404** in this environment — and it returns 404 for
`eggup-core`, `eggup-archive`, and `eggup-acquisition` as well, while
`https://index.crates.io/se/rd/serde` returns 200. The same sparse-index path is
therefore not serving these crate names here, and the read is not evidence about this
publication — see F-3. The registry read that the plan's §7 actually names was used
instead:

```text
$ curl -fsSL -H 'User-Agent: eggup-release-check' \
    https://crates.io/api/v1/crates/eggup-eggpack/versions
0.2.0 yanked=False cksum=33dc1da430938c19c58edd4ac771ce5cdff2b76d88400a7466857d3ea214f1f6 created=2026-10-24T13:51:33.906205Z
0.1.3 yanked=False cksum=952e88b87ffd7b88c7db246e52169542abff9b01771f1086c22e86b4739619c5 created=2026-10-06T20:48:00.300717Z
0.1.2 yanked=False cksum=9dbfdfb74b4faa88d6badaa6f24e71dada3c7322dd0e9877cebe8b0168b13fef created=2026-10-03T02:39:29.522676Z
```

The `0.2.0` record also reports `crate_size 38981`, `rust_version 1.89`,
`license MIT`, and `published_by.login "dbowm91"`. The checksum is corroborated four
ways: the crates.io API, the registry-cached `.crate` file's SHA-256, the registry-only
fixture's lockfile `checksum` field, and the annotated `v0.2.0` tag message.

```text
$ git cat-file -t v0.2.0
tag
$ git rev-parse v0.2.0^{commit}
b61260857fdb3904a091c503b1dff05251ec0bff
$ git rev-parse v0.1.1^{commit} v0.1.2^{commit} v0.1.3^{commit}
881c95ff069d3d465a282cb6a495ba6fcb70cb6f
e8e07eb538d0eef18ea4cb4ace3bb905c316da72
bd4368369aaf0ae1de2ce213fc16689281bdd664
$ git cat-file -p v0.2.0
object b61260857fdb3904a091c503b1dff05251ec0bff
type commit
tag v0.2.0
tagger CI Test <ci@test.example.com> 1792850017 +0000

eggup 0.2.0
… (the message names all seven crates and their checksums, including
 eggup-eggpack 0.2.0  33dc1da4...14f1f6)
```

```text
$ gh release view v0.2.0 --json tagName,name,isDraft,isPrerelease,publishedAt
{"createdAt":"2026-10-24T13:53:37Z","isDraft":false,"isPrerelease":false,
 "name":"0.2.0","publishedAt":"2026-10-24T13:54:03Z","tagName":"v0.2.0"}
```

```text
$ cargo test -p eggup-eggpack --all-targets --all-features --locked --offline
     Running unittests src/lib.rs
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/archive_handoff.rs
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/caller_destinations.rs
test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/digest_reuse.rs
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/interoperability.rs
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**71 passed, 0 failed** — the adapter's whole suite, on this machine, against the
publication source commit's tree.

```text
$ cargo test --manifest-path /tmp/eggup-registry-fixtures/eggpack/Cargo.toml --offline --locked
running 2 tests
test published_eggpack_bounds_the_manifest_size ... ok
test published_eggpack_maps_archive_names_to_formats ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**2 passed, 0 failed** — the registry-only fixture, against the crate resolved from
crates.io.

```text
$ ./scripts/check-docs-truth.sh
note 7: skipped — pass --baseline <ref> to scope the no-production-delta check
note README.md:128: published-version claim with an ambiguous subject; skipped
…
documentation-truth checks passed
EXIT=0
```

```text
$ diff -u <published 0.1.3 src/lib.rs> <published 0.2.0 src/lib.rs>
(the single behavioural hunk quoted above)
$ diff <(cd <published 0.1.3> && find . -type f | sort) \
       <(cd <published 0.2.0> && find . -type f | sort)
> ./tests/digest_reuse.rs
$ git show b612608:crates/eggup-eggpack/src/lib.rs | shasum -a 256
efc896c8ea56aaf7a089fec40ad6dd37ec7e1d204ada69c6067a55a5508e02df  -
$ shasum -a 256 <published 0.2.0 src/lib.rs>
efc896c8ea56aaf7a089fec40ad6dd37ec7e1d204ada69c6067a55a5508e02df
```

The published source is byte-identical to the repository source at the publication
commit.

**Not run, and not claimed.** `cargo publish`, `cargo publish --dry-run`, and
`cargo package` were not run for this closure record; the packaging evidence above is
read from the already-produced artifacts in `target/package/` and from the crates.io
registry copy, not from a new packaging run.

## Per-platform results

Each platform is recorded from the lane that ran on it. No platform result is inferred
from another platform. The adapter's tests execute in the `stable`, `macos`, and
`windows-check` lanes of hosted run `40079833945`; the `msrv` lane is a `cargo check`,
so it compiles the adapter's test targets at the 1.89 floor without executing them.

| Platform | Result |
|---|---|
| **Linux** | **PASS** — hosted run `40079833945`: `Stable checks` (ubuntu-latest) **success**, whose `cargo test --workspace --all-targets --all-features --locked` step executes the adapter's suite; `MSRV check` (ubuntu-latest, Rust 1.89.0) **success**, a `cargo check --workspace --all-targets --locked` that compiles the adapter's test targets at the floor without executing them; `Linux systemd service qualification` (ubuntu-latest) **success** — that lane carries no adapter step. The three `#[cfg(unix)]` adapter fixtures are present on this platform and are green |
| **macOS** | **PASS** — hosted run `40079833945`, `macOS tests` lane (macos-latest): **success**, whose `cargo test --workspace --all-targets --all-features --locked` step executes the adapter's suite. Native, on this machine (darwin arm64): `cargo test -p eggup-eggpack --all-targets --all-features --locked --offline` → **71 passed, 0 failed** (7 inline + 13 + 15 + 8 + 28), with the three `#[cfg(unix)]` fixtures executed. The registry-only fixture was also run natively here: **2 passed, 0 failed** |
| **Windows** | **PASS** — hosted run `40079833945`, `Windows archive, acquisition, and service tests and check` lane (windows-latest): **success**. The lane carries `cargo test -p eggup-eggpack --all-targets --all-features --locked` as an explicit step, and the lane's final `cargo test --workspace --all-targets --locked --no-fail-fast` step runs the adapter's suite again without `--all-features`. The three Unix-only adapter fixtures are `#[cfg(unix)]` and are therefore **absent** from this lane |

| Hosted lane | Platform | Result | Adapter coverage |
|---|---|---|---|
| `stable` | ubuntu-latest (Linux) | success | adapter suite executed via the workspace test step |
| `msrv` | ubuntu-latest (Linux) | success | adapter test targets **compiled** at 1.89.0, not executed |
| `service-systemd` | ubuntu-latest (Linux) | success | none — the lane carries no adapter step |
| `macos` | macos-latest | success | adapter suite executed via the workspace test step |
| `windows-check` | windows-latest | success | adapter suite executed by a dedicated step and again by the workspace step |

The hosted run's **first attempt** failed in exactly one step, in four lanes, with the
same error: `cargo package -p eggup-service --locked --no-verify` resolving the
packaged service crate's `eggup-core = "^0.2"` against crates.io while `eggup-core 0.2.0`
was not yet on the registry. It is a publication state of the train's front, the same
class the repository records as registry finding 16, and it was discharged by the
`eggup-core 0.2.0` upload. The four affected lanes were re-run on the same commit and
passed unchanged; the fifth lane never carried that step. The full account is in the
shared evidence set and in the core plan's §9.1.

## Invariant review

Every invariant the subsystem roadmap and the crate's own contract carry is unchanged
by this publication, and each is checked against the published bytes:

1. **The adapter's public API is unchanged.** Verified item by item against the
   published `src/lib.rs`; the published `0.1.3` and `0.2.0` sources differ in one
   behavioural hunk and one rustdoc hunk.
2. **`AdapterError`'s variant set is unchanged.** Eight variants, `#[non_exhaustive]`,
   `Debug` only — identical in both published artifacts.
3. **The three Eggup edges are exact-pinned to the adapter's own version.** `=0.2.0`
   in the published manifest, asserted by the crate's own fixture through
   `env!("CARGO_PKG_VERSION")`.
4. **`eggpack-manifest` remains `=0.1.0`, registry-only, and not republished here.**
   Its published bytes are unchanged; its version does not move.
5. **The adapter remains the only crate that touches producer types.** No other crate
   in the workspace depends on `eggpack-manifest`; the adapter's own fixture asserts
   that `eggpack-core`, `eggpack-contract`, and `eggpack-bootstrap` appear nowhere in
   its manifest.
6. **The adapter remains a leaf.** Nothing in the workspace depends on
   `eggup-eggpack`.
7. **The digest is computed by `eggup_core::hash_file`.** The workspace holds exactly
   one file-digest implementation; the adapter's direct `sha2` runtime edge is gone and
   `sha2` remains a dev-dependency only.
8. **The symlink rejection and the exact-size check remain, and remain ordered before
   the digest computation.** `lib.rs:513-517` above `:522`.
9. **The digest value, the error variant, and the error payload are unchanged.** Open
   failure, read failure, and digest mismatch all report
   `AdapterError::InvalidAcquiredFile(artifact_name)`.
10. **SHA-256 remains integrity evidence only.** No authenticity, signature,
    provenance, or trust claim is introduced; the crate's textual authority-boundary
    fixture is green.
11. **No release selection, install-root, ownership, permission, elevation, mirror
    fallback, version ordering, service, or publication policy enters the adapter.**
    The published source delta contains no such code.
12. **No new transport or TLS stack.** The published runtime dependency set is the same
    four crates as `0.1.3`'s, minus the direct `sha2` edge.
13. **`#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` are kept**
    (`lib.rs:1-2`), and the workspace's `unsafe_code = "deny"` and clippy `all = warn`
    with `-D warnings` are green in the hosted `stable` lane.
14. **No artifact mutation.** `validate_acquired_archive` remains read-only: it stats,
    hashes, and compares.

## Failure, rollback, and recovery review

`validate_acquired_archive` remains a read-only continuity gate. It mutates nothing,
stages nothing, and has no rollback path. The ordered failure table is unchanged, and
the digest-step rows are now sourced from the core routine:

| Condition | Reported as | Ordered at |
|---|---|---|
| Acquired path is relative | `InvalidAcquiredFile(artifact_name)` | `lib.rs:510-512` |
| `symlink_metadata` fails (missing path) | `InvalidAcquiredFile(artifact_name)` | `lib.rs:513-514` |
| Not a regular file (symlink, directory, FIFO, device) | `InvalidAcquiredFile(artifact_name)` | `lib.rs:515-517` |
| Size ≠ manifest exact size | `InvalidAcquiredFile(artifact_name)` | `lib.rs:515-517` |
| Digest step's open failure | `InvalidAcquiredFile(artifact_name)` | `lib.rs:522-523` |
| Digest step's read failure | `InvalidAcquiredFile(artifact_name)` | `lib.rs:522-523` |
| Digest ≠ manifest digest | `InvalidAcquiredFile(artifact_name)` | `lib.rs:524-526` |

**Distinguishability is unchanged and remains a documented limitation.** All seven
conditions report the same variant and the same payload; they are not distinguishable
from the returned error value. That is the baseline behavior, preserved deliberately,
and narrowing it would change what a published consumer observes. The digest step's
open and read failures are deliberately collapsed into the same variant as the
mismatch, exactly as the adapter's own inline routine reported them — this is what
makes the published error surface identical to `0.1.3`'s despite the delegation.

**Cancellation, restart, and contention** are unchanged: the function performs no
mutation, so there is nothing to roll back; there is no lock; concurrent readers are
harmless; a concurrent writer is resolved by the digest comparison at `:524-526`.

**Rollback of the publication itself is not available and was not attempted.** crates.io
does not permit republishing a version, and this milestone never yanked one. A defect
found in `0.2.0` after acceptance is handled by publishing `0.2.1` under a new plan and
closure record, with its own exact-pin re-pin to `=0.2.1` on the three workspace
dependencies. The `v0.1.1`, `v0.1.2`, and `v0.1.3` tags are not moved and must not be.

## Compatibility and migration review

**The adapter's own API: no migration.** No public signature, field, variant, or
constructor was added, removed, or retyped, and no behavior changed. A consumer that
compiles against `0.1.3`'s adapter API compiles against `0.2.0`'s, **with one
exception that arrives through the pin rather than through the adapter's API**.

**The inherited seam break — the one consumer action.** `eggup-acquisition 0.2.0` is
source-breaking:

- `AcquisitionError::Io(String)` became `AcquisitionError::Io { operation, source }`,
  with a live `source()` chain. A consumer that matched `Io(_)` must match the struct
  form.
- `Clone`, `PartialEq`, and `Eq` are removed from `AcquisitionError`, which now derives
  `Debug` only, because `std::io::Error` implements none of them. A consumer that
  cloned or compared whole error values must match on the variant and its fields
  instead.

The adapter re-exposes the seam's request and limits types through `PlannedAcquisition`
(`lib.rs:310-323`, types imported at `lib.rs:6`), and its own `bind_requests` takes
`eggup_acquisition::AcquisitionRequest` values, so a consumer of `eggup-eggpack 0.2.0`
inherits the break **transitively**. A consumer that also depends on
`eggup-acquisition` directly must widen its own requirement to `0.2` to share the
seam's copy; a `0.1.x` requirement does not accept `0.2.0` and resolves a separate copy
of the seam instead. The published `eggup-acquisition 0.2.0` artifact's own rustdoc
records the removal and instructs consumers to match on the variant and its fields.

**What `0.2.0` carries that the published `0.1.3` did not.**

1. **The M006 digest-reuse change** — a content change, not only a manifest change.
   The published `0.1.3` still carries a direct `sha2 = "0.10.9"` runtime dependency
   and computes the archive digest with its own inline loop; `0.2.0` ships the adapter
   delegating to `eggup_core::hash_file` with the runtime edge removed. The published
   runtime dependency list is therefore one entry shorter than `0.1.3`'s, with an
   unchanged transitive closure — `sha2 v0.10.9` still reaches the adapter through
   `eggup-core` and `eggup-archive`. The digest value, the error variant, and the error
   payload are unchanged, and the symlink rejection and the exact-size check still run
   and are still ordered before the digest computation.
2. **The exact-pin re-pin** from `=0.1.3` to `=0.2.0` on all three in-workspace
   dependencies. This is the consumer-facing mechanism.

**The previous exact-pin cascade, for the record.** `eggup-eggpack 0.1.3` exact-pinned
`=0.1.3` on all three in-workspace dependencies, so a consumer of `0.1.3` resolved all
three at `0.1.3` and inherited their defects. `eggup-eggpack 0.1.2` exact-pinned
`=0.1.2` and never received the M006 fix either. The re-pin is the mechanism by which
this release delivers the workspace's `0.2.0` fixes to adapter consumers: the
registry-only fixture's lockfile resolves `eggup-core 0.2.0`, `eggup-archive 0.2.0`, and
`eggup-acquisition 0.2.0` from crates.io, with one copy of each.

**The older published versions keep their state.** `eggup-eggpack 0.1.3` and `0.1.2`
still carry the pre-re-pin state and the direct `sha2` runtime edge. A consumer pinned
to them does not receive the workspace's `0.2.0` fixes — that is the consumer-unblock
reconciliation this milestone delivers, not a defect in those versions.

**No other behavior change reaches this crate.** `eggup-service`'s two source breaks
(`ServiceError::Manager` retyping and `LifecycleUpdateReceipt`'s new `engine_policy`
field) do not touch the adapter: it does not depend on `eggup-service`.
`eggup-archive`'s now-reachable `cleanup()` success path does not reach the adapter
either: the adapter performs no extraction and no cleanup.

## Security review

**The security-critical property of this release is the pin, and it is proven.** The
exact pins are what prevent a consumer from silently resolving a different published
seam; the registry-only fixture shows the re-pin reached the registry, so the consumer
of `0.2.0` receives the fixed `eggup-core` — the release that carries the
unconditional execution deadline and the bounded-stdin surface — and the fixed
`eggup-archive`, whose `cleanup()` success path is now reachable.

**The symlink-ordering property is preserved.** The guard at `lib.rs:513-517` still
precedes any open of the acquired path. `hash_file` follows symlinks when it opens, so
the adapter's `symlink_metadata` guard remains the **only** link rejection in this
path; it was not moved, not weakened, and not merged into the hash call. The published
source diff shows the guard lines as unchanged context.

**TOCTOU between the guard and the open — carried forward, low severity.** The
`symlink_metadata` (`:513`) and the open inside `hash_file` (`:522`) are separate
pathname operations and are not atomic: a path replaced by a symlink in that window is
followed and the target is hashed. This release neither introduces nor closes the
window — the published `0.1.3` had the same window between `symlink_metadata` and
`fs::File::open`. Closing it requires `O_NOFOLLOW`-style handle-relative opening, which
`eggup-core` does not expose on this path today. It is recorded in
`architecture/eggpack-adapter.md` as a **low**-severity carried-forward item, and
fixing it is a separate milestone with its own design.

**No new security surface.** The published runtime dependency set loses one entry and
gains none. No network stack, no TLS stack, no filesystem write path, no privilege
operation, and no credential-bearing text entered the crate. The crate's own textual
authority-boundary fixture asserts that production source contains none of the
producer or service authority tokens, and it is green on every platform lane.

**Integrity remains SHA-256 checksum evidence only.** No crate in this line makes an
authenticity or signature claim, selects a release, or authorizes a destination, and
this release changes nothing in that respect.

## Documentation and operations evidence

- **Per-crate changelog.** `crates/eggup-eggpack/CHANGELOG.md` § `0.2.0` records the
  publication, the dependency surface (`eggup-core` `=0.2.0`, `eggup-archive` `=0.2.0`,
  `eggup-acquisition` `=0.2.0`, `eggpack-manifest` `=0.1.0`, the last external and not
  republished here), the compatibility statement — this crate's own public API
  unchanged, the published dependency requirements changed, the transitive seam break
  and the widening a consumer must do — the M006 digest-reuse entry, the "no behavior
  change" entry, the `sha2` dev-dependency entry, and the unchanged-boundaries entry.
  The `Unreleased` section states that `0.2.0` is the published baseline on crates.io
  and that no publication milestone authorizes those entries. Both statements are true
  as of the publication timestamp. The section's heading is `## 0.2.0 — 2026-10-23` in
  the published artifact — the release-cut date — and `## 0.2.0 — 2026-10-24` in the
  repository's copy, the publication date, which is the convention every earlier
  per-crate release section follows. The published bytes cannot be advanced; see F-4.
- **Root `CHANGELOG.md`.** The `0.2.0` section aggregates the train and carries an
  `eggup-eggpack 0.2.0` entry naming the re-pin, the delegated digest, the preserved
  guard ordering, the removed `sha2` runtime edge, the unchanged digest value, error
  variant, and error payload, and the unchanged ADR-0004 boundary. It is owned by
  another writer and is not edited here.
- **Crate `README.md`.** The published `0.2.0` README records the runtime dependency
  set, that the archive digest is computed by `eggup_core::hash_file`, that the crate
  declares no direct `sha2` runtime dependency, and that `sha2` remains a
  dev-dependency only, with the integrity-only and no-destination-authorization
  framing intact. The README delta between the published `0.1.3` and `0.2.0` is that
  one paragraph.
- **Normative architecture — drift observed, then repaired by its owner.** When this
  record was written, `architecture/eggpack-adapter.md` still described the
  pre-`0.2.0` published state: its header table said "Published to crates.io at
  `0.1.3`" (the header's `Status` row); its "Runtime dependencies" table listed the
  three Eggup edges at `=0.1.3`; its testing-approach paragraph said the Eggup edges
  "stay `=0.1.3`"; its "Published-crate status" paragraph still said the crate is
  published at `0.1.3`, that the published `0.1.3` manifest carries the direct `sha2`
  runtime dependency, and that "**nothing** has been published under this change"; and
  its header table carried a stale test count ("64 tests" against the suite's 71). The
  file is owned by another writer and is outside this milestone's
  ownership, so this record did not edit it. **The architecture owner repaired all of
  it in the working tree while this record was being written** — the header now reads
  "Published to crates.io at `0.2.0`", the three Eggup edges read `=0.2.0`, the
  testing-approach paragraph reads "Eggup edges stay `=0.2.0`", the test count reads
  71, and the "Published-crate status" paragraph now records the `0.2.0` publication,
  the dropped `sha2` runtime edge, the `=0.2.0` re-pin, and the untouched `0.1.x`
  line. One residual drift remains in that file — see F-5.
- **`docs/` — non-normative, and already consistent.** `docs/` points at the contracts
  rather than restating them, so it is not a normative surface for this milestone.
  `docs/crates.md` currently records the adapter as published at `0.2.0` and states
  that it pins its dependencies exactly (`=0.2.0`, per
  `crates/eggup-eggpack/Cargo.toml`) — consistent with the published manifest.
- **`plans/registry.md` and the subsystem roadmap — not edited here.** They are owned
  by another writer. The required updates are stated under *Roadmap disposition and
  registry updates* below.
- **Operations.** The annotated `v0.2.0` tag and the GitHub release `0.2.0` exist and
  cover the whole train. CI never publishes: the upload was a manual,
  maintainer-authorized step, and the tag and release follow the accepted upload rather
  than the other way round.

## Unresolved findings, with severity

| ID | Finding | Severity | Reason / disposition |
|---|---|---|---|
| F-1 | The published `eggup-eggpack 0.1.3` and `0.1.2` still carry the pre-re-pin state — the direct `sha2 = "0.10.9"` runtime dependency, the inline digest loop, and exact pins at `=0.1.3` and `=0.1.2` — so a consumer pinned to either does not receive the workspace's `0.2.0` fixes | **low** (a published-version state, not a defect) | This is the consumer-unblock reconciliation this milestone delivers, and it is now visible on the registry: `0.2.0` exists and resolves the `0.2.0` set. The older versions are not withdrawn and their tags are not moved — crates.io does not permit republishing a version, and the repository's discipline is that an already-published version is never yanked to correct it. Consumers move forward by adopting `0.2.0`. Recorded so that no consumer believes the `0.1.x` line carries the `0.2.0` fixes |
| F-2 | The registry-only fixture's **behavioural** assertion set is narrower than the plan's §4.2 specified. The surviving fixture carries two tests — `published_eggpack_maps_archive_names_to_formats` and `published_eggpack_bounds_the_manifest_size` — and asserts neither the manifest projection, nor the archive-plan/bound-member round trip, nor the delegated digest, nor the seam typing. The pre-upload fixture (plan §4.1) was not retained in the evidence set either, so its three assertions cannot be re-verified from the surviving artifacts | **low** (an evidence-completeness gap, not a defect in the published artifact) | Every missing assertion has an equivalent source of evidence, and each is named in the matrix: the projection and the archive round trip by the adapter's own shipped test files (`tests/interoperability.rs`, `tests/caller_destinations.rs`, `tests/archive_handoff.rs`) run against the byte-identical published source and green on every platform lane of run `40079833945`; the delegated digest by the published-source diff plus `tests/digest_reuse.rs`; the seam typing by the published `eggup-acquisition 0.2.0` artifact's `AcquisitionError::Io { operation, source }` under `#[derive(Debug)]` only. The milestone's decisive claim — that the re-pin reached the registry — does not depend on the fixture's behavioural assertions; it rests on the lockfile, which is exactly one copy of each dependency at `0.2.0` from crates.io. Closing the gap would mean writing the four assertions into the fixture and re-running it from outside the repository; it is a registry-side evidence improvement, not a publication defect, and it must not be presented as a re-publication requirement |
| F-3 | The crates.io **sparse index** read for this crate returns 404 in the environment where this record was written — as it does for `eggup-core`, `eggup-archive`, and `eggup-acquisition`, while an unrelated crate name returns 200. The sparse-index path is therefore not serving these crate names from here | **low** (an observation about the read, not about the publication) | The registry state was read instead through the crates.io public API, the plan's own §7 named command, which returns `0.2.0` present and unyanked with the checksum. The checksum is corroborated four independent ways: the crates.io API, the SHA-256 of the registry-cached `.crate` file, the registry-only fixture's lockfile `checksum` field, and the annotated `v0.2.0` tag message. Nothing about the publication depends on the sparse-index path |
| F-4 | The published `eggup-eggpack 0.2.0` artifact's `CHANGELOG.md` carries the heading `## 0.2.0 — 2026-10-23`, the date the release was cut, while the repository's copy carries `## 0.2.0 — 2026-10-24`, the date it was published. The two differ by exactly one line, and the divergence is **permanent** | **low** (an immutable registry/repository divergence, not a defect) | The published bytes are the release-cut state — `b612608`'s `CHANGELOG.md` is byte-identical to the one in the `.crate` — and crates.io does not permit republishing a version, so the published heading cannot be advanced. The repository's copy now carries the publication date, which is the convention every earlier per-crate release section follows. The divergence is recorded here and in `plans/registry.md` rather than left for a later reader to rediscover. It is one line of changelog metadata and blocks nothing |
| F-5 | `architecture/eggpack-adapter.md` still described the pre-`0.2.0` published state in four places — the header table's `Status` row, the "Runtime dependencies" table, the testing-approach paragraph, and the "Published-crate status" paragraph — and carried a stale test count (64 against the suite's 71). One residual drift remains in that file: its own "Drift outside this document's ownership" section states that `docs/crates.md` "now states `=0.1.3`", while `docs/crates.md` now states `=0.2.0` | **low** (normative-doc drift) | The file is owned by another writer and is outside this milestone's file ownership, so this record did not edit it. **The architecture owner repaired the four stale statements and the test count in the working tree while this record was being written** — the header now reads "Published to crates.io at `0.2.0`", the three Eggup edges read `=0.2.0`, the testing-approach paragraph reads "Eggup edges stay `=0.2.0`", the count reads 71, and the "Published-crate status" paragraph records the `0.2.0` publication and the untouched `0.1.x` line. **The residual `docs/crates.md` reference was also repaired by the architecture owner afterwards**: the drift bullet now names `=0.2.0` and the updated `docs/crates.md` line numbers, so this finding is fully closed and nothing is handed over. The plan's §2.2 places `architecture/**` outside this milestone's edit scope |
| F-6 | The registry's `eggup-curl` sub-second-deadline tests are load-sensitive on a busy host (`build_curl_args_passes_sub_second_deadlines_to_fake_curl`, `sub_second_total_timeout_kills_child_promptly`) | **low**, and **unrelated to this crate** | Listed here only because it is an open registry finding and this closure record is required to state its severity. It blocks nothing, it concerns `eggup-curl`'s test robustness rather than transport semantics, it was reproduced at a pristine baseline long before this train, and `eggup-eggpack` shares no code, no dependency edge, and no lane step with it. It is registered to Acquisition M012 and is not this milestone's concern |

No **medium-or-higher** finding remains open in M007's own scope. F-2 is the closest to
the line, and it is filed as low because the milestone's substantive claims do not
depend on it and each missing assertion is independently covered.

## Roadmap disposition and registry updates

**M007 does not edit `plans/registry.md` or the subsystem roadmap** — those are another
writer's files, and they are being edited concurrently. The state of each is stated
below, so that this record is consistent with them and so that the remaining work is
visible to its owner:

1. `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` — **the M007 status
   row is already present and already agrees with this record**: status **closed**;
   hosted-qualified on run `40079833945`; published `eggup-eggpack 0.2.0` (checksum
   `33dc1da4…14f1f6`) from `b612608`, the publication source commit, with `v0.2.0` and
   release `0.2.0` created; published last because the crate exact-pins all three of
   its in-workspace dependencies at `=0.2.0`; the registry-only fixture depending only
   on `eggup-eggpack` and resolving `eggup-core`, `eggup-archive`, and
   `eggup-acquisition` at `0.2.0` from crates.io, as the proof that the re-pin
   delivers the workspace's fixes to adapter consumers; the adapter's own public API
   unchanged; and the source break arriving transitively through the seam's retyped
   `AcquisitionError`. **No roadmap change is outstanding from this milestone.**
2. `plans/registry.md` § "Post-0.1.3 open findings" — **entry 9** ("`eggup-eggpack`
   re-implements core's file-digest routine and carries a direct `sha2` dependency for
   that one call site", assessed **medium**) is **closed by publication**, not merely by
   the M006 source change: `eggup-eggpack 0.2.0` now ships the delegated digest and no
   direct `sha2` runtime edge, so a consumer resolving `0.2.0` no longer carries the
   redundancy. The finding's honest framing must be preserved — it was a redundancy and
   a maintenance/supply-chain cost, **not** a correctness or security defect, and no
   failing production report ever existed. The published `0.1.3` and `0.1.2` still
   carry it; that residual is F-1.
3. `plans/registry.md` § "Post-0.1.3 open findings" — **entry 2** (the deferred adapter
   error-surface narrowing) is **unchanged** and remains open. M007 deliberately kept
   the collapse so that the published error surface stays identical to `0.1.3`'s.
   Narrowing remains a candidate for the intentional 0.2/1.0 API boundary and must not
   ride along with a republication.
4. `plans/registry.md` — **entry 1** (the load-sensitive `eggup-curl` sub-second
   deadline tests) is **unchanged**: low severity, blocking nothing, unrelated to this
   crate. It is listed in this record only because the severity must be stated
   (F-6).
5. `plans/registry.md` — the release-aggregation and published-version entries
   **already record** `eggup-eggpack 0.2.0` with checksum `33dc1da4…14f1f6`, published
   2026-10-24 from `b612608`, in the `0.2.0` train table and in the
   "Eggpack manifest interoperability" workstream row; `0.1.2` and `0.1.3` are recorded
   as present, unyanked, and unchanged. No further change is outstanding there, except
   the **entry 9** disposition in item 2.
6. Root `CHANGELOG.md` — the `0.2.0` section already carries this crate's entry; no
   further change is required from this milestone.

## Handoff notes

- **What a consumer must do.** Adopt `eggup-eggpack 0.2.0`. No adapter API migration
  is required. If the consumer also depends on `eggup-acquisition` directly, widen that
  requirement to `0.2` so it shares the seam's copy rather than resolving a second one;
  and expect the seam's retyped `AcquisitionError::Io { operation, source }` and the
  removal of `Clone`, `PartialEq`, and `Eq` from `AcquisitionError`, which this
  adapter inherits transitively through its exact pin.
- **What the next adapter republication would require.** The `0.1.x` line keeps the
  pre-re-pin state (F-1). A future `0.2.1` would need its own plan and closure record,
  its own exact-pin re-pin to `=0.2.1` on the three workspace dependencies, a
  `cargo package` and `cargo publish --dry-run` pass, full hosted qualification, and a
  GitHub release note append that does not move the `v0.2.0` tag. crates.io does not
  permit republishing a version, and this milestone never yanked one.
- **What this milestone did not authorize and did not do.** No change under
  `crates/eggup-eggpack/src/`; no change to `eggup-core`, `eggup-archive`,
  `eggup-acquisition`, or `eggpack-manifest`; no narrowing of the adapter's error
  surface and no new `AdapterError` variant; no authenticity, signature, provenance, or
  trust work; no policy, destination, ownership, permission, release-selection, or
  service change in the adapter; no fix for the TOCTOU window; no movement of the
  `v0.1.1`, `v0.1.2`, or `v0.1.3` tags and no yank of any published version; and no
  edits to `plans/registry.md`, the subsystem roadmap, the root `CHANGELOG.md`, the
  per-crate changelogs, `architecture/**`, `docs/**`, `.github/**`, or `scripts/**`.
