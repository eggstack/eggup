# Closure record — service-lifecycle M016

## Status

**Closed.** All three defects — **D1** (the absent D8 release note and the wrong difference count), **D2** (the unflagged `engine_policy` source break), and **D3** (the M015 closure's stale and self-contradictory test inventory) — are corrected in this working tree. Every correction is derived from a command run against the tree, not asserted.

This milestone is **documentation-only** for the crate: **no production code, no test code, no manifest, no lockfile, no CI file, and no architecture document changed.** The crate's whole suite passes with unchanged per-target counts, clippy is warning-free, and the Windows target compiles.

**One residual is explicitly *not* closed and is not claimed to be:** the published `eggup-service 0.1.3` still carries the stdin-verdict defect (registry finding **15**, M015 § "UF-1"). M016 adds the **release note** for that fix; only the republication milestone can discharge the defect itself.

## Source plan and roadmap

- Source plan: `plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md`
  (`Status: implemented`).
- Roadmap: `plans/subsystems/service-lifecycle-roadmap.md` (service-lifecycle
  subsystem). **The parent owns that file**; the drafted row is in the
  "Appendix — ready-to-paste rows" below.
- Corrects: `plans/closure/service-lifecycle/015-status.md` § "Corrective
  addendum (2026-10-24) — test-inventory correction by Service M016".
- Prerequisite: Service **M015**
  (`plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md`),
  closed and hosted-qualified in run `40062299630` (commit `b9885fa`). M016
  corrects M015's **records**; it does not re-implement M015.
- Finding of record: `plans/registry.md` § "Post-0.1.3 open findings" —
  **finding 15** (**medium**), the published crate's stdin-verdict defect.

## Reviewed repository baseline

- HEAD at implementation time: `bf439f519300354ac14823e74acd48c7995033a6`
  on branch `plans/corrective-batch-2026-10-23`.
- The working tree was **not** clean at implementation time and continued to
  change under other agents while this milestone was in flight. The
  concurrent modifications observed were `.opencode/skills/docs-hygiene/SKILL.md`,
  `.opencode/skills/release-workflow/SKILL.md`,
  `.opencode/skills/verify-workflow/SKILL.md`, `AGENTS.md`, `docs/crates.md`,
  and `plans/registry.md`. **None of them was modified by this milestone**;
  § "Ownership verification" lists the evidence.
- **No file outside this milestone's ownership was modified**, and **no git
  state was mutated**: no stash, checkout, restore, commit, push, tag, clean,
  reset, or branch creation was performed. The published-source evidence was
  produced with `git show` and `git grep`, which are read-only.

## Implementation commits / PRs

None. The working tree is uncommitted; the parent session owns the commit and
the branch. **Nothing was published, no tag moved, and no version was bumped.**

## Executive finding

Three documentation-truthfulness defects, all found by reading the records
against the tree. No production report exists for any of them, and none is
claimed.

**D1 — the crate's own changelog omitted the release note for a fix that ships
in a published crate.** `architecture/service-lifecycle.md:636-655` records
**eight** behaviour differences, **D1–D8**, and states at `:642-644` that *"The
eighth, **D8**, is not a cost: it is a **defect in the runner this
consolidation replaced**, which the consolidation fixes."* D8 is the fix for the
stdin-verdict defect in the **published** `eggup-service 0.1.3` (registry
finding **15**): the replaced runner polled its writer channel with
`try_recv()`, which *removes* the message, so a verdict of `Ok(())` was
consumed and discarded and a **successful** stdin write was reported as
`Manager(Spawn, "manager stdin failed for {program}")`. The crate's changelog
said *"Seven differences are recorded"* and enumerated only **six** — **D4 and
D8 were both absent**. The release note for a fix of a known published defect
was therefore **entirely absent from the crate's own changelog**, and the M015
closure's republication instruction — *"requires the republication milestone,
which must carry the fix and state it in its notes"* (M015 § UF-1) — had
nothing to carry forward.

**D2 — an unflagged source break.** `LifecycleUpdateReceipt` gained the public
field `engine_policy: CommitEnginePolicy`. The struct derives only `Debug`,
has **no `Default` impl**, and has **ten** public fields against the published
`0.1.3`'s **nine** — so a consumer of the published crate that constructs the
struct with a **struct literal** (its only construction path) stops compiling.
The changelog's compatibility preamble named **one** source-breaking change
(the typed manager failure kinds) and was silent about this one. The M012
closure had already recorded it (*"**Source-visible addition.** A consumer that
destructures the receipt **exhaustively** (without `..`) must add the field"*,
`plans/closure/service-lifecycle/012-status.md`, § "Compatibility and
migration") — the **closure** knew; the **changelog** did not.

**D3 — the M015 closure's test inventory was stale and self-contradictory.**
Commit `b9885fa` split the single argv guard into three and added a step to
the hosted `windows-check` lane. The M015 inventory was written at baseline
`20ac770` and never reconciled, so it stated **12** tests where the file has
**14**, cited a **deleted** test name as live evidence in three places, stated
a total of **15** where the tree yields **16**, and stated that the
`windows-check` lane runs **nine** `cargo test` steps where it runs **ten**.
The record contradicted *itself*: its § "Parent-session verification addendum"
already said *"the macOS coverage of the file rose from 12 to 14 tests"*.

## Requirement → evidence matrix

| Requirement | Evidence (`file:line`, command) | Result |
|---|---|---|
| **R-D1** — the changelog states **eight** differences, and names the eighth as a defect fix rather than a cost | `crates/eggup-service/CHANGELOG.md` § "Unreleased", the `**Changed: SystemExecutor's bounded runner now delegates to eggup_core::run_bounded.**` entry ("**Eight** differences are recorded … the **eighth is a defect fix, not a cost**"); enumerated against `architecture/service-lifecycle.md:648-655` (`grep -c '^\| \*\*D[0-9]\*\* \|' architecture/service-lifecycle.md` → `8`) | **Met.** The count is eight, and the enumeration now names all eight: D1+D2 (two stronger outcomes), D3 (timeout-detail normalisation), D4 (descendant-holds-the-pipe detail), D5 (dropped non-UTF-8 environment pair), D6 (unchanged), D7 (deep-corner kind change), D8 (the defect fix). |
| **R-D2** — the changelog carries a dedicated **D8** release note naming the mechanism, the symptom, the impact, and the unpublished status | `crates/eggup-service/CHANGELOG.md` § "Unreleased", the `- **Fixed: a successful stdin write is no longer reported as a stdin failure.**` entry | **Met.** It names `try_recv()` removing the message, the `Manager(Spawn, "manager stdin failed for {program}")` symptom, the `crontab -` impact ("the crate's only production stdin caller"), points at **D8**, and states that *"The published `eggup-service 0.1.3` still carries the defect"*. |
| **R-D3** — the changelog's compatibility call-out names **two** source-breaking changes, and the `engine_policy` entry carries a `**Consumer impact:**` note | `crates/eggup-service/CHANGELOG.md` § "Unreleased" preamble ("**Two** of them are source-breaking changes … or constructs `LifecycleUpdateReceipt` with a struct literal") and the `**Added: two named, separately testable commit-engine policy inputs.**` entry | **Met.** Both the preamble and the specific entry state it, and the entry names the field, its type, the absence of a `Default` impl, and that a consumer which only *reads* the fields is unaffected. |
| **R-D4** — the source-break claim is **true**, established against the published crate | `crates/eggup-service/src/lifecycle_update.rs:90` (`pub engine_policy: CommitEnginePolicy,`), `:66` (`pub struct LifecycleUpdateReceipt {`), `:65` (`#[derive(Debug)]`); `crates/eggup-service/src/commit_engine.rs:201` (`pub struct CommitEnginePolicy`); `grep -rn 'impl Default for LifecycleUpdateReceipt' crates/eggup-service/src/` → **no hits**; `git grep -n 'engine_policy' feb6ae5aea4c9b61c4051957f3662ca49d845f9e -- crates/eggup-service` → **no hits** | **Met.** The field is new since the published `0.1.3`; the receipt has no `Default` impl, so a struct literal is the only construction path; the published source carries no such field. |
| **R-D5** — the M015 closure's stale test count is corrected everywhere it appears | `plans/closure/service-lifecycle/015-status.md` § "Requirement → evidence matrix" row **R-B1** ("all fourteen tests"), § "Per-platform results" **macOS** row ("The fourteen `bounded_executor_consolidation` fixtures"), § "Files touched by this milestone" ("**New.** Fourteen tests"), § "Test inventory added by this milestone" heading ("— 14 tests:") | **Met.** `grep -c '^\s*#\[test\]' crates/eggup-service/tests/bounded_executor_consolidation.rs` → `14`. Every live count now reads **fourteen**; the historical `20ac770` counts are preserved as historical. |
| **R-D6** — the M015 closure's deleted test name is replaced everywhere it was cited as live evidence | `plans/closure/service-lifecycle/015-status.md` rows **R-C1** and **P7**; § "Test inventory added by this milestone" **T8** row (now three rows) | **Met.** `grep -rn 'the_managers_argv_is_byte_identical' crates/eggup-service/tests/` → **no hits**; the three current guards are at `bounded_executor_consolidation.rs:438` (`#[cfg(unix)]` at `:436`), `:610`, and `:762`. The two **historical** mentions — the quoted `20ac770` baseline output and the first hosted run's Windows failure — are left verbatim, as they are historical facts. |
| **R-D7** — the M015 closure's total test count is corrected | `plans/closure/service-lifecycle/015-status.md` § "Test inventory added by this milestone" ("**Count: 16 new tests** … **15 distinct integration tests**, plus **1** crate-private test") | **Met.** 14 attributes + 2 attributes = 16; `eggup_fixture_child` appears in **both** targets and is counted once → 15 distinct; plus `the_session_bus_filter_is_exactly_three_variables` (`crates/eggup-service/src/lib.rs:4268`) → **16**. The old figure of 15 counted the shared entry point twice and used the pre-split target count. |
| **R-D8** — the hosted `windows-check` lane's `cargo test` step count is corrected, and the steps are enumerated | `plans/closure/service-lifecycle/015-status.md` § "Parent-session verification addendum" (lane row and aggregate paragraph: "**ten**"), and the new § "The ten `windows-check` `cargo test` steps, enumerated" | **Met.** `awk 'NR>=116 && NR<=154' .github/workflows/ci.yml \| grep -c 'run: cargo test'` → `10`. The ten steps are enumerated with their line numbers (`:122`, `:123`, `:124`, `:125`, `:126`, `:127`, `:134`, `:138`, `:139`, `:142`); the lane's eleventh step is `cargo package` at `:154` and is not a test step. |
| **R-D9** — the M015 closure carries a **dated** corrective addendum recording the supersession, and its historical evidence is untouched | `plans/closure/service-lifecycle/015-status.md` § "Corrective addendum (2026-10-24) — test-inventory correction by Service M016" | **Met.** The addendum records how the gap went undetected, the ten corrections in a before/after table with their count commands, the ten CI steps, what it does **not** change, and the resolved `AGENTS.md` discrepancy. `git diff` shows no change inside § "Baseline evidence". |
| **R-D10** — **no production code, test code, manifest, lockfile, CI file, or architecture document changed** | `git status --short -- crates/eggup-service/ plans/implementation/service-lifecycle/ plans/closure/service-lifecycle/` | **Met.** Exactly three paths: `M crates/eggup-service/CHANGELOG.md`, `M plans/closure/service-lifecycle/015-status.md`, `?? plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md` (plus this closure record). |
| **R-D11** — the crate's behaviour is unchanged: the whole suite passes with identical per-target counts | `cargo test -p eggup-service --locked --offline` | **Met.** Exit 0. `112` (lib) + `1` + **`14`** + `2` + `3` + `4` + `8` + `5` + `2` + `2` + `14` + `0` + `5` + `1` + `0` + `2` (doc-tests) = **175 passed, 0 failed**. The consolidation target reports **14** and the environment target **2** — the corrected counts, observed natively. |
| **R-D12** — lint and formatting are clean, and the Windows target still compiles | `cargo clippy -p eggup-service --all-targets --locked --offline -- -D warnings`; `cargo fmt -p eggup-service -- --check`; `cargo check --target x86_64-pc-windows-msvc -p eggup-service --locked --offline` | **Met.** clippy exit 0, no warnings; fmt exit 0, no diff; the Windows check exit 0, no warnings. |
| **R-D13** — the published crate's defect is **not** misrepresented, and nothing is published | `crates/eggup-service/CHANGELOG.md` § "Unreleased" (the "Not published by this milestone" disclaimer is unchanged) and the new D8 entry; `git diff -- crates/eggup-service/CHANGELOG.md` | **Met.** The changelog's `## 0.1.2` and `## 0.1.3` sections are **byte-identical** — the diff touches only § "Unreleased". The D8 entry states the published `0.1.3` still carries the defect. M015 § UF-1 and registry finding **15** remain **open**. |
| **R-D14** — the registry and roadmap rows are ready for the parent, and the registry row obeys the 481-byte limit | this record's "Appendix — ready-to-paste rows" | **Met.** The registry row is **423 bytes** (`printf '%s' "$(cat row)" \| wc -c`), within the ≤ 481-byte limit of `plans/registry.md`. Neither `plans/registry.md` nor `plans/subsystems/**` was edited. |

## Production implementation evidence

**There is none, by construction.** This milestone writes no production code.

The complete set of changes is three files:

| File | Change |
|---|---|
| `crates/eggup-service/CHANGELOG.md` | § "Unreleased" only: the difference count corrected to **eight** and the enumeration completed; the new `- **Fixed: a successful stdin write is no longer reported as a stdin failure.**` entry (**D8**); the compatibility preamble's source-break count corrected to **two**; the `**Consumer impact:**` note on the `engine_policy` field. |
| `plans/closure/service-lifecycle/015-status.md` | Ten inventory corrections, plus § "Corrective addendum (2026-10-24) — test-inventory correction by Service M016". |
| `plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md` | **New.** This milestone's plan. |
| `plans/closure/service-lifecycle/016-status.md` | **New.** This record. |

The **production** behaviour of `eggup-service` is bit-identical to the baseline: the same `lib.rs`, the same `Cargo.toml`, the same `Cargo.lock`, the same tests. The 175-passing suite is the evidence, and the per-target counts match the M015 record's command 3 exactly apart from the consolidation target's 12 → 14, which is the split commit `b9885fa`'s doing and not M016's.

## Exact commands run and results

All commands were run from `/Users/davidbowman/projects/eggup` on the **macOS**
lane. Every cargo command is `--offline --locked`; there was **no network**
access.

| # | Command | Result |
|---|---|---|
| 1 | `cargo fmt -p eggup-service -- --check` | exit 0, **no diff** |
| 2 | `cargo clippy -p eggup-service --all-targets --locked --offline -- -D warnings` | exit 0, `Finished \`dev\` profile [unoptimized + debuginfo] target(s)`, **no warnings** |
| 3 | `cargo test -p eggup-service --locked --offline` | exit 0 — `112` (lib) + `1` + `14` + `2` + `3` + `4` + `8` + `5` + `2` + `2` + `14` + `0` + `5` + `1` + `0` + `2` (doc-tests) = **175 passed, 0 failed** |
| 4 | `cargo check --target x86_64-pc-windows-msvc -p eggup-service --locked --offline` | exit 0, **no warnings** |
| 5 | `grep -c '^\s*#\[test\]' crates/eggup-service/tests/bounded_executor_consolidation.rs` | **`14`** |
| 6 | `grep -c '^\s*#\[test\]' crates/eggup-service/tests/bounded_executor_environment.rs` | **`2`** |
| 7 | `grep -n -B2 '^fn the_\(systemd\|launchd\|cron\)_managers_argv_is_byte_identical' crates/eggup-service/tests/bounded_executor_consolidation.rs` | `:436` `#[cfg(unix)]`, `:437` `#[test]`, `:438` `fn the_systemd_managers_argv_is_byte_identical`; `:609` `#[test]`, `:610` `fn the_launchd_managers_argv_is_byte_identical`; `:761` `#[test]`, `:762` `fn the_cron_managers_argv_is_byte_identical` |
| 8 | `grep -rn 'the_managers_argv_is_byte_identical' crates/eggup-service/tests/` | **no hits** — the deleted name is gone from the tree |
| 9 | `awk 'NR>=116 && NR<=154' .github/workflows/ci.yml \| grep -c 'run: cargo test'` | **`10`** |
| 10 | `grep -n 'fn the_session_bus_filter_is_exactly_three_variables' crates/eggup-service/src/lib.rs` | `4268` |
| 11a | `grep -n 'pub engine_policy' crates/eggup-service/src/lifecycle_update.rs` | `90:    pub engine_policy: CommitEnginePolicy,` |
| 11b | `grep -rn 'impl Default for LifecycleUpdateReceipt' crates/eggup-service/src/` | **no hits** — a struct literal is the only construction path |
| 12 | `git grep -n 'engine_policy' feb6ae5aea4c9b61c4051957f3662ca49d845f9e -- crates/eggup-service` | **no hits** — the published `0.1.3` does not carry the field |
| 13 | `grep -c '^\| \*\*D[0-9]\*\* \|' architecture/service-lifecycle.md` | **`8`** — D1–D8 |
| 14 | `git status --short` | this milestone's three paths, plus six concurrent modifications by other agents (`.opencode/skills/{docs-hygiene,release-workflow,verify-workflow}/SKILL.md`, `AGENTS.md`, `docs/crates.md`, `plans/registry.md`) — **none of them touched by M016** |

The **workspace-wide** gate (`./scripts/check-local.sh`) is the **parent's**
step and was **not run** here: it covers crates this milestone does not own,
and other agents were editing the tree concurrently.

### Per-platform results — reported individually, never inferred

| Platform | What was run | Result |
|---|---|---|
| **macOS** (this machine: Darwin 25.6.0 arm64; `rustc 1.89.0`, `cargo 1.89.0`) | commands 1–14 above, including the whole `eggup-service` suite | **All green. 0 failed.** The **fourteen** `bounded_executor_consolidation` fixtures, the **two** `bounded_executor_environment` fixtures, the 112 lib tests, the M014 public-API guard, and the `manager_failure_kinds` taxonomy all pass. **175 passed in total.** The corrected counts were observed natively, not asserted. |
| **Linux** | nothing | **Not run.** No Linux host was available in this session. **No Linux result is inferred from the macOS result.** The hosted `stable`, `msrv`, and `service-systemd` lanes are the parent's qualification step. |
| **Windows** | `cargo check --target x86_64-pc-windows-msvc -p eggup-service --locked --offline` (command 4) | **Compile-only, and warning-free.** The Windows **suite was not executed** — no Windows host was available, and a cross-compile check does **not** execute a single test. The hosted `windows-check` lane is the parent's qualification step, and this record does **not** predict its result. Note that the lane now runs **ten** `cargo test` steps (command 9), one of which is the live SCM fixture added in `b9885fa`. |

## Invariant review

| # | Invariant | Evidence |
|---|---|---|
| 1 | No production behaviour changes | `git status --short` (command 14); the 175-passing suite (command 3) with identical per-target counts |
| 2 | No public API change | `cargo test -p eggup-service --locked --offline --test public_api_surface` → `5 passed; 0 failed` (part of command 3); `crates/eggup-service/tests/public-api-surface.txt` untouched |
| 3 | No test change | `git status --short` lists no path under `crates/eggup-service/tests/`; command 8 confirms the deleted name is absent and commands 5–7 confirm the current names |
| 4 | No manifest or lockfile change | `git status --short` lists neither `crates/eggup-service/Cargo.toml` nor `Cargo.lock`; no dependency was added |
| 5 | No published version section touched | `git diff -- crates/eggup-service/CHANGELOG.md` — only § "Unreleased" hunks |
| 6 | The published crate's defect is not misrepresented | the new D8 entry: *"The published `eggup-service 0.1.3` still carries the defect"*; M015 § UF-1 and registry finding **15** remain open |
| 7 | No historical evidence rewritten | `git diff` shows no change inside M015 § "Baseline evidence"; the addendum's § "What this addendum does **not** change" states the boundary explicitly |
| 8 | No git state mutated | no `stash`, `checkout`, `restore`, `commit`, `push`, `tag`, `clean`, `reset`, or branch creation was performed; the published-source evidence used read-only `git show` / `git grep` |
| 9 | The parent's surfaces are untouched | `git status --short` shows `plans/registry.md` modified, but **not by this milestone** — it was already modified by another agent before and during this work; `plans/subsystems/**` is not listed at all |

## Failure / rollback / recovery review

- **Not applicable, and stated rather than left blank.** This milestone mutates
  **no artifact, spawns no process, starts no service, and takes no lock**.
  There is no failure, cancellation, restart, or contention semantics to define
  for a documentation edit; inventing any would misrepresent the work.
- **A failed edit fails rather than approximating.** Every change is an
  exact-string replacement against the file's current content; a replacement
  that does not match exactly is rejected and nothing is written. (One such
  rejection occurred during implementation — a mis-indented `old_string` — and
  the file was re-read and the edit re-derived, with no partial write.)
- **The counts are the guard.** A milestone that writes an untrue number into a
  closure record is caught by the counting commands (§ "Exact commands run"),
  which derive each number from the tree rather than restating it.
- **No rollback mechanism is claimed.** The crate has no crash-journal for
  documentation and claims none.
- **The published crate is unaffected by any failure here.**
  `eggup-service 0.1.3` is already on crates.io with its defect; nothing this
  milestone does can change that, and nothing it fails to do can make it worse.

## Compatibility and migration review

- **No consumer-visible change.** No code, no API, no behaviour.
- **What a consumer now learns from the changelog that they did not before:**
  1. **D8** — that the source line fixes the misreported stdin write, and that
     the **published `0.1.3` still has the defect**. This is the release note
     the republication milestone must carry.
  2. **The `engine_policy` source break** — that `LifecycleUpdateReceipt` gained
     a public field and that a consumer constructing it with a struct literal
     must add the field.
- **Both remain unpublished.** The § "Unreleased" disclaimers stand:
  `eggup-service 0.1.3` remains the published version, `v0.1.3` is unmoved, and
  republication is a separate, later milestone. Neither migration becomes
  necessary until republication.
- **No dependency change, so no downstream pin cascade.** `eggup-eggpack`'s
  exact pins are untouched.
- **M015's requirement outcomes are unchanged.** Every requirement in M015's
  matrix was met and still is; only the *inventory's* counts and *cited test
  names* were wrong. This is a record correction, not a behaviour change.

## Security review

- **No new privilege, no new filesystem authority, no new network surface, no
  new dependency.** The milestone writes two text files and creates two
  planning documents.
- **No service, subprocess, or system state is touched.** The crate's
  production code is byte-identical, so the service's own security properties
  — the executable allowlist, the cleared child environment, the bounded
  stdin, the bounded output, the unconditional deadline — are untouched by
  construction.
- **No secret is introduced.** The changelog text quotes only an error-detail
  *shape* (`manager stdin failed for {program}`) and a path-free program name.
- **The D8 note is a security-positive disclosure.** The misreported stdin
  write was **fail-closed**, so no state was corrupted and no privilege was
  escalated; stating it plainly is what lets a consumer decide to upgrade.

## Documentation / operations evidence

| Document | What changed |
|---|---|
| `crates/eggup-service/CHANGELOG.md` | § "Unreleased" only: the difference count → **eight** and the enumeration completed; the new **D8** `Fixed` entry; the compatibility preamble's source-break count → **two**; the `**Consumer impact:**` note on `engine_policy`. **No published version section touched.** |
| `plans/closure/service-lifecycle/015-status.md` | Ten inventory corrections (rows **R-C1**, **P7**, **R-B1**; the **macOS** platform row; the "Files touched" row; the inventory heading and its **T8** row; the count; the two `windows-check` step counts) plus § "Corrective addendum (2026-10-24)". |
| `plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md` | **New.** The plan. |
| `plans/closure/service-lifecycle/016-status.md` | **New.** This record. |
| `crates/eggup-service/README.md` | **Not edited.** It states no difference count and no source-break claim, so it is already truthful; read-only evidence. |
| `architecture/service-lifecycle.md` | **Not edited.** It is the norm for D1–D8 and is correct; the changelog is what drifted. |
| `plans/registry.md`, `plans/subsystems/service-lifecycle-roadmap.md` | **Not edited by this milestone** — the parent owns them; the rows are drafted in the appendix below. |
| root `CHANGELOG.md`, `AGENTS.md`, `docs/**`, `.opencode/skills/**`, `.github/**`, `scripts/**`, every other crate | **Not edited.** `AGENTS.md`'s "nine `cargo test` steps" was already corrected to "ten" in the working tree by a concurrent agent, so the two files agree in the final tree. |

### Ownership verification

- `crates/eggup-service/src/**` — **not modified.**
- `crates/eggup-service/tests/**` — **not modified.** `tests/public-api-surface.txt`
  is owned by another agent and was not touched; the public-API guard passes with
  its snapshot unmodified.
- `crates/eggup-service/Cargo.toml`, `Cargo.lock` — **not modified.**
- `crates/eggup-service/README.md` — **not modified.**
- `architecture/**`, `docs/**`, `AGENTS.md`, root `CHANGELOG.md`,
  `.opencode/skills/**`, `.github/**`, `scripts/**`, every other crate — **not
  modified.**
- `plans/registry.md`, `plans/subsystems/**` — **not modified.** `plans/registry.md`
  shows as modified in `git status`, but that modification is **another agent's**,
  pre-existing and concurrent with this milestone.
- **Owned and modified:** `crates/eggup-service/CHANGELOG.md`,
  `plans/closure/service-lifecycle/015-status.md`, and the two new M016
  documents.
- **No git state was mutated**: no `stash`, `checkout`, `restore`, `commit`,
  `push`, `tag`, `clean`, `reset`, or branch creation.

## Unresolved findings with severity

### UF-1 — The published `eggup-service 0.1.3` still reports a successful stdin write as a stdin failure — **medium — OPEN (published exposure, carried forward unchanged)**

**Finding.** Unchanged from M015 § UF-1 and registry finding **15**. The
published crate's bounded runner polls its stdin-writer channel with
`try_recv()`, which **removes** the message it returns, so the verdict `Ok(())`
is consumed and discarded and a **successful** write is reported as
`Manager(Spawn, "manager stdin failed for {program}")`. The crate's only
production stdin caller is the `crontab -` install.

**What M016 did and did not do.** M016 adds the **release note** for the fix —
which is exactly what M015 § UF-1 says the republication milestone *"must
carry … and state it in its notes"* — and it corrects the difference count in
the same changelog entry. It does **not** discharge the defect: the published
crate is unchanged, and the fix is on the source line only.

**Severity: medium, unchanged.** It is not high: it fails closed, it is not a
safety or privilege issue, and the affected crate has no consumer. It is not
low: it is a **functional defect in a published crate's only stdin path**,
reproducible with a two-line probe. Under the repository's rule that
*"medium-or-higher is the de-facto blocking-severity gate"*, this residual is
the parent's decision, exactly as M015 left it: the **republication milestone**
is the corrective that discharges it.

### UF-2 — The `engine_policy` source break is stated but not yet exercised by a consumer — **low — accepted**

**Finding.** M016 states the break in the changelog; nothing else is needed for
truthfulness. The observation that the published `0.1.3` has **no consumer yet**
(the Eggup product itself is not released) is unchanged, so no consumer is
currently broken by it.

**Severity: low.** It is a source-visible addition to a public struct in a
published crate with no consumer, and the migration is a one-line field
addition. **Accepted**, not deferred to a corrective.

### Adjacent findings (recorded, deliberately not fixed here)

- **The `windows-check` lane's `AGENTS.md` discrepancy is resolved in the
  working tree.** `AGENTS.md` § "Verify" said the lane runs "nine `cargo test`
  steps"; it runs **ten** (command 9). The working tree's `AGENTS.md` now reads
  "ten", corrected by a concurrent agent while this milestone was in flight.
  `AGENTS.md` is **not owned by Service M016** and was **not edited** by it; the
  correction is recorded in M015's addendum so the two files agree in the final
  tree.
- **The `crates/eggup-service/tests/published-api-0.1.2/` and
  `published-api-0.1.3/`** fixtures pass unchanged and were not edited.

## Roadmap disposition

**The parent owns `plans/subsystems/service-lifecycle-roadmap.md`.** The
intended row for the milestone-status table (§ 12 of the roadmap), drafted
from the plan's §16 and **not applied** here, is in the appendix below.

The roadmap's header status line advances from *"M001–M015 closed"* to
*"M001–M016 closed"*, and the registry's "Active subsystem roadmaps" row for
**Service lifecycle** stays *"no further service-lifecycle milestone is
registered"* — unless the parent registers the republication milestone that
**UF-1** still needs.

## Registry updates

**The parent owns `plans/registry.md`.** The intended row for §
"Dependency-ready implementation work", drafted from the plan's §16 and **not
applied** here, is in the appendix below.

**Finding 15 stays open.** M016 records the fix's release note; it does not
discharge the published defect, so the registry's "Post-0.1.3 open findings"
entry 15 needs **no** change beyond the note in the appendix.

## Appendix — ready-to-paste rows

### Registry row — `plans/registry.md` § "Dependency-ready implementation work"

**423 bytes**, within the ≤ 481-byte limit. Paste as one line:

```
| Service lifecycle | M016 published-change release-note corrective | closed 2026-10-24 | `plans/closure/service-lifecycle/016-status.md` | Adds the missing **D8** release note — the fix for the published crate's stdin-verdict defect — and corrects the difference count to eight. Flags the unflagged `engine_policy` source break. Corrects the M015 closure's stale test inventory. Documentation only; nothing published |
```

### Roadmap row — `plans/subsystems/service-lifecycle-roadmap.md` § 12 "Milestone status"

```
| M016 | closed 2026-10-24 | `plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md` | `plans/closure/service-lifecycle/016-status.md` | Documentation-only corrective. The crate's `CHANGELOG.md` omitted the release note for **D8** — the fix for the published crate's stdin-verdict defect (registry finding **15**) — and stated "Seven differences" where `architecture/service-lifecycle.md` records **eight**. The **D8** release note is added and the count is corrected. The changelog's compatibility call-out named only one source-breaking change; the unflagged `LifecycleUpdateReceipt::engine_policy` field addition — which stops a consumer that constructs the receipt with a struct literal from compiling — is now stated in the entry that introduces the field. The M015 closure's test inventory is corrected: **14** tests in `bounded_executor_consolidation.rs` (not 12), the three current argv guards replace the deleted `the_managers_argv_is_byte_identical`, the total is **16** new tests (not 15), and the hosted `windows-check` lane runs **ten** `cargo test` steps (not nine). No production code, test code, manifest, lockfile, CI file, or architecture document changed; nothing published; `v0.1.3` unmoved. Registry finding **15** stays open for the published crate until the republication milestone |
```

### Note for the registry's "Post-0.1.3 open findings" list — **no change required**

Entry **15** (the published crate's stdin-verdict defect, **medium**) remains
**open**. M016 adds the release note for its fix to the crate's changelog; the
defect itself is discharged only by the republication milestone. If the parent
wants the finding's text to point at the release note, the optional addition is:

> The release note for this defect's fix is now recorded in the crate's own
> changelog (`crates/eggup-service/CHANGELOG.md` § "Unreleased", the
> "**Fixed: a successful stdin write is no longer reported as a stdin
> failure.**" entry) as difference **D8**. The published crate is unchanged;
> the fix reaches consumers at the next republication.
