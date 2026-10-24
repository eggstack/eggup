# Verified Update Core M019 — Closure and Verification Record

## Status

**closed** — `eggup-core 0.2.0` is on crates.io, the `v0.2.0` tag and the
GitHub release `0.2.0` exist, the registry-only direct proof passes, and the
published package's public API, dependency surface, and file list match the
boundary the plan states. This is the **first** crate of the lockstep `0.2.0`
train of all seven publishable crates.

One deviation from the plan's stop conditions was taken. It was recorded
**before** the upload in `plans/implementation/verified-update-core/019-eggup-core-0.2.0-publication.md`
§9.1 (commit `b612608`), and it is the plan's own section: the hosted matrix
could not be green before the upload, because the packaged-service step
resolves the packaged service crate's `eggup-core = "^0.2"` against crates.io.
The matrix is green **now**, on the same commit, after the upload. The
deviation is described in full below; it is not described as a green matrix
before the upload anywhere in this record.

No medium-or-higher finding about this milestone's own change remains open.
Five low findings remain open; each is named below with its owner.

## Source plan and roadmap

- Plan: `plans/implementation/verified-update-core/019-eggup-core-0.2.0-publication.md`
- Roadmap: `plans/subsystems/verified-update-core-roadmap.md`
- Long-term requirements: `plans/000-long-term-specification.md`,
  `plans/001-terminology-and-domain-model.md`
- Applicable ADRs: `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`,
  `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`,
  `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- Shared evidence set: `/tmp/eggup-release/closure-evidence.md`. This record
  cites that file for the train-wide facts and adds only the evidence specific
  to `eggup-core`.

## Reviewed repository baseline

- Plan baseline — the "release-prep SHA" of plan §10 —
  `b75f73bf1ad138f79e7e16d50f35671e52bafe96` (`b75f73b`).
- Publication source commit `b61260857fdb3904a091c503b1dff05251ec0bff`
  (`b612608`).
- These are **distinct** commits. `b612608` is three commits after `b75f73b`
  on the same branch, and it is the commit the tag `v0.2.0` points at. The
  published `.crate` records exactly
  `b61260857fdb3904a091c503b1dff05251ec0bff` in its `.cargo_vcs_info.json`
  and records no dirty tree.
- Branch: `plans/corrective-batch-2026-10-23`. The working tree was clean
  when the baseline was read (`git status --short` printed nothing at
  `b75f73b`). It is **not** clean at closure time: other writers are working
  on the same branch in parallel for the other six publications of the train,
  and their paths — the sibling implementation plans, the sibling closure
  records, `plans/registry.md`, the subsystem roadmaps, and the changelogs —
  are modified or newly added. None of them is part of this milestone. The
  only path this milestone writes is
  `plans/closure/verified-update-core/019-status.md`.
- Tag targets, read at closure time: `v0.2.0` → `b612608`, `v0.1.1` →
  `881c95f`, `v0.1.2` → `e8e07eb`, `v0.1.3` → `bd43683`. The three `0.1.x`
  tags were not moved.
- Root `Cargo.toml:15` carries `[workspace.package] version = "0.2.0"`, and
  `crates/eggup-core/Cargo.toml:3` inherits it through
  `version.workspace = true`, so the packaged crate version is exactly
  `0.2.0`.

## Implementation commits / PRs

No pull request was opened. The registry publication is this milestone's act;
the repository-side work is the release-prep commit set.

| Commit | Change |
|---|---|
| `afe6201` | the five `0.2.0` publication plans |
| `2161f36` | the workspace version bump to `0.2.0`, the requirement widenings, the `eggup-eggpack` re-pin, the eight changelog cuts, the documentation-truth gate generalisation, and the M016 publication-coupling guard re-point |
| `b612608` | the train-order exception recorded in the core plan's §9.1 — **also the publication source commit and the `v0.2.0` tag target** |
| `2d6e089` | restoring the strict packaged-service step in CI — the follow-on named by registry finding 16. It is **after** the tag and is not part of the published source |

Files this milestone created or changed:

| File | Change |
|---|---|
| `plans/closure/verified-update-core/019-status.md` | **New.** This record. |

No file under `crates/eggup-core/**` was changed by this milestone.
`git diff --stat b75f73b b612608 -- crates/eggup-core/src/` is empty, and
`git diff --stat b75f73b b612608 -- crates/eggup-core/Cargo.toml` is empty.
This milestone authorized an upload; it did not move source.

This session ran no git command that mutates repository state, and no
`cargo publish`, `cargo package`, or other network-touching cargo command.

## Executive finding

`eggup-core 0.2.0` is on crates.io with checksum
`2c86a0ebc94c75465855f661cd3aae91447f53fdb38e6f277092b155dbeb23b8` — the
registry's published-table form is `2c86a0eb…eb23b8` — size 78 576 bytes,
published by `dbowm91` at `2026-10-24T13:46:31.983818Z`, and the locally built
`.crate` in `target/package/` hashes to the same value byte for byte. The
package is exactly the boundary the plan states: `sha2` plus the Windows-only
`self-replace`, no `git`/`path`/`patch` edge, seven `Error` variants with
`Error::Injected` absent from the packaged build, `FailureCategory` still
`#[non_exhaustive]` with the unreachable `RetainedEvidence` variant and its
`retained-evidence` string removed, `MAX_STDIN_BYTES` and
`CommandSpec::stdin_data` added, and **no `docs/` entry** — the crate's
contract documents under `crates/eggup-core/docs/` are not shipped, which is
the open Verified Update Core M016 finding and is not discharged by this
publication. The hosted matrix was green on the publication source commit only
**after** the upload: the first attempt failed in exactly one step, in four
lanes, on `cargo package -p eggup-service --locked --no-verify`, because that
step resolves the packaged service crate's `eggup-core = "^0.2"` against
crates.io — the circularity the plan's §9.1 records before the upload, and the
reason the plan's own stop condition could not be met literally for the first
crate of the train. The re-run on the same commit is green on all five lanes,
the local gate is green in full, the direct registry-only fixture resolves
`eggup-core 0.2.0` from crates.io with no path, git, or patch source and
passes 3 of 3, and the only source break a consumer must make is deleting a
reference to `FailureCategory::RetainedEvidence`.

## The train-order deviation, stated plainly

The plan's stop condition required a green hosted matrix before the upload.
For the **first** crate of the train that condition is circular: the hosted
lanes run `cargo package -p eggup-service --locked --no-verify`, which
resolves the packaged service crate's `eggup-core = "^0.2"` against crates.io,
so it cannot pass until `eggup-core 0.2.0` is on the registry — and
publishing `eggup-core 0.2.0` requires a green matrix. The condition is
unsatisfiable as literally written for the one crate whose publication
discharges it.

The deviation was recorded **before** the upload, in the plan's own §9.1,
"Train-order exception — the first crate of the train", committed as
`b612608`. This is the section that authorizes the upload; it is cited here,
not restated as a justification invented afterwards.

**What was observed.** Hosted run `40079833945`, head SHA
`b61260857fdb3904a091c503b1dff05251ec0bff`. Its **first attempt** failed in
exactly one step, in exactly four lanes, with the same error in each:

```text
error: failed to prepare local package for uploading
Caused by:
  failed to select a version for the requirement `eggup-core = "^0.2"`
  candidate versions found which didn't match: 0.1.3, 0.1.2, 0.1.1, ...
  location searched: crates.io index
  required by package `eggup-service v0.2.0`
```

The failing step is the same step in all four lanes —
`cargo package -p eggup-service --locked --no-verify`. The fifth lane,
`service-systemd`, does not carry that step and was green throughout: it ran
once, in the first attempt, and was carried forward rather than re-executed.
Every other step in every lane was green on both attempts: the workspace
build, the test suites, clippy, rustdoc, the MSRV check, the three
feature-gated footprint configurations, the systemd fixtures, the Windows SCM
fixtures, and the live SCM fixture. The failure is a **publication state**, the
same class the repository already recorded as finding 16, and it is discharged
by this milestone's own upload.

**What qualified instead**, as §9.1 requires, all three parts:

1. The local gate is green in full — fmt, clippy with `-D warnings`, the
   workspace test suite, rustdoc, the `1.89.0` workspace check, and the
   package and dry-run steps. See "Exact commands run and results".
2. The hosted run is green in every step except that single
   registry-resolved packaged-service step, and the failing step's error is
   the registry-selection error above.
3. The packaged crate's own dependency surface resolves entirely against
   versions already on the registry — `sha2`, plus `self-replace` on Windows —
   so `eggup-core 0.2.0` is publishable on its own terms and depends on no
   other crate in the train.

**After the upload**, the four affected lanes were re-run **on the same
commit** and passed unchanged. Run `40079833945` attempt 2 is green on all
five lanes. The attempt timings corroborate the account: attempt 1 started
`2026-10-24T08:54:04Z`, the upload landed `13:46:31Z`, and the four re-run
lanes started `13:47:26Z`.

**What the exception did not authorize.** It authorized no other upload. Every
later crate in the train was published only after the full hosted matrix was
green, which was possible only because this upload had already put
`eggup-core 0.2.0` on the registry. After an accepted upload, registry state
is immutable: a defect in the published `0.2.0` requires a new patch version
(`0.2.1`), never an overwrite, a re-upload, or a yank-and-republish.

## Requirement → evidence matrix

| # | Requirement (plan §8 / §3 gate) | Evidence | Command and result |
|---|---|---|---|
| R1 | `eggup-core 0.2.0` is on crates.io with the recorded checksum, size, publisher, and publication timestamp; the version list shows `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0`, and no version is yanked *(AC1, G1)* | crates.io version and version-list endpoints; `audit_actions` carries exactly one `publish` entry and no `yank` | `curl -fsSL https://crates.io/api/v1/crates/eggup-core/versions` → `0.2.0 \| yanked False \| size 78576 \| created 2026-10-24T13:46:31.983818Z \| cksum 2c86a0eb`, and `0.1.3`/`0.1.2`/`0.1.1`/`0.1.0` present, none yanked. `curl -fsSL https://crates.io/api/v1/crates/eggup-core/0.2.0` → `num = 0.2.0`, `yanked = False`, `crate_size = 78576`, `published_by.login = dbowm91`, `audit_actions = [{action: publish, user: dbowm91, time: 2026-10-24T13:46:31.983818Z}]` |
| R2 | The published version is exactly `0.2.0`, matching `[workspace.package] version` at the publication source commit *(AC2, G3)* | `Cargo.toml:15` (`version = "0.2.0"`); `crates/eggup-core/Cargo.toml:3` (`version.workspace = true`); the normalized packaged manifest's `version = "0.2.0"` | Read of `target/package/eggup-core-0.2.0/Cargo.toml`; `git show 2161f36 -- Cargo.toml` carries the bump; crates.io `num = 0.2.0` |
| R3 | The public API boundary holds: seven `Error` variants with `Error::Injected` absent from the packaged build; `FailureCategory` `#[non_exhaustive]`; `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` present; `FailureCategory::RetainedEvidence` absent; the three `Error::Io` operation strings as stated *(AC3, G4)* | `crates/eggup-core/src/error.rs:28-63` — the seven variants; `:61-62` — `#[cfg(test)] Injected(String)`; `crates/eggup-core/src/transaction.rs:136` — `#[non_exhaustive]`; `:141-169` — seven members, no `RetainedEvidence`; `crates/eggup-core/src/candidate.rs:27` — `MAX_STDIN_BYTES = 1024 * 1024`; `:125` — `stdin_data`; `:333`, `:410`, `:515` — `"spawning bounded candidate"`, `"polling bounded candidate"`, `"writing bounded stdin"`; `crates/eggup-core/src/lib.rs:34` — the re-export | `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` → exit 0; `cargo test -p eggup-core --all-targets --locked --offline` → **110 passed, 0 failed**; the registry-only fixture → **3 passed, 0 failed** |
| R4 | The dependency surface is unchanged: `sha2`, plus `self-replace` under `cfg(windows)`; no dependency added, removed, or re-versioned; the Unix/macOS graph unchanged *(AC4, G5)* | The crates.io dependency list for the published version carries exactly two entries — `sha2 ^0.10` with no target, `self-replace ^1.5` with `target = cfg(windows)`. The packaged manifest carries `[dependencies.sha2] version = "0.10"` and `[target."cfg(windows)".dependencies.self-replace] version = "1.5"`. `git diff --stat b75f73b b612608 -- crates/eggup-core/Cargo.toml` is empty | `curl -fsSL https://crates.io/api/v1/crates/eggup-core/0.2.0/dependencies` → `sha2 ^0.10` (no target) and `self-replace ^1.5` (`cfg(windows)`), both `optional = False`; the fixture lockfile resolves `sha2 0.10.9` and `self-replace 1.5.0`, both from the registry |
| R5 | The packaged manifest carries no `git`, `path`, or `patch` dependency and names no dependency absent from the registry *(AC5, G6)* | The normalized packaged manifest has no such entry; both named dependencies are on crates.io (`sha2 0.10.9`, `self-replace 1.5.0`), as the fixture lockfile's registry checksums show | Read of `target/package/eggup-core-0.2.0/Cargo.toml`; `grep -nE 'path =\|git =\|\[patch'` over the fixture `Cargo.lock` → no match |
| R6 | The package file list is recorded and contains no `docs/` entry; the record states that the contract documents are not shipped *(AC6, G7)* | The tarball carries 23 entries: `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, six `examples/*.rs`, and eleven `src/*.rs`. **Zero** `docs/` entries. The exclusion is declared at `crates/eggup-core/Cargo.toml:15-19` and preserved in the normalized packaged manifest | `tar tzf target/package/eggup-core-0.2.0.crate \| sort` → 23 entries; `tar tzf … \| grep -c 'docs/'` → `0` |
| R7 | The hosted matrix is green on the publication source commit, under the train-order exception recorded in §9.1 of the plan *(AC7, G8)* | Hosted run `40079833945`, head SHA `b61260857fdb3904a091c503b1dff05251ec0bff`. Attempt 1: four lanes failed on the single step `cargo package -p eggup-service --locked --no-verify`; the fifth lane was green. Attempt 2: all five lanes green. The attempt-1 job log carries the registry-selection error verbatim. See "The train-order deviation" | `gh run view 40079833945 --json headSha,conclusion,jobs` → `headSha = b612608…`, `conclusion = success`; `gh api …/attempts/1/jobs` → four `failure`, one `success`; `gh api …/attempts/2/jobs` → five `success`; `gh api …/actions/jobs/<id>/logs` → the error text quoted above |
| R8 | The local gate is green: fmt, clippy, test, doc, the `1.89.0` check, and the package/dry-run steps *(AC8)* | The shared evidence set's local-gate table on the publication source commit; plus this session's first-hand runs on macOS | `cargo fmt --all -- --check` → clean; `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` → exit 0; `cargo test -p eggup-core --all-targets --locked --offline` → **110 passed, 0 failed**; `cargo +1.89.0 check --workspace --all-targets --locked --offline` → exit 0; the shared evidence set records `cargo test --workspace --all-targets --all-features --locked` → **528 passed, 0 failed, 1 ignored** and `cargo doc --workspace --no-deps --locked` → clean |
| R9 | The changelog is truthful: it attributes M014, M015, M017, and M018 to `0.2.0` and attributes M010, M011, the earlier Core audit fixes, and the M012 correction to `0.1.3` *(AC9, G9)* | `crates/eggup-core/CHANGELOG.md:3-21` — the `0.2.0` entry names M017, M018, M015, and M014 as carried, and states that `0.1.3` still carries every one of them; the same entry attributes M010, M011, the earlier audit fixes, and the M012 correction to `0.1.3`; `:52-59` — the one consumer action | Read of `crates/eggup-core/CHANGELOG.md`; the same text is the packaged `CHANGELOG.md` (`tar tzf …` entry `eggup-core-0.2.0/CHANGELOG.md`) |
| R10 | No unclassified source delta: no file under `crates/eggup-core/src/**` differs from the qualified baseline *(AC10, G10)* | `git diff --stat b75f73b b612608 -- crates/eggup-core/src/` is empty; `git diff --stat b75f73b b612608 -- crates/eggup-core/Cargo.toml` is empty | Read-only `git diff --stat` at closure time |
| R11 | Maintainer authorization was recorded before the upload, and the upload is the only publication act performed under this milestone *(AC11, G11)* | The authorization and the substitute qualification are the plan's §9.1, committed as `b612608` — before the `13:46:31Z` upload. The crates.io `audit_actions` for `0.2.0` contains exactly one entry, `publish`, and no `yank`. The six sibling crates' `0.2.0` versions are published under their own milestones, in the order the release notes record: core, archive, acquisition, curl, eggfetch, service, eggpack | `git show b612608 --stat` (the §9.1 commit precedes the upload); crates.io `audit_actions` read above; the GitHub release body's per-package table |
| R12 | Fixture A passes: its lockfile resolves `eggup-core 0.2.0` from crates.io with no path, git, or patch source, and its test suite passes *(AC12)* | The fixture is a fresh crate **outside** this repository at `/tmp/eggup-registry-fixtures/core/`. Its manifest declares `eggup-core = "=0.2.0"` and nothing else, with no `[patch]`, path, or git override and no `.cargo/config.toml`. Its `Cargo.lock` carries `eggup-core 0.2.0` with `source = "registry+https://github.com/rust-lang/crates.io-index"` and `checksum = "2c86a0ebc94c75465855f661cd3aae91447f53fdb38e6f277092b155dbeb23b8"`, and **no** entry anywhere carries a `path` or `git` source. Three tests pass | `cargo test --locked --offline` in `/tmp/eggup-registry-fixtures/core` → `3 passed; 0 failed` — `published_core_exposes_the_bounded_stdin_surface`, `published_core_error_is_display_and_debug`, `published_core_failure_category_has_no_retained_evidence` |
| R13 | Fixture B is explicitly outstanding from this milestone, with the owning milestone named *(AC13)* | The plan's §4 "Ordering" defers Fixture B until `eggup-service 0.2.0` is registry-visible, and its W9 assigns it to the service publication milestone. The shared evidence set's registry table records `eggup-service 0.2.0` on crates.io, so the precondition is now met — but the proof belongs to that milestone's own record. Read-only observation at closure time: the service fixture's lockfile resolves `eggup-service 0.2.0` and `eggup-core 0.2.0` both from `registry+https://github.com/rust-lang/crates.io-index`, the core entry carrying checksum `2c86a0eb…eb23b8` | `grep -A4 'name = "eggup-core"' /tmp/eggup-registry-fixtures/service/Cargo.lock` → `version = "0.2.0"`, `source = "registry+…"`, `checksum = "2c86a0eb…"`; the test result is carried by `plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md`'s closure record, not by this one |
| R14 | Tag and release state is correct: `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved; `v0.2.0` points at the publication source commit and was created only after the registry-only proofs passed; the release notes state each package's exact publication source SHA and do not claim that the package ships the crate's contract documents *(AC14)* | `v0.2.0` → `b61260857fdb3904a091c503b1dff05251ec0bff`; `v0.1.1`/`v0.1.2`/`v0.1.3` → `881c95f`/`e8e07eb`/`bd43683`, unmoved. The GitHub release `0.2.0` exists at `https://github.com/eggstack/eggup/releases/tag/v0.2.0`, `isDraft = false`, `isPrerelease = false`, published `2026-10-24T13:54:03Z`. Its body carries the per-package table — `eggup-core 0.2.0` published from `b612608` with checksum `2c86a0eb…eb23b8` — and makes **no** claim that the package ships the crate's contract documents | `git rev-list -n1 v0.2.0` and the same for the three `0.1.x` tags; `gh release view v0.2.0 --json name,tagName,url,isDraft,isPrerelease,publishedAt,body` → the values above |
| R15 | The closure record exists with the requirement→evidence matrix and the consumer-unblock reconciliation *(AC15, §10)* | This file: the matrix above, the plan §10 items in "Reviewed repository baseline", "Platform evidence", and "Registry updates" | `plans/closure/verified-update-core/019-status.md` — this file |

## Production implementation evidence

This milestone publishes; it does not change source. The production
implementation evidence is therefore the published surface itself: the
packaged manifest, the packaged file list, and the public API anchors in the
sources that were packaged.

### The published manifest

`target/package/eggup-core-0.2.0/Cargo.toml` — the normalized manifest cargo
produced for the upload — carries `name = "eggup-core"`, `version = "0.2.0"`,
`edition = "2021"`, `rust-version = "1.89"`, `license = "MIT"`, and
`[lints.rust] unsafe_code = "deny"`. Its dependency section is exactly:

```toml
[dependencies.sha2]
version = "0.10"

[target."cfg(windows)".dependencies.self-replace]
version = "1.5"
```

There is no `git`, `path`, or `patch` entry. The crates.io dependency list for
the published version agrees entry for entry, which is the registry's own
statement of the published graph rather than the source tree's intent. Because
`self-replace` is gated on `cfg(windows)`, the published Unix and macOS graph
is `sha2` alone; on Windows it is `sha2` plus `self-replace 1.5.0`. No
dependency was added, removed, or re-versioned by this milestone: the
manifest diff between the plan baseline `b75f73b` and the publication source
commit `b612608` is empty.

### The published file list

The uploaded tarball carries 23 entries:

```text
eggup-core-0.2.0/.cargo_vcs_info.json
eggup-core-0.2.0/CHANGELOG.md
eggup-core-0.2.0/Cargo.lock
eggup-core-0.2.0/Cargo.toml
eggup-core-0.2.0/Cargo.toml.orig
eggup-core-0.2.0/README.md
eggup-core-0.2.0/examples/{custom_validator,multi_member,one_member,ownership,receipts,stale_lock_recovery}.rs
eggup-core-0.2.0/src/{candidate,current_exe,domain,error,failure_taxonomy_tests,integrity,lib,lock,stage,test_support,transaction}.rs
```

There is **no `docs/` entry**. `crates/eggup-core/Cargo.toml:15-19` declares
`exclude = ["docs/", "tests/", "benches/"]`, so the crate's contract documents
under `crates/eggup-core/docs/{domain,verification,transaction}.md` are not
shipped. Neither this record nor the release notes claims otherwise; the
contracts are reachable from the repository at the tagged commit. See the
unresolved findings for the owning milestone.

`src/failure_taxonomy_tests.rs` and `src/test_support.rs` ship as source
files, but both modules are `#[cfg(test)]`-gated at
`crates/eggup-core/src/lib.rs:55-59`, so neither is compiled into a shipped
build. `Error::Injected` is likewise `#[cfg(test)]`-only
(`crates/eggup-core/src/error.rs:61-62`) and is in no packaged build.

### The published public API anchors

- `crates/eggup-core/src/error.rs:28-63` — `Error` with exactly the seven
  published variants: `InvalidInput`, `UnknownMember`, `Io`, `UpdateInProgress`,
  `DestinationConflict`, `VerificationFailed`, `CandidateExecution`, plus the
  `#[cfg(test)]`-only `Injected` at `:61-62`. The enum is closed, so an
  exhaustive downstream `match` compiles unchanged against `0.1.3` and `0.2.0`.
- `crates/eggup-core/src/transaction.rs:136-170` — `FailureCategory` keeps
  `#[non_exhaustive]` and exactly seven members: `LockContention`,
  `OwnershipConflict`, `Verification`, `Filesystem`, `InvalidInput`,
  `Injected`, `PostCommitCheck`. `RetainedEvidence` is gone, and its
  `retained-evidence` string with it. The remaining seven `as_str()` values
  are byte-identical to what `0.1.3` published.
- `crates/eggup-core/src/candidate.rs:27` — `MAX_STDIN_BYTES: usize = 1024 *
  1024`, the inclusive bound, re-exported at `crates/eggup-core/src/lib.rs:34`.
- `crates/eggup-core/src/candidate.rs:125` — `CommandSpec::stdin_data`, the
  infallible builder that attaches the bounded payload.
- `crates/eggup-core/src/candidate.rs:333`, `:410`, `:515` — the three
  `Error::Io` operation strings. `"spawning bounded candidate"` and
  `"polling bounded candidate"` are unchanged from `0.1.3`;
  `"writing bounded stdin"` is new in `0.2.0`, added by M017.

### What `0.2.0` carries that `0.1.3` did not

- **M014 — the bounded candidate deadline.** `run_bounded` evaluated its
  wall-clock deadline only while the direct child was alive, so a descendant
  holding the output pipe open pinned the call indefinitely, and the timeout
  path left both reader threads unjoined. The deadline is now an unconditional
  maximum bound on the whole bounded execution, and a reader still blocked at
  the deadline is abandoned rather than joined.
- **M015 — the public-failure-taxonomy correction.** The unreachable
  `FailureCategory::RetainedEvidence` variant and its `retained-evidence`
  string are removed, and every public fallible item now carries an `# Errors`
  section.
- **M017 — the bounded stdin surface.** `CommandSpec::stdin_data` attaches a
  payload, `MAX_STDIN_BYTES` bounds it, an over-bound payload is rejected with
  `Error::InvalidInput` before any child is spawned, and the payload is
  written on a dedicated thread whose failure kills and reaps the child and
  returns a typed `Error::Io` rather than hanging.
- **M018 — deadline priority over the stdin writer's verdict.** The outcome is
  classified by the instant the **writer** observed the write, not by the
  runner's poll phase, so the same input and the same timeout no longer
  produce two different public classifications.

**M010**, **M011**, the earlier Core audit fixes, and the **M012** public-error
compatibility correction were published at `0.1.3` and are **not** new here.
The published changelog says so, and this record does not present them as new.

## Exact commands run and results

### Recorded by the shared evidence set, on the publication source commit

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | exit 0 |
| `cargo test --workspace --all-targets --all-features --locked` | **528 passed, 0 failed, 1 ignored** |
| `cargo doc --workspace --no-deps --locked` | clean |
| `./scripts/check-docs-truth.sh` | "documentation-truth checks passed" |

The one ignored test is the pre-existing ignored `eggup-curl` unit test, whose
reason is recorded in the acquisition M016 closure record. `cargo tree
--workspace --locked` was read for the dependency-boundary review; it is a
review step, not a gate.

The plan's §7.3 package and dry-run steps (`cargo package -p eggup-core
--locked`, `cargo package -p eggup-core --list --locked`, `cargo publish -p
eggup-core --dry-run --locked`) were executed by the publication session. This
session did **not** re-run them: `cargo package` and `cargo publish` are
outside this record's read-only scope. Their artifacts were inspected
read-only instead — `target/package/eggup-core-0.2.0.crate` and its extracted
directory — and the result is recorded in R4, R5, and R6.

### Run by this closure session, first-hand, on macOS

| Command | Result |
|---|---|
| `./scripts/check-docs-truth.sh` | "documentation-truth checks passed", exit 0 |
| `cargo fmt --all -- --check` | clean, exit 0 |
| `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` | exit 0 |
| `cargo test -p eggup-core --all-targets --locked --offline` | **110 passed, 0 failed** across five test binaries (59 + 12 + 14 + 11 + 14) |
| `cargo +1.89.0 check --workspace --all-targets --locked --offline` | exit 0 |
| `cargo test --locked --offline` in `/tmp/eggup-registry-fixtures/core` | **3 passed, 0 failed** |
| `shasum -a 256 target/package/eggup-core-0.2.0.crate` | `2c86a0ebc94c75465855f661cd3aae91447f53fdb38e6f277092b155dbeb23b8` — byte-identical to the registry checksum |
| `tar tzf target/package/eggup-core-0.2.0.crate \| sort` | 23 entries; `grep -c 'docs/'` → `0` |
| `curl -fsSL https://crates.io/api/v1/crates/eggup-core/versions` and `…/0.2.0` and `…/0.2.0/dependencies` | the registry readback recorded in R1, R4, R5 |
| `gh run view 40079833945 …` and `gh api …/attempts/{1,2}/jobs` | the per-lane results recorded in R7 |
| `gh api …/actions/jobs/120830884380/logs --allow-escape-sequences` | the attempt-1 failure text quoted verbatim above |
| `gh release view v0.2.0 --json …` | the tag and release state recorded in R14 |

Toolchain identity actually used: `rustc --version` → `rustc 1.89.0
(29483883e 2025-08-04)`; `rustup toolchain list` reports
`1.89.0-aarch64-apple-darwin` installed, and the active default is
`1.89-aarch64-apple-darwin`. The MSRV check therefore ran on the 1.89 floor,
not on a newer toolchain.

The `cargo +1.89.0 test --workspace --all-targets --locked` variant of plan
§7.2 was **not** run by this session. The hosted `msrv` lane's proof is the
same shape — `cargo check --workspace --all-targets --locked` on the 1.89.0
toolchain, plus the three feature-gated footprint checks — so the floor is
proven by a compile at 1.89.0, not by a test run at 1.89.0.

## Platform evidence

Reported per platform. No platform's evidence is inferred from another's.

### Linux

- **Hosted — `stable` lane** ("Stable checks", `runs-on: ubuntu-latest`): green
  in attempt 2 of run `40079833945`. In attempt 1 it failed on the single
  step `cargo package -p eggup-service --locked --no-verify` and on no other
  step.
- **Hosted — `msrv` lane** ("MSRV check", `runs-on: ubuntu-latest`): green in
  attempt 2. In attempt 1 it failed on the same single packaged-service step
  and on no other step.
- **Hosted — `service-systemd` lane** ("Linux systemd service qualification",
  `runs-on: ubuntu-latest`): green throughout. It ran once — started
  `2026-10-24T08:54:07Z`, completed `08:54:47Z`, in the first attempt — and was
  carried forward into attempt 2 rather than re-executed, because it had
  already succeeded. This lane does not carry the packaged-service step, and
  it covers the systemd fixtures.
- **Native local Linux: not run.** The local gate and every command in this
  session ran on macOS.

### macOS

- **Hosted — `macos` lane** ("macOS tests", `runs-on: macos-latest`): green in
  attempt 2. In attempt 1 it failed on the single packaged-service step and on
  no other step.
- **Native local macOS** (this session, `darwin` arm64): `cargo fmt --all --
  --check` clean; `cargo clippy -p eggup-core --all-targets --locked --offline
  -- -D warnings` exit 0; `cargo test -p eggup-core --all-targets --locked
  --offline` **110 passed, 0 failed**; `cargo +1.89.0 check --workspace
  --all-targets --locked --offline` exit 0; the registry-only fixture
  `cargo test --locked --offline` **3 passed, 0 failed**. The
  documentation-truth gate exits 0.

### Windows

- **Hosted — `windows-check` lane** ("Windows archive, acquisition, and service
  tests and check", `runs-on: windows-latest`): green in attempt 2. In attempt 1
  it failed on the single packaged-service step and on no other step. This
  lane is the workflow's only Windows test coverage and runs
  `cargo test --workspace --all-targets --locked --no-fail-fast`, so the
  Windows-native Core M010/M011 fixtures and the SCM fixtures execute there.
- **Native local Windows: not run.**

The registry-only direct fixture's native evidence is **macOS only** — the
fixture was run on this session's macOS host. The hosted `windows-check` lane
exercises the workspace test suite on Windows, not the registry-only fixture.

## Invariant review

| Invariant | Review |
|---|---|
| Integrity evidence is SHA-256 checksum only; no authenticity or signature claim | Holds. The published manifest's `description` says "checksum integrity only, no transport or service manager"; `crates/eggup-core/src/lib.rs` states "Integrity here is checksum evidence only. No authenticity or signature claim is made: there is no authenticity verifier in this crate." No `0.2.0` change touches it. |
| Core carries no transport, service-manager, or Eggpack dependency | Holds. The published dependency list is exactly `sha2` and the Windows-only `self-replace`. `self-replace` is the deferred deletion of a replaced running image, not a transport. |
| `unsafe` is denied workspace-wide | Holds. The packaged manifest carries `[lints.rust] unsafe_code = "deny"`; `crates/eggup-core/src/lib.rs:1` carries `#![forbid(unsafe_code)]`. |
| Ownership is caller-proven; `Owned` is never inferred | Unchanged. The transaction, staging, locking, commit, rollback, and ownership machinery is untouched: `git diff --stat b75f73b b612608 -- crates/eggup-core/src/` is empty. |
| `validate` requires every member to carry `IntegrityStatus::Verified`; `IntegrityRequirement::None` always fails | Unchanged, and no source change could have moved it. |
| Core never creates live destination parents and never decides a lock is stale | Unchanged. The M010/M011 surfaces that implement the current-executable plan and the opt-in stale-lock recovery were published at `0.1.3` and are not restated as new here. |
| Failure outcomes after the lock is held are receipts, not `Err` | Unchanged. M015 moved retained-evidence reporting into `RecoveryError`; the receipt semantics are those already reviewed in the M015 and M018 closure records. |
| The public error surface is stable | Holds. `Error` keeps exactly the seven published variants, so an exhaustive `match` compiles unchanged; `FailureCategory` keeps `#[non_exhaustive]`. |

## Failure, rollback, and recovery review

This milestone changes no failure behaviour in the transaction machinery. The
only failure-path changes that reach consumers in `0.2.0` are the ones already
reviewed in their own closure records and carried here unchanged:

- The bounded-execution deadline is an unconditional maximum bound. A call
  that could previously hang past its own documented bound now returns at the
  deadline with `timed_out` set — the descendant-holds-the-pipe case is
  bounded and fail-closed rather than indefinite.
- A bounded stdin write failure observed **before** the deadline is an `Err`:
  the child is killed and reaped and the call fails closed. Observed **at or
  after** the deadline it is the deadline outcome — `Ok` with `timed_out` set
  — because the deadline already decided it. An undelivered payload is never
  reported as a success.
- An over-bound stdin payload is rejected with `Error::InvalidInput` **before
  any child is spawned**, so the bound fails closed and no process is created.
- The failure taxonomy now contains exactly seven categories, each of which a
  producer can actually produce; the unreachable variant is gone.

The rollback and recovery surface is unchanged from `0.1.3`:
`RecoveryError` keeps its two variants — `Core(Error)` and
`RecoveryRequired { evidence, detail }` — and its `#[non_exhaustive]`
attribute (`crates/eggup-core/src/error.rs:71-88`);
`CleanupDisposition` keeps its three published members — `Cleaned`,
`RetainedForRecovery`, `DeferredToProcessExit`
(`crates/eggup-core/src/transaction.rs:43-57`); and
`TransactionReceipt::recovery_path` still carries a real path. A defect in the
published `0.2.0` would require a new patch version (`0.2.1`); the published
version is immutable.

## Compatibility and migration review

**The compatibility surface of `eggup-core 0.2.0` is one removal.**

- `Error` keeps exactly the seven variants `0.1.3` published, so an exhaustive
  `match` compiles unchanged. `Error::Injected` remains `#[cfg(test)]`-only and
  is in no packaged build. The registry-only fixture asserts `Error` is still
  `Display` and `Debug`; M012's byte-identical exhaustive-match fixture is the
  proof that the match itself compiles unchanged against both versions
  (recorded in `plans/closure/verified-update-core/013-status.md`).
- `FailureCategory` keeps `#[non_exhaustive]` and its seven categories. Its
  seven `as_str()` strings are byte-identical to the published ones; the only
  string that disappears is `retained-evidence`, and no runtime failure could
  ever have carried it, because no producer existed.
- `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` are **additive**. The
  `Error::Io` `operation` strings are additive: no published operation string
  changes value.
- The only source break is `FailureCategory::RetainedEvidence`. `eggup-core
  0.1.3` on crates.io **does** publish that variant, so a downstream crate
  that named it explicitly — a match arm, an equality assertion, or a constant
  `as_str()` call — stops compiling against `0.2.0`. Deleting the name
  reference is the whole of the required work. It is a compile-time break, not
  a silent behaviour change.

**Consumer action.** One: delete any reference to
`FailureCategory::RetainedEvidence`. Beyond that, no consumer migration is
required — no data format changed, no configuration changed, and nothing else
is needed beyond picking up the new version.

**Migration shape for a crate that pins core.** A caret requirement
(`eggup-core = "^0.2"`) resolves the new version on the next resolve, with no
consumer action beyond the one removal. An exact pin (`eggup-core = "=0.1.3"`)
does not move: `eggup-eggpack`'s `=0.2.0` re-pin is the train's own handling
of that case, and a seam fix published after `0.2.0` still needs a separate
adapter republication before `eggup-eggpack` inherits it.

## Security review

- **No authenticity or signature claim.** Unchanged and restated in the
  published crate's own documentation. The crate selects no release, infers
  no version, and authorizes no destination.
- **No new attack surface.** No dependency was added, removed, or re-versioned.
  The published dependency list is `sha2` and the Windows-only `self-replace`,
  both already-present registry crates.
- **No `unsafe`.** `#![forbid(unsafe_code)]` in the crate and
  `unsafe_code = "deny"` in the packaged manifest.
- **Bounded resources.** Two of the four carried corrections are
  resource-exhaustion bounds, and both fail closed: the wall-clock deadline is
  an unconditional maximum bound on the whole bounded execution, so a
  descendant holding the output pipe cannot pin the call indefinitely; and the
  stdin payload is bounded at 1 MiB inclusive, with an over-bound payload
  rejected before any child is spawned.
- **No test-only code in a shipped build.** `Error::Injected`,
  `src/test_support.rs`, and `src/failure_taxonomy_tests.rs` are all
  `#[cfg(test)]`-gated, so a published build cannot produce the `injected`
  category or construct an injected error.
- **No secret material in the package.** The file list is 23 entries and
  carries no fixture data, credential, or test data; `tests/` and `benches/`
  are excluded by the manifest. The two test-support source files that do ship
  are `#[cfg(test)]`-gated and are not compiled into a shipped build.
- **Registry integrity.** The registry checksum equals the locally built
  artifact's SHA-256 byte for byte, so the published tarball is the artifact
  this repository qualified. The fixture lockfile's checksum matches it
  independently.

## Documentation and operations evidence

- **The crate's changelog is truthful** and is the published
  `CHANGELOG.md`: `crates/eggup-core/CHANGELOG.md:3-21` attributes M017, M018,
  M015, and M014 to `0.2.0`, states that `0.1.3` still carries every one of
  them, and attributes M010, M011, the earlier audit fixes, and the M012
  correction to `0.1.3`. `:52-59` states the one consumer action.
- **The contract documents are not shipped, and nothing claims they are.**
  `crates/eggup-core/Cargo.toml:15-19` excludes `docs/`; the published
  tarball has no `docs/` entry; this record and the GitHub release body both
  say so explicitly rather than implying the package carries the contracts.
  The contracts are reachable from the repository at the tagged commit.
  The owning milestone is **Verified Update Core M016 — `eggup-core`
  contract-doc packaging**, named as the follow-on by planning C013 and
  recorded as `plans/registry.md` open finding 14. It is **not** discharged by
  this publication and needs its own republication.
- **The release notes state each package's exact publication source SHA** —
  all seven published from `b612608`, which is the tag — and the
  `eggup-core 0.2.0` registry checksum `2c86a0eb…eb23b8`. The release exists
  as `0.2.0` and is not a draft or a prerelease.
- **The documentation-truth gate passes** at closure time:
  `./scripts/check-docs-truth.sh` → "documentation-truth checks passed",
  exit 0.
- **No source drift rode along.** `git diff --stat b75f73b b612608 --
  crates/eggup-core/src/` and `… -- crates/eggup-core/Cargo.toml` are both
  empty, so the published artifact is the qualified tree and nothing else.
- **Operations.** The publication is manual — one publish act, the plan's §7.7
  `cargo publish -p eggup-core --locked` from the clean publication source
  commit, which the crates.io `audit_actions` list records as a single
  `publish` entry by `dbowm91`. CI never publishes. The registry state after
  the publication is immutable: a defect in the published `0.2.0` requires
  `0.2.1`, never an overwrite or a yank-and-republish.

## Unresolved findings

| # | Finding | Severity | Disposition |
|---|---|---|---|
| U1 | **Core M016 — the contract documents are not shipped in the published package.** `crates/eggup-core/Cargo.toml:15-19` excludes `docs/`, so the published `eggup-core 0.2.0` carries no `crates/eggup-core/docs/{domain,verification,transaction}.md`, and a consumer on crates.io or docs.rs cannot read the contract it is bound to. The gap is **not** discharged by this publication; the publication leaves it exactly as it was. | **low** for this milestone | Owned by **Verified Update Core M016 — `eggup-core` contract-doc packaging**, named as the follow-on by planning C013 and recorded as `plans/registry.md` open finding 14. The manifest change alone has no consumer effect without the `eggup-core` republication that would deliver it, so M016 needs its own milestone and its own publication, and it must not be treated as a packaging one-liner. `plans/registry.md` rates that finding **medium** as a consumer-facing documentation gap; this record does not restate or change that rating. On the repository's "medium-or-higher forces a new corrective" rule: a corrective **exists and is named** — Core M016, with its republication to be authorized separately — so the rule is satisfied by a registered, owned follow-on rather than by closing the finding here. The gap is a pre-existing one that this publication neither introduced nor enlarged; shipping the contract docs inside the `0.2.0` package would have exceeded this milestone's authorized scope, and the published `0.2.0` is not withdrawn for it. |
| U2 | **Registry finding 1 — the `eggup-curl` sub-second-deadline tests remain load-sensitive on a busy host.** | **low** | Pre-existing, outside this crate's publication surface, blocks nothing here, and unrelated to `eggup-core`. Worth a corrective; the acquisition subsystem owns it. The shared evidence set records the same condition. |
| U3 | **The published `eggup-core 0.1.3` still carries the M014, M017, and M018 defects.** A consumer pinned to `0.1.3` — including one whose graph resolves through an exact pin — keeps the unbounded candidate deadline, the missing bounded stdin surface, and the classification flap. | **low** as a standing exposure | Not a defect of this milestone: `0.2.0` is the version that carries the corrections, and this milestone is the consumer-unblock reconciliation for them. The `0.1.3` version and its `v0.1.3` tag are immutable and are not withdrawn. |
| U4 | **The registry-only fixture's assertions cover a subset of the plan §4 list.** Plan §4 enumerates six behaviours for Fixture A. The fixture as built asserts three: the bounded-stdin surface exists and `MAX_STDIN_BYTES` is 1 MiB; `Error` is `Display` and `Debug`; the published category set is seven and carries no `retained-evidence` string. The over-bound payload rejection, a bounded run reaching its deadline with `timed_out` set, an exhaustive `match` over the seven `Error` variants, and `InstallPlan::for_current_executable` planning are **not** exercised by the registry-only proof. | **low** | Bounds the reach of the registry-only evidence; it is not a defect in the published crate. Those behaviours are covered by the crate's own suite (110 tests) and, for the exhaustive match, by M012's byte-identical fixture recorded in `plans/closure/verified-update-core/013-status.md`. Recorded so no reader overstates what the registry-only proof establishes. |
| U5 | **The version heading date does not match the publication date.** `crates/eggup-core/CHANGELOG.md:3` headed the version `## 0.2.0 — 2026-10-23` and the root `CHANGELOG.md` recorded the crates as published on 2026-10-23, while the registry timestamp for `0.2.0` is `2026-10-24T13:46:31.983818Z`. The changelog entry was cut on 2026-10-24 (commit `2161f36`, `2026-10-24T08:48:47Z`), so the heading matched neither the cut date nor the publication date, and the two repository changelogs disagreed with `plans/registry.md`, which already recorded the train as "published 2026-10-24". | **low** | **Corrected before finalisation.** The repository's copies of the seven per-crate `## 0.2.0` headings now read `## 0.2.0 — 2026-10-24`, the publication date, per the repository convention that the heading carries the publication date; the root `CHANGELOG.md` likewise records 2026-10-24. The **published** `.crate` tarballs still carry `## 0.2.0 — 2026-10-23`, the cut date, and cannot be re-cut — crates.io accepts no overwrite. That difference is immutable registry state and is recorded in `plans/registry.md` rather than left silent. |

## Roadmap disposition

**M019 — `eggup-core 0.2.0` publication — closed.** The milestone is complete:
the crate is on the registry, the tag exists, the release exists, and the
registry-only proof passes.

The roadmap file is owned by another writer and was not modified by this
milestone. The intended row for `plans/subsystems/verified-update-core-roadmap.md`
§12 is:

```markdown
| M019 | closed | `plans/implementation/verified-update-core/019-eggup-core-0.2.0-publication.md` | `plans/closure/verified-update-core/019-status.md` | Published from `b612608`; hosted run `40079833945` green on all five lanes after the plan §9.1 train-order exception; `v0.2.0` + release `0.2.0` created; registry-only direct fixture green (3/3); the crate's contract documents are **not** shipped — Verified Update Core M016 owns that |
```

**M016** — `eggup-core` contract-doc packaging — has no implementation plan and
no closure record. It remains the named follow-on for U1 and is not authored by
this milestone.

## Registry updates

**Registry state.** `eggup-core` now lists `0.1.0`, `0.1.1`, `0.1.2`,
`0.1.3`, and `0.2.0`. None is yanked. `0.2.0`'s checksum is
`2c86a0ebc94c75465855f661cd3aae91447f53fdb38e6f277092b155dbeb23b8` (registry
published-table form `2c86a0eb…eb23b8`), size 78 576 bytes, published by
`dbowm91` at `2026-10-24T13:46:31.983818Z`, from source commit
`b61260857fdb3904a091c503b1dff05251ec0bff`. The `0.1.x` line is intact.

**Consumer-unblock reconciliation.** Publishing `eggup-core 0.2.0` was the
first step of the train, and it is what unblocked the other six: each of them
names `eggup-core` in its manifest, and `cargo package` resolves that name
from the registry rather than from the in-tree path dependency. The shared
evidence set's registry table now shows all seven publishable crates at
`0.2.0`, published in dependency order — core, archive, acquisition, curl,
eggfetch, service, eggpack — from the one source commit `b612608`.
`eggup-transport-footprint` remains `publish = false` and is never published.
Nothing remained blocked on this milestone's upload once it landed.

**Source break for consumers.** `eggup-core`'s own break is the removal of
`FailureCategory::RetainedEvidence` and its `retained-evidence` string. The
train's other breaks — `AcquisitionError::Io` becoming a struct variant with
`Clone`/`PartialEq`/`Eq` removed, `ServiceError::Manager` becoming a struct
variant, and `LifecycleUpdateReceipt` gaining `engine_policy` — are carried by
the seam and the service crate and are recorded in their own closure records;
the shared evidence set states them so that no closure record understates
them.

**Intended `plans/registry.md` updates**, handed over rather than applied —
that file is owned by another writer:

1. The verified-update-core **workstream row**: add M019 to the closed range,
   record `eggup-core 0.2.0` published 2026-10-24, hosted run `40079833945`
   green on all five lanes, and the plan §9.1 train-order exception.
2. The verified-update-core **subsystem status row**: the same, with M016
   still the next milestone.
3. A **milestone row** for M019, in the form used by the M013 row: published
   from `b612608`, `v0.2.0` + release created, registry-only direct fixture
   green, and the contract-doc gap carried as M016.
4. The registry's open finding 16 — the deferred packaged-service verification
   — is discharged by this upload, and the strict step is restored in CI by
   `2d6e089`. The registry row should record that.

The registry's `0.2.0` train table and the `0.1.3` table are already present
and already carry `eggup-core 0.2.0` from `b612608` with checksum
`2c86a0eb…eb23b8`; no change to either is needed for this milestone.

## Appendix — facts this record could not establish

- The **publisher identity's** token scope and the crates.io audit log beyond
  the `audit_actions` entry for `0.2.0` were not inspected.
- The **GitHub release body's** wording for the six sibling crates was read
  (it carries their checksums), but each sibling crate's own package contents
  and registry readback belong to that crate's closure record, not this one.
- The **service fixture's test result** was not run or claimed here; the
  read-only lockfile observation in R13 is recorded as an observation, and the
  result belongs to the service publication milestone's record.
- The **`cargo publish --dry-run`** step of plan §7.3 was not re-run by this
  session. The uploaded artifact it produced was inspected read-only instead,
  and its checksum matches the registry's.
- The **`cargo +1.89.0 test --workspace --all-targets --locked`** step of plan
  §7.2 was not run by this session. The MSRV floor is proven by a workspace
  *check* at 1.89.0 — run locally here and by the hosted `msrv` lane — not by a
  test run at 1.89.0.
- The **`cargo tree` for a Windows target** of `eggup-core` (plan §10) was not
  run. The published graph is instead established by the crates.io dependency
  list, which carries `self-replace` with `target = cfg(windows)` and `sha2`
  with no target.
