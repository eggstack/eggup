# Archive Extraction M006 — Closure and Verification Record

## Status

**Closed.**

`eggup-archive 0.2.0` is on crates.io, unyanked, published from the shared
`0.2.0` train commit `b61260857fdb3904a091c503b1dff05251ec0bff`. The `v0.2.0`
annotated tag points at that commit, GitHub release `0.2.0` exists against it,
and the registry-only proof passes against the registry rather than against the
workspace. The milestone's own scope — the upload of this one crate — is
complete and green on all five hosted lanes.

The archive's public API is **unchanged**: the published `0.2.0` `src/lib.rs`
and the published `0.1.3` `src/lib.rs` have byte-identical `pub fn` signature
lines and byte-identical `pub` item lines. The single caller-visible change is
a **behaviour** change: the `cleanup()` success path is now reachable, so a
caller that wrote `cleanup().unwrap_err()` now panics on the success path
instead of compiling against an always-`Err` result.

Two findings stay open at low severity and block nothing: the date heading
discrepancy between the published tarball and the repository's changelog
(F-1), and the deferred `lib.rs` module decomposition, whose named follow-up
number was consumed by this milestone (F-3). Two more are carried forward
unchanged from M005 at low severity (F-2, F-4). One registry finding that is
**not** an archive concern stays open at low severity and is recorded here for
visibility only: registry finding 1, the load-sensitive `eggup-curl`
sub-second-deadline tests (F-5).

CI never publishes. `.github/workflows/ci.yml` contains **no** `cargo publish`
step — `grep -n "cargo publish" .github/workflows/ci.yml` returns nothing — and
the publication was manual, maintainer-authorized, and plan-gated.

Platform evidence is reported per platform below. A lane that was not run is
recorded as **not run**, never inferred from another platform.

## Source plan and roadmap

- Source plan: `plans/implementation/archive-extraction/006-eggup-archive-0.2.0-publication.md`
- Source roadmap: `plans/subsystems/archive-extraction-roadmap.md`
- Predecessor closure records: `plans/closure/archive-extraction/001-status.md`
  through `001d-status.md` (the extraction lineage), `004-status.md` (the
  `0.1.3` correctness publication), and `005-status.md` (the cleanup-contract
  corrective whose work `0.2.0` publishes)
- Sibling records for the same train:
  `plans/closure/verified-update-core/019-status.md`,
  `plans/closure/acquisition-transport/017-status.md`,
  `plans/closure/service-lifecycle/017-status.md`,
  `plans/closure/eggpack-manifest-interoperability/007-status.md`

## Reviewed repository baseline

| Item | Value |
|---|---|
| Branch | `plans/corrective-batch-2026-10-23` |
| Plan baseline (qualified baseline) | `b75f73bf1ad138f79e7e16d50f35671e52bafe96` (`b75f73b`) |
| Publication source commit | `b61260857fdb3904a091c503b1dff05251ec0bff` (`b612608`) |
| Relationship | **Distinct.** The publication source commit is four commits after the qualified baseline and carries the release-prep work; it is not the baseline |
| Tag | `v0.2.0`, annotated (`git cat-file -t v0.2.0` → `tag`), resolving to `b61260857fdb3904a091c503b1dff05251ec0bff` |
| HEAD at review | `2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219`, one commit after the publication source commit |
| Working tree at review | dirty, by design: the parent-owned close-out edits (root `CHANGELOG.md`, the seven per-crate `CHANGELOG.md` date corrections, `plans/registry.md`, the five subsystem roadmaps) and the five closure records themselves are uncommitted at review time. No crate source is modified. |

The working tree is **not** clean, and this record does not claim it is. The
`git status --porcelain` output at review time lists exactly those
parent-owned edits plus the sibling closure records; `git diff --stat b612608
-- crates/eggup-archive/src/` is **empty**, so the archive's sources under
review are the published sources.

## Implementation commits / PRs

| Commit | Subject | Role |
|---|---|---|
| `afe6201430cb4eba78e809059edef8a7d1e36ebf` | `plans: add the five 0.2.0 publication plans` | authored this milestone's plan and its four siblings' |
| `2161f36967b2d9afda4e2b05f7eb260e664fe761` | `release: bump workspace to 0.2.0 and cut the 0.2.0 changelogs` | the release-prep commit: workspace version bump to `0.2.0`, the dependency widenings, the `eggup-eggpack` re-pin, the eight `0.2.0` changelog cuts, the `check-docs-truth.sh` generalisation, and the **publication-coupling guard re-point** |
| `b61260857fdb3904a091c503b1dff05251ec0bff` | `plans(core): record the train-order exception for the first crate` | **the publication source commit**; the train-order exception recorded in the core plan's §9.1 |
| `2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219` | `ci: restore the strict packaged-service verification` | one commit after the publication source commit; CI only, no crate source |

No pull request was used; the plan, the release-prep commit, the train-order
exception, and the publication were committed directly on the branch.

**The archive needed no source commit of its own.** `git diff --stat
b75f73b..b612608 -- crates/eggup-archive/src/` is empty: the work `0.2.0`
publishes was already merged and closed under M005, and this milestone
authorized the upload of an already-qualified source.

## Executive finding

Archive Extraction M006 published `eggup-archive 0.2.0` as the **second**
crate of the lockstep `0.1.3 → 0.2.0` train, immediately after
`eggup-core 0.2.0`. The published bytes come from the shared train commit
`b612608`; the published tarball's `.cargo_vcs_info.json` records
`"sha1": "b61260857fdb3904a091c503b1dff05251ec0bff"` and records **no** dirty
tree.

The archive's publication is the **quietest** in the train, and that is the
substantive finding. `crates/eggup-archive/Cargo.toml:23-24` declares
`eggup-core` as a **path-only dev-dependency**, and dev-dependencies are
stripped when a crate is packaged. The published archive therefore carries
**no `eggup-core` dependency at all** — the crates.io dependency readback for
`0.2.0` names exactly five normal dependencies (`flate2`, `fs_at`, `sha2`,
`tar`, `zip`) and no `eggup-core` entry, and the registry-only fixture's
lockfile and `cargo tree --locked` contain **zero** occurrences of
`eggup-core`. Unlike `eggup-curl`, `eggup-eggfetch`, `eggup-service`, and
`eggup-eggpack`, the archive needed **no dependency widening** to become
packageable against the registry. That is this milestone's slice of registry
finding 17's closure, and it is a **positive** finding rather than a
correction: the crate was already registry-packageable before the upload.

The second finding is that the publication is **behaviour-changing but not
API-changing**. The published `0.2.0` `src/lib.rs` and the published `0.1.3`
`src/lib.rs` have identical `pub fn` signature lines and identical `pub` item
lines; the only production-code difference is the body of
`remove_owned_root_with_hook`, which now returns `Ok(())` when the owned root
was emptied instead of discarding the emptying result and unconditionally
returning `Err(CleanupFailed)`. Because the `Ok` arm was unreachable in
`0.1.3`, a caller that wrote `cleanup().unwrap_err()` — the idiom the crate's
own documentation invited — now **panics** at runtime rather than compiling
and failing to compile differently. That is a runtime behaviour change in a
release whose public surface did not move, and it is the only consumer-visible
break in this crate.

Third, the archive published **under no deviation**. The core plan's §9.1
records, **before the first upload of the train**, that the hosted
packaged-service step resolves `eggup-core` against crates.io and therefore
cannot be green before the train's first upload. That exception was recorded
before the archive's own upload, `eggup-core 0.2.0` was already registry-
visible when the archive was published, and hosted run `40079833945` is green
on **all five** lanes on the publication source commit. Nothing in this
record should be read as the archive having been published under a deviation.

Fourth, the train order is provable from the registry itself, not only from
the plan. The crates.io `created_at` timestamps are monotonic: core
`13:46:31.983818Z`, archive `13:50:16.471564Z`, acquisition `13:50:19.661533Z`,
curl `13:50:29.938295Z`, eggfetch `13:51:02.514990Z`, service
`13:51:17.639169Z`, eggpack `13:51:33.906205Z`. The archive is second, as
planned.

## Publication record

| Field | Value |
|---|---|
| Registry version | `0.2.0` |
| Checksum (SHA-256) | `ce35edd5d2246f91a0b98a7eddbecc25ad5e944c1e92193887e6490e11ff57ad` |
| Published-table form | `ce35edd5…ff57ad` |
| Crate size | 34 275 bytes |
| Publisher | `dbowm91` |
| Published at | `2026-10-24T13:50:16.471564Z` |
| Yanked | no |
| Source commit in tarball | `b61260857fdb3904a091c503b1dff05251ec0bff` (`path_in_vcs`: `crates/eggup-archive`) |
| Dirty tree recorded | no |
| Published dependency list | `flate2 ^1.1.10`, `fs_at ^0.2.1`, `sha2 ^0.10.9`, `tar ^0.4.46`, `zip ^6.0.0` — five normal dependencies, **no `eggup-core`** |
| Package file list | `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `src/lib.rs` — **no `tests/` entry** |

Prior versions, all unyanked: `eggup-archive` `0.1.2` (published
`2026-09-28T21:53:39.799017Z`, checksum `758e9564…1c8292`) and `0.1.3`
(published `2026-10-06T20:44:03.580599Z`, checksum `a28af652…abe9824c`). No
version was re-uploaded, re-published, or yanked by this milestone.

## Requirement → evidence matrix

| # | Requirement (plan gate / AC) | Evidence (file:line or artifact) | Command and result |
|---|---|---|---|
| 1 | `eggup-archive 0.2.0` is on crates.io with its checksum, size, publisher, and timestamp (G1, AC1) | crates.io version readback: `0.2.0`, checksum `ce35edd5…ff57ad`, `crate_size = 34275`, `created_at = 2026-10-24T13:50:16.471564Z` | `curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' https://crates.io/api/v1/crates/eggup-archive/versions` → `0.2.0` present; passed |
| 2 | No existing version was re-published, re-uploaded, or yanked (G1, AC1) | the same version list: `0.1.2` and `0.1.3` both report `yanked = false`; `0.2.0` reports `yanked = false` | same readback; passed |
| 3 | The publishing identity holds publish authority (G2, AC1) | crates.io owners readback: `dbowm91` (David Bowman); the two pre-existing versions were published by the same identity | `GET /api/v1/crates/eggup-archive/owners` → `dbowm91`; passed |
| 4 | The published version is exactly `0.2.0`, matching `[workspace.package] version` (G3, AC2) | published tarball's normalized `Cargo.toml`: `version = "0.2.0"`; the repository's `Cargo.toml` at `b612608` carries `[workspace.package] version = "0.2.0"`, inherited through `version.workspace = true` | `tar -xzOf … Cargo.toml`; passed |
| 5 | The public API surface is unchanged (G4, AC3) | published `0.1.3` vs published `0.2.0` `src/lib.rs`: every `pub fn` signature line identical, every `pub` item line identical; 13 top-level `pub` items in each | `diff` of the two published tarballs' `src/lib.rs` → no signature delta; passed |
| 6 | The dependency surface is `flate2`, `fs_at`, `sha2`, `tar`, `zip` and nothing else (G5, AC4) | published tarball's normalized `Cargo.toml` `[dependencies]`; crates.io dependency readback for `0.2.0` | `GET /api/v1/crates/eggup-archive/0.2.0/dependencies` → exactly those five, all `kind = normal`; passed |
| 7 | The packaged manifest carries no `git`, `path`, or `patch` dependency, and **no `eggup-core` entry at all** (G6, AC5) | the published tarball's normalized `Cargo.toml` has an **empty** `[dev-dependencies]` section — the path-only dev-dependency was stripped; crates.io readback has no `eggup-core` | `tar -xzOf … Cargo.toml` → `[dev-dependencies]` empty; passed |
| 8 | The package file list is recorded and contains no `tests/` entry (G7, AC6) | published tarball listing: 7 entries — `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `src/lib.rs` | `tar -tzf` on the published `.crate`; passed |
| 9 | The hosted matrix is green on the publication source commit (G8, AC7) | run `40079833945`, `headSha = b61260857fdb3904a091c503b1dff05251ec0bff`, `conclusion = success`; all five lanes green | `gh run view 40079833945 --json jobs` → 5/5 `success`; passed |
| 10 | The Windows lane **executed** the archive tests (G8, AC7) | job `120888982306` (windows-latest): `eggup_archive` lib 43 passed, `m005_cleanup_contract` 3 passed, `m005_member_write_parity` 3 passed, 0 failed | raw job log; passed — see *Per-platform results* |
| 11 | The changelog is truthful (G9, AC9) | `crates/eggup-archive/CHANGELOG.md` `## 0.2.0` section: attributes the cleanup-contract correction and the shared member-write helper to `0.2.0`, states the `cleanup()` result-handling migration, records the five-dependency surface and the stripped `eggup-core` dev-edge, and does not present `0.1.3` content as new | read at review; passed |
| 12 | No unclassified source delta under `crates/eggup-archive/src/**` (G10, AC10) | `git diff --stat b75f73b..b612608 -- crates/eggup-archive/src/` → empty; `git diff --stat b612608 -- crates/eggup-archive/src/` → empty | both diffs empty; passed |
| 13 | Train order: `eggup-core 0.2.0` was registry-visible before the archive upload (G11, AC11) | crates.io `created_at`: core `2026-10-24T13:46:31.983818Z` < archive `2026-10-24T13:50:16.471564Z`; the archive's own published dependency list contains **no** `eggup-core`, so this is a train-order gate, not a dependency gate | crates.io readback of both crates' version lists; passed |
| 14 | Maintainer authorization was recorded before the upload, and the upload was the only publication act (G12, AC12) | the publication was performed under this plan's W5/W6 gate; the crates.io readback shows exactly one new version for this crate (`0.2.0`) and no other version touched | crates.io version list; passed |
| 15 | The local gate is green on the publication source commit (§7.1, AC8) | `./scripts/check-local.sh` → **exit 0**: fmt clean, clippy exit 0, `cargo test --workspace --all-targets --all-features --locked` → **528 passed, 0 failed, 1 ignored**, doc clean, `documentation-truth checks passed` | run at the publication source commit; the single ignored test is the pre-existing ignored `eggup-curl` unit test at `crates/eggup-curl/src/lib.rs:1583` — **not** introduced by this milestone, and **not** an archive test; passed |
| 16 | The MSRV floor of 1.89 is checked (§7.2, AC8) | hosted `msrv` lane (Rust `1.89.0`, ubuntu-latest) green in run `40079833945`; the review machine's active toolchain **is** the floor: `rustc 1.89.0 (29483883e 2025-08-04)` / `cargo 1.89.0`, with `1.89.0-aarch64-apple-darwin` installed | `cargo +1.89.0 check --workspace --all-targets --locked` → exit 0 on macOS; passed |
| 17 | The package inventory and the packaged manifest's dependency section are recorded (§7.3, §10) | read from the **published** tarball rather than from a local re-package — see the note in *Exact commands run* | `tar -tzf` and `tar -xzOf` on the published `.crate`; passed |
| 18 | The published bytes originate from the publication source commit (§10, AC2) | `.cargo_vcs_info.json` in the published tarball: `"sha1": "b61260857fdb3904a091c503b1dff05251ec0bff"`, `"path_in_vcs": "crates/eggup-archive"`, no `dirty` key | `tar -xzOf … .cargo_vcs_info.json`; passed |
| 19 | The registry-only fixture passes against the published crate (§4, §7.5, AC13) | `/tmp/eggup-registry-fixtures/archive/` — manifest `eggup-archive = "=0.2.0"`; the lockfile's only source line is `source = "registry+https://github.com/rust-lang/crates.io-index"`; **zero** `eggup-core` entries in the lockfile | `cargo test --locked --offline` → **2 passed, 0 failed**: `published_archive_extracts_a_bounded_allowlisted_member`, `published_archive_rejects_an_empty_plan`; passed |
| 20 | The registry-only fixture's tree resolves from the registry only (§4, §7.5, AC13) | `cargo tree --locked --offline --depth 1` → `registry-fixture-archive` → `eggup-archive v0.2.0`, `flate2`, `sha2`, `tar`; every entry carries the registry source line; no `path`, `git`, or `patch` override in the manifest, the workspace table, or a `.cargo/config.toml` (none exists) | `cargo tree --locked --offline` in the fixture; passed |
| 21 | The tag and release state is correct (§5, AC14, §10) | `git tag -l` → `v0.1.1`, `v0.1.2`, `v0.1.3`, `v0.2.0`; `git rev-list -n1 v0.2.0` → `b61260857fdb3904a091c503b1dff05251ec0bff`; `git cat-file -t v0.2.0` → `tag` (annotated); the earlier tags were not moved (`v0.1.1` → `881c95ff…`, `v0.1.2` → `e8e07eb5…`, `v0.1.3` → `bd436836…`); release `0.2.0` published `2026-10-24T13:54:03Z` | `gh release view v0.2.0` → `https://github.com/eggstack/eggup/releases/tag/v0.2.0`; passed |
| 22 | The consumer-unblock reconciliation is recorded (§10, AC15) | the crates.io readback lists all seven `0.2.0` train crates as published and unyanked, with monotonic timestamps; this milestone covers the archive's own slice | crates.io readback of all seven crates; passed |
| 23 | The published package ships no `tests/` fixtures, and the `cleanup().unwrap_err()` migration note is carried (AC6, §10) | published tarball listing (row 8); the migration note is in the published `CHANGELOG.md` itself, in the four `cleanup()` rustdoc blocks, and in the `README.md` | `tar -tzf`; `tar -xzOf … CHANGELOG.md`; passed |
| 24 | The closure record exists at the required path (AC15) | this file, `plans/closure/archive-extraction/006-status.md` | present at review; passed |

## Production implementation evidence

### The published bytes are the repository's bytes

`git diff --stat b612608 -- crates/eggup-archive/src/` is empty, and the
published tarball's `.cargo_vcs_info.json` names `b61260857fdb3904a091c503b1dff05251ec0bff`
with no dirty-tree flag. The archive's sources under review are the published
sources, byte for byte.

### The public API did not move

Comparing the **published** `0.1.3` and `0.2.0` `src/lib.rs` extracted from the
two registry tarballs:

```text
$ diff <(grep -E "^\s*pub fn " v013/src/lib.rs) \
       <(grep -E "^\s*pub fn " v020/src/lib.rs)
(identical pub fn signatures)

$ diff <(grep -E "^\s*pub (enum|struct|fn|const|type) " v013/src/lib.rs) \
       <(grep -E "^\s*pub (enum|struct|fn|const|type) " v020/src/lib.rs)
(identical pub item lines)
```

Both versions expose 13 top-level `pub` items. `ArchiveFormat`,
`ArchiveLimits` (including `strict_default()`), `ArchiveMember`,
`ArchivePlan`, `ExtractionErrorKind` with its fourteen variants,
`ExtractionError` with `kind()` and `residue_path()`, `ExtractedMember`,
`ExtractedArchive`, `PersistedExtraction`, `BoundMember`, `BoundExtraction`,
`DeferredCleanup`, and the free function `extract` are all present and
unchanged. The four `cleanup()` entry points keep the signature
`fn cleanup(self) -> Result<(), ExtractionError>`; every `root()` accessor is
present.

### The one production behaviour change

The only production-code hunk between the two published versions is the body
of the private helper `remove_owned_root_with_hook`:

```diff
     if let Some(hook) = hook {
         hook(&path);
     }
-    let _ = empty_result;
-    Err(ExtractionError::new(ExtractionErrorKind::CleanupFailed).with_residue(path))
+    match empty_result {
+        Ok(()) => Ok(()),
+        Err(_) => Err(ExtractionError::new(ExtractionErrorKind::CleanupFailed).with_residue(path)),
+    }
 }
```

Before, the emptying result was discarded (`let _ = empty_result;`) and the
function unconditionally returned `Err(CleanupFailed)`, so the `Ok` arm of
`Result<(), ExtractionError>` was **unreachable in a published crate** while
the rustdoc on all four entry points said they "remove" the extraction
directory. After, the `Ok` arm is reachable and truthful: `Ok(())` means the
owned root object held no entries at the final handle-relative listing;
`Err(..)` with `kind() == CleanupFailed` means it could not be fully emptied,
the residue may be partially emptied, its emptiness is not guaranteed, and
`residue_path()` names the recorded root path. The crate still never removes
the root directory itself and never traverses the root pathname; the residual
empty directory is the caller's to remove.

The remaining diff between the two published versions is rustdoc and test
bodies: the four `cleanup()` rustdoc blocks were rewritten to the shared
`Ok`/`Err` contract, `ExtractionErrorKind::CleanupFailed` and
`ExtractionError::residue_path` gained the two-meaning and advisory-path
wording, and the crate's inline tests were migrated off the
`cleanup().unwrap_err()` idiom — including the new
`cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence`
fixture.

### The dependency surface and the stripped dev-edge

`crates/eggup-archive/Cargo.toml:16-21` declares the five runtime
dependencies; `:23-24` declares `eggup-core` as a **path-only dev-dependency**.
The workspace tree shows both edges:

```text
$ cargo tree -p eggup-archive --locked --offline
eggup-archive v0.2.0 (/Users/davidbowman/projects/eggup/crates/eggup-archive)
├── flate2 v1.1.10 …
├── fs_at v0.2.1 …
├── sha2 v0.10.9 …
├── tar v0.4.46 …
└── zip v6.0.0 …
[dev-dependencies]
└── eggup-core v0.2.0 (/Users/davidbowman/projects/eggup/crates/eggup-core)
```

The **published** manifest, by contrast, carries an empty `[dev-dependencies]`
section and no `eggup-core` entry anywhere. That is the proof that the
path-only dev-dependency is stripped at packaging, and it is why this crate
needed no dependency widening — the published `0.1.3` registry dependency list
was already the same five, with no `eggup-core`.

### The package contents

`crates/eggup-archive/Cargo.toml:14` sets `exclude = ["tests/", "benches/"]`,
and the published tarball confirms it: 7 entries, no `tests/` and no `benches/`
directory. The M005 integration fixtures (`m005_cleanup_contract.rs`,
`m005_member_write_parity.rs`) are **not** shipped. The package ships
`src/lib.rs`, `README.md`, `CHANGELOG.md`, the normalized `Cargo.toml`,
`Cargo.toml.orig`, `Cargo.lock`, and `.cargo_vcs_info.json`.

## Exact commands run, with results

### Run at review, on this machine (macOS, darwin arm64)

```text
$ rustc --version
rustc 1.89.0 (29483883e 2025-08-04)

$ cargo --version
cargo 1.89.0 (c24e10642 2025-06-23)

$ cargo test -p eggup-archive --locked --offline
running 50 tests
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
(57 archive tests passed on macOS)

$ cargo +1.89.0 check --workspace --all-targets --locked
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.04s
(exit 0)

$ cargo tree -p eggup-archive --locked --offline
(five runtime dependencies; `eggup-core` under [dev-dependencies] only)

$ git diff --stat b75f73b..b612608 -- crates/eggup-archive/src/
(empty)

$ git diff --stat b612608 -- crates/eggup-archive/src/
(empty)

$ git status --porcelain
 M CHANGELOG.md
 M crates/eggup-acquisition/CHANGELOG.md
 M crates/eggup-archive/CHANGELOG.md
 M crates/eggup-core/CHANGELOG.md
 M crates/eggup-curl/CHANGELOG.md
 M crates/eggup-eggfetch/CHANGELOG.md
 M crates/eggup-eggpack/CHANGELOG.md
 M crates/eggup-service/CHANGELOG.md
 M plans/registry.md
 M plans/subsystems/acquisition-transport-roadmap.md
 M plans/subsystems/archive-extraction-roadmap.md
 M plans/subsystems/eggpack-manifest-interoperability-roadmap.md
 M plans/subsystems/service-lifecycle-roadmap.md
 M plans/subsystems/verified-update-core-roadmap.md
?? plans/closure/acquisition-transport/017-status.md
?? plans/closure/service-lifecycle/017-status.md
(parent-owned close-out edits; no crate source modified)

$ git tag -l
v0.1.1
v0.1.2
v0.1.3
v0.2.0

$ git rev-list -n1 v0.2.0
b61260857fdb3904a091c503b1dff05251ec0bff

$ git cat-file -t v0.2.0
tag

$ git rev-list -n1 v0.1.1 ; git rev-list -n1 v0.1.2 ; git rev-list -n1 v0.1.3
881c95ff069d3d465a282cb6a495ba6fcb70cb6f
e8e07eb538d0eef18ea4cb4ace3bb905c316da72
bd4368369aaf0ae1de2ce213fc16689281bdd664

$ gh run view 40079833945 --json databaseId,headSha,conclusion,status,url
{"conclusion":"success","databaseId":40079833945,"headSha":"b61260857fdb3904a091c503b1dff05251ec0bff","status":"completed","url":"https://github.com/eggstack/eggup/actions/runs/40079833945"}

$ gh run view 40079833945 --json jobs
Windows archive, acquisition, and service tests and check => success
MSRV check => success
macOS tests => success
Stable checks => success
Linux systemd service qualification => success

$ gh run view 40100087952 --json databaseId,headSha,conclusion,status,url
{"conclusion":"success","databaseId":40100087952,"headSha":"2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219","status":"completed",…}

$ gh release view v0.2.0 --json tagName,name,isDraft,isPrerelease,publishedAt,url
{"isDraft":false,"isPrerelease":false,"name":"0.2.0","publishedAt":"2026-10-24T13:54:03Z","tagName":"v0.2.0","url":"https://github.com/eggstack/eggup/releases/tag/v0.2.0"}

$ grep -n "cargo publish" .github/workflows/ci.yml
(no output — CI never publishes)
```

### Registry readback (read-only GETs)

```text
$ curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' \
    https://crates.io/api/v1/crates/eggup-archive/versions
0.2.0 2026-10-24T13:50:16.471564Z yanked=False crate_size=34275
       checksum=ce35edd5d2246f91a0b98a7eddbecc25ad5e944c1e92193887e6490e11ff57ad
0.1.3 2026-10-06T20:44:03.580599Z yanked=False crate_size=30920
       checksum=a28af652ec11b5309eb7b628d0c875262be2c491a7fb3c5d3fb6c50eabe9824c
0.1.2 2026-09-28T21:53:39.799017Z yanked=False crate_size=27102
       checksum=758e9564359dd9fdf9a7ee4fab04f049986ff058564c5793664f51085b1c8292

$ curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' \
    https://crates.io/api/v1/crates/eggup-archive/0.2.0/dependencies
normal flate2 ^1.1.10 optional=False
normal fs_at  ^0.2.1  optional=False
normal sha2   ^0.10.9 optional=False
normal tar    ^0.4.46 optional=False
normal zip    ^6.0.0  optional=False
(no `eggup-core` entry)

$ curl -fsSL -H 'User-Agent: eggup-publication-check/0.2.0' \
    https://crates.io/api/v1/crates/eggup-archive/owners
dbowm91  David Bowman
```

The seven train crates' `0.2.0` `created_at` timestamps, read in one pass:

```text
eggup-core       2026-10-24T13:46:31.983818Z  2c86a0eb…eb23b8  size=78576
eggup-archive    2026-10-24T13:50:16.471564Z  ce35edd5…ff57ad  size=34275
eggup-acquisition 2026-10-24T13:50:19.661533Z 2ef74b4d…299fc3  size=21944
eggup-curl       2026-10-24T13:50:29.938295Z  15cf536f…90d3f7  size=30971
eggup-eggfetch   2026-10-24T13:51:02.514990Z  9fc8f648…658721  size=24363
eggup-service    2026-10-24T13:51:17.639169Z  31dd2e22…fe1b4a  size=94031
eggup-eggpack    2026-10-24T13:51:33.906205Z  33dc1da4…14f1f6  size=38981
```

### Published artifact inspection

```text
$ tar -tzf eggup-archive-0.2.0.crate
eggup-archive-0.2.0/.cargo_vcs_info.json
eggup-archive-0.2.0/CHANGELOG.md
eggup-archive-0.2.0/Cargo.lock
eggup-archive-0.2.0/Cargo.toml
eggup-archive-0.2.0/Cargo.toml.orig
eggup-archive-0.2.0/README.md
eggup-archive-0.2.0/src/lib.rs

$ shasum -a 256 eggup-archive-0.2.0.crate
ce35edd5d2246f91a0b98a7eddbecc25ad5e944c1e92193887e6490e11ff57ad  eggup-archive-0.2.0.crate

$ tar -xzOf eggup-archive-0.2.0.crate eggup-archive-0.2.0/.cargo_vcs_info.json
{
  "git": {
    "sha1": "b61260857fdb3904a091c503b1dff05251ec0bff"
  },
  "path_in_vcs": "crates/eggup-archive"
}
```

### Registry-only fixture

```text
$ cd /tmp/eggup-registry-fixtures/archive
$ grep -c 'eggup-core' Cargo.lock
0
$ grep -n 'source =' Cargo.lock | sort -u -t'"' -k2
8:source = "registry+https://github.com/rust-lang/crates.io-index"
$ ls .cargo/config.toml
(no such file)
$ cargo tree --locked --offline --depth 1
registry-fixture-archive v0.0.0 (/private/tmp/eggup-registry-fixtures/archive)
├── eggup-archive v0.2.0
├── flate2 v1.1.10
├── sha2 v0.10.9
└── tar v0.4.46
$ cargo test --locked --offline
running 2 tests
test published_archive_rejects_an_empty_plan ... ok
test published_archive_extracts_a_bounded_allowlisted_member ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The fixture's `published_archive_extracts_a_bounded_allowlisted_member` builds
a tar.gz, declares one member with its exact size and SHA-256, calls
`extract(&plan, &dir)`, asserts `bytes_written()` and `sha256()` against the
declared digest, captures the residual path from `root()` **before** the
consuming call, and then **matches** the `cleanup()` result — asserting on the
`Ok` arm that the root still exists, holds zero entries, and is the caller's
to remove. It deliberately does not call `cleanup().unwrap_err()`.

### Commands **not** run, and why

- **`cargo package -p eggup-archive --locked`, `cargo package … --list`, and
  `cargo publish … --dry-run` (plan §7.3) — NOT run at review time.** The
  review session's command set excludes `cargo package` and `cargo publish`.
  The package inventory this plan asks for was instead read from the
  **published** `.crate` tarball, which is the authoritative artifact and a
  stronger check than a local re-package's listing: the file list, the
  normalized manifest, the dependency section, the empty
  `[dev-dependencies]`, and `.cargo_vcs_info.json` all come from the bytes
  crates.io actually holds.
- **`cargo publish -p eggup-archive --locked` — NOT run at review time.** The
  upload was performed under the plan's W5/W6 gate before this review; the
  registry readback is the evidence of its outcome.
- **`cargo test --workspace --all-targets --all-features --locked` — NOT run
  at review time.** The whole-workspace gate was run at the publication
  source commit and is recorded in the shared evidence file
  (`/tmp/eggup-release/closure-evidence.md`): exit 0, **528 passed, 0 failed,
  1 ignored**. The single ignored test is the pre-existing ignored
  `eggup-curl` unit test at `crates/eggup-curl/src/lib.rs:1583`
  (`#[ignore = "environment-conditional: needs a host that blackholes the
  route to 10.255.255.1:81 …"]`). It is **not** introduced by this milestone
  and **not** an archive test.
- **`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets
  --all-features --locked -- -D warnings`, `cargo doc --workspace --no-deps
  --locked`, `./scripts/check-local.sh` — NOT re-run at review time.** They
  were run at the publication source commit and are recorded in the shared
  evidence file. `./scripts/check-docs-truth.sh` **was** re-run at review
  time and passes (see below).

### Documentation-truth gate, re-run at review

```text
$ ./scripts/check-docs-truth.sh
…
documentation-truth checks passed
```

## Per-platform results

Each platform's result is reported from the lane that ran on it. No platform
result is inferred from another.

| Platform | Result |
|---|---|
| **macOS (darwin arm64, this machine)** | **RUN — native.** `cargo test -p eggup-archive --locked --offline`: 50 inline + 4 (`m005_cleanup_contract`) + 3 (`m005_member_write_parity`) = **57 tests passed, 0 failed, 0 ignored**. `cargo +1.89.0 check --workspace --all-targets --locked` → exit 0. The registry-only fixture ran here too: **2 passed, 0 failed**. The whole-workspace local gate was run at the publication source commit and exited 0 (528 passed / 0 failed / 1 ignored); it was **not** re-run at review time. |
| **Linux** | **PASS — hosted.** Run `40079833945`, job `120888982464` (`Stable checks`, ubuntu-latest) and job `120888982366` (`MSRV check`, ubuntu-latest, Rust `1.89.0`), plus job `120888983425` (`Linux systemd service qualification`, ubuntu-latest) — all `success`. The `Stable checks` lane executed the archive tests: `eggup_archive` lib **50 passed**, `m005_cleanup_contract` **4 passed**, `m005_member_write_parity` **3 passed**, 0 failed. The `MSRV check` lane runs `cargo check --workspace --all-targets --locked` — it **checks** the archive at the 1.89 floor but does **not** execute the archive tests. |
| **Windows** | **PASS — hosted.** Run `40079833945`, job `120888982306` (`Windows archive, acquisition, and service tests and check`, windows-latest) — `success`. The lane **executed** the archive tests: `eggup_archive` lib **43 passed**, `m005_cleanup_contract` **3 passed**, `m005_member_write_parity` **3 passed**, 0 failed. The lower Windows count is the platform gating, not a gap: the archive's inline suite is 41 platform-neutral tests plus 9 `#[cfg(unix)]` tests (macOS/Linux: 50) plus 2 `#[cfg(windows)]` tests (Windows: 43), and `m005_cleanup_contract`'s foreign-replacement fixture is `#[cfg(unix)]`. The two `#[cfg(windows)]` inline fixtures — `deterministic_race_windows_reparse_after_authority_boundary_is_preserved` and `windows_reparse_replacement_is_not_followed_by_materialization` — **did** execute here. |
| **Hosted CI lanes (stable / msrv / service-systemd / macos / windows-check)** | **PASS** — all five lanes green in run `40079833945` on commit `b612608`. The run's **first attempt** failed in exactly one step, in four lanes, with the same registry-selection error (`failed to select a version for the requirement `eggup-core = "^0.2"` … candidate versions found which didn't match: 0.1.3, 0.1.2, 0.1.1, …`); that step is the packaged-service step, which resolves `eggup-core` against crates.io and therefore could not be green before the train's first upload. The exception was **recorded before the first upload** in the core plan's §9.1, and after the uploads the four affected lanes were re-run on the same commit and passed unchanged. The archive was published **after** the exception was recorded and **after** `eggup-core 0.2.0` was registry-visible, so the archive itself needed no exception. |
| **Packaged-service strict step (finding 16)** | **PASS** — restored in commit `2d6e089` (all four CI lanes now run `cargo package -p eggup-service --locked`, dropping `--no-verify`), and hosted run `40100087952` on `2d6e089` is green. This is a CI change, not an archive change; it is recorded because it is what makes the four lanes' re-run meaningful. |

## Invariant review

The archive subsystem's own invariants (`plans/subsystems/archive-extraction-roadmap.md` §2)
and the crate's safety contract are re-confirmed against the **published**
`0.2.0` bytes, not against the repository's working tree:

| # | Invariant | Evidence | Result |
|---|---|---|---|
| 1 | `eggup-core` remains archive-format independent | the published manifest has no `eggup-core` entry at all; the archive reaches no other Eggup crate | pass |
| 2 | Archive bytes are integrity-verified before extraction is trusted as deployment input | unchanged — the crate consumes an already-verified archive and verifies each declared member's size and SHA-256 after writing | pass |
| 3 | Only caller-declared members may be materialized | unchanged — exact-equality allowlist lookup in the shared `write_declared_member` helper | pass |
| 4 | Extraction never writes into the live installation root | unchanged — no destination-path authority exists in the crate | pass |
| 5 | Traversal, link, special-file, alias, and overwrite cases fail closed | unchanged — path validation and the Windows reserved-device set are byte-identical between the published `0.1.3` and `0.2.0` | pass |
| 6 | Per-member, aggregate, entry-count, path, and diagnostic bounds are finite | unchanged — `ArchiveLimits::strict_default()` and `ArchivePlan::new` are byte-identical between the two published versions | pass |
| 7 | Extracted member size/digest evidence is verified before success | unchanged — verified by the registry-only fixture against the published crate (`bytes_written`, `sha256`) | pass |
| 8 | Cleanup removes only operation-owned extraction state | unchanged in mechanics — handle-relative emptying through the retained root handle; only the `Ok`/`Err` reporting changed | pass |
| 9 | SHA-256 is integrity evidence only — no authenticity or signature claim | no signature, trust-root, provenance, or attestation code or wording is added by this milestone; the published `CHANGELOG.md` states "Integrity remains SHA-256 checksum evidence only; no authenticity or signature claims" | pass |
| 10 | No transport, no live-installation policy, no new dependency | the published dependency list is the same five crates as `0.1.3`; no transport or service dependency | pass |
| 11 | No path-based creation or deletion of the owned root | unchanged — the crate still never removes the root directory and never traverses the root pathname | pass |
| 12 | `unsafe_code` denied; `missing_docs` denied | the published manifest carries `[lints.rust] unsafe_code = "deny"`; the crate's `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` attributes are untouched, and `cargo doc --workspace --no-deps --locked` is clean | pass |
| 13 | The crate's public surface is stable | proved by the published-`0.1.3`-vs-published-`0.2.0` diff (rows 5 of the matrix): identical `pub` lines | pass |

## Failure, rollback, and recovery review

- **Registry immutability is the recovery boundary.** A defect in the published
  `0.2.0` requires a **new patch version** (`0.2.1`), never an overwrite, a
  re-upload, or a yank-and-republish of `0.2.0`. No version of this crate was
  yanked by this milestone and the `0.1.x` line is intact.
- **The `0.1.x` line remains published and unyanked.** A consumer pinned to
  `0.1.2` or `0.1.3` still receives the old always-`Err` cleanup contract.
  That is a property of immutable registry versions, not a defect here; it is
  recorded as F-4.
- **Cleanup that cannot empty the root stays fail-closed.** `Err` with
  `kind() == CleanupFailed` and `residue_path()` naming the recorded root;
  success is never reported and emptiness is never claimed. The deterministic
  inline fixture
  `cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence`
  exceeds the crate's private `CLEANUP_MAX_DEPTH` bound and is published in
  the crate's `src/lib.rs`; it ran in every lane that executes the archive
  tests.
- **Partial emptying.** The bounded emptying may remove some content before
  failing; the result is still `Err(CleanupFailed)`, the residue may be
  partially emptied, and the crate does not re-verify emptiness by pathname on
  that path.
- **Cleanup racing with a rename or replacement.** Unchanged: the emptying
  targets the owned object through the retained handle; a renamed or replaced
  pathname is never followed, traversed, or removed.
- **Concurrent cleanup attempts.** Unchanged: every entry point consumes
  `self`, so exactly one of drop or explicit cleanup runs.
- **Cancellation / restart.** The crate has no cancellation API and none was
  added; no durable state exists, so restart is not applicable.
- **Rollback of this milestone.** The milestone is a publication plus a
  closure record. Rolling it back would mean a new `0.2.1`; there is no
  source change to revert, because `crates/eggup-archive/src/**` is byte-
  identical between the qualified baseline and the publication source commit.

## Compatibility and migration review

- **Published baseline.** `eggup-archive 0.1.3`, published on crates.io
  `2026-10-06T20:44:03.580599Z` from `bd43683`, registry checksum
  `a28af652…abe9824c`. `0.2.0` is the successor.
- **No public API change.** No public signature, field, type, or constructor
  was added, removed, or retyped. `Result<(), ExtractionError>` is retained on
  all four `cleanup()` entry points; `ExtractionErrorKind` keeps its fourteen
  variants; `ArchiveLimits` defaults, plan and limit validation, path
  validation, the Windows reserved-device set, and the finite
  member/aggregate/entry/path bounds are unchanged. This is proved by
  diffing the two **published** tarballs, not by reading the source.
- **One behaviour change, and it is the reason this release needs a migration
  note.** The `cleanup()` success path is now reachable. A caller that wrote
  `cleanup().unwrap_err()` — the idiom the crate's own `0.1.3` rustdoc
  invited, because the `Ok` arm was unreachable — now **panics** on the
  success path. A caller that wrote `cleanup()?` and aborted the flow on error
  now continues on success: the intended fix, but a control-flow change to
  review. A caller that needs the residual path must capture it from
  `root()` **before** the consuming call, because the call consumes the value.
  Apart from that result handling, no consumer migration is required. This is
  a **behaviour** change and not an **API** change precisely because the
  signatures and types did not move: the same call sites compile, and the
  difference is observable only at run time.
- **The migration note is shipped.** It is in the published `CHANGELOG.md`
  itself, in all four `cleanup()` rustdoc blocks, and in the published
  `README.md` — so a consumer reading the registry, docs.rs, or the crate
  source sees it without reading this record.
- **The dependency surface is unchanged**, so a consumer's resolution does not
  move: five runtime dependencies, the same as `0.1.3`.
- **The published package ships no `tests/` fixtures.** The M005 integration
  fixtures are excluded by `Cargo.toml:14` and are absent from the published
  tarball. No release note or claim states otherwise.
- **No authenticity, signature, provenance-attestation, or crash-journaling
  claim is made.** SHA-256 remains integrity evidence only.
- **Downstream.** `eggup-eggpack` pins `eggup-archive` exactly, and the
  `0.2.0` train re-pinned it to `=0.2.0` in the same release-prep change set;
  the published `eggup-eggpack 0.2.0` therefore carries the corrected cleanup
  contract.

## Security review

- **No path-based deletion.** The crate never removes the root directory and
  never recursively traverses the root pathname; `remove_owned_root_with_hook`
  performs only `fs_at` `*at` operations relative to the retained handle. The
  publication changes the **reporting** of that operation, not its authority.
- **No path-based creation.** `create_private_file_at` and its `create_new` /
  `follow(false)` semantics are untouched.
- **Path validation unchanged.** `validate_archive_path` and
  `validate_output_name` are byte-identical between the published `0.1.3` and
  `0.2.0`; the Windows reserved-device set is unchanged and still covered by
  the crate's regression guard, which **executed** in the Windows lane.
- **No authenticity or signature language introduced.** Integrity remains
  SHA-256 checksum evidence only.
- **No new dependency, no runtime `eggup-core` edge, no transport dependency,
  no Eggpack dependency.** The published manifest carries the same five
  runtime dependencies as `0.1.3` and an empty `[dev-dependencies]`.
- **`unsafe_code` still denied** at the workspace level and in the published
  manifest; the crate's `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`
  attributes are untouched and `cargo doc` is clean.
- **The recorded root path stays advisory.** The rustdoc says so explicitly
  for all four entry points and for `residue_path()`, and the published
  `0.2.0` carries that wording.
- **Fail-closed preserved.** A cleanup that cannot empty the root never
  reports success and never claims the residue is empty.
- **No secret, credential, or token material** is introduced by the
  publication, and the published artifact contains only the seven files listed
  above.

## Documentation and operations evidence

| Artifact | State at publication |
|---|---|
| `crates/eggup-archive/CHANGELOG.md` (repository) | `## 0.2.0 — 2026-10-24`. Attributes the cleanup-contract correction and the shared member-write helper to `0.2.0`, states the `cleanup()` result-handling migration and the `unwrap_err()` panic consequence, records the five-dependency surface and the stripped `eggup-core` dev-edge, and states that integrity is SHA-256 evidence only. It does not present `0.1.3` content as new. |
| `crates/eggup-archive/CHANGELOG.md` (published tarball) | `## 0.2.0 — 2026-10-23` — the **cut** date, not the publication date. See F-1. The body text is otherwise identical. |
| `crates/eggup-archive/README.md` (published tarball) | Carries the four-entry-point `Ok`/`Err` cleanup contract, the caller-owned residual directory, the advisory-path statement, and the `unwrap_err()` migration note. |
| `crates/eggup-archive/src/lib.rs` (published tarball) | The four `cleanup()` rustdoc blocks carry the shared contract; `ExtractionErrorKind::CleanupFailed`, `ExtractionError::residue_path`, the `ExtractedArchive` struct doc, and `extract`'s drop-semantics paragraph carry the corrected wording. |
| `architecture/archive-extraction.md` | Updated by M005; this milestone changes no architecture file. The M006 publication adds no new architectural fact — the contract it publishes is the contract M005 already documented. |
| `docs/` | **No change.** The `docs/` tree is non-normative and carries no archive cleanup or residue wording to correct. |
| Operations | No new configuration, no operator migration step, no runtime behaviour change beyond the documented `cleanup()` result handling. |
| CI | `.github/workflows/ci.yml` contains no publish step; the workflow builds, checks, tests, packages, and documents. Publication is manual and plan-gated. |

**The date discrepancy, recorded plainly.** The repository's per-crate
`## 0.2.0` heading reads **2026-10-24** (the publication date, per the
repository convention every earlier per-crate release section follows), while
the **published** `.crate` tarball carries **2026-10-23** (the cut date) in
that heading — verified by extracting the published `CHANGELOG.md` from the
tarball and comparing it with the repository's copy. Registry state is
immutable, so the published heading cannot be corrected there. The difference
is recorded in `plans/registry.md`; this record states it rather than papering
over it. The body text of the two files is otherwise identical.

## Unresolved findings with severity

### F-1 — the published tarball's `## 0.2.0` heading date differs from the repository's

**Severity: low. Open — not fixable on the registry.**

The published `eggup-archive 0.2.0` tarball carries `## 0.2.0 — 2026-10-23`
(the cut date); the repository's copy carries `## 0.2.0 — 2026-10-24` (the
publication date, the repository's convention). The body text is identical.
Registry state is immutable: the published heading cannot be corrected
without a new version, and publishing a `0.2.1` to correct a heading date
would be a worse outcome than the discrepancy. **Why low:** documentation-only,
no behavioural or safety consequence, and the difference is disclosed in the
published changelog's own surrounding text and in `plans/registry.md`.
**Disposition:** recorded, not corrected. This is the archive's slice of the
repository-wide one-line difference the registry already records for all seven
crates.

### F-2 — the `Err(CleanupFailed)` path is still not proven from an external integration test

**Severity: low. Open — carried forward from M005 (F-2), unchanged.**

The crate exposes no portable public failure trigger; the only deterministic
mechanism is exceeding the crate's private cleanup bounds
(`CLEANUP_MAX_DEPTH`, `CLEANUP_MAX_ENTRIES_PER_PASS`), which are not public
constants, so the fixture must live inline in `src/lib.rs`. It does, and it
is published. The registry-only fixture therefore exercises the `Ok` arm and
the bounded-plan rules, not the `Err` arm. **No external coverage of the
failure path is claimed.** The milestone did not change this and does not
worsen it.

### F-3 — the deferred `lib.rs` module decomposition has no milestone of its own

**Severity: low. Open.**

M005 deferred the decomposition of `crates/eggup-archive/src/lib.rs` into
modules as a pure code move with no consumer-visible effect, and named the
follow-up **"Archive Extraction M006 — `eggup-archive` module decomposition"**.
The M006 number was subsequently consumed by **this** milestone, the
`0.2.0` publication, so the deferred work has **no plan and no milestone
number**. The archive's `src/lib.rs` remains a single module. **Why low:** no
consumer-visible effect, no safety property at stake, and a pure code move
when it is eventually planned. **Disposition:** the numbering collision is
recorded here so the deferral is not silently lost; a future milestone needs a
new number and a plan. The parent owns the roadmap and should register it.

### F-4 — the published `0.1.x` line still carries the pre-`0.2.0` cleanup contract

**Severity: low. Open — not fixable on the registry.**

`eggup-archive 0.1.2` and `0.1.3` remain published and unyanked, and both
carry the always-`Err` cleanup contract with its unreachable `Ok` arm. A
consumer pinned to those versions still receives the old behaviour. **Why
low:** this is a property of immutable registry versions, not a defect in this
milestone; the fix reaches such a consumer only by resolving `0.2.0`. The
`0.1.x` line staying intact is a deliberate policy in this repository.

### F-5 — registry finding 1 (load-sensitive `eggup-curl` sub-second-deadline tests) — **out of scope, visible**

**Severity: low. Open. Blocks nothing. Not an archive concern.**

`build_curl_args_passes_sub_second_deadlines_to_fake_curl` and
`sub_second_total_timeout_kills_child_promptly` carry sub-second wall-clock
budgets and fail intermittently on a loaded host. Hosted CI passes both on
every lane. The defect is test robustness in `eggup-curl`, not transport
semantics and not anything in `eggup-archive`. It is recorded here only so
that a reader of this closure record sees the whole open set of the train
rather than discovering it elsewhere. The owning subsystem is Acquisition
Transport; this milestone neither advances nor blocks it.

### F-6 — the external foreign-replacement fixture stays Unix-gated

**Severity: low. Open — a permanent platform limitation, carried forward from M005 (F-3).**

`foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched` is
`#[cfg(unix)]` because Windows refuses to rename a directory while its handle
is held. The Windows equivalent — a reparse point installed at the recorded
pathname — is covered by the crate's inline `#[cfg(windows)]` fixture
`windows_reparse_replacement_is_not_followed_by_materialization`, which
**executed and passed** in this run's Windows lane (job `120888982306`). The
limitation is structural, not a qualification gap: no platform result can
settle it.

### Out of scope, stated so it is not claimed

- **Core M016 — `eggup-core` contract-doc packaging** is recorded as
  *authorable; plan intentionally unwritten*. It is **not** part of this
  milestone and no plan exists for it. The archive's `0.2.0` package likewise
  ships **no** contract or architecture documentation: its seven files are
  listed in the *Production implementation evidence* section, and no
  architecture or contract document is among them. This record does not claim
  otherwise.
- **No archive finding of medium-or-higher severity is open.** Nothing in this
  milestone's scope forces a corrective.

## Roadmap disposition

- **Archive Extraction M006 → closed.** The milestone's own scope — the
  registry upload of `eggup-archive 0.2.0`, its package qualification, its
  registry-only proof, its tag and release contribution, and this closure
  record — is complete and green on all five hosted lanes of run `40079833945`.
- **The published state.** `eggup-archive 0.2.0` is on crates.io, unyanked,
  checksum `ce35edd5…ff57ad`, published `2026-10-24T13:50:16.471564Z` by
  `dbowm91` from `b612608`, the second crate of the train. `v0.2.0` and
  release `0.2.0` point at that commit; the `v0.1.1`/`v0.1.2`/`v0.1.3` tags
  and the published `0.1.x` versions were not moved or withdrawn.
- **The M005 work is now published.** The cleanup-contract truthfulness and
  the single member-write path that M005 closed in source reach consumers
  through `0.2.0`, and the archive's `0.1.3` baseline is superseded.
- **This milestone's contribution to registry finding 17** is positive and
  should be stated as such in the registry: the archive was **already**
  packageable against the registry and needed **no** dependency widening,
  because its only Eggup edge is a path-only dev-dependency that packaging
  strips. The widening was needed by `eggup-curl`, `eggup-eggfetch`,
  `eggup-service`, and `eggup-eggpack`, not here.
- **The deferred module decomposition (F-3) needs a new milestone number.**
  The M005 follow-up named "M006", which this publication consumed. The
  parent owns the roadmap and should register the work under a fresh number
  with a plan, so the deferral is not lost.
- **Archive Extraction M007** is not registered by this milestone. If the
  module decomposition is taken up, M007 is the next free number in the
  archive subsystem.

## Registry updates (intended, for the parent to apply)

The parent session owns `plans/registry.md`; this closure describes the
intended edits rather than making them.

1. **Record the archive's slice of the `0.2.0` train block** (already
   present in the file at review time): `eggup-archive 0.2.0` published from
   `b612608`, registry checksum `ce35edd5…ff57ad`. The block already states the
   train's one-source-commit ordering and the `v0.2.0` tag and release.
2. **Add the publication detail that the registry table does not carry**: the
   full checksum
   `ce35edd5d2246f91a0b98a7eddbecc25ad5e944c1e92193887e6490e11ff57ad`, the
   crate size 34 275 bytes, the publisher `dbowm91`, and the publication
   timestamp `2026-10-24T13:50:16.471564Z`.
3. **Record the positive slice of finding 17 for the archive.** The finding's
   disposition (closed on 2026-10-24 by Acquisition Transport M017) covers the
   adapters that needed widening. The archive's contribution is distinct and
   should not be folded into it: `eggup-archive` was already packageable
   against the registry and needed no widening, because its only Eggup edge is
   a path-only dev-dependency stripped at packaging. The published dependency
   list names the five runtime dependencies and **no** `eggup-core`.
4. **Record the archive's `cleanup()` behaviour change in the compatibility
   surface.** The shared evidence file already lists it; the registry should
   carry it in the same place as the acquisition and service breaks, so the
   whole `0.2.0` compatibility surface is in one place: no public API change,
   one behaviour change — the `cleanup()` success path is now reachable, so a
   caller that wrote `cleanup().unwrap_err()` now panics and must match on
   the result.
5. **Note the published-heading date difference for the archive.** The
   registry's existing paragraph covers the seven `.crate` files generically
   (`## 0.2.0 — 2026-10-23` in the packages, `## 0.2.0 — 2026-10-24` in the
   repository). The archive adds no new instance of it — F-1 is that same
   one-line difference, verified for this crate — so no new entry is needed,
   only the archive's confirmation that it applies here too.
6. **Do not close finding 1.** The `eggup-curl` sub-second-deadline tests stay
   open at low severity, blocks nothing, and hosted CI passes them on every
   lane (F-5 here).
7. **Do not record the module decomposition as M006.** The M005 follow-up name
   collides with this milestone; the registry should not carry a pointer to a
   milestone number that means something else (F-3).

## Closure checklist against `plans/closure/README.md`

| Required element | Present |
|---|---|
| status | yes — **closed** |
| source plan and roadmap | yes — plan `006-eggup-archive-0.2.0-publication.md`, roadmap `plans/subsystems/archive-extraction-roadmap.md` |
| reviewed repository baseline | yes — `b75f73bf1ad138f79e7e16d50f35671e52bafe96`, with the publication source commit `b61260857fdb3904a091c503b1dff05251ec0bff` identified as **distinct** |
| implementation commits/PRs | yes — `afe6201`, `2161f36`, `b612608` (the publication source commit), `2d6e089`; no PR |
| executive finding | yes — the archive's quiet publication, the no-widening slice of finding 17, the behaviour-not-API change, the no-deviation publication, and the registry-proven train order |
| requirement→evidence matrix | yes — 24 rows, each with a command and an observed result |
| production implementation evidence | yes — the published-bytes proof, the published-`0.1.3`-vs-published-`0.2.0` public-API diff, the `remove_owned_root_with_hook` hunk, the dependency surface and stripped dev-edge, the package contents |
| exact commands run and results | yes — review-time commands, registry readbacks, published-artifact inspection, the fixture run, and an explicit list of the commands **not** run and why |
| per-platform results reported separately | yes — macOS **run** natively (57 archive tests, MSRV check, fixture 2/2); Linux **pass** in hosted run `40079833945` (archive tests executed in the `Stable checks` lane; the `MSRV check` lane checks but does not execute); Windows **pass** in the same run (archive tests executed, 43 + 3 + 3) |
| invariant review | yes — 13 invariants, item by item, against the published bytes |
| failure/rollback/recovery review | yes — including registry immutability, fail-closed cleanup, and the partial-emptying and race semantics |
| compatibility and migration review | yes — the published baseline, the proved absence of an API change, the `cleanup().unwrap_err()` panic, the `root()` capture requirement, the unchanged dependency surface, and the truthful "no `tests/` shipped" statement |
| security review | yes — including the explicit statement that SHA-256 is integrity evidence only |
| documentation/operations evidence | yes — including the published-vs-repository heading date difference |
| unresolved findings with severity | yes — F-1 (low, open), F-2 (low, open), F-3 (low, open), F-4 (low, open), F-5 (low, open, out of scope), F-6 (low, open, permanent platform limitation). **No medium-or-higher finding is open** |
| roadmap disposition | yes |
| registry updates | yes — prose; the parent applies the edits |
