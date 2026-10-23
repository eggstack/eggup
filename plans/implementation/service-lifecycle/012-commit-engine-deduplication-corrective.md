# Service Lifecycle Milestone 012 — commit-engine deduplication and ownership-type unification corrective

Status: closed; see `plans/closure/service-lifecycle/012-status.md` (implemented and green on the native macOS lane; hosted-qualified on Linux, macOS, and Windows, run `40008039843`)

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Original closed work affected:

- `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`
- `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`
- `plans/closure/service-lifecycle/005-status.md`
- `plans/closure/service-lifecycle/006-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **5**: "`eggup-service` carries two divergent copies of the commit/rollback engine, and three structurally identical ownership enums across two crates." Registered at baseline `c480541` alongside this plan. The finding is assessed **high** severity and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: invariant

## 1. Objective

Establish **one** commit-and-rollback engine in `eggup-service`, **one**
canonical ownership type, and **no** panic path in the service lifecycle.

1. `crates/eggup-service/src/disposition.rs` and
   `crates/eggup-service/src/lifecycle_update.rs` no longer carry independent
   copies of the transition / rollback-quiesce / restore mechanics. Exactly one
   implementation of each mechanic exists in the crate, and both public entry
   points (`commit_with_disposition`, `commit_with_lifecycle`) reach it. The two
   paths' remaining differences — ownership-loss tolerance and post-transition
   re-verification — become **explicit, named, separately-testable policy
   inputs**, not divergent code.
2. `eggup_service::Ownership` and `eggup_core::Ownership` are **the same type**,
   not two structurally identical types. The same holds for `CronOwnership`.
3. The `unreachable!("only Running and Stopped are valid desired states")` at
   `crates/eggup-service/src/lifecycle_update.rs:541` is removed. A desired
   state outside `{Running, Stopped}` produces a typed, bounded failure — never
   a panic — with no change to the `#[non_exhaustive]` `LifecycleState` enum.

A fourth, low-severity outcome: the crate declares **one** byte ceiling for
manager-output diagnostic text, and every production truncation site for that
data class uses it.

## 2. Why this milestone is ready

- Every defect is **fully characterized by reading the source at the baseline
  SHA**; each claim in §3 carries a `file:line` citation.
- The service-lifecycle subsystem has **no open milestone**: the roadmap status
  table (`plans/subsystems/service-lifecycle-roadmap.md` §12) records M001–M011
  closed; `eggup-service 0.1.3` was published 2026-10-09 (roadmap §11, M011).
- Scope is bounded to **one crate** (`eggup-service`) plus its own
  `README.md`/`CHANGELOG.md` and `architecture/service-lifecycle.md`. No other
  crate's production code is modified; `eggup-core` is a dependency only.
- Severity, stated explicitly per the planning-workflow severity gate:
  **Defect A (two divergent copies of the commit/rollback engine) — HIGH**; the
  duplicated code performs destructive service mutations and artifact rollback
  and has already drifted in three safety-relevant ways (A1–A3, §3). **Defect B
  (three structurally identical ownership enums across two crates) — MEDIUM**.
  **Defect C (a second public `CommandOutput` type with the same name) —
  MEDIUM**. **Defect D (two byte caps for the same data class) — LOW**.
- **Honest detection statement:** there is **no failing production report** for
  any of these defects; they were found by static reading at the baseline SHA.
  Both copies currently pass their own test suites, which is precisely why the
  divergence is invisible — each copy is tested against the behavior it
  implements, and no test asserts the two paths agree. A "medium-or-higher"
  issue is open, so per the severity gate this corrective must land before the
  subsystem can be considered clean.

## 3. Current implementation evidence

### 3.1 Defect A — two divergent copies of the same commit/rollback engine

`crates/eggup-service/src/` holds four source files: `lib.rs` (4,959 lines),
`disposition.rs` (2,502), `lifecycle_update.rs` (1,538), `windows_scm.rs`
(1,604). `disposition.rs` and `lifecycle_update.rs` are **in the same crate**
and each implement a full commit-and-rollback orchestrator. They share no
transition code: `disposition.rs` calls only its own
`restore_owned_state`/`safe_transition_owned_to`/`transition_owned_to` (`:483`,
`:726`, `:754`, `:838`, `:874`); `lifecycle_update.rs` calls only its own
`safe_restore_state`/`restore_state`/`transition_to` (`:323`, `:378`, `:408`,
`:474`, `:582`, `:610`). The only cross-module reuse is the crate-root
`pub use` at `crates/eggup-service/src/lib.rs:14` and `:19`.

| Mechanic | `disposition.rs` | `lifecycle_update.rs` |
|---|---|---|
| deadline helper | `remaining` at `:296-298` | `remaining` at `:680-682` (identical body) |
| detail truncation | `bound_detail` at `:291-294` | `bounded_detail` at `:684-687` (identical body, different name) |
| panic-safe inspect | `safe_manager_inspect` at `:300-315` | `safe_inspect` at `:585-600` |
| panic-safe restore/transition | `safe_transition_owned_to` at `:866-884` | `safe_restore_state` at `:602-620` |
| transition | `transition_owned_to` at `:886-953` (68 LOC) | `transition_to` at `:502-556` (55 LOC) |
| rollback quiesce | `quiesce_manager_for_rollback` at `:955-1029` | `quiesce_for_rollback` at `:622-678` (57 LOC) |
| restore primitive | `restore_owned_state` at `:595-659` | `restore_state` at `:575-583` |
| orchestration | `commit_with_disposition` at `:321-593` (273 LOC) | `commit_with_lifecycle` at `:254-453` (200 LOC) |

`restore_owned_state` (`disposition.rs:595-659`) is a **third** in-crate copy of
the transition match body — the same `(wanted, current.state)` arms, ownership
check, and `service transition did not complete` message, with a different
phase label. The copies have already diverged in three safety-relevant ways.

**A1 — Post-transition verification (the disposition copy does not re-inspect).**
`lifecycle_update.rs:552-555` re-inspects and validates after the transition:

```rust
let after = manager
    .inspect(spec)
    .map_err(|e| LifecycleFailure::new(spec, phase, e, None))?;
validate_observed(spec, &after, wanted, phase)
```

`validate_observed` (`:558-573`) rejects a post-transition snapshot whose
ownership is not `Owned` or whose state is not the wanted state.
`disposition.rs:918-944` does **not** re-inspect; it accepts the manager's own
`result.completed` flag and returns `Ok(())` (`if !r.completed { return
Err(mk_failure(spec, phase, "service transition did not complete", ...)); } Ok(())`).
Consequence: the disposition path can report a **successful** transition the
lifecycle path would have rejected — including one the manager reported as
complete but whose observed state never changed, or whose ownership changed
during the transition.

**A2 — A panic path that survives only because of a distant guard.**
`lifecycle_update.rs:541`:

```rust
_ => unreachable!("only Running and Stopped are valid desired states"),
```

`disposition.rs:946-951` handles the same situation with a typed error
(`_ => Err(mk_failure(spec, phase, "service state is Unknown or Transitioning",
Some(&current)))`). `LifecycleState` is `#[non_exhaustive]` with `Transitioning`
and `Unknown` variants (`crates/eggup-service/src/lib.rs:146-155`); the
`unreachable!()` arm is reached when the **desired** state is `Transitioning` or
`Unknown` while the **current** state is `Running` or `Stopped` (the `(_,
LifecycleState::Transitioning | LifecycleState::Unknown)` arm at `:539` already
covers the current-state cases).

**The two engines feed structurally identical values into two different
implementations.** `RestoreIntent::Preserve` maps to the pre-commit snapshot's
state in both paths: `lifecycle_update.rs:464` and `disposition.rs:833`
(`crate::RestoreIntent::Preserve => before.state`). The lifecycle path filters
that snapshot at `lifecycle_update.rs:265-269` (`before.ownership !=
Ownership::Owned || !matches!(before.state, Running | Stopped)` → `Preflight`
error). The disposition path does **not** filter the state: `disposition.rs:440`
assigns `let before = fresh_manager;` with no state restriction, and
`before.state` flows into `restore_owned_state` at `:726`, `:754`, and `:833`.

**Verified consequence:** today an unfiltered `Transitioning`/`Unknown`
`before.state` produces a typed error in the disposition engine and is
unreachable in the lifecycle engine, so the panic at `:541` is **latent, not
live** — not reachable through the current public API of `commit_with_lifecycle`.
It *is* reachable through the crate-internal transition primitive (a unit test
in `lifecycle_update.rs` calls `transition_to` directly at the baseline SHA),
and it is one guard relocation away from being live: the guard is ~200 lines
from the point of use, in a different function, and the `RestoreIntent::Preserve`
mapping at `:464` is exactly the value carrier that a future change to
restore-intent plumbing would feed a non-`{Running, Stopped}` value into.

This is the concrete hazard of the duplication: **merging by copying the
lifecycle engine into the disposition path would convert the disposition path's
typed error into a panic**, because the disposition path legitimately passes an
unfiltered `before.state` into its restore primitive.

**A3 — Ownership-loss tolerance.** `disposition.rs:904-907` tolerates ownership
loss when the wanted state is `Stopped` and the unit is already stopped
(`if wanted == LifecycleState::Stopped && current.state == LifecycleState::Stopped
{ return Ok(()); }`). `lifecycle_update.rs:520-527` always errors
(`current.ownership != Ownership::Owned` → `LifecycleFailure`). The two engines
disagree about whether an unproven ownership state can produce a **successful**
transition outcome.

### 3.2 Defect B — three structurally identical ownership enums across two crates

1. `crates/eggup-core/src/domain.rs:116` — `pub enum Ownership { Absent, Owned, Foreign, Unknown }`.
   Canonical. Its doc states that `Owned` is never inferred from an
   `InstallPlan` and is returned only by a consumer-supplied
   `OwnershipVerifier`. Re-exported at `crates/eggup-core/src/lib.rs:31-37`.
2. `crates/eggup-service/src/lib.rs:132` — `pub enum Ownership { Absent, Owned, Foreign, Unknown }`.
   Byte-structurally identical duplicate. Its own doc comment claims *"Canonical
   ownership classification, shared with `eggup-core` vocabulary"* while not
   reusing the type.
3. `crates/eggup-service/src/lib.rs:2773` — `pub enum CronOwnership { Absent, Owned, Foreign, Unknown }`.
   A third copy, produced by `cron_classify` at `:2842` and mapped back to
   `Ownership` at `:3074-3083`.

There is **no** `From`/`TryFrom` between any pair (`grep -rn "impl From<.*Ownership>" crates/`
returns no match) and no cross-type `PartialEq`. `eggup-service` depends on
`eggup-core` (`crates/eggup-service/Cargo.toml`:
`eggup-core = { version = "0.1.0", path = "../eggup-core" }`), so the
duplication is **not** required by any boundary rule. It is already visible
inside the crate: the test verifier at `disposition.rs:1875-1886` returns
`eggup_core::Ownership` while the crate's own public API exposes
`service::Ownership`, and `crates/eggup-service/src/windows_scm.rs:774`
(`registration_ownership`) uses the crate-internal type.

Consequences: a consumer of two published crates must hand-map between two
identical enums; the failure taxonomy can drift; the `Foreign`/`Unknown`
fail-closed rule is stated in two places
(`crates/eggup-core/src/domain.rs:112-115`, `crates/eggup-service/src/lib.rs:129-131`).

### 3.3 Defect C — a second public `CommandOutput` type with the same name

- `crates/eggup-core/src/candidate.rs:84` — `pub struct CommandOutput { exit_code: Option<i32>, stdout: Vec<u8>, stderr: Vec<u8>, timed_out: bool, output_limited: bool }`; fields private, accessors at `:92-121`; re-exported at `crates/eggup-core/src/lib.rs:26-29`.
- `crates/eggup-service/src/lib.rs:621` — `pub struct CommandOutput { pub status: Option<i32>, pub stdout: Vec<u8>, pub stderr: Vec<u8> }`; public fields; `stdout_text`/`stderr_text` at `:630-640`.

Both crates are published and `eggup-service` depends on `eggup-core`, so a
consumer importing both faces a **name collision** and must disambiguate. The
shapes also differ: core uses `exit_code` plus timeout/overflow flags; service
uses `status` and carries neither flag.

### 3.4 Related observation — the bounded subprocess runner (duplication observation only)

`eggup-service` implements its own bounded runner: `pub struct SystemExecutor`
at `crates/eggup-service/src/lib.rs:671`, `fn run` at `:700`, spawn at `:723`,
`read_bounded_generic` at `:952`. This duplicates `eggup-core`'s `run_bounded`
(`crates/eggup-core/src/candidate.rs:125`) and is not required by a boundary
rule, because `eggup-service` already depends on `eggup-core`. A third copy in
`eggup-curl` (`crates/eggup-curl/src/lib.rs:573` `run_curl_to_file`, spawn at
`:594`) **is** justified: `eggup-curl` may depend only on `eggup-acquisition`.

Two concrete obstacles, stated honestly:

- **No bounded stdin.** `run_bounded` sets `.stdin(Stdio::null())`
  unconditionally (`crates/eggup-core/src/candidate.rs:130`). The service path
  needs optional bounded stdin for `crontab -`
  (`crates/eggup-service/src/lib.rs:652-654` documents `stdin_data`; piped stdin
  at `:732`, bounded writer thread at `:787`, result binding at `:773`).
- **Core's primitive is currently defective.** Its wall-clock deadline is
  enforced only while the direct child is alive: the kill-on-deadline branch at
  `crates/eggup-core/src/candidate.rs:187-190` sits inside the polling path
  reached only while the child is running. A separate core corrective,
  `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`,
  is being authored to fix this. **That file does not exist at this baseline
  SHA**; it is a planned successor, not a closed milestone.

**Scope decision:** consolidating `SystemExecutor` onto a core bounded-execution
primitive is **out of scope for M012** and is deferred (§5, §6.6, §16).

### 3.5 Defect D — two byte caps for the same data class

- `crates/eggup-service/src/lib.rs:1784-1786` — `fn truncate(s: &str) -> String` caps at **256** bytes; used for manager stderr text at `:1418`, `:1812`, `:3024`, `:3047`.
- `crates/eggup-service/src/lib.rs:310-312` — `ServiceError::bounded` caps at **512**; used by `ServiceError::conflict`/`manager` (`:314-322`).
- `crates/eggup-service/src/lib.rs:1795` — `permission_hint` re-caps to `512 - HINT.len()`.
- `crates/eggup-service/src/lib.rs:286` — the crate doc states the cap as **512** without qualification: *"Error strings produced by crate helpers are capped at 512 UTF-8-safe bytes."*

The same data class (manager-output text) is capped at two different values, and
the declared contract matches only one of them. The existing test at
`crates/eggup-service/src/lib.rs:3402` (`assert!(truncate(&unicode).len() <= 256);`)
encodes the 256 ceiling and must move with any change to it.

## 4. Invariants that must not regress

- Ownership `Foreign` and `Unknown` always fail closed with a **receipt**
  (`RolledBack`/`RecoveryRequired`), never as `Err`; unifying the enum types
  must not weaken or re-scope this rule. `Owned` is never inferred — it is
  returned only by a caller-proven ownership check.
- The ADR-0002 transaction model is unchanged: `InstallPlan → Prepared →
  Verified → Validated → Receipt` in `eggup-core`, and the service crate's
  compose-over-Core model is unchanged. The service crate still creates **no
  live destination parent**, makes no artifact-level decision, and remains
  manager-neutral.
- The manager program allowlist (`resolve_manager_program` at
  `crates/eggup-service/src/lib.rs:879`, `validate_executable_path` at `:906`)
  and the minimal manager environment (`filtered_manager_environment` at `:929`)
  apply unchanged.
- Absolute operation deadlines, per-stream output bounds
  (`MAX_COMMAND_OUTPUT_BYTES` at `:611`), bounded stdin (`MAX_CRONTAB_BYTES` at
  `:613`), literal argv with no shell interpolation, cleared environment, and
  kill-and-reap child cleanup are unchanged.
- Windows SCM behavior is unchanged: ownership classification
  (`crates/eggup-service/src/windows_scm.rs:774`), the `MarkedForDelete →
  Unknown` mapping (`:319`, `:427`), and the re-check before mutation (`:460`,
  `:471`, `:484`, `:493`).
- `LifecycleState` stays `#[non_exhaustive]` with the same four variants; no
  public enum is extended or narrowed. `eggup-service` gains **no** dependency.

## 5. Scope

### In scope

- One commit-and-rollback engine inside `eggup-service`, with the two paths'
  behavioral differences expressed as explicit named policy inputs.
- Removal of the `unreachable!()` panic path, with a typed failure replacing it.
- Unification of `Ownership` and `CronOwnership` onto `eggup_core::Ownership`
  by type identity, with source-compatible paths preserved.
- Resolution of the `CommandOutput` name collision by renaming the service-side
  type; one declared byte ceiling for manager-output diagnostic text, used by
  every production truncation site for that class.
- Regression fixtures for each of the above, plus `README.md` /
  `CHANGELOG.md` and `architecture/service-lifecycle.md` updates.

### Explicitly out of scope

- **No transport, archive, or Eggpack dependencies.** `eggup-service` keeps
  exactly `eggup-core`, `windows-args` (`=0.2.0`), and `windows-service`
  (`=0.8.1`, Windows target only). No new dependency of any kind.
- **No Windows SCM backend execution coverage** — service-lifecycle milestone
  `014` (not yet authored at this baseline).
- **No `ServiceError` taxonomy change**, **no timeout-model change**, and **no
  cron inspection fix** — all service-lifecycle milestone `013`
  (`plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`,
  authored after this baseline, not closed).
- **No consolidation of `SystemExecutor` onto a core bounded-execution
  primitive** (§3.4, §6.6, §16).
- **No republication.** `eggup-service 0.1.3` is published (2026-10-09, source
  `feb6ae5`, checksum `9f7f7ea8…32124dc`); M012 must not publish and must not
  move the `v0.1.3` tag. Republication is a separate later milestone.
- **No change to the manager adapters' external command surface** — the argv
  built for `systemctl`/`launchctl`/`crontab` is unchanged.
- **No change to `eggup-core` production code**; `eggup-core` is read-only here.

## 6. Required production changes

Requirements are observable outcomes, not code.

### 6.1 Defect A — one engine, differences as named policy inputs

- **R-A1.** Exactly one implementation of the transition, rollback-quiesce,
  panic-safe-inspect, panic-safe-restore, deadline, and detail-truncation
  mechanics exists in `eggup-service`; both public orchestrators reach it.
- **R-A2.** The post-transition re-inspection and validation of
  `lifecycle_update.rs:552-555` + `:558-573` is available to **both** paths as
  an explicit, named, separately testable policy input, **on by default**; the
  disposition path's current no-re-inspection behavior survives only as an
  explicitly named, non-default input.
- **R-A3.** The ownership-loss tolerance of `disposition.rs:904-907` is
  available as an explicit, named, separately testable policy input, **off by
  default** (an ownership that is no longer `Owned` is always a typed failure);
  when the permissive input is selected, that fact must be observable in the
  returned receipt/failure evidence.
- **R-A4.** No `unreachable!`, `unwrap`, `expect`, or arithmetic panic remains
  reachable on any manager-reported state or caller-supplied desired state, in
  either path.
- **R-A5.** The failure taxonomy is unchanged: the same
  `LifecycleFailure`/`LifecycleUpdatePhase` vocabulary, the same
  `LifecycleUpdateError` shapes, and the same receipt dispositions
  (`RolledBack`/`RecoveryRequired`). Only the producing code path changes.

### 6.2 Defect A2 — required behavior for a non-`{Running, Stopped}` desired state

- **R-A6.** A desired state of `Transitioning` or `Unknown` produces a typed,
  bounded failure whose detail states that the desired state is not a valid
  `Running`/`Stopped` target — never a panic, never a fabricated success.
- **R-A7.** The validity check lives **at the point of use**, inside the unified
  engine's transition mechanic, so the engine rejects an invalid desired state
  regardless of which caller supplied it.
- **R-A8.** The current-state cases (`Transitioning`/`Unknown` observed on the
  manager) keep the existing typed error and message
  (`"service state is Unknown or Transitioning"`, `disposition.rs:946-951`).
- **R-A9.** `LifecycleState` remains `#[non_exhaustive]` with the same four
  variants; no public enum change is made or required.

### 6.3 Defect B — required type identity and compatibility alias

- **R-B1.** `eggup_service::Ownership` and `eggup_core::Ownership` are **the
  same type**. Preferred mechanism: a re-export of the canonical type
  (`pub use eggup_core::Ownership;`), so `eggup_service::Ownership` continues
  to resolve for existing consumers and no conversion is needed anywhere.
- **R-B2.** `eggup_service::CronOwnership` is reconciled with the canonical type
  such that the two are **the same type**. Preferred mechanism: a public type
  alias to the canonical type, so `CronOwnership::Absent` and friends keep
  resolving and `cron_classify`'s public signature keeps its spelling.
  `CronOwnership` must not remain a distinct enum.
- **R-B3.** The re-export carries rustdoc stating, in service terms, that this
  is `eggup-core`'s canonical ownership classification and that `Owned` is never
  inferred; core's destination-oriented wording must not stand unqualified for
  the service context.
- **R-B4.** The `Foreign`/`Unknown` fail-closed rule is stated once, in the
  canonical type's documentation; the service crate's rustdoc points at it.
- **R-B5.** Classification *logic* is unchanged: `cron_classify`
  (`crates/eggup-service/src/lib.rs:2842-2862`) and the mapping at `:3074-3083`
  keep the same outcomes. Only the result type changes.

### 6.4 Defect C — required naming outcome

- **R-C1.** `eggup-service` must not export a canonical public type named
  `CommandOutput`; the canonical service-side name states that it is a *manager*
  command output.
- **R-C2.** The `CommandExecutor::run` signature
  (`crates/eggup-service/src/lib.rs:655-660`) names the canonical type.
- **R-C3.** A source-compatibility alias under the old name **may** be retained
  so existing explicit-path callers and existing `CommandExecutor`
  implementations continue to compile. The alias is a type alias to the
  canonical type, so implementations written against either spelling still
  satisfy the trait. The alias must **not** be marked `#[deprecated]` here:
  deprecation is a publication-policy decision owned by the later republication
  milestone, and a `#[deprecated]` alias would break downstream `-D warnings`
  builds.
- **R-C4.** The closure records the **residual** ambiguity for consumers who
  glob-import both crates as an unresolved finding with an explicit severity,
  deferred to the intentional 0.2/1.0 boundary — consistent with the
  already-registered `CleanupDisposition` finding in `plans/registry.md` §
  "Post-0.1.3 open findings" item 2.

### 6.5 Defect D — one declared byte ceiling

- **R-D1.** The crate declares **exactly one** byte ceiling for manager-output
  diagnostic text; the declared value appears in the crate doc at
  `crates/eggup-service/src/lib.rs:286` and matches every production truncation
  site for that data class.
- **R-D2.** The `permission_hint` re-cap at
  `crates/eggup-service/src/lib.rs:1795` is documented as a composition of that
  ceiling (ceiling minus hint length), not as a second ceiling.
- **R-D3.** No diagnostic produced by the crate exceeds the declared ceiling,
  and a test asserts the bound.
- **R-D4.** The existing assertion at `crates/eggup-service/src/lib.rs:3402` is
  updated together with the declared ceiling, not left contradicting the crate
  doc.

### 6.6 Bounded runner — explicit non-change

- **R-E1.** `SystemExecutor`'s execution behavior is **not** changed in M012; no
  reuse of `eggup_core::run_bounded` occurs, so M012 does not depend on core's
  currently-defective deadline behavior.
- **R-E2.** The deferral is recorded as a named follow-up obligation in §16 and
  as a closure finding, with the prerequisite that core's bounded-execution
  deadline corrective closes first and that bounded-stdin support is added to
  the core primitive before any service-side reuse.

## 7. Ordered work packages

Each work package is independently verifiable and leaves the tree green.

- **WP1 — Decide and extract the shared mechanics.** Enumerate the mechanics
  from §3.1 and select one home for each inside `eggup-service` (a new internal
  module, or one of the two existing modules, whichever keeps the dependency
  direction acyclic and keeps `windows_scm.rs` free of the orchestration types).
  Extract the transition, rollback-quiesce, panic-safe-inspect,
  panic-safe-restore, deadline, and truncation mechanics into that single home;
  keep both public entry points' signatures unchanged. *Verify:* the crate
  builds and the existing disposition and lifecycle suites stay green at every
  step.
- **WP2 — Unify the ownership types.** Apply R-B1…R-B5: replace
  `crates/eggup-service/src/lib.rs:132` with the canonical re-export and
  `crates/eggup-service/src/lib.rs:2773` with the alias, and update every
  in-crate use site — `crates/eggup-service/src/windows_scm.rs:9` and `:774`,
  the test verifier at `crates/eggup-service/src/disposition.rs:1875-1886`.
  *Verify:* `cargo test -p eggup-service --locked` green; the §10 compile-level
  type-identity assertion is in place and passing.
- **WP3 — Remove the panic path.** Replace
  `crates/eggup-service/src/lifecycle_update.rs:541` with the typed failure of
  R-A6 and move the desired-state validity check to the point of use per R-A7;
  do not change `LifecycleState`. *Verify:* the §10 fixture that panics at the
  baseline returns a typed failure.
- **WP4 — Post-transition verification and ownership-loss policy.** Introduce the
  two named policy inputs of R-A2 and R-A3, both defaulting to the stricter
  behavior, and route both public entry points through the unified engine.
  *Verify:* the parity, post-transition-verification, and ownership-loss
  fixtures of §10 are green, and the parity fixture shows the two former paths
  now take the same engine.
- **WP5 — Resolve the `CommandOutput` collision.** Apply R-C1…R-C3. *Verify:*
  `cargo doc -p eggup-service --no-deps --locked` shows one canonical
  manager-command-output type plus the compatibility alias.
- **WP6 — One byte ceiling.** Apply R-D1…R-D4. *Verify:* the §10 bound test is
  green and `crates/eggup-service/src/lib.rs:286` matches the enforced ceiling.
- **WP7 — Tests.** Add every fixture listed in §10, each proved to fail against
  the pre-M012 baseline by the command in §11.
- **WP8 — Docs and changelog.** Apply §12.

## 8. Failure, cancellation, restart, and contention semantics

Where the two copies disagreed, the **stricter** behavior wins, and the reason
is recorded: the strict copy is the one that cannot report success for an
unobserved or unproven state, and the workspace's fail-closed rule makes
"cannot prove" a failure, not a success.

- **Manager transition failure.** A manager `start`/`stop` that errors, or
  returns `completed == false`, produces the existing typed failure
  (`"service transition did not complete"`) with the pre-transition snapshot
  attached, under the existing phase vocabulary. Both paths behave identically
  post-M012.
- **Post-transition validation failure.** With R-A2's default (on), a manager
  that reports `completed == true` but whose post-transition inspection shows
  an ownership that is not `Owned`, or a state that is not the wanted state,
  produces the typed failure of `lifecycle_update.rs:558-573`. This stricter
  behavior wins; the disposition path's silent acceptance (`:918-944`) is
  removed as a default. Rationale: a `completed` flag is a manager self-report,
  not evidence of observed state; the M010 quiescence work already established
  that a nominal completion is not proof.
- **Ownership lost mid-transition.** With R-A3's default (strict), an ownership
  that is no longer `Owned` at the transition point is always a typed failure —
  `lifecycle_update.rs:520-527`. The disposition copy's tolerance
  (`:904-907`, allowed only when the wanted state is `Stopped` and the unit is
  already stopped) is not the default. Rationale: `Owned` is never inferred, and
  unproven ownership must not be able to produce a successful outcome.
- **Desired state not `Running`/`Stopped`.** Always a typed, bounded failure
  (R-A6); never a panic, on any path, under any policy. The engine validates
  the desired state itself (R-A7), so the guard does not depend on a distant
  caller's pre-filter.
- **Ownership classification `Foreign`/`Unknown`.** Unchanged: fail closed with
  a receipt (`RolledBack`/`RecoveryRequired`), never `Err`. Type unification
  must not alter this; a foreign or unknown registration is never destructively
  mutated.
- **Manager panic.** Still caught at the mechanic that is executing, per the
  existing `safe_*` wrappers, and recorded under the phase actually executing.
  The unified engine keeps one catch boundary per mechanic, not a single outer
  catch.
- **Deadline exhaustion.** Unchanged: a zero or exhausted budget produces the
  existing typed failure; no deadline arithmetic changes in M012.
- **Cancellation / restart / contention.** Unchanged. M012 introduces no new
  concurrency, lock, or ordering between quiesce, commit, post-commit check, and
  rollback. The unified engine must preserve the existing ordering exactly,
  because that ordering is what the closed M005/M006 closures were qualified
  against.

## 9. Compatibility and migration

`eggup-service 0.1.3` is **published** (2026-10-09, source `feb6ae5`, checksum
`9f7f7ea8…32124dc`). The crate's API is public. M012 does **not** publish and
does **not** move the `v0.1.3` tag; the changes land on the source line and
reach consumers only through a separate, later republication milestone.

- **`eggup_service::Ownership` keeps resolving.** R-B1 uses a re-export, so
  `use eggup_service::Ownership;` and `eggup_service::Ownership::Owned` keep
  compiling, and the type is now *identical* to `eggup_core::Ownership` — a
  consumer that previously hand-mapped between the two no longer has to.
  Source-compatible and strictly widening.
- **`eggup_service::CronOwnership` keeps resolving.** R-B2 uses a public type
  alias, so `CronOwnership::Absent` and the `cron_classify` return spelling keep
  compiling. Source-compatible.
- **`eggup_service::CommandOutput` is renamed.** The canonical name becomes the
  manager-command-output name; the old spelling survives only as a type alias
  (R-C3). A type rename is a **source-visible** change: consumers writing
  `eggup_service::CommandOutput` in a type position keep compiling through the
  alias; `CommandExecutor` implementations written against either spelling
  still satisfy the trait, because the alias denotes the same type; consumers
  who glob-import both crates and then name `CommandOutput` unqualified remain
  ambiguous, and that residual is recorded per R-C4 rather than silently
  accepted.
- **Two behavior changes are visible to consumers of the disposition path.**
  Both are fail-closed **tightenings**: (1) a transition the manager reports as
  complete but whose observed state or ownership does not match is now a typed
  failure instead of a success (`disposition.rs:918-944`); (2) ownership that
  is no longer `Owned` at a transition is now always a typed failure instead of
  the tolerated already-stopped case (`disposition.rs:904-907`). A consumer that
  relied on either permissive outcome must select the corresponding named policy
  input explicitly. The closure must record both with their `file:line` sites.
- **No `ServiceError` variant, `LifecycleState` variant, `ServiceManager` trait
  method signature, or `LifecycleUpdateError` shape changes.** No new
  dependency, no protocol/schema/lock migration, and no consumer migration step
  is required for the type-identity changes.

## 10. Required tests

Each new fixture must be proved to **fail** against the pre-M012 implementation
(§11 gives the exact command).

1. **Parity fixture — both former paths now take ONE engine.** Drive
   `commit_with_lifecycle` and `commit_with_disposition` through the same
   manager double over a matrix (owned running, owned stopped, ownership lost,
   manager `completed == false` with a mismatching post-transition state,
   manager `completed == false` with a matching state, inspect error, manager
   panic) and assert the two paths produce the **same** observable outcome class
   per scenario. At the baseline this fails: the paths disagree on the
   ownership-loss and post-transition scenarios (A1, A3).
2. **Non-`{Running, Stopped}` desired state.** A crate-internal unit test, in
   the module that owns the unified engine, invoking the transition mechanic
   with a desired state of `Transitioning` and with `Unknown` against a manager
   double reporting `Running`, asserting a **typed failure**, not a panic. At
   the baseline `crates/eggup-service/src/lifecycle_update.rs:541` **panics**.
   The fixture is a crate-internal unit test because the panic is not reachable
   through the current public API (§3.1, A2).
3. **Post-transition verification.** A manager double whose `start`/`stop`
   returns `completed == true` while a subsequent `inspect` reports a state
   different from the wanted state, plus a case where the subsequent inspect
   reports ownership that is not `Owned`. Both must produce the typed validation
   failure. At the baseline the disposition path returns success.
4. **Ownership-loss tolerance.** A manager double reporting ownership that is
   not `Owned`, wanted state `Stopped`, current state `Stopped`. The default must
   be the strict typed failure; the explicitly named permissive policy input
   must produce the tolerant outcome **and** make the selected policy observable
   in the returned evidence. At the baseline the disposition path succeeds and
   the lifecycle path fails — the fixture pins the new default.
5. **Ownership type identity (compile-level).** A bidirectional static assertion
   proving the two paths denote one type — e.g. a `const _: () = { … }` block
   with assignments in both directions between `eggup_core::Ownership` and
   `eggup_service::Ownership`, and the same for `eggup_service::CronOwnership`.
   It must be a **compile-time** check, so a future divergence is a build
   failure, not a runtime assertion.
6. **Byte-ceiling test.** Assert that every manager-output diagnostic the crate
   produces is bounded by the declared ceiling of R-D1 and that the crate doc
   value matches the enforced value; update the existing assertion at
   `crates/eggup-service/src/lib.rs:3402` with it.
7. **Existing suites stay green.** The full `eggup-service` suite, including the
   disposition and lifecycle suites, stays green. No existing test may be
   deleted or weakened to accommodate the change. If an existing test encoded a
   permissive behavior that R-A2/R-A3 deliberately remove, the closure must name
   that test and record the behavior change.

## 11. Required verification commands

Exact strings:

```text
cargo test -p eggup-service --locked
cargo test -p eggup-service --locked disposition
cargo test -p eggup-service --locked lifecycle
cargo clippy -p eggup-service --all-targets --locked -- -D warnings
cargo doc -p eggup-service --no-deps --locked
cargo test --workspace --all-targets --all-features --locked
./scripts/check-local.sh
```

`./scripts/check-local.sh` is the local gate: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
`cargo test --workspace --all-targets --all-features --locked`,
`cargo doc --workspace --no-deps --locked`, plus `cargo tree --workspace --locked`
as a human review step.

**Proving the new fixtures fail against the pre-M012 baseline**, verbatim:

```text
git worktree add ../eggup-m012-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
(cd ../eggup-m012-baseline && cargo test -p eggup-service --locked disposition)
(cd ../eggup-m012-baseline && cargo test -p eggup-service --locked lifecycle)
```

The baseline worktree then receives the new fixtures (copied source files for
the crate-internal unit test, and/or `crates/eggup-service/tests/*.rs` for the
integration fixtures), and each fixture must be shown to **fail** — or to panic,
for fixture 2 — in the baseline worktree:

```text
(cd ../eggup-m012-baseline && cargo test -p eggup-service --locked <fixture_name>)
```

The closure records, per fixture: the fixture name, the exact baseline command,
and the exact observed failure output. Remove the worktree afterwards
(`git worktree remove ../eggup-m012-baseline`).

Hosted qualification, run and recorded separately: `stable`, `msrv`,
`service-systemd`, `macos`, `windows-check`. A lane that was not run is recorded
as **not run**, never inferred.

## 12. Documentation updates

- **rustdoc on the unified engine**: what it is, what it guarantees, and the two
  named policy inputs with their defaults and their fail-closed direction.
- **rustdoc on the `Ownership` re-export** (R-B3): the service-context meaning,
  the pointer to `eggup-core`'s canonical documentation, and the statement that
  `Owned` is never inferred and that `Foreign`/`Unknown` fail closed.
- **rustdoc on the replacement for the removed panic path** (the typed failure a
  non-`{Running, Stopped}` desired state produces), on the renamed
  manager-command-output type, and on the compatibility alias (its purpose and
  its 0.2/1.0 disposition).
- **`crates/eggup-service/CHANGELOG.md`** — an `Unreleased` entry carrying the
  crate's standard no-publication / no-migration disclaimer: the changes are not
  published by this milestone, no consumer migration is required for the
  type-identity changes, and republication is a separate milestone.
- **`crates/eggup-service/README.md`** — if it names the ownership type, the
  `CommandOutput` type, or the two orchestrators, update it to the new names and
  to the single-engine reality.
- **`architecture/service-lifecycle.md`** — the "Source layout" section and the
  per-file sections for `lib.rs` / `lifecycle_update.rs` / `disposition.rs`, the
  ownership section ("Ownership and authorization"), and the `SystemExecutor` /
  process-execution section must reflect the single engine, the canonical
  ownership type, and the renamed command-output type.
- **`docs/` is non-normative.** If a consumer-facing page names the affected
  types, update it to point at the contracts; do not introduce a rule there.
- `plans/registry.md` § "Post-0.1.3 open findings" and the service-lifecycle
  roadmap status table are updated by the **closure**, not by this plan.

## 13. Acceptance criteria

M012 closes only when all of the following are checkable:

1. Exactly one implementation of the transition, rollback-quiesce,
   panic-safe-inspect, panic-safe-restore, deadline, and truncation mechanics
   exists in `eggup-service`; both public orchestrators reach it.
2. Post-transition verification and ownership-loss tolerance are named,
   separately testable policy inputs, both defaulting to the **stricter**
   existing behavior, and both paths run the same engine by default.
3. No `unreachable!`/panic remains on any desired-state or manager-state input
   in either path; the §10 item 2 fixture returns a typed failure where the
   baseline panics.
4. `eggup_service::Ownership` and `eggup_core::Ownership` are the same type,
   proven by a compile-time bidirectional assertion; likewise
   `eggup_service::CronOwnership`.
5. `eggup-service` exports no canonical public type named `CommandOutput`; the
   manager-command-output type is canonical under its new name, with the
   compatibility alias retained and documented.
6. Exactly one byte ceiling is declared for manager-output diagnostic text; the
   crate doc matches it; every production truncation site for that class uses it.
7. Every new fixture in §10 is proved to fail against the baseline SHA
   `c4805411f74b85158e0ae5d0a219a32c4209d98e` via the §11 worktree command, and
   passes at the implementation SHA.
8. `cargo test -p eggup-service --locked`, the disposition and lifecycle filtered
   runs, `cargo clippy -p eggup-service --all-targets --locked -- -D warnings`,
   `cargo doc -p eggup-service --no-deps --locked`, `cargo test --workspace
   --all-targets --all-features --locked`, and `./scripts/check-local.sh` are
   green, with exact outputs recorded.
9. Hosted `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` lanes
   are recorded with real results; any lane not run is recorded as **not run**.
10. No new dependency appears in `crates/eggup-service/Cargo.toml`; `cargo tree
    -p eggup-service --locked` still shows only `eggup-core`, `windows-args`,
    and (Windows target only) `windows-service`.
11. `eggup-service 0.1.3` remains the published version; no publication occurred;
    the `v0.1.3` tag is unmoved.
12. No medium-or-higher finding remains open against the service-lifecycle
    subsystem as a result of this milestone.

## 14. Stop conditions

Halt and escalate rather than proceed when:

- The unification cannot be achieved **without** a breaking public API change
  that must instead be sequenced at the 0.2/1.0 boundary — in particular, if
  the ownership unification cannot be expressed as a re-export plus an alias
  while keeping `eggup_service::Ownership` and `eggup_service::CronOwnership`
  resolving, stop; do not remove them.
- The shared engine cannot preserve the **stricter** fail-closed semantics
  (post-transition verification on, ownership loss always a failure). If the
  permissive disposition behavior cannot be expressed as an explicit named policy
  input without weakening the default, stop.
- The work would require reusing the currently-defective `eggup_core::run_bounded`
  before core's bounded-execution deadline corrective
  (`plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`)
  closes. **M012 must not reuse it at all**; if reuse appears necessary, stop and
  re-sequence.
- Removing the `unreachable!()` would require adding or removing a
  `LifecycleState` variant, or making `LifecycleState` exhaustive.
- Any change requires a new dependency, a transport/archive/Eggpack dependency,
  or a `ServiceError` taxonomy change.
- Any change alters the manager argv surface, the manager environment policy, the
  executable allowlist, or the deadline arithmetic.
- The unified engine cannot reproduce the closed M005/M006 ordering, or an
  existing M005/M006 test must be deleted or weakened rather than updated.
- A hosted lane fails for a reason attributable to this milestone.

## 15. Closure evidence required

`plans/closure/service-lifecycle/012-status.md` must contain:

- **Status**: closed / conditionally closed / corrective pass required / blocked;
  source plan and roadmap; reviewed repository baseline; implementation commits.
- **Executive finding**: what the two copies were, what they had drifted into,
  and what the unified engine now guarantees.
- **Requirement→evidence matrix**: one row per requirement R-A1…R-A9, R-B1…R-B5,
  R-C1…R-C4, R-D1…R-D4, R-E1, R-E2, each with the exact command and exact
  observed output that satisfies it.
- **Production implementation evidence**: the single home of the shared
  mechanics, with `file:line` for each; the removal sites of the duplicated
  helpers; the two policy inputs with their defaults.
- **Baseline-failure evidence**: per new fixture, the exact baseline command, the
  baseline worktree SHA, and the exact observed failure or panic output. A
  fixture that does not fail at the baseline is not evidence and must be
  corrected or withdrawn.
- **Exact commands and results**, including §11's list, with real output.
- **Separate platform results**: Linux, macOS, and Windows reported individually.
  A lane that was not run is recorded as **not run**, never inferred from another
  platform.
- **Invariant review**: §4's invariants, each marked preserved / changed /
  violated, with evidence.
- **Failure, rollback, and recovery review**: §8's semantics, each with the
  fixture that proves it.
- **Compatibility and migration review**: §9's items, including the two
  fail-closed behavior tightenings with their `file:line` sites, the alias
  retention, and the statement that no consumer migration is required for the
  type-identity changes.
- **Security review**: the fail-closed direction, the `Owned`-never-inferred
  rule, the unchanged allowlist/environment/stdin/output bounds, and the
  confirmation that no panic path remains on manager- or caller-supplied input.
- **Documentation/operations evidence**: the rustdoc, `CHANGELOG.md` `Unreleased`,
  `README.md`, and `architecture/service-lifecycle.md` updates.
- **Unresolved findings with severity**: including the residual `CommandOutput`
  glob-import ambiguity (R-C4) and the deferred `SystemExecutor` consolidation
  (R-E2), each with an explicit severity and reason.
- **Roadmap disposition** and **registry updates**, including the new numbered
  entry in `plans/registry.md` § "Post-0.1.3 open findings".

## 16. Handoff notes

- **Sequencing obligation.** If the deferred bounded-execution consolidation is
  later taken, it must wait for
  `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`
  to close, and that core milestone must first add bounded-stdin support to the
  core primitive (core's `run_bounded` sets `.stdin(Stdio::null())`
  unconditionally at `crates/eggup-core/src/candidate.rs:130`, while the service
  path needs bounded stdin for `crontab -`). Only after both conditions hold may
  `eggup-service`'s `SystemExecutor` delegate to a core primitive. **M012 itself
  neither reuses the primitive nor waits for it** — it is sequenced entirely
  independently, because depending on the defective core behavior would import a
  known defect into this corrective.
- **What service-lifecycle milestone `013` owns** (named by this plan's scope
  decision; authored after this baseline, not closed): the `ServiceError`
  taxonomy, the timeout model, and the cron inspection fix. M012 changes the
  *type* of the cron classification result and nothing about cron inspection.
- **What service-lifecycle milestone `014` owns**: Windows SCM backend execution
  coverage. M012 changes no SCM behavior.
- **What core milestone `014` owns**: the bounded-execution wall-clock deadline
  defect in `eggup-core`'s `run_bounded`.
- **What this plan does NOT authorize**: publication of `eggup-service` or any
  other crate; moving the `v0.1.3` tag; a new dependency; a `ServiceError`
  taxonomy change; a timeout-model change; a cron inspection change; Windows SCM
  backend execution coverage; reuse of the defective core bounded-execution
  primitive; a change to the `LifecycleState` variant set; or any change to
  `eggup-core` production code.
- **Reuse warning for the next author.** The two orchestrators differ in more
  than the three divergences named in §3.1. WP1 must diff the two orchestrators
  function-by-function before extracting and must record every additional
  difference it finds. An additional safety-relevant difference becomes a named
  policy input or a new work package — it must not be silently resolved by
  picking one side.
