# Closure record — service-lifecycle M017

## Status

**Closed.** `eggup-service 0.2.0` is on crates.io, published 2026-10-24 from the
train's shared publication source commit `b612608`, checksum
`31dd2e222f93457317d909c2daf1426945f3315d75b7cb591932c58904fe1b4a`, crate size
94031 bytes, publisher `dbowm91`, no version yanked. The registry-only proof
passes: a fresh crate **outside** this repository, depending on
`eggup-service = "=0.2.0"` and on nothing else, resolves **both**
`eggup-service 0.2.0` and `eggup-core 0.2.0` from crates.io, and its two tests
pass. The hosted qualification is green on all five lanes in run `40079833945`,
and the local gate is green at the current head.

This is the **sixth** crate of the lockstep `0.2.0` train of the seven
publishable crates, and the **first** publication of this crate that carries the
fix for the stdin-verdict defect the published `0.1.3` still carries.

Two registry findings are discharged by this milestone, in different senses, and
the distinction is recorded rather than blurred: finding **15** is closed **for
consumers** — the fix is now reachable from the registry — while the published
`eggup-service 0.1.3` still carries the defect and the `v0.1.3` tag was not
moved; finding **16** is closed outright, because `eggup-core 0.2.0` is on the
registry and the strict `cargo package -p eggup-service --locked` step was
restored in commit `2d6e089` across all four lanes that carry it.

No line of `crates/eggup-service/src/**` changed in this milestone. The
production bytes of the published crate are the qualified baseline's bytes.

## Source plan and roadmap

- Source plan: `plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md`.
  Its `Status:` line now reads *"closed 2026-10-24; `eggup-service 0.2.0`
  published from `b612608` (run `40079833945` green on all five lanes);
  closure record: `plans/closure/service-lifecycle/017-status.md`"* — the plan's
  owner updated it concurrently with this record. When this record began, the
  line still read `ready for handoff`; see **UF-3**.
- Roadmap: `plans/subsystems/service-lifecycle-roadmap.md` (service-lifecycle
  subsystem). **Another writer owns that file**, and it has already applied the
  M017 row and the updated header status line; see "Roadmap disposition".
- Findings discharged: `plans/registry.md` § "Post-0.1.3 open findings" —
  **finding 15** (the published crate's stdin-verdict defect) and **finding 16**
  (the deferred packaged-service verification).
- Prerequisites: Service **M012**, **M013**, and **M014** (closed and
  hosted-qualified in run `40008039843`), **M015** (closed and hosted-qualified
  in run `40062299630`), and **M016** (closed and hosted-qualified in run
  `40067964510`, commit `a431aaa`). Verified Update Core **M019**
  (`plans/implementation/verified-update-core/019-eggup-core-0.2.0-publication.md`)
  published `eggup-core 0.2.0` before this milestone's package step ran; the
  core publication's §9.1 records the train-order exception that the four
  packaged-service lanes could not satisfy before the first upload.

## Reviewed repository baseline

- HEAD at closure time: `2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219` on branch
  `plans/corrective-batch-2026-10-23`.
- The working tree was **dirty by design** when this record's work began. The
  parent-owned close-out edits for this train were already uncommitted:
  `plans/registry.md`, the root `CHANGELOG.md`, the seven crate changelogs, the
  five `0.2.0` publication plans, and the five subsystem roadmaps — plus the
  sibling closure records for this train as they landed
  (`plans/closure/verified-update-core/019-status.md` and
  `plans/closure/acquisition-transport/017-status.md`). **Every one of those
  files was read only here.** None of them is this milestone's work, and the
  repository's `crates/eggup-service/src/**` was not touched by any of it. The
  `Status:` header of this milestone's plan was corrected by its owner
  concurrently with this record; see UF-3.
- **No file outside this milestone's ownership was modified**, and **no git
  state was mutated**: no `stash`, `checkout`, `restore`, `commit`, `push`,
  `tag`, `clean`, `reset`, or branch creation was performed. Every git
  inspection was read-only (`git log`, `git show`, `git rev-parse`, `git
  rev-list`, `git cat-file`, `git diff`, `git blame`, `git status`).
- Two SHAs are in play and they are **distinct**:
  - the plan's repository baseline, `b75f73bf1ad138f79e7e16d50f35671e52bafe96`
    (short `b75f73b`), which is what the source-delta check is scoped to; and
  - the **publication source commit**,
    `b61260857fdb3904a091c503b1dff05251ec0bff` (short `b612608`), from which
    the published `.crate` was uploaded.
  Two release-prep commits sit between them, and both carry changes the
  publication needed: `afe6201` authored the five `0.2.0` publication plans,
  and `2161f36` carried the workspace version bump, the requirement widenings,
  the `eggup-eggpack` re-pin, the eight changelog cuts, and the
  documentation-truth gate generalisation.

## Implementation commits / PRs

| Commit | What it carries |
|---|---|
| `afe6201430cb4eba78e809059edef8a7d1e36ebf` | The five `0.2.0` publication plans, including this milestone's plan. |
| `2161f36967b2d9afda4e2b05f7eb260e664fe761` | The workspace version bump to `0.2.0`, the requirement widenings, the `eggup-eggpack` re-pin, the eight changelog cuts, the documentation-truth gate generalisation, and the M016 publication-coupling guard re-point. |
| `b61260857fdb3904a091c503b1dff05251ec0bff` | The train-order exception recorded in the core plan's §9.1. **This is the publication source commit** — every published `.crate` in the train carries exactly this SHA in `.cargo_vcs_info.json`, and none records a dirty tree. |
| `2d6e08924fa5d7a9e147595c2f76fc7cc0e9d219` | Restoring the strict packaged-service verification in CI — the follow-on named by registry finding **16**. |

- **Tag.** `v0.2.0`, annotated (tag object
  `aae1d2d1d033e33d3f8857cad43f42684cc88e98`), pointing at `b612608`. The
  `v0.1.1`, `v0.1.2`, and `v0.1.3` tags were **not** moved: `v0.1.1` remains
  the lightweight tag `881c95ff…`, `v0.1.2` the annotated tag over
  `e8e07eb5…`, and `v0.1.3` the annotated tag over `bd436836…`.
- **Release.** GitHub Release `0.2.0`,
  `https://github.com/eggstack/eggup/releases/tag/v0.2.0`, published
  2026-10-24T13:54:03Z, the repository's "Latest" release. Its notes state each
  package's exact publication source SHA, including
  `eggup-service 0.2.0 | b612608 (this tag) | 31dd2e22…fe1b4a`.
- **The upload.** `cargo publish -p eggup-service --locked` from the clean
  publication source commit — the only upload this milestone authorized. This
  record did not perform it and does not claim to have: the evidence that it
  happened is the crates.io entry read back in command 3 and the release notes
  in command 9. No PR was opened; the work is on the branch
  `plans/corrective-batch-2026-10-23`.

## Executive finding

`eggup-service 0.2.0` is on crates.io, published 2026-10-24 from the train's
shared source commit `b612608`, and it is the **first** publication of this
crate that carries the fix for the stdin-verdict defect the published `0.1.3`
still carries: the replaced `SystemExecutor` runner polled its stdin writer's
verdict with `try_recv()`, which *removes* the message it returns, so a verdict
of `Ok(())` was discarded and the later receive observed an empty, disconnected
channel and reported `Manager(Spawn, "manager stdin failed for …")` for a write
that had succeeded — on `crontab -`, the crate's only production stdin caller.
The publication also delivers what M012 through M016 put on the source line: one
commit/rollback engine, the typed `ServiceError::Manager { kind, detail,
os_code }` taxonomy with `manager_kind()` and `manager_os_code()` as the
programmatic discriminators, the typed Windows SCM surface with its live-fixture
registration, the consolidation of the bounded runner onto
`eggup_core::run_bounded`, and the release note for the eighth behaviour
difference — the note M015's closure required the republication to carry. The
registry-only proof is what makes the widened `eggup-core = "0.2"` edge
load-bearing rather than nominal: a consumer's lockfile resolves
`eggup-core 0.2.0` from crates.io and cannot resolve `eggup-core 0.1.3`, which
lacks `CommandSpec::stdin_data` and therefore could not compile the published
crate at all. The two registry findings are discharged in the right directions:
finding **15** is closed **for consumers**, because the fix is now reachable
from the registry, while the published `0.1.3` keeps the defect and `v0.1.3`
was not moved; finding **16** is closed outright, because `eggup-core 0.2.0` is
on the registry and the strict `cargo package -p eggup-service --locked` step
was restored in `2d6e089`. Nothing in this milestone changed a line of
`crates/eggup-service/src/**`, and the source-delta check against the
qualified baseline is empty.

## Requirement → evidence matrix

| Requirement | Evidence (`file:line`, command) | Result |
|---|---|---|
| **R-G1** — registry preflight: `0.2.0` absent before the upload, `0.1.0`–`0.1.3` present, none yanked | crates.io versions readback (command 3) | **Met.** The registry lists exactly `0.1.0`, `0.1.1`, `0.1.2`, `0.1.3`, `0.2.0`, and **no** version carries `yanked = true`. No existing version was re-published, re-uploaded, or withdrawn; crates.io rejects an overwrite, so the single `0.2.0` entry created at 2026-10-24T13:51:17Z is the only upload this milestone made. |
| **R-G2** — publish authority for the crate name | crates.io versions readback (command 3), the `published_by` field | **Met.** `published_by` is `dbowm91` (id 391532) for `0.2.0` — the same publisher that published `0.1.0`, `0.1.1`, `0.1.2`, and `0.1.3`. |
| **R-G3** — version identity: the published version is exactly `0.2.0` | `Cargo.toml:15` (`version = "0.2.0"` under `[workspace.package]`); `crates/eggup-service/Cargo.toml` (`version.workspace = true`); `target/package/eggup-service-0.2.0/Cargo.toml` (`version = "0.2.0"`) | **Met.** The workspace version is inherited, the packaged manifest reads `0.2.0`, and the registry entry is `0.2.0`. |
| **R-G4** — the packaged manifest carries the widened `eggup-core` edge | `crates/eggup-service/Cargo.toml` (`eggup-core = { version = "0.2", path = "../eggup-core" }`); `target/package/eggup-service-0.2.0/Cargo.toml` (`[dependencies.eggup-core]` / `version = "0.2"`); crates.io dependencies endpoint (command 4) | **Met.** The registry records `eggup-core ^0.2` as a normal dependency of `eggup-service 0.2.0`. The path edge is rewritten to the registry edge by cargo's normalisation, which is exactly the requirement that the packaged crate has to carry. |
| **R-G5** — the public API boundary of the packaged crate | `crates/eggup-service/src/lib.rs:419` (`ServiceError::Manager { kind, detail, os_code }`), `:339` (`ManagerFailureKind`, six variants), `:486` (`manager_kind`), `:495` (`manager_os_code`), `:176` (`pub use eggup_core::Ownership`), `:3122` (`pub type CronOwnership = Ownership`), `:921` / `:953` (`ManagerCommandOutput` / `pub type CommandOutput = ManagerCommandOutput`), `:912` (`MAX_DIAGNOSTIC_BYTES: usize = 512`); `crates/eggup-service/src/lifecycle_update.rs:90` (`pub engine_policy: CommitEnginePolicy`); `crates/eggup-service/src/commit_engine.rs:159`, `:180`, `:201` | **Met.** Every item of the plan's §2 surface is present in the source that was packaged. The registry-only fixture exercises the retyped `Manager` variant and the `Ownership` type identity against the **published** bytes, not against the workspace. |
| **R-G6** — the dependency surface is unchanged in shape | `crates/eggup-service/Cargo.toml`; `cargo tree -p eggup-service --locked --offline --depth 1` (command 10) → `eggup-core v0.2.0`, `windows-args v0.2.0`; the same command with `--target x86_64-pc-windows-msvc` → `eggup-core v0.2.0`, `windows-args v0.2.0`, `windows-service v0.8.1`; crates.io dependencies endpoint (command 4) | **Met.** `windows-args` is declared unconditionally because the SCM command-line parser is not `cfg`-gated; `windows-service` is under `cfg(windows)` only, and crates.io records its target as `cfg(windows)`. No dependency was added, removed, or re-versioned. |
| **R-G7** — no `git`, `path`, or `patch` edge in the packaged manifest | `target/package/eggup-service-0.2.0/Cargo.toml` — the normalised manifest names three registry dependencies and no source override; `target/package/eggup-service-0.2.0/Cargo.lock` — 37 registry source lines, **0** `path` or `git` sources | **Met.** The packaged crate's own lockfile resolves `eggup-core 0.2.0` from `registry+https://github.com/rust-lang/crates.io-index` with checksum `2c86a0eb…eb23b8`. |
| **R-G8** — the package file list, and the manifest's `exclude` | the shared evidence set's package qualification, cited; `tar -tzf target/package/eggup-service-0.2.0.crate` (command 6) | **Met.** The shipped bytes are `src/` (`commit_engine.rs`, `disposition.rs`, `lib.rs`, `lifecycle_update.rs`, `windows_scm.rs`), `README.md`, `CHANGELOG.md`, the normalised `Cargo.toml`, and cargo's own `Cargo.lock`, `Cargo.toml.orig`, and `.cargo_vcs_info.json`. **No `tests/` entry and no `benches/` entry** — the manifest's `exclude = ["tests/", "benches/"]` holds, so the crate's integration tests are not shipped. |
| **R-G9** — the hosted matrix is green on the publication source commit | run `40079833945`, head SHA `b61260857fdb3904a091c503b1dff05251ec0bff`, conclusion `success` (command 14 and the per-lane logs, commands 15–17) | **Met.** All five lanes green: `stable` 528 passed, `msrv` check-only by design, `service-systemd` 9 passed, `macos` 528 passed, `windows-check` 650 passed. The run's **first attempt** failed in exactly one step, in four lanes, with the `eggup-core = "^0.2"` resolution error; the four lanes were re-run **on the same commit** and passed unchanged. That first-attempt failure is a publication state, recorded in the core plan's §9.1, and it is discharged by this milestone's own upload. |
| **R-G10** — changelog truthfulness | `crates/eggup-service/CHANGELOG.md` § "0.2.0 — 2026-10-23" | **Met.** The section attributes M012, M013, M014, M015, and M016 to `0.2.0`; names both source breaks (`ServiceError::Manager` as a struct variant, and the `engine_policy` field with the struct-literal construction path); names both consumer-visible behaviour changes (the cron managed block body must embed the absolute executable path, and one `OperationDeadline` per public operation); states that `0.1.3` still carries the stdin-verdict defect that `0.2.0` fixes; and states the SHA-256-only integrity caveat. It does not present `0.1.3` content as new. |
| **R-G11** — maintainer authorization, recorded with its date and its decision | `plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md` §3 gate **G11** | **Met.** The authorization is the **lockstep `0.2.0` train decision** that commissions the publication of the seven publishable crates, dated **2026-10-24**, and the plan itself is that decision's service half: it was authored in commit `afe6201` on 2026-10-24, and the upload followed on the same date. |
| **R-G12** — no unclassified source delta | `git diff --name-only b75f73b -- crates/eggup-service/src/` (command 8) | **Met.** **No output.** No file under `crates/eggup-service/src/**` differs from the qualified baseline. The only crate changes since that baseline are the manifest's `eggup-core` requirement widening and the `0.2.0` changelog cut, both carried by the release-prep commits and both inside the plan's boundary. |
| **R-§4** — the registry-only fixture | `/tmp/eggup-registry-fixtures/service/`; its `Cargo.toml` (`[workspace]`, `eggup-service = "=0.2.0"`, `publish = false`, and no other dependency), its `Cargo.lock`, `cargo tree --locked` (command 12), and `cargo test --locked --offline` (command 11) | **Met.** The fixture is a fresh crate outside the repository, with **no** `path`, `git`, or `patch` source in its manifest, its workspace table, or any `.cargo/config.toml`. Its lockfile resolves `eggup-service 0.2.0` (checksum `31dd2e22…fe1b4a`) **and** `eggup-core 0.2.0` (checksum `2c86a0eb…eb23b8`) from `registry+https://github.com/rust-lang/crates.io-index`. `cargo tree --locked` shows `registry-fixture-service → eggup-service v0.2.0 → eggup-core v0.2.0, windows-args v0.2.0`. **2 of 2 tests pass**: `published_service_manager_error_is_struct_shaped` and `published_service_ownership_is_the_core_ownership_type`. |
| **R-§5** — tag and release state | `git rev-list -n1 v0.2.0` → `b61260857fdb3904a091c503b1dff05251ec0bff`; `git cat-file -t v0.2.0` → `tag`; `gh release view v0.2.0` (command 9) | **Met.** `v0.2.0` is annotated and points at the publication source commit. `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved. The release notes state each package's exact source SHA and do not claim that the published package ships the crate's integration tests. |
| **R-F15** — registry finding **15** is discharged | the **published** bytes, read out of the uploaded `.crate`: `eggup-service-0.2.0/src/lib.rs:1080` calls `eggup_core::run_bounded(&spec)` on an `eggup_core::CommandSpec` built at `:1062`, and the file contains **zero** occurrences of `std::process::Command`; the crate's published `CHANGELOG.md` carries the **D8** release note; the registry readback (command 3) | **Met, for consumers.** `eggup-service 0.2.0` carries the fix, so the defect is no longer reachable by a consumer who resolves `0.2.0`. The published `0.1.3` still carries it — that version is not withdrawn, its checksum is unchanged, and the `v0.1.3` tag was **not** moved. |
| **R-F16** — registry finding **16** is discharged | commit `2d6e089` (`git show 2d6e089 -- .github/workflows/ci.yml`, command 7) | **Met.** The strict step `cargo package -p eggup-service --locked` is restored in all four lanes that carry it — `stable`, `msrv`, `macos`, and `windows-check` — replacing `cargo package -p eggup-service --locked --no-verify`. `eggup-core 0.2.0` is on the registry, so the packaged crate's `eggup-core = "^0.2"` requirement resolves and the packaged sources are compiled, not merely packaged. |
| **R-W11** — consumer-unblock reconciliation | the registry readback for all seven crates (command 5) | **Met.** `eggup-service 0.2.0` unblocks nothing that was blocked on an earlier step of the train — core, archive, seam, curl, and eggfetch all landed before it — and `eggup-eggpack`, the train's last crate, does **not** depend on the service. The train is complete: all seven crates are on the registry at `0.2.0` from `b612608`. |
| **R-AC16** — the closure record exists with the required matrix | this record | **Met.** `plans/closure/service-lifecycle/017-status.md`, with the requirement→evidence matrix the plan's §10 requires. |

## Production implementation evidence

**There is no production implementation in this milestone, by construction.** The
published crate's production bytes are the qualified baseline's bytes: the
source-delta check is empty (`git diff --name-only b75f73b --
crates/eggup-service/src/` → no output). What this milestone produced is a
**registry artifact**, and the evidence below is the artifact itself.

### The published package

| Element | Value | How it was read |
|---|---|---|
| Package file list | `src/` (5 modules), `README.md`, `CHANGELOG.md`, normalised `Cargo.toml`, plus cargo's own `Cargo.lock`, `Cargo.toml.orig`, `.cargo_vcs_info.json` — **no `tests/`, no `benches/`** | `tar -tzf target/package/eggup-service-0.2.0.crate` |
| Packaged manifest's dependency section | `[dependencies.eggup-core] version = "0.2"`; `[dependencies.windows-args] version = "=0.2.0"`; `[target."cfg(windows)".dependencies.windows-service] version = "=0.8.1"` | `target/package/eggup-service-0.2.0/Cargo.toml` |
| Packaged lockfile's sources | 37 registry source lines, **0** `path` or `git` sources; `eggup-core 0.2.0` resolved from crates.io with checksum `2c86a0eb…eb23b8` | `target/package/eggup-service-0.2.0/Cargo.lock` |
| Source commit in the published tarball | `b61260857fdb3904a091c503b1dff05251ec0bff`, `path_in_vcs: "crates/eggup-service"`, no dirty-tree marker | `target/package/eggup-service-0.2.0/.cargo_vcs_info.json` |
| The published crate runs its manager commands through the one bounded runner | `eggup-service-0.2.0/src/lib.rs:1080` calls `eggup_core::run_bounded(&spec)` on an `eggup_core::CommandSpec` built at `:1062`; the file holds **zero** occurrences of `std::process::Command` | the uploaded `.crate` extracted and read |
| Local checksum of the uploaded tarball | `31dd2e222f93457317d909c2daf1426945f3315d75b7cb591932c58904fe1b4a` — **byte-identical to the registry's checksum for `eggup-service 0.2.0`** | `shasum -a 256 target/package/eggup-service-0.2.0.crate` |
| The packaged crate compiles against the registry's `eggup-core` | the verify step's build output is present under `target/package/eggup-service-0.2.0/target/` — fingerprints for `eggup-service` and `eggup-core`, and the registry-resolved `eggup-core 0.2.0` in the packaged lockfile | directory listing and lockfile read |

The last row is the local counterpart of registry finding **16**: the packaged
sources were compiled against the **registry** copy of `eggup-core 0.2.0`, not
merely packaged. That is the property the restored strict CI step asserts.

### The registry-only fixture

`/tmp/eggup-registry-fixtures/service/` — a fresh crate outside this repository.

- `Cargo.toml`: a `[workspace]` table, `publish = false`, and exactly one
  dependency — `eggup-service = "=0.2.0"`. **No** `path`, `git`, or `patch`
  source in the manifest, in the workspace table, or in any
  `.cargo/config.toml`.
- `Cargo.lock`: `eggup-service 0.2.0` (checksum `31dd2e22…fe1b4a`) **and**
  `eggup-core 0.2.0` (checksum `2c86a0eb…eb23b8`), both from
  `source = "registry+https://github.com/rust-lang/crates.io-index"`.
- **The transitive resolution is the point.** The fixture does not depend on
  `eggup-core`; it resolves `eggup-core 0.2.0` only because the published
  `eggup-service 0.2.0` requires it at `^0.2`. A consumer's lockfile therefore
  cannot resolve `eggup-core 0.1.3`, which carries neither
  `CommandSpec::stdin_data` nor core's unconditional execution deadline and
  could not compile `eggup-service 0.2.0` at all.
- `cargo tree --locked`:

  ```text
  registry-fixture-service v0.0.0
  └── eggup-service v0.2.0
      ├── eggup-core v0.2.0
      └── windows-args v0.2.0
  ```

- `cargo test --locked` → **2 passed; 0 failed**:
  `published_service_manager_error_is_struct_shaped` and
  `published_service_ownership_is_the_core_ownership_type`. The second one
  proves `eggup_service::Ownership` **is** `eggup_core::domain::Ownership`
  without the fixture depending on `eggup-core` directly, by asserting the
  type's own name.
- **No live host manager is mutated.** The fixture exercises the published
  crate's types only; it installs, starts, stops, and uninstalls nothing, and
  writes to no real crontab. The real-systemd behaviour is proven by the hosted
  `service-systemd` lane, not by this fixture.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` on the **macOS**
lane (Darwin 25.6.0 arm64). **No `cargo publish`, no `cargo package`, and no
mutating git command was run.** Every cargo command below is `--offline
--locked` and reads the existing cache; the only network access was read-only
`curl` against crates.io and read-only `gh` against the hosted run.

| # | Command | Result |
|---|---|---|
| 1 | `git log --oneline -12`; `git rev-parse b75f73b b612608 afe6201 2161f36 2d6e089` | the four release-prep commits and both full SHAs, as recorded above |
| 2 | `git rev-parse v0.2.0`; `git rev-list -n1 v0.2.0`; `git cat-file -t v0.2.0`; the same for `v0.1.1`, `v0.1.2`, `v0.1.3` | `v0.2.0` → tag object `aae1d2d1…`, annotated, commit `b61260857fdb3904a091c503b1dff05251ec0bff`. `v0.1.1`/`v0.1.2`/`v0.1.3` unmoved |
| 3 | `curl -fsSL -A "eggup-release-check/1.0" https://crates.io/api/v1/crates/eggup-service/versions` | five versions — `0.2.0` (checksum `31dd2e22…fe1b4a`, size 94031, created 2026-10-24T13:51:17.639169Z, publisher `dbowm91`, **not yanked**), `0.1.3`, `0.1.2`, `0.1.1`, `0.1.0` — **none yanked** |
| 4 | `curl -fsSL -A "eggup-release-check/1.0" https://crates.io/api/v1/crates/eggup-service/0.2.0/dependencies` | `eggup-core ^0.2` (normal), `windows-args =0.2.0` (normal), `windows-service =0.8.1` (normal, target `cfg(windows)`) |
| 5 | the same versions endpoint for `eggup-core`, `eggup-archive`, `eggup-acquisition`, `eggup-curl`, `eggup-eggfetch`, `eggup-eggpack` | all seven crates carry `0.2.0`; **no version yanked** in any of them; every `0.1.x` line intact |
| 6 | `tar -tzf target/package/eggup-service-0.2.0.crate` | 11 entries: `.cargo_vcs_info.json`, `CHANGELOG.md`, `Cargo.lock`, `Cargo.toml`, `Cargo.toml.orig`, `README.md`, `src/commit_engine.rs`, `src/disposition.rs`, `src/lib.rs`, `src/lifecycle_update.rs`, `src/windows_scm.rs` — **no `tests/`, no `benches/`** |
| 7 | `git show 2d6e089 -- .github/workflows/ci.yml` | the strict `cargo package -p eggup-service --locked` restored in **four** lanes — `stable`, `msrv`, `macos`, `windows-check` — replacing the `--no-verify` form |
| 8 | `git diff --name-only b75f73b -- crates/eggup-service/src/`; `git diff --stat b612608 -- crates/eggup-service/src/` | **no output** in both — no production source drift against the qualified baseline and none between the publication source commit and the current head |
| 9 | `gh release view v0.2.0 --json name,tagName,url,isDraft,isPrerelease,publishedAt` | `0.2.0`, `v0.2.0`, `https://github.com/eggstack/eggup/releases/tag/v0.2.0`, not a draft, not a prerelease, published 2026-10-24T13:54:03Z |
| 10 | `cargo tree -p eggup-service --locked --offline --depth 1`, with and without `--target x86_64-pc-windows-msvc` | Unix: `eggup-core v0.2.0`, `windows-args v0.2.0`. Windows: `eggup-core v0.2.0`, `windows-args v0.2.0`, `windows-service v0.8.1` |
| 11 | `cd /tmp/eggup-registry-fixtures/service && cargo test --locked --offline` | **2 passed; 0 failed** — `published_service_manager_error_is_struct_shaped`, `published_service_ownership_is_the_core_ownership_type` |
| 12 | `cd /tmp/eggup-registry-fixtures/service && cargo tree --locked --offline` | `registry-fixture-service → eggup-service v0.2.0 → eggup-core v0.2.0, windows-args v0.2.0` — registry sources only |
| 13 | `shasum -a 256 target/package/eggup-service-0.2.0.crate`; `cat target/package/eggup-service-0.2.0/.cargo_vcs_info.json` | `31dd2e222f93457317d909c2daf1426945f3315d75b7cb591932c58904fe1b4a` — **identical to the registry's checksum**; source SHA `b61260857fdb3904a091c503b1dff05251ec0bff` |
| 14 | `gh run view 40079833945 --json databaseId,headSha,status,conclusion,createdAt,updatedAt,jobs` | run `40079833945`, head `b61260857fdb3904a091c503b1dff05251ec0bff`, conclusion `success`, created 2026-10-24T08:54:04Z, updated 2026-10-24T13:49:45Z, **five jobs, all `success`** |
| 15 | `gh run view 40079833945 --job <windows-check> --log` | 650 passed, 0 failed; the service's five dedicated steps and the live SCM fixture green; the `EGGUP-SCM-LIVE-SKIP` and `EGGUP-SCM-LIVE-CLEANUP-FAILED` markers appear **zero** times |
| 16 | `gh run view 40079833945 --job <service-systemd> --log` | 9 passed, 0 failed |
| 17 | `gh run view 40079833945 --job <stable> --log` and `--job <macos> --log` | 528 passed, 0 failed in each |
| 18 | `./scripts/check-local.sh` at head `2d6e089` | **exit 0** — `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` exit 0 with no warnings; `cargo test --workspace --all-targets --all-features --locked` **528 passed, 0 failed, 1 ignored**; `cargo doc --workspace --no-deps --locked` clean; `cargo tree --workspace --locked` read for the boundary review; `./scripts/check-docs-truth.sh` → "documentation-truth checks passed" |
| 19 | `./scripts/check-docs-truth.sh` | **"documentation-truth checks passed"**, exit 0 — unchanged by this record |
| 20 | `grep -n` over `crates/eggup-service/src/` for `MAX_DIAGNOSTIC_BYTES`, `manager_kind`, `manager_os_code`, `pub use eggup_core::Ownership`, `pub type CronOwnership`, `pub type CommandOutput`, `pub struct ManagerCommandOutput`, `pub enum ManagerFailureKind`, `pub engine_policy` | the locations cited in rows **R-G5**; `MAX_DIAGNOSTIC_BYTES: usize = 512` and the six `ManagerFailureKind` variants confirmed in the source |

The **workspace gate** was run at the current head `2d6e089`, which differs from
the publication source commit `b612608` only in `.github/workflows/ci.yml` and
`AGENTS.md` — no crate source, no manifest, no lockfile — so the gate result
carries for the published bytes.

### Per-platform results — reported individually, never inferred

| Platform | What was run | Result |
|---|---|---|
| **macOS** (this machine: Darwin 25.6.0 arm64) | commands 1–14 and 17–20, including the whole workspace gate (command 18) and the registry-only fixture (command 11) | **All green. 0 failed.** The local gate: 528 passed, 0 failed, 1 ignored — the one ignored test is the pre-existing ignored `eggup-curl` unit test, whose reason is recorded in the acquisition M016 closure. The registry-only fixture: 2 passed, 0 failed. The tarball's local SHA-256 is **byte-identical** to the registry's checksum. The hosted `macos` lane of run `40079833945` is macOS too and is green — 528 passed, 0 failed (command 17). |
| **Linux** | the hosted `stable` lane and the hosted `service-systemd` lane of run `40079833945` — `cargo test --workspace --all-targets --all-features --locked` on `ubuntu`, and the real-systemd fixtures as root | **Green.** `stable`: 528 passed, 0 failed. `service-systemd`: **9 passed, 0 failed** — the `published-api-0.1.2` fixture's negative control `published_0_1_2_cannot_complete_stop_for_owned_failed_unit` (1) and its three `registry_0_1_2_*` inspection controls (3); the current source's `systemd_failed_service` fixture `real_systemd_owned_failed_service_stop_is_observed_and_quiescent` (1); and the `published-api-0.1.3` fixture's `real_systemd_owned_failed_service_stop_is_observed_and_quiescent` (1) and its three `registry_0_1_3_*` controls (3). The published `0.1.x` crates are therefore still exercised against the real Service Manager, unchanged. The `msrv` lane is also Linux and is **check-only by design** — it runs `cargo check --workspace --all-targets --locked` on `1.89.0` and **no test at all**, so it contributes no test result and none is inferred for it. |
| **Windows** | the hosted `windows-check` lane of run `40079833945` — the ten `cargo test` steps, including the three named service diagnostic tests, the SCM unit tests, and the live SCM fixture | **Green, native, and executed — not compile-only.** 650 passed, 0 failed. The service's own steps: `diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges` (1 passed), `service_errors_output_and_permission_remediation_stay_utf8_bounded` (1 passed), `lifecycle_failure_detail_is_utf8_safe_bounded_and_control_free` (1 passed), `windows_scm::tests` (**16 passed**), and the live SCM fixture `windows_scm_live` (**2 passed**) against the genuine Service Control Manager, registering a disposable service through the crate's **public** API. The `EGGUP-SCM-LIVE-SKIP` and `EGGUP-SCM-LIVE-CLEANUP-FAILED` markers appear **zero** times in the lane log, so the skip path was not taken and the disposable registration was removed. The remaining five of the ten steps cover acquisition/archive/curl, eggpack, Core `current_executable` (8 passed) and Core `stale_lock_recovery` (14 passed), and the whole workspace. |

No platform's result is inferred from another's. The Linux evidence is the
hosted `stable` and `service-systemd` lanes; the macOS evidence is this
machine's local gate plus the registry-only fixture; the Windows evidence is
the hosted `windows-check` lane.

## Invariant review

| # | Invariant | Evidence |
|---|---|---|
| 1 | **No production source changed** in this milestone | `git diff --name-only b75f73b -- crates/eggup-service/src/` → no output (command 8). The published bytes are the qualified baseline's bytes. |
| 2 | **The published package does not ship the crate's tests** | the tarball listing has no `tests/` or `benches/` entry (command 6); the manifest's `exclude = ["tests/", "benches/"]` is carried into the normalised packaged manifest. |
| 3 | **The published crate depends on the registry's `eggup-core 0.2.0`, never on a path edge** | the packaged manifest names `[dependencies.eggup-core] version = "0.2"`; the packaged lockfile has 37 registry source lines and **0** path or git sources. |
| 4 | **A consumer cannot resolve `eggup-core 0.1.3`** against the published service | the registry-only fixture's lockfile resolves `eggup-core 0.2.0` from crates.io; the crates.io dependencies endpoint records `eggup-core ^0.2` for `eggup-service 0.2.0`. |
| 5 | **The dependency boundary of the workspace is unchanged** | `cargo tree -p eggup-service --locked --offline --depth 1` shows `eggup-core` and `windows-args` on Unix, and `windows-service` additionally on the Windows target. No service-manager, transport, or Eggpack edge. |
| 6 | **The public API surface of the published crate is the one the plan states** | the source locations cited in row **R-G5**, and the registry-only fixture's two tests exercising the retyped `Manager` variant and the `Ownership` type identity against the published bytes. |
| 7 | **The public API surface did not change in this milestone either** | `git diff --name-only b75f73b -- crates/eggup-service/src/` → no output. The retyped `Manager` variant and the `engine_policy` field are the qualified baseline's state, carried by M012 and M013. |
| 8 | **No published version was re-published, withdrawn, or yanked** | the crates.io readback shows five versions, none yanked; `v0.1.1`, `v0.1.2`, and `v0.1.3` are unmoved; the release notes state the `0.1.x` line stays published. |
| 9 | **Integrity evidence stays SHA-256 checksum only** | the crate's `0.2.0` changelog section and the release notes both say so; the record's only integrity claim is the checksum equality in command 13. No authenticity, signature, release-selection, or destination-authorization claim is made anywhere in this milestone. |
| 10 | **No git state was mutated, and no file outside this milestone's ownership was modified** | the command list in this record contains no mutating git operation; `git status --porcelain` showed only the concurrently added `plans/registry.md`, which was read and not written. |

## Failure / rollback / recovery review

- **The publication is a one-way act, and that is stated rather than papered
  over.** crates.io does not accept an overwrite. A defect in the published
  `eggup-service 0.2.0` would require a **new patch version** (`0.2.1`); it
  could not be corrected by re-uploading `0.2.0`, and a yank-and-republish
  would be a different artifact under the same number, which this repository's
  own rules forbid. The plan's §9 states this, and this milestone honours it:
  no existing version was touched.
- **The `0.1.x` line is the fallback, and it is intact.** Every consumer pinned
  to `0.1.0`–`0.1.3` continues to resolve, and the in-repository
  `published-api-0.1.2` and `published-api-0.1.3` fixtures — separate
  workspaces pinned to `=0.1.2` and `=0.1.3` — still build and still run their
  real-systemd controls in the hosted `service-systemd` lane.
- **The tag cannot be rolled back either.** `v0.2.0` is the train's single
  shared source commit, so a defect in the service half cannot be corrected by
  moving the tag; the correction would be a new version from a new commit, and
  the train's later publications append to the same release notes without
  moving the tag.
- **The one failure this milestone's own run observed was a publication state,
  not a defect.** The hosted run's first attempt failed in exactly one step, in
  four lanes, with `failed to select a version for the requirement `eggup-core =
  "^0.2"` / candidate versions found which didn't match: 0.1.3, 0.1.2, 0.1.1,
  ...`. That step resolves the packaged service crate's `eggup-core` against
  crates.io, and `eggup-core 0.2.0` was not on the registry when the first
  crate of the train was uploaded. The four affected lanes were re-run **on the
  same commit** after the upload and passed unchanged, and the fifth lane,
  `service-systemd`, which does not carry the packaged-service step, was green
  throughout. The condition is recorded in the core plan's §9.1, and this
  milestone's upload is what discharges it.
- **The crate's own failure semantics are untouched by the publication.** The
  fail-closed properties — the executable allowlist, the cleared child
  environment, the bounded stdin, the bounded output, the unconditional
  deadline, and the typed `ManagerFailureKind` taxonomy — are properties of the
  source line, which this milestone did not change. The publication widens
  their **reach**, not their shape.
- **No rollback or crash-journal mechanism is claimed for the crate**, and none
  is invented here.

## Compatibility and migration review

The published `0.2.0` is the first `eggup-service` release that is not a
`0.1.x`. Its compatibility surface is exactly the one the shared evidence set
states for the train, and it is stated here for this crate.

**Two source breaks.**

1. `ServiceError::Manager(detail)` became
   `ServiceError::Manager { kind, detail, os_code }`
   (`crates/eggup-service/src/lib.rs:419`), carrying a `ManagerFailureKind`
   with exactly six variants — `Spawn`, `Timeout`, `Permission`, `NotFound`,
   `Protocol`, `LocalFilesystem`. A consumer that matched the tuple form must
   match the struct form and branch on `kind` — or on the programmatic
   discriminators `ServiceError::manager_kind()` (`:486`) and
   `ServiceError::manager_os_code()` (`:495`) — and must **not** branch on the
   message text.
2. `LifecycleUpdateReceipt` gained the public field `engine_policy`
   (`CommitEnginePolicy`) (`crates/eggup-service/src/lifecycle_update.rs:90`).
   The struct derives only `Debug` and has **no `Default` impl**, so a struct
   literal is its only construction path: a consumer that constructs the
   receipt with a struct literal must add the field and stops compiling until
   it does. A consumer that only reads the receipt's fields is unaffected.

**No consumer migration is required for the type-identity changes.**
`eggup_service::Ownership` re-exports `eggup_core::Ownership`
(`lib.rs:176`), `eggup_service::CronOwnership` is a public alias of it
(`lib.rs:3122`), and `eggup_service::CommandOutput` survives as a
**non-deprecated** alias of `ManagerCommandOutput` (`lib.rs:953`, with
`ManagerCommandOutput` at `lib.rs:921`). All three keep resolving under their
existing spellings, and no conversion is needed anywhere.

**Two consumer-visible behaviour changes.**

1. **The cron managed block body must now embed the absolute executable path
   of the spec.** A `desired_block` that omits `spec.executable()` receives
   `ServiceError::InvalidInput` from `inspect`, `install`, and `uninstall`,
   where it previously received success (`install`) or a normal result. The
   gate runs before any listing or mutation, so a non-conforming request never
   reaches the crontab.
2. **Exactly one `OperationDeadline` per public operation.** Every adapter
   spends exactly one, so an operation that previously overran its declared
   budget now fails with a `Timeout` kind instead of overrunning. This is a
   fail-closed tightening, not a relaxation.

**The published `0.1.3` still carries the stdin-verdict defect.** A consumer
pinned to `0.1.3` — or to `0.1.0`, `0.1.1`, or `0.1.2` — has not received the
fix and must move to `0.2.0` to get it. The `v0.1.3` tag was not moved and the
version was not withdrawn.

**No dependency change, so no downstream pin cascade.** `eggup-service` still
depends on `eggup-core` (`^0.2`), `windows-args` (`=0.2.0`), and
`windows-service` (`=0.8.1`, Windows only). `eggup-eggpack`'s exact pins are
untouched by this crate's publication — they moved in the train's release-prep
commit `2161f36`, and `eggup-eggpack` does not depend on `eggup-service` at
all.

## Security review

- **No new privilege, no new filesystem authority, no new network surface, no
  new dependency, and no production code change.** The published bytes are the
  qualified baseline's bytes, so the crate's security properties are the
  baseline's properties by construction.
- **The allowlist is still the only path to a binary.** The service's
  pre-flight still runs before anything is spawned, so a rejected request
  creates no process at all.
- **The child environment is still cleared**, and only the explicit
  user-scoped session-bus variables are injected.
- **The stdin payload is still bounded before the spawn**, and the output bound
  still discards an over-bound stream rather than returning a truncated buffer.
- **The stdin-verdict fix is a security-positive change.** The defect it
  repairs was **fail-closed** — nothing was corrupted, no state was misreported
  in the permissive direction, and no privilege was escalated — but a consumer
  that installs a cron block through the published `0.1.3` receives a typed
  `Spawn` failure where a successful install was intended. Publishing the fix
  removes that false negative from the reachable path.
- **The publication does not weaken the integrity story.** Integrity evidence
  remains SHA-256 checksum only. The crate's `0.2.0` changelog, the release
  notes, and this record make no authenticity or signature claim, select no
  release, and authorize no destination.
- **The registry-only fixture mutates no host state.** It exercises the
  published crate's types; it installs, starts, stops, and uninstalls nothing
  and writes to no real crontab. The real-systemd behaviour is proven by the
  hosted `service-systemd` lane, which runs as root on a throwaway host, not
  by the fixture.
- **The disposable Windows service registration is cleaned up.** The
  `EGGUP-SCM-LIVE-CLEANUP-FAILED` marker appears zero times in the Windows
  lane log, so the registration the live SCM fixture made was removed.

## Documentation / operations evidence

| Document | What it carries |
|---|---|
| `crates/eggup-service/CHANGELOG.md` § "0.2.0" | The published release note: the train's order and shared source commit, the dependency surface and the reason the `eggup-core` requirement widened, the SHA-256-only integrity caveat, both source breaks with their exact migrations, both consumer-visible behaviour changes, the "no consumer migration required" statement for the type-identity changes, and the statement that this release is the first to carry the stdin-verdict fix that the published `0.1.3` lacks. The eight differences D1–D8 are enumerated, with **D8** named as the defect fix rather than a cost. **No published version section below it was touched.** The section heading reads `## 0.2.0 — 2026-10-24` in the repository; the **published** artifact's copy reads `## 0.2.0 — 2026-10-23`, which cannot now be changed — see **UF-6**. |
| `crates/eggup-service/README.md` | **Not edited by this milestone.** It states no version number and no source-break claim, so it is already truthful; read-only evidence. |
| `architecture/service-lifecycle.md` | **Not edited by this milestone.** It is the norm for D1–D8 and the crate's dependency boundary; read-only evidence. |
| GitHub Release `0.2.0` | The per-package publication source SHA table — `eggup-service 0.2.0` from `b612608`, checksum `31dd2e22…fe1b4a` — the publication order and its reason, the three source breaks, the statement that the `0.1.x` versions are not withdrawn, and the SHA-256-only integrity caveat. |
| `plans/registry.md` | **Edited by its owning writer, concurrently with this record, not by this milestone.** See "Registry updates". |
| `plans/subsystems/service-lifecycle-roadmap.md` | **Edited by its owning writer, concurrently with this record.** The M017 row and the updated header status line are applied; see "Roadmap disposition". |
| `plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md` | **Edited by its owning writer, concurrently with this record.** The `Status:` line now reads `closed 2026-10-24`. |
| root `CHANGELOG.md` | **Edited by its owning writer, concurrently with this record.** Its header now reads `## 0.2.0 — 2026-10-24, completed 2026-10-24` and its opening paragraph is in the past tense. See **UF-5**. |
| `plans/closure/service-lifecycle/017-status.md` | **New.** This record. |

**Operational state after this milestone.** `eggup-service 0.2.0` is on
crates.io with no version yanked; `v0.2.0` and the GitHub Release `0.2.0` point
at `b612608`; the strict packaged-service step is restored in CI; the `0.1.x`
line remains published and remains exercised by the in-repository
`published-api-0.1.2` and `published-api-0.1.3` fixtures in the hosted
`service-systemd` lane.

## Unresolved findings with severity

### UF-1 — The published `eggup-service 0.1.3` still reports a successful stdin write as a failure — **medium — closed for consumers; the exposure remains on the `0.1.3` artifact**

**Finding.** Unchanged in substance from registry finding **15** and from
service M015 § UF-1 and M016 § UF-1. The published crate's bounded runner
polls its stdin-writer channel with `try_recv()`, which *removes* the message
it returns, so the verdict `Ok(())` is consumed and discarded by a check that
only looks for `Err(_)`, and the later receive observes an empty, disconnected
channel and concludes the write failed. The affected path is `crontab -`, the
crate's only production stdin caller, so a crontab install through the
published `0.1.3` reports `manager(Spawn, "manager stdin failed for …")` after
a write that in fact succeeded.

**What this milestone did.** `eggup-service 0.2.0` is published and carries the
fix, so the defect is no longer reachable by a consumer who resolves `0.2.0`.
That is the disposition the registry now records — *closed for consumers on
2026-10-24*.

**What this milestone did not do, and did not claim.** The published
`eggup-service 0.1.3` still carries the defect. That version is not withdrawn,
its checksum is unchanged, and the `v0.1.3` tag was **not** moved. A consumer
pinned to `0.1.x` carries the exposure until it moves to `0.2.0`.

**Severity: medium, unchanged for the `0.1.3` artifact.** It is not high: it
fails closed, it is not a safety or privilege issue, and the affected crate has
no consumer yet — the Eggup product itself is not released. It is not low: it
is a functional defect in a published crate's only stdin path, reproducible with
a two-line probe.

**Disposition.** The source-line defect was closed by service **M015**; the
release note that carries the fix was added by service **M016**; this
milestone delivers both to consumers. Nothing further is owed to the `0.1.3`
artifact, because a published version cannot be corrected in place — only
superseded.

### UF-2 — The restored strict packaged-service step — **low — CLOSED by hosted run `40100087952`**

**Finding.** The strict step `cargo package -p eggup-service --locked` was
restored in commit `2d6e089`, which is **after** the run that qualified this
publication: run `40079833945` has head SHA `b612608`, at which the workflow
still ran the `--no-verify` form in its four lanes.

**Resolution.** The restored form has since executed in a hosted lane. Run
`40100087952`, on commit `2d6e089` — the commit that removed `--no-verify` — is
**green on all five lanes** (`stable`, `msrv`, `macos`, `windows-check`,
`service-systemd`), and the step named *Run cargo package -p eggup-service
--locked* — the strict form, with `--no-verify` absent — **succeeded on all
four lanes that carry it**. The `service-systemd` lane does not carry the step
and was green throughout.

This run is the hosted qualification of registry finding 16's follow-on: the
deferral is **discharged**, not merely recorded. Finding 16 is closed.

The earlier draft of this finding, written before the run was checked, recorded
the step as exercised only locally. That was true of the run that qualified
this publication (`40079833945`) and is not true of the hosted record as a
whole; the hosted record now covers it.

**Severity: low, and now closed.** It was a qualification-coverage gap, not a
behavioural or security defect, and it blocked nothing.

### UF-3 — The source plan's `Status:` line read `ready for handoff` — **RESOLVED during this record's work**

**Finding.** When this record began,
`plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md`
carried `Status: ready for handoff` in its header, although the milestone it
plans was published and closed.

**Resolution.** The plan's owner corrected the line concurrently with this
record. It now reads *"closed 2026-10-24; `eggup-service 0.2.0` published from
`b612608` (run `40079833945` green on all five lanes); closure record:
`plans/closure/service-lifecycle/017-status.md`"*. The plan file was read only
here and was **not** edited by this record.

**Severity: low, and now closed.** It was a status line, not a behavioural or
security claim, and it blocked nothing.

### UF-4 — Registry finding 1, the load-sensitive `eggup-curl` sub-second-deadline tests — **low — OPEN, blocking nothing, unrelated to this crate**

**Finding.** `build_curl_args_passes_sub_second_deadlines_to_fake_curl` and
`sub_second_total_timeout_kills_child_promptly` carry sub-second wall-clock
budgets and fail intermittently on a loaded host. The defect is test
robustness, not transport semantics, and it predates and is unrelated to the
`0.2.0` train.

**Severity: low.** It blocks nothing, it is unrelated to `eggup-service`, and it
is recorded here only because the registry's finding list is the place the
train's open items are read from. **This milestone neither closes nor touches
it.**

### UF-5 — The root `CHANGELOG.md` framed the `0.2.0` release as in progress — **RESOLVED during this record's work**

**Finding.** When this record began, the root changelog's header read
`## 0.2.0 — 2026-10-23, in progress`, and its opening paragraph said the
`0.2.0` source version "is being published", although all seven crates of the
train are on crates.io at `0.2.0`.

**Resolution.** The root changelog's owner corrected it concurrently with this
record. The header now reads `## 0.2.0 — 2026-10-24, completed 2026-10-24` and
the opening paragraph is in the past tense. The file was read only here and was
**not** edited by this record.

**Severity: low, and now closed.** It was a documentation-truthfulness matter
about a summary file, not a claim about any crate's behaviour, and it blocked
nothing.

### UF-6 — The **published** release note's heading date precedes the registry publication by one day — **low — accepted, and not correctable**

**Finding.** The published artifact's release note is headed
`## 0.2.0 — 2026-10-23` — read directly from
`eggup-service-0.2.0/CHANGELOG.md` inside the uploaded `.crate` — while the
registry records the publication at 2026-10-24T13:51:17Z. The same one-day
offset appeared in all seven crates of the train, so it was the train's
changelog-cut date rather than a service-specific drift.

**Resolution of the repository copy.** The seven crate changelogs were corrected
concurrently with this record and now read `## 0.2.0 — 2026-10-24` in the
workspace, matching the registry timestamp. The changelog files were read only
here and were **not** edited by this record.

**What remains.** The **published** `eggup-service 0.2.0` artifact still carries
the `2026-10-23` heading, and it cannot be changed — crates.io does not accept
an overwrite. The published artifact's heading therefore permanently differs
from the repository's copy of the same file.

**Severity: low.** It is a date on a section heading of an immutable artifact;
it does not misstate the crate's behaviour, its dependency surface, or its
integrity evidence, and it blocks nothing. A consumer reading the published
changelog sees a release date one day earlier than the upload, which is
accurate about when the note was written and not misleading about what was
published.

## Roadmap disposition

**The roadmap is owned by another writer, and that writer has already applied
the M017 updates.** `plans/subsystems/service-lifecycle-roadmap.md` now carries:

- a header status line that reads *"M001–M017 closed"* and names
  *"M017 published `eggup-service 0.2.0` 2026-10-24 from `b612608` (hosted run
  `40079833945` green on all five lanes)"*; and
- the § 12 "Milestone status" row for **M017**, marked *"closed;
  hosted-qualified (run `40079833945`)"*, naming this closure record, the
  checksum `31dd2e22…fe1b4a`, the publication source commit `b612608`, the
  `v0.2.0` tag and the release, the closure of registry finding 15 for
  consumers resolving `0.2.0`, the registry-only fixture's transitive
  `eggup-core 0.2.0` resolution, and the restoration of the strict
  `cargo package -p eggup-service --locked` step in `2d6e089` as the closure
  of registry finding 16.

**Nothing in the roadmap was edited by this record** — the file was read only.
This record's own drafted row is retained in the appendix for comparison; the
applied row above is the authoritative one.

## Registry updates

**`plans/registry.md` is owned by another writer, and that writer has already
applied the M017 updates.** The following were observed in the working tree
during this record's work and are reported, not proposed:

- § "Post-0.1.3 open findings" entry **15** now carries the disposition
  *"closed for consumers on 2026-10-24"*: Service Lifecycle M017 published
  `eggup-service 0.2.0` from `b612608`, which carries the fix, so the defect
  is no longer reachable by a consumer who resolves `0.2.0`; the published
  `0.1.3` still carries it — that version is not withdrawn and `v0.1.3` was not
  moved — so a consumer pinned to `0.1.3` still carries the exposure, and the
  fix reaches it only by resolving `0.2.0`. Closure:
  `plans/closure/service-lifecycle/017-status.md`.
- § "Post-0.1.3 open findings" entry **16** now carries the disposition
  *"closed on 2026-10-24"*: the `eggup-core` republication landed —
  `eggup-core 0.2.0` is on crates.io, published from `b612608` — and the strict
  step was restored in commit `2d6e089`, so all four lanes that carry it now
  run `cargo package -p eggup-service --locked`, and the packaged sources are
  compiled against the registry copy of `eggup-core 0.2.0` rather than only
  packaged. The registry-only fixture asserts the same resolution from outside
  the repository.
- The § "Dependency-ready implementation work" table carries the row for
  **Service lifecycle | M017 eggup-service 0.2.0 publication | closed
  2026-10-24**, naming this record and the checksum `31dd2e22…fe1b4a`, the
  source commit `b612608`, the green run `40079833945`, the registry-only
  fixture's `eggup-core 0.2.0` resolution, and the two finding dispositions.
- The train's registry summary names the service among the seven published
  crates with its source commit and checksum, and the subsystem status row
  for **Service lifecycle** reads *"M001–M017 closed"* with the strict
  packaged-service step's restoration in `2d6e089` named.

**Nothing in `plans/registry.md` was edited by this record.** The file was read
only, and it now carries the complete M017 disposition — the two finding
entries, the milestone-status row, the train summary, and the subsystem status
row. No hand-off to the registry's owner remains open from this record.

## Appendix — this record's own drafted rows (superseded)

The two blocks below are what this record drafted for the roadmap's owner. **Both
have since been applied by that owner**, in the wording the owner chose — the
applied wording is reported in "Roadmap disposition" and is the authoritative
one. The drafts are retained here only so the parent can see what this record
proposed and reconcile the two wordings if it wants to.

### Draft roadmap row — `plans/subsystems/service-lifecycle-roadmap.md` § 12 "Milestone status"

```
| M017 | closed 2026-10-24; hosted-qualified (run `40079833945`) | `plans/implementation/service-lifecycle/017-eggup-service-0.2.0-publication.md` | `plans/closure/service-lifecycle/017-status.md` | Publishes `eggup-service 0.2.0` (checksum `31dd2e22…fe1b4a`) from the train's shared source commit `b612608`, the sixth of the seven publishable crates. This is the first publication that carries the fix for the stdin-verdict defect the published `0.1.3` still carries: the replaced runner consumed the stdin writer's `Ok(())` verdict with `try_recv()`, so a successful write was reported as `manager(Spawn, "manager stdin failed for …")` on the `crontab -` path. The publication also delivers M012's single commit/rollback engine, M013's typed `ServiceError::Manager { kind, detail, os_code }` and one-deadline-per-operation model, M014's typed Windows SCM surface with its live-fixture registration, M015's consolidation onto `eggup_core::run_bounded`, and M016's release note for the eighth difference. Registry-only proof: a fresh crate outside the repository, depending on `eggup-service = "=0.2.0"` and nothing else, resolves `eggup-service 0.2.0` and `eggup-core 0.2.0` from crates.io, so a consumer's lockfile cannot resolve `eggup-core 0.1.3`, which could not compile the published crate; 2 of 2 fixture tests pass. Hosted run `40079833945` green on all five lanes — `stable` 528 passed, `msrv` check-only by design, `service-systemd` 9 passed, `macos` 528 passed, `windows-check` 650 passed with the live SCM fixture green and no skip marker. Two registry findings discharged: **15** closed for consumers on 2026-10-24 (the published `0.1.3` still carries the defect and `v0.1.3` was not moved) and **16** closed on 2026-10-24 (the strict `cargo package -p eggup-service --locked` step restored in `2d6e089` across all four lanes that carry it). Two source breaks — the retyped `ServiceError::Manager` and the `LifecycleUpdateReceipt::engine_policy` field — and two consumer-visible behaviour changes — the cron managed block must embed the absolute executable path, and one `OperationDeadline` per public operation. No production source changed; the published bytes are the qualified baseline's bytes |
```

### Draft roadmap header status line — the intended replacement for line 3

```
Status: active; M001–M017 closed; M012, M013, and M014 closed 2026-10-23 (hosted run `40008039843`); M015 bounded executor consolidation closed 2026-10-23 (hosted run `40062299630`); M016 published-change release note closed 2026-10-24; `eggup-service 0.2.0` published 2026-10-24 from `b612608` (hosted run `40079833945`); downstream wg-basic M004 and M005 technical work closed; production signing/publication remains externally blocked
```

### Note for the root `CHANGELOG.md` — **resolved during this record's work**

When this record began, the root changelog's header read
`## 0.2.0 — 2026-10-23, in progress` and its opening paragraph said the `0.2.0`
source version "is being published", although all seven crates of the train are
on crates.io at `0.2.0`. The file's owner has since corrected it to
`## 0.2.0 — 2026-10-24, completed 2026-10-24` with a past-tense opening
paragraph. This record applied nothing. See **UF-5**.
