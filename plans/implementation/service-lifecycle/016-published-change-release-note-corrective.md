# Service Lifecycle Milestone 016 — Published-change release-note corrective

Status: implemented

Repository baseline: `bf439f519300354ac14823e74acd48c7995033a6`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Finding of record:

- `plans/closure/service-lifecycle/015-status.md` § "Corrective addendum (2026-10-24) — test-inventory correction by Service M016" — the M015 closure's test inventory was stale and self-contradictory.
- `plans/registry.md` § "Post-0.1.3 open findings" — **finding 15** (**medium**): the published `eggup-service 0.1.3` reports a *successful* stdin write as a stdin failure. The source-line fix is **D8**; the release note for it was absent from the crate's own changelog.

Prerequisite milestone (not owned by this plan):

- `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md` — Service M015, closed and hosted-qualified in run `40062299630` (commit `b9885fa`). M016 corrects M015's **records**; it does not re-implement M015.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: polish

> **Classification note.** M016 is classified `polish` because it adds **no capability**, creates **no new code path**, and changes **no public item**. It is a **documentation-correctness** corrective: the crate's own release notes and the milestone records that certify them did not say what the code does. Its acceptance is nevertheless stated as a **truthfulness invariant** — a published-change release note must state the change, the source break it causes (if any), and the fact that it is unpublished, and a closure record's inventory must match the tree it certifies.

## 1. Objective

1. **The release note for D8 exists in the crate's own changelog.** The consolidation in `architecture/service-lifecycle.md` § "Behaviour differences of the consolidation" records **eight** behaviour differences, **D1–D8**. The eighth, **D8**, is not a cost: it is a **defect in the runner the consolidation replaced**, which the consolidation fixes — the fix for the stdin-verdict defect in the **published** `eggup-service 0.1.3` (registry finding **15**). `crates/eggup-service/CHANGELOG.md` stated *"Seven differences are recorded"* and enumerated only six, so **the release note for a fix of a known published defect was entirely absent from the crate's own changelog**. This milestone adds it and makes the count consistent with the architecture.
2. **The one unflagged source break is flagged.** `LifecycleUpdateReceipt` gained a public field `engine_policy: CommitEnginePolicy`. The struct has **no `Default` impl**, so a struct literal is its only construction path; a consumer of the published `0.1.3` that constructs the struct with a literal therefore **stops compiling**. The changelog's compatibility call-out names **one** source-breaking change (the typed manager failure kinds) and is **silent** about this one. This milestone states it, in the entry that introduces the field.
3. **The M015 closure's test inventory is corrected.** `plans/closure/service-lifecycle/015-status.md` states a test count, a test name, and a CI step count that do not match the tree it certifies, and it contradicts itself about the test count. This milestone corrects every one of them and adds a dated addendum recording the supersession.

## 2. Why this milestone is ready

- **The defects are recorded, not detected.** All three were found by reading the records against the tree, not by a failing test. No production report exists and none is claimed.
- **D1 is a publication-truthfulness defect, and it is the highest-value of the three.** The repository's own rule is that a crate's changelog is the record a consumer reads before upgrading. A fix that turns a **failure into a success** on the crate's only production stdin path (`crontab -`) is exactly the kind of change a consumer must be told about. Its absence from the changelog means the published-crate defect described in **UF-1** and registry finding **15** has **no release note at all**, so the republication milestone — which must "carry the fix and state it in its notes" (`plans/closure/service-lifecycle/015-status.md` § UF-1) — has nothing to carry forward.
- **D2 is a publication-truthfulness defect for a different consumer shape.** The M012 closure already records the field as a *"Source-visible addition. A consumer that destructures the receipt **exhaustively** (without `..`) must add the field"* (`plans/closure/service-lifecycle/012-status.md`, § "Compatibility and migration"). The **closure** knows; the **changelog** does not. A consumer reading only the changelog is not warned.
- **D3 is a record-correctness defect that has already produced one self-contradiction.** Commit `b9885fa` split the single argv guard into three and added a step to the hosted `windows-check` lane. The M015 closure's inventory sections were written at baseline `20ac770` and never reconciled. The record's own § "Parent-session verification addendum" states the new count — *"the macOS coverage of the file rose from 12 to 14 tests"* — which is what makes the earlier sections contradict it rather than merely lag it.
- **Scope is bounded to documentation.** Only `crates/eggup-service/CHANGELOG.md` and `plans/{implementation,closure}/service-lifecycle/**` are touched. No production code, no test code, no manifest, no lockfile, no CI file, no architecture document.
- **Severity, stated explicitly** per the planning-workflow severity gate. **D1** is **medium**: it is the release-note omission for a fix of a known **published** defect, and the repository's rule is that *"medium-or-higher is the de-facto blocking-severity gate"*. **D2** is **medium**: it is an unflagged source break against a published crate. **D3** is **low**: it is a documentation-only defect in a closure record, with no behaviour and no requirement outcome changed. Each is recorded with its reason, not assumed on a shared scale.

## 3. Current implementation evidence

All anchors below are at the baseline SHA `bf439f519300354ac14823e74acd48c7995033a6`.

### 3.1 D1 — the architecture records eight differences; the changelog recorded six

`architecture/service-lifecycle.md`, § "Behaviour differences of the consolidation" (`:636-655`):

| # | Difference | Direction | Row |
|---|---|---|---|
| D1 | A mid-stream reader error is now fail-closed | Fail-closed (stronger) | `:648` |
| D2 | Core kills the child on an output overflow | Strictly safer | `:649` |
| D3 | The timeout detail text is normalised | Detail text only | `:650` |
| D4 | A descendant holding the pipe open ends at the deadline with the timeout detail | Same kind, different detail text | `:651` |
| D5 | A non-UTF-8 environment pair is dropped | Narrowing, fail-closed | `:652` |
| D6 | The spawn-failure `NotFound` detail names the resolved program | No change | `:653` |
| D7 | A stdin writer still blocked at the deadline is reported as a timeout | Kind change, in a deep corner | `:654` |
| **D8** | **A successful stdin write is no longer reported as a stdin failure** | **Defect fixed** (a failure becomes a success) | `:655` |

The section's own prose at `:642-644` states: *"The eighth, **D8**, is not a cost: it is a **defect in the runner this consolidation replaced**, which the consolidation fixes."*

`crates/eggup-service/CHANGELOG.md` § "Unreleased", the consolidation entry, at the baseline stated:

> Seven differences are recorded in `architecture/service-lifecycle.md` § "Behaviour differences of the consolidation": two are stronger (a reader error and an output overflow now fail closed, and an overflowing child is now killed and reaped), one normalises the timeout detail text, one drops a non-UTF-8 environment pair rather than passing it through lossily, one is unchanged, and one changes the kind in a deep corner that has no portable fixture.

The sentence says **seven** and enumerates **six** (D1+D2 grouped, D3, D5, D6, D7). **D4 and D8 are both absent**, and the count is wrong by two.

### 3.2 D2 — the `engine_policy` field is a source break for a literal-constructing consumer

| Item | Site | Evidence |
|---|---|---|
| The field | `crates/eggup-service/src/lifecycle_update.rs:90` | `pub engine_policy: CommitEnginePolicy,` |
| The struct | `crates/eggup-service/src/lifecycle_update.rs:66` | `pub struct LifecycleUpdateReceipt {` |
| Its derives | `crates/eggup-service/src/lifecycle_update.rs:65` | `#[derive(Debug)]` — **no `Default`** |
| The field's type | `crates/eggup-service/src/commit_engine.rs:201` | `pub struct CommitEnginePolicy {` with `#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]` at `:200` |
| The published `0.1.3` struct | `git show feb6ae5…:crates/eggup-service/src/lifecycle_update.rs` → `:159-178` | **nine** fields, **no `engine_policy`** |

`grep -rn 'impl Default for LifecycleUpdateReceipt' crates/eggup-service/src/` returns **no hits**, and the only `#[derive(Default)]` in `lifecycle_update.rs` (`:466`) belongs to the test-only `ScriptedManager`. So an external consumer's **only** construction path for `LifecycleUpdateReceipt` is a struct literal that must now list a tenth field. `git grep -n 'engine_policy' feb6ae5… -- crates/eggup-service` returns **no hits**, confirming the field is new since the published `0.1.3`.

The changelog's compatibility preamble at the baseline named **one** source-breaking change:

> One of them — the typed manager failure kinds — is a source-breaking change for a consumer that matches `ServiceError::Manager` positionally or branches on its message text; that migration becomes necessary only at republication, and the exact replacement is named in its entry below.

The `**Added: two named, separately testable commit-engine policy inputs.**` entry mentioned `engine_policy` only as *"observable in the returned receipt's `engine_policy` field"*, with **no** source-break call-out.

### 3.3 D3 — the M015 closure's inventory does not match the tree

| # | Site in `plans/closure/service-lifecycle/015-status.md` | Stated | Actual | Count command |
|---|---|---|---|---|
| 1 | `:693` — inventory heading | 12 tests | **14** | `grep -c '^\s*#\[test\]' crates/eggup-service/tests/bounded_executor_consolidation.rs` |
| 2 | `:707` — the **T8** row | `the_managers_argv_is_byte_identical` | **deleted in `b9885fa`**, split into three | `grep -n -B2 '^fn the_\(systemd\|launchd\|cron\)_managers_argv_is_byte_identical' …` |
| 3 | `:134` — row **R-C1** | cites the deleted test as live evidence | three current tests | `cargo test -p eggup-service --locked --offline` |
| 4 | `:200` — row **P7** | cites the deleted test | three current tests | same |
| 5 | `:131` — row **R-B1** | "all twelve tests … pass" | **fourteen** | same `grep -c` |
| 6 | `:350` — **macOS** platform row | "The twelve … fixtures" | **fourteen** | same `grep -c` |
| 7 | `:350` — "Files touched" | "**New.** Twelve tests." | **Fourteen** | same `grep -c` |
| 8 | `:723` — the count | "**Count: 15 new tests** (13 in the two new integration targets, 1 in the fixture module's shared helper … and 1 crate-private)" | **16** (15 distinct integration tests + 1 crate-private) | `grep -c '^\s*#\[test\]'` on both targets → `14` and `2`; the shared `eggup_fixture_child` entry point appears in **both** |
| 9 | `:736` — `windows-check` lane row | "nine targeted `cargo test` steps" | **ten** | `awk 'NR>=116 && NR<=154' .github/workflows/ci.yml \| grep -c 'run: cargo test'` → `10` |
| 10 | `:740` — aggregate paragraph | "nine separate `cargo test` steps" | **ten** | same |

The current test names and their anchors:

| Test | `crates/eggup-service/tests/bounded_executor_consolidation.rs` | Gate |
|---|---|---|
| `the_systemd_managers_argv_is_byte_identical` | `:438` | `#[cfg(unix)]` at `:436`, `#[test]` at `:437` |
| `the_launchd_managers_argv_is_byte_identical` | `:610` | none — platform-neutral |
| `the_cron_managers_argv_is_byte_identical` | `:762` | none — platform-neutral |

The `windows-check` lane's `cargo test` steps are at `.github/workflows/ci.yml:122`, `:123`, `:124`, `:125`, `:126`, `:127`, `:134`, `:138`, `:139`, `:142` — **ten**. The lane's eleventh step, `cargo package -p eggup-service --locked --no-verify` at `:154`, is a packaging step and not a `cargo test` step.

## 4. Invariants that must not regress

1. **No production behaviour changes.** No file under `crates/eggup-service/src/**` is modified. `cargo test -p eggup-service --locked --offline` must produce the identical per-target result set it produced before this milestone.
2. **No public API change.** No item is added, removed, or re-signatured. `crates/eggup-service/tests/public-api-surface.txt` stays byte-identical and its guard still passes.
3. **No test change.** No file under `crates/eggup-service/tests/**` is modified; the test names, counts, and gates that this milestone *records* are the ones that already exist.
4. **No manifest or lockfile change.** `crates/eggup-service/Cargo.toml` and `Cargo.lock` stay byte-identical; no new dependency.
5. **No published version section is touched.** `crates/eggup-service/CHANGELOG.md` § "0.1.3" and § "0.1.2" stay byte-identical. This milestone edits **only** the § "Unreleased" section. `eggup-service` remains published at `0.1.3`; `v0.1.3` is unmoved; **nothing is published**.
6. **The published crate's defect is not misrepresented.** The changelog must state, in the D8 entry, that the **published `0.1.3` still carries the defect** and that the fix reaches consumers at the next republication. The M015 closure's **UF-1** and registry finding **15** stay **open**.
7. **No historical evidence is rewritten.** The M015 closure's baseline evidence, its quoted `20ac770` test output, and its record of the first hosted run's Windows failure are **historical facts** and are left verbatim. Only the **live** inventory claims are corrected.
8. **No git state is mutated.** No `stash`, `checkout`, `restore`, `commit`, `push`, `tag`, `clean`, `reset`, or branch creation. The parent owns all commits.
9. **The parent owns the shared control surfaces.** `plans/registry.md`, `plans/subsystems/**`, the root `CHANGELOG.md`, `AGENTS.md`, `architecture/**`, `docs/**`, `.opencode/skills/**`, `.github/**`, and `scripts/**` are **not edited** by this milestone. Ready-to-paste rows are drafted in the closure record instead.

## 5. Scope

### In scope

- `crates/eggup-service/CHANGELOG.md` — § "Unreleased" only.
- `plans/closure/service-lifecycle/015-status.md` — the stale inventory, corrected, plus a dated corrective addendum.
- `plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md` — this plan.
- `plans/closure/service-lifecycle/016-status.md` — this milestone's closure record.

### Explicitly out of scope

- **Any production code.** `crates/eggup-service/src/**` is read-only here.
- **Any test code.** `crates/eggup-service/tests/**` is read-only here. Another agent owns `crates/eggup-service/tests/public-api-surface.txt`; this milestone does not touch it.
- **The published crate.** No version bump, no `cargo publish`, no tag movement, no republication milestone. The published `0.1.3` keeps its defect; this milestone only **records** that.
- **`architecture/service-lifecycle.md`.** It is the *source of truth* for D1–D8 and is **correct**; the changelog is what drifted. Editing the architecture document would be editing the norm.
- **`crates/eggup-service/README.md`.** It makes no difference-count and no source-break claim, so it needs no change; it is read-only evidence.
- **The registry and the roadmap.** `plans/registry.md` and `plans/subsystems/service-lifecycle-roadmap.md` are the parent's. Ready-to-paste rows go in this milestone's closure record, in an appendix.
- **The root `CHANGELOG.md`, `AGENTS.md`, `docs/**`, `.opencode/skills/**`, `.github/**`, `scripts/**`, and every other crate.**
- **The full workspace gate.** `./scripts/check-local.sh` covers crates this milestone does not own, and other agents were editing the tree concurrently. The parent runs it.

## 6. Required production changes

**None.** This milestone is documentation-only for the crate. No `src/` file is modified, no public item changes, no behaviour changes.

If implementing this milestone reveals that a **code** change is required — for example, if the `engine_policy` field turned out to be constructible in a way that is *not* source-breaking, or if a D8 release note could not be written truthfully without one — **stop and report it** rather than making it.

## 7. Ordered work packages

1. **WP1 — D1: the D8 release note and the corrected difference count.**
   1. In `crates/eggup-service/CHANGELOG.md` § "Unreleased", in the `**Changed: SystemExecutor's bounded runner now delegates to eggup_core::run_bounded.**` entry: change *"Seven differences"* to *"**Eight** differences"*, and complete the enumeration so that all eight are named — D1 and D2 grouped as the two stronger outcomes, D3 the timeout-detail normalisation, D4 the descendant-holds-the-pipe detail change, D5 the dropped non-UTF-8 environment pair, D6 the unchanged spawn-failure detail, D7 the deep-corner kind change, and the eighth stated as **a defect fix, not a cost**.
   2. Add a dedicated `- **Fixed: a successful stdin write is no longer reported as a stdin failure.**` entry immediately after the consolidation entry, carrying the `try_recv()` mechanism, the `Manager(Spawn, "manager stdin failed for {program}")` symptom, the `crontab -` impact, the **D8** pointer, and the statement that the **published `0.1.3` still carries the defect**.
2. **WP2 — D2: the source-break note.**
   1. In the § "Unreleased" compatibility preamble, change *"One of them — the typed manager failure kinds —"* to *"**Two** of them"*, and extend the list of affected consumer shapes to include *"constructs `LifecycleUpdateReceipt` with a struct literal"*.
   2. In the `**Added: two named, separately testable commit-engine policy inputs.**` entry, add a `**Consumer impact:**` note naming the new public field `engine_policy` (`CommitEnginePolicy`), stating that the struct has **no `Default` impl** so a struct literal is its only construction path, and stating that a consumer that only *reads* the receipt's fields is unaffected.
3. **WP3 — D3: the M015 closure inventory corrections.** Apply the ten corrections in §3.3's table, each with the count command that establishes the corrected value.
4. **WP4 — the M015 closure's dated corrective addendum.** Add § "Corrective addendum (2026-10-24) — test-inventory correction by Service M016" at the end of `plans/closure/service-lifecycle/015-status.md`, recording: how the gap went undetected; the ten corrections in a before/after table with their count commands; the enumeration of the ten `windows-check` `cargo test` steps; what the addendum does **not** change (the historical baseline evidence, the historical `12` in command 3, the historical Windows failure); and the `AGENTS.md` "nine steps" discrepancy, flagged for the parent because `AGENTS.md` is not owned here.
5. **WP5 — verification.** Run §11's commands and record the results.
6. **WP6 — the closure record.** Write `plans/closure/service-lifecycle/016-status.md` with the requirement → evidence matrix, the exact commands and their observed results, the per-platform results reported separately, and the **appendix** carrying the ready-to-paste registry row and roadmap row.

## 8. Failure, cancellation, restart, and contention semantics

**Not applicable, and stated rather than left blank.** This milestone mutates **no artifact, spawns no process, starts no service, and takes no lock**. There is no failure, cancellation, restart, or contention semantics to define for a documentation edit, and inventing any would misrepresent the work.

What *is* stated, because it is the only failure surface that exists:

- **A failed edit is a failed edit.** No partial write is possible: each change is an exact-string replacement against the file's current content, and a replacement that does not match exactly fails rather than writing something approximate.
- **Verification is the guard.** A milestone that writes an untrue number into a closure record is caught by §11's commands, which count the attributes and steps from the tree rather than restating them. The M016 closure record carries those counts as its own evidence.
- **No rollback mechanism is claimed.** This crate has no crash-journal for documentation and claims none.
- **The published crate is unaffected by any failure here.** `eggup-service 0.1.3` is already on crates.io with its defect; nothing this milestone does can change that, and nothing it fails to do can make it worse.

## 9. Compatibility and migration

- **No consumer-visible change.** No code, no API, no behaviour. The M016 edits are to a changelog and to two planning records.
- **What the changelog now tells a consumer that it did not before.** Two facts, both about **unpublished** source-line changes:
  1. **D8** — that the source line fixes a defect in which a *successful* stdin write was reported as a failure, and that the **published `0.1.3` still has the defect**. This is the release note the republication milestone needs to carry (M015 § UF-1: *"requires the republication milestone, which must carry the fix and state it in its notes"*).
  2. **The `engine_policy` source break** — that `LifecycleUpdateReceipt` gained a public field and that a consumer constructing it with a struct literal must add the field. The M012 closure already recorded this (`plans/closure/service-lifecycle/012-status.md`, § "Compatibility and migration"): *"**Source-visible addition.** A consumer that destructures the receipt **exhaustively** (without `..`) must add the field."* The changelog now says it too.
- **Both remain unpublished.** The § "Unreleased" section's existing disclaimers stand: `eggup-service 0.1.3` remains the published version, the `v0.1.3` tag is unmoved, and republication is a separate, later milestone. Neither migration becomes necessary until republication.
- **No dependency change, so no downstream pin cascade.** `eggup-eggpack`'s exact pins are untouched.
- **The M015 closure's requirement outcomes are unchanged.** Every requirement in M015's matrix was met and still is; only the *inventory's* counts and *cited test names* were wrong. The M016 closure record states this explicitly so a reader does not mistake a record correction for a behaviour change.

## 10. Required tests

**No new test is required, and none may be added.** This milestone changes no code path, so there is no behaviour to pin. The milestone's acceptance is stated in terms of **truthfulness against the tree**, and the evidence is the **counting commands** in §11 plus the existing suite's green result.

Specifically:

1. The existing `eggup-service` suite must still pass, unchanged, at the same per-target counts — in particular `tests/bounded_executor_consolidation.rs` at **14 passed** and `tests/bounded_executor_environment.rs` at **2 passed**. This is simultaneously the **regression check** for invariant 1 and the **evidence** for the corrected counts.
2. The corrected inventory numbers must be **derivable** from the tree by the exact commands recorded in the M016 closure's requirement → evidence matrix, so a reviewer can re-derive each number rather than trust it.
3. The corrected CI step count must be **derivable** from `.github/workflows/ci.yml` by the recorded command, and the ten steps must be **enumerated** in the M015 addendum with their line numbers, so the number is checkable rather than asserted.

## 11. Required verification commands

Run from the repository root. Every cargo command is `--offline --locked`; there is no network.

```sh
# 1 — formatting
cargo fmt -p eggup-service -- --check

# 2 — lint, all targets, warnings denied
cargo clippy -p eggup-service --all-targets --locked --offline -- -D warnings

# 3 — the crate's whole suite (regression check for invariant 1)
cargo test -p eggup-service --locked --offline

# 4 — Windows cross-compile check (compile-only; it does NOT execute the suite)
cargo check --target x86_64-pc-windows-msvc -p eggup-service --locked --offline

# 5 — D3 evidence: the consolidation target's test count
grep -c '^\s*#\[test\]' crates/eggup-service/tests/bounded_executor_consolidation.rs

# 6 — D3 evidence: the environment target's test count
grep -c '^\s*#\[test\]' crates/eggup-service/tests/bounded_executor_environment.rs

# 7 — D3 evidence: the three argv guards and their gates
grep -n -B2 '^fn the_\(systemd\|launchd\|cron\)_managers_argv_is_byte_identical' \
  crates/eggup-service/tests/bounded_executor_consolidation.rs

# 8 — D3 evidence: the deleted name is gone
grep -rn 'the_managers_argv_is_byte_identical' crates/eggup-service/tests/

# 9 — D3 evidence: the windows-check lane's cargo test step count
awk 'NR>=116 && NR<=154' .github/workflows/ci.yml | grep -c 'run: cargo test'

# 10 — D3 evidence: the crate-private session-bus test's anchor
grep -n 'fn the_session_bus_filter_is_exactly_three_variables' crates/eggup-service/src/lib.rs

# 11 — D2 evidence: the field, its type, and the absence of a Default impl
grep -n 'pub engine_policy' crates/eggup-service/src/lifecycle_update.rs
grep -rn 'impl Default for LifecycleUpdateReceipt' crates/eggup-service/src/

# 12 — D2 evidence: the published 0.1.3 does not carry the field
git grep -n 'engine_policy' feb6ae5aea4c9b61c4051957f3662ca49d845f9e -- crates/eggup-service

# 13 — D1 evidence: the architecture records eight differences, D1-D8
grep -c '^| \*\*D[0-9]\*\* |' architecture/service-lifecycle.md

# 14 — ownership: this milestone touched only the four paths it owns
git status --short
```

The workspace-wide gate `./scripts/check-local.sh` is the **parent's** step and is **not** run here.

## 12. Documentation updates

| Document | Change |
|---|---|
| `crates/eggup-service/CHANGELOG.md` | § "Unreleased" only: the **D8** `Fixed` entry; the difference count corrected to **eight** and the enumeration completed; the compatibility preamble's source-break count corrected to **two**; a `**Consumer impact:**` note on the `engine_policy` field. **No published version section touched.** |
| `plans/closure/service-lifecycle/015-status.md` | The ten inventory corrections (rows **R-C1**, **P7**, **R-B1**; the **macOS** platform row; the "Files touched" row; the inventory heading and its **T8** row; the count; the two `windows-check` step counts), plus § "Corrective addendum (2026-10-24) — test-inventory correction by Service M016". |
| `plans/implementation/service-lifecycle/016-published-change-release-note-corrective.md` | **New.** This plan. |
| `plans/closure/service-lifecycle/016-status.md` | **New.** This milestone's closure record, with the ready-to-paste registry and roadmap rows in its appendix. |
| `plans/registry.md`, `plans/subsystems/service-lifecycle-roadmap.md` | **Not edited.** The parent owns them; the rows are drafted in the M016 closure's appendix. |
| `crates/eggup-service/README.md` | **Not edited.** It states no difference count and no source-break claim, so it is already truthful. Read-only evidence. |
| `architecture/service-lifecycle.md` | **Not edited.** It is the norm for D1–D8 and is correct. |
| root `CHANGELOG.md`, `AGENTS.md`, `docs/**`, `.opencode/skills/**`, `.github/**`, `scripts/**` | **Not edited.** The parent owns the root changelog and `AGENTS.md`; the rest are out of scope. |

## 13. Acceptance criteria

| # | Criterion | How it is checked |
|---|---|---|
| **AC1** | The crate's changelog states **eight** differences and names the eighth as a defect fix. | `grep -n 'Eight' crates/eggup-service/CHANGELOG.md`; read the entry against `architecture/service-lifecycle.md:636-655`. |
| **AC2** | The changelog carries a dedicated release note for **D8**, naming the `try_recv()` mechanism, the `manager stdin failed for {program}` symptom, the `crontab -` impact, and that the **published `0.1.3` still carries the defect**. | Read the `**Fixed: a successful stdin write …**` entry. |
| **AC3** | The changelog's compatibility call-out names **two** source-breaking changes and the `engine_policy` entry carries a `**Consumer impact:**` note. | Read § "Unreleased"'s preamble and the policy-inputs entry. |
| **AC4** | The changelog's published version sections (`0.1.2`, `0.1.3`) are byte-identical to the baseline. | `git diff -- crates/eggup-service/CHANGELOG.md` shows only § "Unreleased" hunks. |
| **AC5** | Every stale inventory claim in the M015 closure is corrected, and each corrected number is derivable from the tree by a recorded command. | The M015 addendum's before/after table; §11's commands 5–10. |
| **AC6** | The M015 closure has a dated corrective addendum recording the supersession, and its historical baseline evidence is untouched. | Read § "Corrective addendum (2026-10-24)"; `git diff` shows no change inside § "Baseline evidence". |
| **AC7** | The `windows-check` lane's step count is **ten**, and the ten steps are enumerated with their `.github/workflows/ci.yml` line numbers. | §11's command 9; the M015 addendum's step table. |
| **AC8** | No production code, test code, manifest, lockfile, CI file, or architecture document changed. | `git status --short` lists only the four owned paths. |
| **AC9** | The crate's whole suite still passes with unchanged per-target counts; clippy is warning-free; formatting is clean; the Windows target compiles. | §11's commands 1–4. |
| **AC10** | The M016 closure record has a requirement → evidence matrix, the exact commands and their observed results, per-platform results reported separately with "not run" where a lane was not run, and the ready-to-paste registry and roadmap rows in an appendix. | Read `plans/closure/service-lifecycle/016-status.md`. |
| **AC11** | No git state was mutated, and nothing was published. | `git status --short`; the `v0.1.3` tag and the crates.io listing are untouched. |

## 14. Stop conditions

The implementer **stops and reports** rather than proceeding if any of these fire:

1. **A code change appears to be required.** If the changelog cannot state D1, D2, or D3 truthfully without changing `crates/eggup-service/src/**` or `crates/eggup-service/tests/**`, stop and report. This milestone is documentation-only by construction.
2. **`crates/eggup-service/tests/public-api-surface.txt` shows a diff.** Another agent owns that file. If it changed, it is not this milestone's doing; report it and do not revert it.
3. **The counts in §3.3 do not reproduce.** If `grep -c '^\s*#\[test\]'` on the two targets does not return `14` and `2`, or the `windows-check` step count does not return `10`, the tree has moved under this milestone; re-derive every number from the tree and report the discrepancy.
4. **The published `0.1.3` source is unavailable.** `git show feb6ae5…` must work to establish the D2 baseline. If it does not, the D2 note may still be written from the workspace source and the M012 closure's own record, but the published-source evidence must be reported as unavailable rather than assumed.
5. **The M015 closure's historical evidence appears to need changing.** The baseline section and the first-hosted-run account are historical facts. If correcting the inventory seems to require rewriting them, stop: the addendum's "What this addendum does **not** change" section is the boundary.
6. **A concurrent edit lands in a file this milestone owns.** Another agent editing `crates/eggup-service/CHANGELOG.md` or the M015 closure concurrently would make the exact-string edits fail; re-read, re-derive, and report rather than forcing the edit.

## 15. Closure evidence required

The M016 closure record must carry:

1. **Status**, **source plan and roadmap**, **reviewed repository baseline**, **implementation commits/PRs**, and **executive finding** — per `plans/closure/README.md`.
2. **A requirement → evidence matrix** with the columns `requirement | evidence | result`, one row per requirement, each evidence cell citing a `file:line` **and** the command that produced it, and each result cell stating the observed outcome.
3. **Production implementation evidence** — stating explicitly that there is **none**, because the milestone is documentation-only, and naming the exact hunks that were written instead.
4. **Exact commands run and results** — §11's fourteen commands, each with its exit status and the observed output that matters.
5. **Invariant review** — the nine invariants of §4, each with its evidence.
6. **Failure/rollback/recovery review** — §8's statements.
7. **Compatibility and migration review** — §9's statements.
8. **Security review** — stating that no privilege, filesystem authority, network surface, or dependency changed.
9. **Documentation/operations evidence** — §12's table.
10. **Unresolved findings with severity** — with a stated severity and reason for each.
11. **Per-platform results, reported separately** — **Linux**, **macOS**, and **Windows** each on its own line, with anything not run marked **"not run"** and never inferred.
12. **Roadmap disposition** and **registry updates** — as a **ready-to-paste appendix**, because the parent owns `plans/registry.md` and `plans/subsystems/service-lifecycle-roadmap.md`. The registry row must be **≤ 481 bytes**, matching the existing format in `plans/registry.md`.

## 16. Handoff notes

- **What the parent must apply.** The appendix of `plans/closure/service-lifecycle/016-status.md` carries a ready-to-paste **registry row** (≤ 481 bytes) and a ready-to-paste **roadmap row**, plus a note on the `plans/registry.md` § "Post-0.1.3 open findings" list: **finding 15 stays open** for the published crate. M016 records the fix's release note; it does **not** discharge the published defect.
- **What the parent should also consider.** `AGENTS.md` § "Verify" said the `windows-check` lane runs "nine `cargo test` steps". It runs **ten**. `AGENTS.md` is not owned by this milestone and was not edited by it; the working tree's `AGENTS.md` already reads "ten" (corrected concurrently by another agent), so the two files agree in the final tree.
- **What this milestone does not do.** It does not republish `eggup-service`, does not move `v0.1.3`, does not bump the version, does not touch `architecture/**`, does not touch the root `CHANGELOG.md`, and does not run the workspace gate.
- **Severity summary for the parent's gate.** D1 **medium**, D2 **medium**, D3 **low**. All three are addressed by this milestone's own edits; the residual that remains open is the **published** defect itself (registry finding **15**), which only the republication milestone can discharge.
- **No consumer migration is required by this milestone.** It changes documentation only. The two migrations the changelog now names become necessary only at republication.
