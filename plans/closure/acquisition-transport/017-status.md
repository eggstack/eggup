# Closure record — acquisition-transport M017

## Status

**Closed.**

`eggup-acquisition 0.2.0`, `eggup-curl 0.2.0`, and `eggup-eggfetch 0.2.0` are
on crates.io, unyanked, published from the shared `0.2.0` train commit
`b61260857fdb3904a091c503b1dff05251ec0bff`. The `v0.2.0` annotated tag points
at that commit, release `0.2.0` exists against it, and all three
registry-only proofs pass against the registry rather than against the
workspace. The milestone's stated reason to exist — registry open finding 17,
that the two transport adapters could not be packaged against the registry —
is discharged.

One finding stays open at low severity and blocks nothing: registry finding 1,
the load-sensitive `eggup-curl` sub-second-deadline tests. One bookkeeping
hand-off was raised during the review and has since been discharged: the source
plan's `Status:` header read `ready for handoff`; it now reads `closed`, naming
this record. The registry and the acquisition-transport roadmap both carry
their M017 rows.

Platform evidence is reported per platform below. A lane that was not run is
recorded as **not run**, never inferred from another platform.

## Source plan and roadmap

- Source plan: `plans/implementation/acquisition-transport/017-eggup-acquisition-0.2.0-publication.md`
- Source roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Predecessor closure records: `plans/closure/acquisition-transport/009-status.md`,
  `010-status.md`, `011-status.md` (the `0.1.x` publications), and
  `016-status.md` (the M016 coupling correction whose guard this milestone
  re-points)

## Reviewed repository baseline

| Item | Value |
|---|---|
| Branch | `plans/corrective-batch-2026-10-23` |
| Plan baseline (qualified baseline) | `b75f73bf1ad138f79e7e16d50f35671e52bafe96` (`b75f73b`) |
| Publication source commit | `b61260857fdb3904a091c503b1dff05251ec0bff` (`b612608`) |
| Relationship | **Distinct.** The publication source commit is four commits after the qualified baseline and carries the release-prep work; it is not the baseline |
| Tag | `v0.2.0`, annotated, resolving to `b61260857fdb3904a091c503b1dff05251ec0bff` |
| Working tree at review | dirty, by design: the parent-owned close-out edits (root `CHANGELOG.md`, the seven per-crate `CHANGELOG.md` date corrections, `plans/registry.md`, the five subsystem roadmaps) and this closure record itself are uncommitted at review time. No crate source is modified. |
| HEAD at review | `2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219`, one commit after the publication source commit |

The publication source commit is the same commit for all three crates: each
published tarball's `.cargo_vcs_info.json` records
`"sha1": "b61260857fdb3904a091c503b1dff05251ec0bff"`.

## Implementation commits / PRs

| Commit | Subject | Role |
|---|---|---|
| `afe6201430cb4eba78e809059edef8a7d1e36ebf` | `plans: add the five 0.2.0 publication plans` | authored this milestone's plan and its four siblings' |
| `2161f36967b2d9afda4e2b05f7eb260e664fe761` | `release: bump workspace to 0.2.0 and cut the 0.2.0 changelogs` | the release-prep commit: workspace version bump to `0.2.0`, the dependency widenings, the `eggup-eggpack` re-pin, the eight `0.2.0` changelog cuts, the `docs/releases.md` and `docs/crates.md` updates, the `check-docs-truth.sh` generalisation, and the **publication-coupling guard re-point** |
| `b61260857fdb3904a091c503b1dff05251ec0bff` | `plans(core): record the train-order exception for the first crate` | the publication source commit; the train-order exception recorded in the core plan's §9.1 |
| `2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219` | `ci: restore the strict packaged-service verification` | one commit after the publication source commit; CI only, no crate source |

No pull request was used; publication and the train-order exception were
committed directly on the branch.

## Executive finding

Acquisition Transport M017 published the seam and both of its adapters as one
coupled chain — `eggup-acquisition 0.2.0`, `eggup-curl 0.2.0`,
`eggup-eggfetch 0.2.0` — from the single shared train commit `b612608`, and
the milestone's reason to exist is discharged: registry open finding 17 is
closed, because the two transport adapters can now be packaged against the
registry. The before state was a measured publication failure, not a
suspicion. `cargo package -p eggup-curl` exited 101 with 11 compile errors
and `-p eggup-eggfetch` exited 101 with 4, because both declared
`eggup-acquisition = { version = "0.1.0", path = ... }` — a caret requirement
that does not accept `0.2.0` — while calling the seam's `__adapter_io` and
`__adapter_bound_detail` entry points, which exist only in the unpublished
seam. The same cascade reached `crates/eggup-eggpack/Cargo.toml` and
`crates/eggup-transport-footprint/Cargo.toml`. The requirements were widened
in this publication (`"0.2"` for the two adapters, `=0.2.0` for the leaf), the
seam published first, and each adapter then published against the
registry-visible seam.

The proof that the widening actually took effect is the eggfetch registry-only
fixture, which declares no Eggup dependency other than `eggup-eggfetch` and
whose lockfile resolves `eggup-acquisition 0.2.0` from crates.io
**transitively** — only possible because the adapter's own requirement was
widened to accept the published seam. The curl fixture proves the same for
the other adapter and additionally exercises the seam's retyped
`AcquisitionError`.

The seam's `0.2.0` is source-breaking in exactly two ways, and in no other
way. `AcquisitionError::Io(String)` became
`AcquisitionError::Io { operation, source }` with a live `source()` chain, and
the enum now derives **only** `Debug` — `Clone`, `PartialEq`, and `Eq` are
removed, because `std::io::Error` implements none of them. The `Display`
rendering stays bounded at 512 bytes and truncates only at a UTF-8 character
boundary; `Io` stays terminal under every `CompositionPolicy`; the two
`#[doc(hidden)]` adapter-facing entry points are source-compatible
additions. `eggup-curl 0.2.0` and `eggup-eggfetch 0.2.0` carry no
source-breaking change of their own and are transitively source-breaking
through the seam's `^0.2` requirement. The curl adapter's own public API is
additive — the caller-settable User-Agent, validated as an HTTP field value
and emitted as a literal `--user-agent` argv element ahead of the `--`
separator that isolates the URL; `CurlConfig` is untouched, so the observable
wire behaviour does not change. The eggfetch adapter's public API is
unchanged and `EggfetchTransport` remains shareable across threads.

One production change had to be made by this milestone beyond the upload: the
M016 publication-coupling guard enforced a **pre-publication** invariant — the
seam's `## Unreleased` section had to declare a *next release*, and every
in-workspace dependent had to accept it or record the widening. This
publication discharged that premise, so commit `2161f36` re-pointed the guard
at the seam's **published** version. The published version is now **derived**
from the highest `## X.Y.Z` heading in the seam's own changelog, never from a
literal; every in-workspace dependent must accept it or have the widening
recorded; an unknown shape is a failure, never an assumption of acceptance;
the file still has zero dependencies and still reads the repository's sources
at compile time. The three test names are unchanged because
`crates/eggup-acquisition/tests/assertion_guard.rs` names them as the
required behaviour witnesses. The re-pointing was proven red-first: the
re-target still discriminates.

One finding stays open at low severity — registry finding 1, the
load-sensitive `eggup-curl` sub-second-deadline tests — and it blocks nothing;
hosted CI passes both tests on every lane. The published `0.1.x` line
(`eggup-acquisition 0.1.3`, `eggup-curl 0.1.2`, `eggup-eggfetch 0.1.3`) still
carries the pre-coupling state, so consumers pinned to those versions do not
receive the widening; that is a property of immutable registry versions, not a
defect in this milestone.

## Publication record

| Field | `eggup-acquisition` | `eggup-curl` | `eggup-eggfetch` |
|---|---|---|---|
| Registry version | `0.2.0` | `0.2.0` | `0.2.0` |
| Checksum (SHA-256) | `2ef74b4dce2b9f861b2759a45a5bc96377d8273892b04d45843c2f9348299fc3` | `15cf536fff9bbb44a7dc3ff33f24643386134c25b0cef350989919467c90d3f7` | `9fc8f6480cd6717ed854ac764d0cfc18c6325b7367940149ea2f45f2d5658721` |
| Published-table form | `2ef74b4d…299fc3` | `15cf536f…90d3f7` | `9fc8f648…658721` |
| Crate size | 21 944 bytes | 30 971 bytes | 24 363 bytes |
| Publisher | `dbowm91` | `dbowm91` | `dbowm91` |
| Published at | `2026-10-24T13:50:19.661533Z` | `2026-10-24T13:50:29.938295Z` | `2026-10-24T13:51:02.514990Z` |
| Yanked | no | no | no |
| Source commit in tarball | `b612608…` | `b612608…` | `b612608…` |
| Published dependency list | *(empty)* | `eggup-acquisition ^0.2` | `eggup-acquisition ^0.2`, `eggfetch-core ^0.2.0`, `tokio ^1`, `futures-util ^0.3` |

Prior versions, all unyanked: `eggup-acquisition` `0.1.0`, `0.1.1`, `0.1.2`,
`0.1.3`; `eggup-curl` `0.1.2`; `eggup-eggfetch` `0.1.0`, `0.1.1`, `0.1.2`,
`0.1.3`. No version was re-uploaded or yanked by this milestone.

## Requirement → evidence matrix

| # | Requirement (plan §) | Evidence (file:line or artifact) | Command and result |
|---|---|---|---|
| 1 | All three crates are on crates.io at exactly `0.2.0` (G1, G3, AC1, AC2) | crates.io version readback for `eggup-acquisition`, `eggup-curl`, `eggup-eggfetch` | `GET /api/v1/crates/<crate>/versions` → `0.2.0` present for each; passed |
| 2 | No existing version was re-published, re-uploaded, or yanked (G1, AC1) | same version lists; every prior version reports `yanked = false` | passed |
| 3 | The published version matches `[workspace.package] version` (G3) | `Cargo.toml` `[workspace.package] version = "0.2.0"`; each published tarball's normalised `Cargo.toml` `version = "0.2.0"` | read back from the published tarballs; passed |
| 4 | The published bytes originate from the shared train commit (AC18, §10) | `.cargo_vcs_info.json` in all three tarballs: `"sha1": "b61260857fdb3904a091c503b1dff05251ec0bff"` | `tar -xzOf … .cargo_vcs_info.json`; passed |
| 5 | The seam's published public API matches the stated boundary (G4, AC3) | packaged `src/lib.rs`; compile-fail probe against the registry copy (E0164, E0599, E0369 below) | probe compiled offline against `registry+https://github.com/rust-lang/crates.io-index`; passed |
| 6 | The curl adapter's published public API matches the stated boundary (G5, AC4) | registry fixture B: `published_curl_exposes_the_user_agent_control`, `published_curl_rejects_an_empty_executable` | `cargo test` in the fixture → 2/2; passed |
| 7 | The eggfetch adapter's published public API matches the stated boundary (G5, AC4) | registry fixture C: `published_eggfetch_builds_a_transport`, `published_eggfetch_reports_a_version` | `cargo test` in the fixture → 2/2; passed |
| 8 | The seam keeps **zero** `[dependencies]`, before and after (G6, AC5) | published tarball's normalised `Cargo.toml` has an empty `[dependencies]`; crates.io dependency list for `0.2.0` is empty; `cargo tree -p eggup-acquisition --depth 1` prints the crate alone | passed |
| 9 | The curl adapter's published dependency surface is `eggup-acquisition ^0.2` only (G6, AC5) | normalised manifest `[dependencies.eggup-acquisition] version = "0.2"`; crates.io readback `req = ^0.2`, one normal dependency | passed |
| 10 | The eggfetch adapter's published dependency surface (G6, AC5) | normalised manifest: `eggfetch-core 0.2.0`, `eggup-acquisition 0.2`, `futures-util 0.3`, `tokio 1`; crates.io readback agrees | passed |
| 11 | No published manifest carries a `git`, `path`, or `patch` dependency (G7, AC6) | normalised manifests contain no `path`, `git`, or `[patch]`; crates.io readback shows only registry requirements | passed |
| 12 | Package contents ship no `tests/` or `benches/` entry (G8, AC7) | tarball listings: seam = 7 files (`src/lib.rs`, `README.md`, `CHANGELOG.md`, `Cargo.toml`, `Cargo.toml.orig`, `Cargo.lock`, `.cargo_vcs_info.json`); curl and eggfetch add `examples/<crate>_fetch.rs` only | `tar -tzf` on the cached `.crate` files; passed |
| 13 | The hosted matrix is green on the publication source commit (G9, AC8) | run `40079833945`; five lanes green — see the shared evidence file for the per-lane results and the first-attempt failure and re-run | passed |
| 14 | The local gate is green on the publication source commit (§7.1, AC9) | `./scripts/check-local.sh` exit 0: fmt clean, clippy exit 0, test 528 passed / 0 failed / 1 ignored, doc clean, documentation-truth passed. The one ignored test is the pre-existing ignored `eggup-curl` unit test, whose reason is recorded in the M016 closure; it is not introduced by this milestone | passed |
| 15 | The MSRV floor of 1.89 is checked (§7.2) | hosted `msrv` lane green in run `40079833945`; local `rustup toolchain list` shows `1.89.0-aarch64-apple-darwin` installed and `1.89-aarch64-apple-darwin` active | passed |
| 16 | The dependency trees are as stated (§7.3, AC5) | `cargo tree -p <crate> --locked --offline --depth 1` for the three crates | seam: no edges; curl: `eggup-acquisition` only; eggfetch: `eggfetch-core`, `eggup-acquisition`, `futures-util`, `tokio`; passed |
| 17 | The changelogs are truthful about the break and the publication (G10, AC10) | `crates/eggup-acquisition/CHANGELOG.md:3`, `crates/eggup-curl/CHANGELOG.md:3`, `crates/eggup-eggfetch/CHANGELOG.md:3` — each `## 0.2.0` section states the source break, the published baseline, and names this closure record; each `## Unreleased` section states that `0.2.0` is the published baseline | read at review; passed |
| 18 | The M016 coupling guard is reconciled with the cut changelog, not weakened (G11, AC11) | commit `2161f36` re-pointed `crates/eggup-acquisition/tests/publication_coupling.rs`; `plans/closure/acquisition-transport/016-status.md` records the change as an addendum | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` → 3 passed, 0 failed; passed |
| 19 | The guard derives the published version from the seam's changelog, never from a literal (G11) | `crates/eggup-acquisition/tests/publication_coupling.rs` — the published version is read from the highest `## X.Y.Z` heading in `crates/eggup-acquisition/CHANGELOG.md`; no version literal is embedded | read at review; passed |
| 20 | Every in-workspace dependent accepts the published version or records the widening (G11) | `crates/eggup-curl/Cargo.toml` and `crates/eggup-eggfetch/Cargo.toml` require `eggup-acquisition` `0.2`; `crates/eggup-transport-footprint/Cargo.toml` requires `0.2`; `crates/eggup-eggpack/Cargo.toml` requires `=0.2.0` | guard test 2 green at review; passed |
| 21 | An unknown shape is a failure, never an assumption of acceptance (G11) | the guard's requirement parser returns an error for an unrecognised shape and the test fails on it; the red-first evidence below exercises exactly that path | red-first probe; passed |
| 22 | The guard keeps zero dependencies and reads the repository's sources at compile time (G11) | the seam's published normalised manifest has an empty `[dependencies]`; the guard embeds the repository's changelogs and manifests with `include_str!` | passed |
| 23 | No source drift under the three crates' `src/**` (G12, AC12) | `git diff --stat b75f73b..b612608 -- crates/eggup-acquisition/src crates/eggup-curl/src crates/eggup-eggfetch/src` → empty | passed |
| 24 | The upload order was respected (AC17) | registry timestamps are monotonic — seam `13:50:19.661533Z`, curl `13:50:29.938295Z`, eggfetch `13:51:02.514990Z` | crates.io readback; passed |
| 25 | Registry finding 17 is discharged, with its before state on the record | before state: `cargo package -p eggup-curl` exited 101 with 11 errors and `-p eggup-eggfetch` exited 101 with 4 (`plans/closure/acquisition-transport/016-status.md`; `plans/registry.md` finding 17). After state: both adapters published against the registry seam, and their registry-only fixtures resolve the seam from crates.io | passed |
| 26 | Registry-only fixture A passes (AC14) | `/tmp/eggup-registry-fixtures/acquisition/` — manifest `eggup-acquisition = "=0.2.0"`; lockfile resolves `eggup-acquisition 0.2.0` from the registry with no path, git, or patch source | 3/3 — `published_seam_validates_fetch_limits`, `published_seam_io_variant_is_struct_shaped`, `published_seam_error_is_debug_but_not_clone`; passed |
| 27 | Registry-only fixture B passes (AC15) | `/tmp/eggup-registry-fixtures/curl/` — manifest `eggup-acquisition = "=0.2.0"`, `eggup-curl = "=0.2.0"`; lockfile resolves both from the registry | 2/2 — `published_curl_exposes_the_user_agent_control`, `published_curl_rejects_an_empty_executable`; passed |
| 28 | Registry-only fixture C passes (AC16) | `/tmp/eggup-registry-fixtures/eggfetch/` — manifest declares only `eggup-eggfetch = "=0.2.0"`; the lockfile resolves `eggup-acquisition 0.2.0` **transitively** from the registry | 2/2 — `published_eggfetch_builds_a_transport`, `published_eggfetch_reports_a_version`; passed |
| 29 | The compile-fail expectation is documented with observed compiler output (AC14, §10) | the three probes below, compiled against the registry copy of `eggup-acquisition 0.2.0` | E0164, E0599, E0369 observed; passed |
| 30 | The tag and release state is correct (AC18, §10) | `git tag -l` → `v0.1.1`, `v0.1.2`, `v0.1.3`, `v0.2.0`; `git rev-list -n1 v0.2.0` → `b612608…`; the tag is annotated (`git cat-file -t v0.2.0` → `tag`); the earlier tags were not moved (shared evidence file) | passed |
| 31 | The consumer-unblock reconciliation is recorded (AC19, §10) | the shared evidence file's registry-state table lists all seven `0.2.0` train crates on crates.io; this milestone covers the seam and the two adapters | passed |
| 32 | The closure record exists at the required path (AC19) | this file, `plans/closure/acquisition-transport/017-status.md` | present at review; passed |

## Production implementation evidence

### The publication-coupling guard re-point

The M016 guard in `crates/eggup-acquisition/tests/publication_coupling.rs`
enforced a **pre-publication** invariant: the seam's `## Unreleased` section
had to declare a *next release*, and every in-workspace dependent had to
accept it or record the widening. This publication discharged that premise —
`0.2.0` is now the published baseline, and there is no next release to
declare — so commit `2161f36` re-pointed the guard.

| | Before the re-point | After the re-point |
|---|---|---|
| Subject of the invariant | The seam's *next* release, declared under `## Unreleased` | The seam's **published** version |
| Source of the version | The `## Unreleased` section's declared next release | The highest `## X.Y.Z` heading in the seam's own changelog — derived, never a literal |
| Dependent rule | Every dependent must accept the declared next release or record the widening | Every dependent must accept the published version or record the widening |
| Unknown shape | — | A failure, never an assumption of acceptance |
| Dependencies | zero | zero |
| Source access | read the repository's sources at compile time | read the repository's sources at compile time |
| Test names | three | three — unchanged, because `crates/eggup-acquisition/tests/assertion_guard.rs` names them as the required behaviour witnesses |

The re-point was proven red-first; the re-target still discriminates:

| Probe | Expected | Observed |
|---|---|---|
| A changelog with no version heading | all three tests fail | all three fail |
| A `## 0.3.0` heading added while the dependents still accept `0.2` | two of the three fail, with the version in the message | two fail; the message carries the version |
| The widening record added alongside that heading | the "recorded, not performed" fallback discharges | discharged |

The guard is green at the reviewed HEAD:

```text
$ cargo test -p eggup-acquisition --locked --offline --test publication_coupling
running 3 tests
test dependent_requirements_accept_the_next_release_or_record_the_widening ... ok
test seam_changelog_declares_exactly_one_unreleased_section_with_a_next_release ... ok
test adapter_changelogs_record_the_widening_and_assert_no_contradicted_republication_version ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The change is recorded as an addendum in
`plans/closure/acquisition-transport/016-status.md`.

### The registry-only proofs

Three fresh crates outside the repository, each with no path, git, or patch
source anywhere:

| Fixture | Manifest | Resolves from crates.io | Tests |
|---|---|---|---|
| `/tmp/eggup-registry-fixtures/acquisition/` | `eggup-acquisition = "=0.2.0"` | `eggup-acquisition 0.2.0` | 3/3 — `published_seam_validates_fetch_limits`, `published_seam_io_variant_is_struct_shaped`, `published_seam_error_is_debug_but_not_clone` |
| `/tmp/eggup-registry-fixtures/curl/` | `eggup-acquisition = "=0.2.0"`, `eggup-curl = "=0.2.0"` | `eggup-acquisition 0.2.0`, `eggup-curl 0.2.0` | 2/2 — `published_curl_exposes_the_user_agent_control`, `published_curl_rejects_an_empty_executable` |
| `/tmp/eggup-registry-fixtures/eggfetch/` | `eggup-eggfetch = "=0.2.0"` | `eggup-acquisition 0.2.0` (**transitively**), `eggup-eggfetch 0.2.0` | 2/2 — `published_eggfetch_builds_a_transport`, `published_eggfetch_reports_a_version` |

The eggfetch fixture is the direct proof of registry finding 17: it depends
only on `eggup-eggfetch`, and its lockfile shows the seam resolving to
`0.2.0` from crates.io — which is only possible because the adapter's
requirement was widened to `eggup-acquisition = "0.2"`. The curl fixture
proves the same for the other adapter, and additionally exercises the seam's
own retyped `AcquisitionError`.

### The compile-fail expectation

The plan's §10 requires the documented negative expectation for
`AcquisitionError::Io(_)` and for whole-value cloning and comparison, with
the observed compiler output. The three probes below were compiled against
the registry copy of `eggup-acquisition 0.2.0` (resolved offline from the
local cargo cache; the lockfile records
`source = "registry+https://github.com/rust-lang/crates.io-index"` and the
published checksum).

Snippet A — the tuple-variant matcher:

```rust
matches!(err, eggup_acquisition::AcquisitionError::Io(_))
```

```text
error[E0164]: expected tuple struct or tuple variant, found struct variant `eggup_acquisition::AcquisitionError::Io`
 --> src/lib.rs:3:19
  |
3 |     matches!(err, eggup_acquisition::AcquisitionError::Io(_))
  |                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ not a tuple struct or tuple variant
  |
help: the struct variant's fields are being ignored
  |
3 -     matches!(err, eggup_acquisition::AcquisitionError::Io(_))
3 +     matches!(err, eggup_acquisition::AcquisitionError::Io { operation: _, source: _ })
  |
For more information about this error, try `rustc --explain E0164`.
```

Snippet B — whole-value cloning of an owned error:

```rust
err.clone()
```

```text
error[E0599]: no method named `clone` found for enum `AcquisitionError` in the current scope
 --> src/lib.rs:3:9
  |
3 |     err.clone()
  |         ^^^^^ method not found in `AcquisitionError`
  |
For more information about this error, try `rustc --explain E0599`.
```

Snippet C — whole-value comparison of owned errors:

```rust
a == b
```

```text
error[E0369]: binary operation `==` cannot be applied to type `AcquisitionError`
   --> src/lib.rs:3:7
    |
3   |     a == b
    |     - ^^ - AcquisitionError
    |     |
    |     AcquisitionError
    |
note: the foreign item type `AcquisitionError` doesn't implement `PartialEq`
   --> …/eggup-acquisition-0.2.0/src/lib.rs:258:1
    |
258 | pub enum AcquisitionError {
    | ^^^^^^^^^^^^^^^^^^^^^^^^^ not implement `PartialEq`
```

## Exact commands run and results

Commands run at this record's review, all read-only, all offline unless noted:

| # | Command | Result |
|---|---|---|
| 1 | `git log --oneline -12` | branch head `2d6e089`; the four train commits identified above |
| 2 | `git status --short` | the parent-owned close-out edits and this record are uncommitted — no crate source is modified |
| 3 | `git tag -l` | `v0.1.1`, `v0.1.2`, `v0.1.3`, `v0.2.0` |
| 4 | `git rev-list -n1 v0.2.0` | `b61260857fdb3904a091c503b1dff05251ec0bff` |
| 5 | `git cat-file -t v0.2.0` | `tag` — annotated |
| 6 | `git diff --stat b75f73b..b612608 -- crates/eggup-acquisition/src crates/eggup-curl/src crates/eggup-eggfetch/src` | empty — no source drift (G12, AC12) |
| 7 | `cargo test -p eggup-acquisition --locked --offline --test publication_coupling` | 3 passed, 0 failed |
| 8 | `cargo tree -p eggup-acquisition --locked --offline --depth 1` | the crate alone — zero dependencies |
| 9 | `cargo tree -p eggup-curl --locked --offline --depth 1` | `eggup-acquisition` only |
| 10 | `cargo tree -p eggup-eggfetch --locked --offline --depth 1` | `eggfetch-core`, `eggup-acquisition`, `futures-util`, `tokio`; dev-dependencies `tokio`, `url` |
| 11 | `curl -fsSL -A "eggup-release-check/1.0" https://crates.io/api/v1/crates/<crate>/versions` | `0.2.0` present for each of the three crates; every prior version unyanked |
| 12 | `curl -fsSL -A "eggup-release-check/1.0" https://crates.io/api/v1/crates/<crate>/0.2.0` | checksum, crate size, publisher `dbowm91`, publication timestamp, `yanked = false` |
| 13 | `curl -fsSL -A "eggup-release-check/1.0" https://crates.io/api/v1/crates/<crate>/0.2.0/dependencies` | the seam: empty; curl: `eggup-acquisition ^0.2`; eggfetch: four normal dependencies |
| 14 | `tar -tzf <cached .crate>` for the three published tarballs | the package file lists in row 12 of the matrix |
| 15 | `tar -xzOf <cached .crate> …/.cargo_vcs_info.json` for the three tarballs | `sha1 = b612608…` in each |
| 16 | `tar -xzf <cached .crate>` then read the normalised `Cargo.toml` of each | the published dependency surfaces in rows 8–11 |
| 17 | `cargo build --offline` of the three compile-fail probes against the registry copy | E0164, E0599, E0369 — the outputs above |
| 18 | `rustup toolchain list` | `1.89.0-aarch64-apple-darwin` installed; `1.89-aarch64-apple-darwin` active and default |
| 19 | `./scripts/check-docs-truth.sh` | `documentation-truth checks passed`, exit 0 — before and after this record |

Commands run at the publication source commit and reported in the shared
evidence file, not re-run here: the hosted matrix (run `40079833945`), the
local gate (`./scripts/check-local.sh`, exit 0 with 528 passed / 0 failed /
1 ignored), the `cargo package` and `cargo publish --dry-run` steps, and the
three registry-only fixture test runs. The published artefacts read back in
commands 11–16 are the downstream proof that the packaging steps succeeded.

## Per-platform results

| Platform | Local gate | Hosted lane | `eggup-acquisition` tests | `eggup-curl` tests | `eggup-eggfetch` tests | Status |
|---|---|---|---|---|---|---|
| **Linux** (ubuntu) | not run — no Linux host in the review environment | `stable`, `msrv`, `service-systemd` — green in run `40079833945` | run in the workspace test step | run in the workspace test step | run in the workspace test step | **Run — green** |
| **macOS** (darwin 25.6.0, arm64) | `./scripts/check-local.sh` exit 0 — 528 passed, 0 failed, 1 ignored | `macos` — green in run `40079833945` | run | run | run | **Run — green** |
| **Windows** (windows-latest) | not run — no Windows host in the review environment | `windows-check` — green in run `40079833945` | run in `cargo test -p eggup-acquisition -p eggup-archive -p eggup-curl --locked` and in the full-workspace step | run in both steps | run in `cargo test --workspace --all-targets --locked --no-fail-fast` | **Run — green** |

The Windows lane's steps are named explicitly in `.github/workflows/ci.yml`
(`windows-check`); the `eggup-curl` and `eggup-eggfetch` tests both run there.
The `#[cfg(not(windows))]` gate on `eggup-curl`'s live-loopback tests is
retained, so those specific tests are skipped on Windows and run on Linux and
macOS; the Windows lane runs the portable acquisition fixtures and the curl
process and error-path tests. `eggup-eggfetch` has no platform gate, so its
tests run on all three platforms.

## Invariant review

| Invariant | Status | Evidence |
|---|---|---|
| The seam has zero dependencies, before and after | Held | published normalised manifest; crates.io dependency list; `cargo tree` |
| The adapters depend on the seam only, with no embedded HTTP or TLS stack in `eggup-curl` | Held | published normalised manifests; crates.io readback |
| Transport composition lives in the seam, never in an adapter | Held | no published change to the seam's composition types; the adapters' public APIs are additive only |
| The seam's `0.2.0` is source-breaking in exactly two ways | Held | E0164, E0599, E0369; the seam's `## 0.2.0` changelog section states the same two |
| The `Display` rendering stays bounded at 512 bytes and truncates only at a UTF-8 character boundary | Held | the seam's `## 0.2.0` changelog section; the source is unchanged since the qualified baseline (row 23) |
| `Io` stays terminal under every `CompositionPolicy` | Held | the seam's `## 0.2.0` changelog section; the source is unchanged since the qualified baseline |
| The two `#[doc(hidden)]` adapter-facing entry points are source-compatible additions | Held | the adapters' published sources call them unchanged; the adapters' fixtures build against the published seam |
| The curl adapter's User-Agent is validated as an HTTP field value, emitted as a literal `--user-agent` argv element ahead of the `--` separator, and a rejected value is a terminal `AcquisitionError::InvalidInput` — never `Unavailable` | Held | `published_curl_exposes_the_user_agent_control`, `published_curl_rejects_an_empty_executable` |
| `CurlConfig` is untouched, so the observable wire behaviour does not change | Held | `git diff` over the curl crate's `src/**` is empty since the qualified baseline |
| `EggfetchTransport` remains shareable across threads | Held | `published_eggfetch_builds_a_transport`; the source is unchanged since the qualified baseline |
| The published seam's `Clone`, `PartialEq`, and `Eq` removals are the only derive change | Held | E0599 and E0369; the published `AcquisitionError` derives `Debug` only |
| The coupling guard derives its subject version rather than hard-coding it | Held | the guard's published-version derivation; red-first evidence above |
| The coupling guard has zero dependencies and reads the repository's sources at compile time | Held | the seam's published normalised manifest; the guard's `include_str!` embedding |
| SHA-256 is integrity evidence only — no authenticity or signature claim is made | Held | no signature, attestation, or provenance mechanism was added by this milestone |
| CI never publishes | Held | `.github/workflows/ci.yml` contains no publish step |

## Failure, rollback, and recovery review

- **A registry version is immutable.** A defect discovered in a published
  `0.2.0` cannot be corrected by overwriting the tarball, re-uploading the
  version, or yanking and republishing under the same number. The only
  correction is a new patch version (`0.2.1`), which requires its own
  milestone, its own plan, and its own authorization. No such corrective is
  open against these three crates at this record's review.
- **The publication is not transactional.** The three uploads were accepted
  individually, in dependency order: the seam first, then the two adapters.
  A failure after the seam's upload but before the adapters' would have left
  the seam published and the adapters unpublished — a resumable state, not a
  corrupt one, because the adapters' requirements were widened to accept the
  already-published seam. The outcome observed was the complete one.
- **The train-order exception was recorded before it was relied on.** The
  packaged-service CI step resolves `eggup-core` against the registry, so it
  could not be green before the train's first upload; commit `b612608`
  records that exception in the core plan's §9.1. The four affected lanes
  were re-run on the same commit after the uploads and passed unchanged.
- **The guard fails closed.** An unknown dependency shape, a changelog without
  a version heading, or a dependent that neither accepts the published
  version nor records the widening is a test failure, not a warning and not
  an assumption of acceptance. This is the same direction of failure as the
  M016 guard it replaces: loud, not silent.
- **The registry-only fixtures are the recovery check.** If a published
  artefact is ever found to differ from the workspace, the fixtures are the
  reproduction path — they consume only the registry, so they fail for a
  consumer exactly as they would fail for a third party.

## Compatibility and migration review

The seam's `0.2.0` is source-breaking in exactly two ways:

1. `AcquisitionError::Io(String)` became
   `AcquisitionError::Io { operation, source }`. A consumer that matched the
   tuple variant must change to the struct-variant pattern; the compiler
   names the replacement (E0164 above). The variant carries a live `source()`
   chain, so error inspection is preserved rather than flattened.
2. The enum now derives **only** `Debug`. `Clone`, `PartialEq`, and `Eq` are
   removed, because `std::io::Error` implements none of them. A consumer
   that cloned or compared a whole `AcquisitionError` must move to
   field-wise handling or to its own comparison policy (E0599 and E0369
   above).

Nothing else is consumer-visible. The `Display` rendering stays bounded at
512 bytes and truncates only at a UTF-8 character boundary; `Io` stays
terminal under every `CompositionPolicy`; the two `#[doc(hidden)]`
adapter-facing entry points are source-compatible additions.

`eggup-curl 0.2.0` carries **no** source-breaking change of its own but is
transitively source-breaking through the seam's `^0.2` requirement. Its own
public API is additive: the caller-settable User-Agent
(`CurlTransport::with_executable_and_user_agent` and
`CurlTransport::user_agent`), validated as an HTTP field value and emitted as
a literal `--user-agent` argv element ahead of the `--` separator that
isolates the URL; a rejected value is a terminal
`AcquisitionError::InvalidInput` and never `Unavailable`. `CurlConfig` is
untouched, so the observable wire behaviour does not change.

`eggup-eggfetch 0.2.0` likewise carries no source-breaking change of its
own and is transitively source-breaking through the same seam requirement.
Its public API is unchanged and `EggfetchTransport` remains shareable across
threads.

**What `0.2.0` carries that `0.1.3` did not:** the M016 coupling correction.
The seam's changelog declares its next release and records the widening each
dependent requires, and the two adapters' changelogs no longer claim a
non-breaking `0.1.3`. The adapters' M012–M015 qualification work was
published at `0.1.2`/`0.1.3` and is unchanged in substance.

**Migration for a consumer pinned to the `0.1.x` line:** the published
`eggup-acquisition 0.1.3`, `eggup-curl 0.1.2`, and `eggup-eggfetch 0.1.3`
still carry the pre-coupling state, so a consumer pinned to them does not
receive the widening and must move to `0.2.0` deliberately. Registry versions
are immutable, so this cannot be repaired in place.

## Security review

- **No authenticity claim is made or implied.** The published checksums are
  SHA-256 integrity evidence only. No signature, attestation, provenance
  mechanism, or transparency-log entry was added by this milestone, and the
  registry's `published_by` identity is the registry's own record, not a
  signature.
- **The published artefacts carry no credential material.** The three
  tarballs ship only their source, `README.md`, `CHANGELOG.md`, the manifest
  pair, a lockfile, and `.cargo_vcs_info.json`; the curl and eggfetch
  tarballs add one `examples/` entry each. No token, key, or environment
  file is included.
- **The seam's zero-dependency property is preserved in the published
  artefact**, not merely in the workspace manifest — the published
  normalised manifest has an empty `[dependencies]` section.
- **The User-Agent control is validated, not interpolated.** The caller
  supplies a header field value; it is rejected as a terminal
  `InvalidInput` when malformed, and it is emitted as a literal argv
  element ahead of the `--` separator that isolates the URL, so it cannot
  become a second argument vector. The adapters' fixtures exercise both the
  accepted and the rejected path.
- **The `#[doc(hidden)]` adapter-facing entry points remain
  source-compatible additions**, which keeps the seam's adapter seam narrow
  without widening the consumer-visible surface.
- **The publication was performed by the crate's existing owner**, `dbowm91`,
  which the registry records as `published_by` for all three crates and for
  their earlier versions — so no new publish authority was created.

## Documentation / operations evidence

Changed by the release-prep commit `2161f36` and relevant to these three
crates:

| Document | Change |
|---|---|
| `crates/eggup-acquisition/CHANGELOG.md` | the `## 0.2.0` section states the two source breaks, the published baseline, and names this closure record; the `## Unreleased` boilerplate states that `0.2.0` is the published baseline |
| `crates/eggup-curl/CHANGELOG.md` | the `## 0.2.0` section states the transitive break through the seam and the additive User-Agent control |
| `crates/eggup-eggfetch/CHANGELOG.md` | the `## 0.2.0` section states the transitive break through the seam and the unchanged public API |
| `crates/eggup-curl/Cargo.toml`, `crates/eggup-eggfetch/Cargo.toml` | the seam requirement widened to `0.2` |
| `crates/eggup-eggpack/Cargo.toml`, `crates/eggup-transport-footprint/Cargo.toml` | the seam requirement widened to `=0.2.0` and `0.2` respectively |
| `crates/eggup-acquisition/tests/publication_coupling.rs` | the guard re-pointed at the published version |
| `crates/eggup-acquisition/tests/assertion_guard.rs` | unchanged — it is what fixes the three witness test names |
| `docs/crates.md`, `docs/releases.md`, `README.md`, `architecture/overview.md`, `architecture/tooling-governance.md`, `architecture/transport-footprint.md` | the published-version claims updated to `0.2.0` |
| `scripts/check-docs-truth.sh` | generalised so the published-version checks are not pinned to one crate |

Deliberately **not** changed by this milestone: any file under the three
crates' `src/**` (verified empty by the diff in row 23); the seam's public
API beyond the two breaks already recorded; the `docs/` consumer-facing
guides' policy content, which are non-normative and point at the contracts.

Operations: publication is a tracked milestone, never a side effect, and CI
never publishes. The `v0.2.0` tag is annotated and points at the publication
source commit; the `v0.1.1`, `v0.1.2`, and `v0.1.3` tags were not moved. The
release `0.2.0` exists against the tag.

## Unresolved findings

| ID | Finding | Severity | Blocking | Notes |
|---|---|---|---|---|
| U1 | Registry finding 1 — the load-sensitive `eggup-curl` sub-second-deadline tests (`build_curl_args_passes_sub_second_deadlines_to_fake_curl` and `sub_second_total_timeout_kills_child_promptly`) are sensitive to machine load and can flake on a busy runner. | Low | Blocks nothing | Hosted CI passes both on every lane. The tests are timing-bound, not correctness-bound; the failure mode is a slow-runner timeout, not a wrong result. The published `0.2.0` does not change this. |
| U2 | The published `eggup-acquisition 0.1.3`, `eggup-curl 0.1.2`, and `eggup-eggfetch 0.1.3` still carry the pre-coupling state; consumers pinned to them do not receive the widening. | Low | Blocks nothing | Registry versions are immutable, so this cannot be repaired in place. It is a property of the published `0.1.x` line, not a defect in `0.2.0`. |
| U3 | The source plan's `Status:` header read `ready for handoff`; per `plans/implementation/README.md` it becomes `implemented` once the closure record exists. | Low | Blocks nothing | **Discharged before finalisation.** The source plan's `Status:` header now reads `closed 2026-10-24; ...`, naming this closure record. The registry and the roadmap already carry their M017 rows. |

No finding of medium-or-higher severity is open against this milestone.

Two further items were observed earlier in this review and were resolved by
their owners before the record was finalised, so they are recorded here rather
than left to be rediscovered as unexplained inconsistencies: the three
per-crate changelog sections were headed `## 0.2.0 — 2026-10-23` and the root
`CHANGELOG.md` recorded the crates as published on 2026-10-23 while crates.io
recorded the upload at `2026-10-24T13:50Z`–`13:51Z`; and the root
`CHANGELOG.md`'s `## 0.2.0` section was headed "in progress". At
finalisation both read `2026-10-24`, the root section reads
`## 0.2.0 — 2026-10-24, completed 2026-10-24`, and each crate is recorded as
published 2026-10-24 by Acquisition Transport M017.

## Roadmap disposition

`plans/subsystems/acquisition-transport-roadmap.md` requires an M017 row in
its status table. The row is present at this record's finalisation, and the
roadmap's `Status:` header names M017 as published on 2026-10-24 from
`b612608` with hosted run `40079833945` green on all five lanes. The roadmap
is owned by another writer and was not modified here. The row's content is
consistent with this record's evidence and is summarised below.

- Acquisition Transport M017 is **closed**. Published `eggup-acquisition
  0.2.0` (`2ef74b4d…299fc3`), `eggup-curl 0.2.0` (`15cf536f…90d3f7`), and
  `eggup-eggfetch 0.2.0` (`9fc8f648…658721`) from `b612608`, the publication
  source commit, with `v0.2.0` and release `0.2.0` created. The registry-only
  fixtures resolve the seam from crates.io — the eggfetch fixture transitively,
  which is the direct proof of the widening that discharges registry finding
  17. The seam's `0.2.0` is source-breaking in exactly two ways: the retyped
  `AcquisitionError::Io { operation, source }` and the removal of `Clone`,
  `PartialEq`, and `Eq` from the error enum. The M016 publication-coupling
  guard was re-pointed at the seam's published version in `2161f36` and was
  proven red-first. Registry finding 1 remains open at low severity and blocks
  nothing.

The preceding milestone, M016, remains closed; the addendum in
`plans/closure/acquisition-transport/016-status.md` records the guard's
re-pointing.

## Registry updates

`plans/registry.md` is owned by another writer and was not modified here. At
this record's finalisation the registry carries the M017 state, and this
record is consistent with it:

- The workstream summary row records M001–M017 closed and states that M017
  published `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch 0.2.0`
  (2026-10-24) from `b612608`.
- The subsystem summary row records M001–M017 closed, M012 conditionally
  closed, M017 published from `b612608` with run `40079833945` green, and
  notes that no further acquisition milestone is registered and that **open
  finding 1 stays open**.
- The detailed milestone table carries the M017 acquisition row — closed
  2026-10-24, with the three published checksums in their published-table
  form and "finding 17 closed".
- Registry finding 17 carries the disposition: closed on 2026-10-24 by
  Acquisition Transport M017, with the before state on the record —
  `cargo package -p eggup-curl` exited 101 with 11 errors and
  `-p eggup-eggfetch` exited 101 with 4 — and the after state, both adapters
  packaged and published against the published seam. The finding is closed,
  not deferred.

The registry-side bookkeeping item recorded as U3 during the review — the
source plan's `Status:` header — was discharged before this record was
finalised: the header now reads `closed 2026-10-24` and names this record.
