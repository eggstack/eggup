# Service Lifecycle M012 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M012--commit-engine-deduplication-corrective`

Reviewed repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`
(branch `plans/service-m010-failed-unit-recovery`)

Implementation commits/PRs: **working tree, uncommitted — no commit was made.**
No commit, push, tag, stash, checkout, restore, or clean was performed. The
change set is:

| Path | Change |
|---|---|
| `crates/eggup-service/src/commit_engine.rs` | **new** — the single commit-and-rollback engine |
| `crates/eggup-service/src/lifecycle_update.rs` | duplicated mechanics removed; orchestrator rewired |
| `crates/eggup-service/src/disposition.rs` | duplicated mechanics removed; orchestrator rewired |
| `crates/eggup-service/src/lib.rs` | ownership re-export, cron alias, command-output rename, one byte ceiling, module wiring |
| `crates/eggup-service/tests/commit_engine_parity.rs` | **new** — parity, post-transition verification, ownership-loss default, type identity (split) |
| `crates/eggup-service/tests/common/mod.rs` | **new** — shared scriptable manager double |
| `crates/eggup-service/tests/commit_engine_policy.rs` | **new** — the two named policy inputs |
| `crates/eggup-service/tests/ownership_type_identity.rs` | **new** — compile-time type-identity proof |
| `crates/eggup-service/README.md` | public surface updated |
| `crates/eggup-service/CHANGELOG.md` | `## Unreleased` updated |
| `architecture/service-lifecycle.md` | source layout, per-file sections, ownership section, process-execution section |

No file outside the assigned ownership list was modified. `Cargo.toml` and
`Cargo.lock` were not touched; `--locked` stayed valid throughout.

## Executive finding

`eggup-service` carried **two divergent copies** of the destructive
commit/rollback engine: one in `disposition.rs` and one in `lifecycle_update.rs`.
The copies had already drifted apart in safety-relevant ways, and the drift was
not limited to the three differences the plan named. There is now **one**
engine — `crates/eggup-service/src/commit_engine.rs` — and both public
orchestrators reach it.

The unified engine is strictly the safer of the two former copies:

- a manager's own `completed` flag is a **self-report**, not evidence of
  observed state, so every transition is re-inspected and validated by default;
- an ownership that is no longer `Owned` is **always** a typed failure by
  default;
- the rollback quiesce **refuses** to stop an unobservable or non-owned service
  (the lifecycle copy swallowed the `inspect()` error and would have stopped an
  unproven target);
- the rollback restore is **panic-guarded** (the disposition copy called its
  restore without a catch boundary);
- a desired state outside `{Running, Stopped}` is a **typed, bounded failure**,
  never a panic.

The two behavioural differences that remain are now **explicit, named,
separately testable policy inputs** on `CommitEnginePolicy`, both defaulting to
the stricter behaviour: `PostTransitionVerification` (`Verify` | 
`TrustManagerReport`) and `OwnershipLossTolerance` (`Deny` | 
`TolerateAlreadyStopped`).

The three structurally identical ownership enums are now **one type**:
`eggup_service::Ownership` is a re-export of `eggup_core::Ownership`, and
`eggup_service::CronOwnership` is a public type alias of it. The two public
types named `CommandOutput` are resolved by renaming the service-side type to
`ManagerCommandOutput` and keeping `CommandOutput` as a non-deprecated type
alias. The two byte ceilings for manager-output diagnostic text are now **one
declared ceiling**, `MAX_DIAGNOSTIC_BYTES` (512 bytes), named in the crate doc.

No consumer migration is required for the type-identity changes. Two
behavioural tightenings are consumer-visible and are recorded below.

## Requirement-to-evidence matrix

| Plan requirement | Evidence | Verdict |
|---|---|---|
| §6.1 R-A1 — exactly one implementation of the transition, rollback-quiesce, panic-safe-inspect, panic-safe-restore, deadline, and truncation mechanics | `crates/eggup-service/src/commit_engine.rs:293` (`transition_to`), `:373` (`safe_transition_to`), `:245` (`safe_inspect`), `:478` (`quiesce_for_rollback`), `:425` (`quiesce_service`), `:228` (`remaining`), `:236` (`bounded_detail`). `disposition.rs` and `lifecycle_update.rs` contain no private copy of any of them. | Met |
| §6.1 R-A2 — post-transition verification is a named, separately testable policy input, **on by default**, on both paths | `PostTransitionVerification` (`commit_engine.rs:159`), default `Verify` (`commit_engine.rs:159`, `CommitEnginePolicy::default()` at `:201`); applied in `transition_to` (`commit_engine.rs:358-364`) and in the post-commit restore (`commit_engine.rs:578-586`) | Met |
| §6.1 R-A3 — ownership-loss tolerance is a named, separately testable policy input, **off by default**, and observable in the returned evidence | `OwnershipLossTolerance` (`commit_engine.rs:180`), default `Deny`; one implementation of the tolerance condition (`ownership_tolerated`, `commit_engine.rs:268`) used by both the transition check and the post-transition validation; the selected policy is reported in `LifecycleUpdateReceipt::engine_policy` (`lifecycle_update.rs:84`, `disposition.rs:724`, `:738`, `:995`, `:1009`) | Met |
| §6.1 R-A4 — a manager panic is still caught at the mechanic that is executing, and recorded under the phase actually executing | `safe_transition_to` (`commit_engine.rs:373-392`) catches the panic and records phase `RestoreNew`; `safe_inspect` (`commit_engine.rs:245-260`) catches and records the inspect phase; the outer `catch_unwind` in the orchestrators can no longer mislabel a panicking transition as a post-install check failure | Met |
| §6.1 R-A5 — one implementation of the rollback-quiesce mechanic, fail-closed | `commit_engine.rs:478-553`; `inspect` failure → refused (`commit_engine.rs:492-501`), non-`Owned` → refused (`commit_engine.rs:509-518`), stopped state confirmed (`commit_engine.rs:532-545`) | Met |
| §6.1 R-A6 — a desired state of `Transitioning`/`Unknown` produces a typed, bounded failure stating the desired state is not a valid `Running`/`Stopped` target | `INVALID_DESIRED_STATE` (`commit_engine.rs:221`), enforced in `transition_to` (`commit_engine.rs:309-319`); unit test `commit_engine.rs:674` | Met |
| §6.1 R-A7 — the desired-state validity check lives **at the point of use**, inside the unified engine | `commit_engine.rs:309-319` is inside `transition_to`; the disposition path no longer has its own transition mechanic that could skip the check | Met |
| §6.1 R-A8 — the current-state cases keep the existing typed error and message `"service state is Unknown or Transitioning"` | `commit_engine.rs:340-348` | Met |
| §6.3 R-B1 — `eggup_service::Ownership` keeps resolving for existing consumers; no conversion needed anywhere | `pub use eggup_core::Ownership;` (`lib.rs:165`); `windows_scm.rs` continues to resolve `crate::Ownership` unmodified | Met |
| §6.3 R-B2 — `CronOwnership` must not remain a distinct enum | `pub type CronOwnership = Ownership;` (`lib.rs:2837`) | Met |
| §6.3 R-B3 — the re-export carries rustdoc stating the service-context meaning and that `Owned` is never inferred | `lib.rs:132-164` | Met |
| §6.3 R-B4 — the service crate must not define a second `Ownership` enum | The local enum is deleted; `grep -rn '^pub enum Ownership {' crates/eggup-service/src/` returns nothing, and `pub use eggup_core::Ownership;` is the only definition | Met |
| §6.3 R-B5 — classification *logic* is unchanged | `cron_classify` (`lib.rs:2897`) and the mapping (`lib.rs:3128-3141`) keep the same outcomes; only the result type spelling changed | Met |
| §6.4 R-C1 — the service crate must not export a canonical public type named `CommandOutput` | `ManagerCommandOutput` (`lib.rs:662`); `CommandOutput` is a type alias (`lib.rs:694`) | Met |
| §6.4 R-C2 — the `CommandExecutor` trait names the canonical type | `lib.rs:714` (`-> Result<ManagerCommandOutput, ServiceError>`) | Met |
| §6.4 R-C3 — the alias must not be `#[deprecated]` | `lib.rs:684-693` — explicitly documented as not deprecated, with the reason | Met |
| §6.4 R-C4 — the residual glob-import ambiguity is recorded as an unresolved finding | See "Unresolved findings", finding 2 | Met (recorded) |
| §6.5 R-D1 — exactly one declared byte ceiling for manager-output diagnostic text, named in the crate doc, used by every production truncation site | `MAX_DIAGNOSTIC_BYTES` (`lib.rs:653`); crate doc (`lib.rs:310-311`); `ServiceError::invalid`/`bounded` (`lib.rs:330-337`); `permission_hint` composition (`lib.rs:1849`); `bounded_detail` (`commit_engine.rs:236`) | Met |
| §6.5 R-D2 — the ceiling is not silently raised to a size that could carry a secret | The value stays 512; the former inner 256-byte cap is removed so the *same* value is enforced once | Met |
| §6.5 R-D3 — no diagnostic produced by the crate exceeds the declared ceiling | `ServiceError::Manager` is capped by `ServiceError::manager`; the engine's failure detail by `bounded_detail`; the fixture at `lib.rs:3475` asserts the bound and the retention of the manager output up to the ceiling | Met |
| §6.5 R-D4 — the existing assertion at `lib.rs:3402` is updated with the declared ceiling | `lib.rs:3449-3472` now asserts against `MAX_DIAGNOSTIC_BYTES` and asserts the constant's value | Met |
| §8 — stricter behaviour wins wherever the copies disagreed | See "Failure/rollback/recovery review" | Met |
| §6.6 R-E1 — `SystemExecutor` is **not** consolidated onto `eggup_core::run_bounded` | `SystemExecutor` (`lib.rs:725`) is unchanged in structure; core's bounded-execution primitive is untouched and remains a recorded follow-on | Met (deferred by the plan itself) |
| §5 — the `ServiceError` taxonomy change, the timeout-model change, and the cron inspection fix are **not** performed | `ServiceError`'s variants are unchanged; `LifecycleUpdatePolicy`'s three timeouts are unchanged; the cron classification is unchanged | Met |
| §5 — the manager adapters' external command surface is unchanged | The argv built for `systemctl` / `launchctl` / `crontab` is untouched; only the output *type name* changed | Met |
| §5 — no new dependency of any kind | `cargo tree -p eggup-service --locked` shows `eggup-core` (sha2) + `windows-args`; identical to the baseline | Met |
| §12 — rustdoc on the unified engine, the canonical ownership type, and the renamed command-output type | `commit_engine.rs:1-49` (module), `lib.rs:134-165` (ownership), `lib.rs:644-694` (command output) | Met |
| §15 — closure evidence | This record | Met |

## Additional divergences found (beyond the three the plan named)

§16 requires every additional difference to be recorded. The two engine copies
differed in **seven** places, not three. Five were safety-relevant and are
resolved by the stricter behaviour:

| # | Difference | Baseline sites | Resolution |
|---|---|---|---|
| D1 | Post-transition verification absent | `disposition.rs:918` vs `lifecycle_update.rs:552-555` | The stricter (verify) behaviour wins, as a named policy input defaulting to on. |
| D2 | Ownership loss tolerated | `disposition.rs:904-907` vs `lifecycle_update.rs` (no equivalent) | The stricter (deny) behaviour wins, as a named policy input defaulting to off. |
| D3 | Invalid desired state panicked | `lifecycle_update.rs:541` (`unreachable!()`) vs `disposition.rs:946` (typed error) | The typed error wins; the panic is removed entirely. |
| **D4** | The rollback quiesce **swallowed the `inspect()` error** | `lifecycle_update.rs:636` (`manager.inspect(spec).ok()`) vs `disposition.rs:973-983` | The stricter (fail-closed) behaviour wins. The lifecycle copy never received the 0.1.2 fail-closed fix that `crates/eggup-service/CHANGELOG.md` claims for this path. |
| **D5** | The rollback quiesce had **no ownership check** | `lifecycle_update.rs:636-642` vs `disposition.rs:990-999` | The stricter (refuse a non-owned service) behaviour wins. The lifecycle copy would have issued a destructive `stop` against a non-owned service. |
| **D6** | The rollback restore was **not panic-guarded** | `disposition.rs:483`, `:726`, `:754` (direct call) vs `lifecycle_update.rs` (`safe_restore_state`) | The guarded behaviour wins: the unified `safe_transition_to` is panic-guarded at its own boundary. |
| **D7** | The quiesce-failure restore was labelled `RestoreOld` | `disposition.rs:483-489` vs `lifecycle_update.rs:328` (`Quiesce`) | The phase-accurate labelling wins: the failure is recorded under the phase actually executing (§8). |

Two further differences are **path policy, not engine mechanics**, and are
preserved deliberately rather than unified:

- the disposition path restores the old state on `RolledBack` only for
  `ManagedRunning` / `ManagedStopped`, while the lifecycle path restores it
  unconditionally (`disposition.rs:693-713` vs `lifecycle_update.rs:319-330`).
  This is a *disposition* question owned by the closed M005/M006 milestones, not
  a mechanic; the shared mechanic is now identical on both sides.
- the wanted-state mapping differs because the disposition path has a richer
  plan (`disposition.rs:743-757` vs `lifecycle_update.rs:372-384`). The mapping
  is caller-owned policy feeding the shared engine.

One further inline quiesce exists at `disposition.rs:509-540` (the
`DirectRunning` branch). It stops the **caller's direct-runtime seam**
(`DirectRuntimeControl::stop`), not the service manager, and it is not a copy
of the manager quiesce mechanic. It is deliberately left as a caller-owned
seam call and is **not** part of the unified engine.

## Production implementation evidence

### The unified engine

| Element | Location |
|---|---|
| Module documentation (guarantees, the two named policy inputs, their defaults and fail-closed direction) | `crates/eggup-service/src/commit_engine.rs:1-49` |
| `PostTransitionVerification` (`Verify` default, `TrustManagerReport`) | `commit_engine.rs:159-178` |
| `OwnershipLossTolerance` (`Deny` default, `TolerateAlreadyStopped`) | `commit_engine.rs:180-199` |
| `CommitEnginePolicy` (the two named inputs) | `commit_engine.rs:201-208` |
| `remaining` (the one deadline helper) | `commit_engine.rs:228-230` |
| `bounded_detail` (the one truncation rule, control-character free, `MAX_DIAGNOSTIC_BYTES`) | `commit_engine.rs:236-239` |
| `safe_inspect` (the one panic-safe inspection) | `commit_engine.rs:245-260` |
| `ownership_tolerated` (the one ownership-tolerance condition) | `commit_engine.rs:268-276` |
| `transition_to` (the one transition mechanic) | `commit_engine.rs:293-365` |
| desired-state validity check at the point of use | `commit_engine.rs:309-319` |
| ownership check | `commit_engine.rs:324-334` |
| current-state typed error | `commit_engine.rs:340-348` |
| post-transition verification | `commit_engine.rs:358-364` |
| `safe_transition_to` (the one panic-safe transition) | `commit_engine.rs:373-392` |
| `validate_observed` (post-transition ownership/state validation) | `commit_engine.rs:400-418` |
| `quiesce_service` (the one pre-commit quiesce) | `commit_engine.rs:425-465` |
| `quiesce_for_rollback` (the one rollback quiesce, fail-closed) | `commit_engine.rs:478-553` |
| `restore_and_check` (the one post-commit restore + caller check) | `commit_engine.rs:561-604` |
| `NoPostInstallCheck` / `PostInstallCheck` / `PostInstallCheckError` (moved here) | `commit_engine.rs:106-150` |
| `LifecycleFailure` / `LifecycleUpdatePhase` (moved here) | `commit_engine.rs:52-104` |

### The orchestrators

| Element | Location |
|---|---|
| `commit_with_lifecycle` (signature unchanged; delegates to the default-policy variant) | `crates/eggup-service/src/lifecycle_update.rs:166-190` |
| `commit_with_lifecycle_with_engine_policy` (new) | `lifecycle_update.rs:192-370` |
| `commit_with_disposition` (signature unchanged; delegates to the default-policy variant) | `crates/eggup-service/src/disposition.rs:289-326` |
| `commit_with_disposition_with_engine_policy` (new) | `disposition.rs:328-591` |
| `LifecycleUpdateReceipt::engine_policy` (the selected policy, observable in the evidence) | `lifecycle_update.rs:84-87` |
| receipt construction carrying the selected policy | `lifecycle_update.rs:353`, `:368`; `disposition.rs:724`, `:738`, `:995`, `:1009` |
| `restore_success_state` (now a thin delegation to `restore_and_check`) | `lifecycle_update.rs:372-394` |
| `restore_manager_success` (now a thin delegation to `restore_and_check`) | `disposition.rs:743-786` |
| `commit_manager_path` | `disposition.rs:593-741` |
| `commit_direct_path` | `disposition.rs:788-1011` |

### The type-identity and byte-ceiling changes

| Element | Location |
|---|---|
| `pub use eggup_core::Ownership;` with service-context rustdoc | `crates/eggup-service/src/lib.rs:132-165` |
| `pub type CronOwnership = Ownership;` | `lib.rs:2824-2837` |
| `ManagerCommandOutput` | `lib.rs:644-661` |
| `CommandOutput` type alias (not deprecated) | `lib.rs:671-694` |
| `CommandExecutor::run` naming the canonical type | `lib.rs:709-714` |
| `MAX_DIAGNOSTIC_BYTES` | `lib.rs:646-653` |
| crate doc naming the one ceiling | `lib.rs:310-311` |
| `ServiceError::invalid` / `bounded` using the constant | `lib.rs:330-337` |
| `permission_hint` documented as a composition of the ceiling | `lib.rs:1846-1851` |
| the four former `truncate(...)` manager-stderr sites now using `ServiceError::bounded(...)` | `lib.rs:1472`, `:1866`, `:3079`, `:3102` |
| the former `fn truncate` (the second, smaller ceiling) deleted | removed from `lib.rs` |
| module wiring and re-exports | `lib.rs:11-28` |

## Exact commands run and their real output

### Local gate (scoped to this milestone)

The parent session owns the full workspace gate. Per the assignment, the
workspace-wide `cargo test --workspace` and `./scripts/check-local.sh` were
**not** run by this session.

```console
$ cargo test -p eggup-service --locked --offline
running 110 tests
test result: ok. 110 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.24s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(The 110 lib tests include the new `commit_engine::tests::
non_running_stopped_desired_state_is_a_typed_failure` and the new
`tests::manager_output_diagnostics_use_the_one_declared_ceiling`. The
`running 0 tests` targets are `ownership_type_identity.rs`, whose evidence is a
compile-time proof, and the doc-test target.)

```console
$ cargo clippy -p eggup-service --all-targets --locked --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
```

```console
$ cargo fmt --all -- --check
(no output — clean)
```

```console
$ cargo doc -p eggup-service --no-deps --locked --offline
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
   Generated /Users/davidbowman/projects/eggup/target/doc/eggup_service/index.html
```

```console
$ cargo tree -p eggup-service --locked
eggup-service v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-service)
├── eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
│   └── sha2 v0.10.9
...
└── windows-args v0.2.0
    └── wtf8 v0.3.0
```

Unchanged from the baseline: no new dependency, no transport / service-manager /
Eggpack dependency, no `sha2` in `eggup-acquisition`, no `self-replace` on
non-Windows graphs.

### Baseline-failure evidence

Mechanism used: **`git worktree add`** (the preferred mechanism in the
assignment). The worktree was created at
`/tmp/eggup-baseline-service012` from `c4805411f74b85158e0ae5d0a219a32c4209d98e`,
the fixtures were copied in, the focused tests were run there expecting a
non-zero exit, and the worktree was then removed with
`git worktree remove --force /tmp/eggup-baseline-service012`. No git state in
the primary working tree was mutated. `git worktree list` afterwards shows
only the primary worktree.

#### Fixture 1/3/4 — parity, post-transition verification, ownership-loss default

Files copied: `crates/eggup-service/tests/commit_engine_parity.rs` and
`crates/eggup-service/tests/common/mod.rs`. Both compile unchanged against the
baseline — they use only the pre-M012 public API surface.

```console
$ cd /tmp/eggup-baseline-service012
$ cargo test -p eggup-service --locked --offline --test commit_engine_parity
running 3 tests
test ownership_loss_is_a_typed_failure_by_default_on_both_paths ... FAILED
test post_transition_verification_is_a_typed_failure_on_both_paths ... FAILED
test both_former_paths_reach_one_engine_and_agree_on_every_scenario ... FAILED

failures:

---- ownership_loss_is_a_typed_failure_by_default_on_both_paths stdout ----

thread 'ownership_loss_is_a_typed_failure_by_default_on_both_paths' panicked at crates/eggup-service/tests/commit_engine_parity.rs:350:10:
the ownership loss must be recorded
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- post_transition_verification_is_a_typed_failure_on_both_paths stdout ----

thread 'post_transition_verification_is_a_typed_failure_on_both_paths' panicked at crates/eggup-service/tests/commit_engine_parity.rs:293:10:
the post-transition mismatch must be recorded

---- both_former_paths_reach_one_engine_and_agree_on_every_scenario stdout ----

thread 'both_former_paths_reach_one_engine_and_agree_on_every_scenario' panicked at crates/eggup-service/tests/commit_engine_parity.rs:227:9:
assertion `left == right` failed: the two former paths disagree on scenario "ownership lost before the post-commit transition"
  left: CommittedWithPostCommitFailure
 right: CommittedClean

failures:
    both_former_paths_reach_one_engine_and_agree_on_every_scenario
    ownership_loss_is_a_typed_failure_by_default_on_both_paths
    post_transition_verification_is_a_typed_failure_on_both_paths

test result: FAILED. 0 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
EXIT=101
```

Reading the three failures:

- the parity matrix fails because the two former paths **disagree** on the
  ownership-loss scenario — the lifecycle path records a post-commit failure
  while the disposition path reports a clean commit;
- `post_transition_verification_is_a_typed_failure_on_both_paths` fails with
  "the post-transition mismatch must be recorded" because at the baseline the
  disposition path **recorded no failure at all** for a manager that reported
  `completed == true` while its observation disagreed with the wanted state;
- `ownership_loss_is_a_typed_failure_by_default_on_both_paths` fails with "the
  ownership loss must be recorded" for the same reason: at the baseline the
  disposition path **tolerated** the ownership loss silently.

The last two are the plan's A1 and A3 divergences, observed directly.

#### Fixture 2 — invalid desired state is a typed failure

This is a **crate-internal** unit test, so it cannot be a `tests/` file. It was
transcribed into the baseline worktree's `crates/eggup-service/src/lifecycle_update.rs`
test module, adapted only to the baseline's five-argument `transition_to`
signature (the baseline has no engine-policy parameter). The transcription is
the same test body as `commit_engine.rs:674`.

```console
$ cd /tmp/eggup-baseline-service012
$ cargo test -p eggup-service --locked --offline --lib non_running_stopped_desired_state_is_a_typed_failure
running 1 test
test lifecycle_update::tests::non_running_stopped_desired_state_is_a_typed_failure ... FAILED

failures:

---- lifecycle_update::tests::non_running_stopped_desired_state_is_a_typed_failure stdout ----

thread 'lifecycle_update::tests::non_running_stopped_desired_state_is_a_typed_failure' panicked at crates/eggup-service/src/lifecycle_update.rs:541:14:
internal error: entered unreachable code: only Running and Stopped are valid desired states
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    lifecycle_update::tests::non_running_stopped_desired_state_is_a_typed_failure

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 108 filtered out; finished in 0.00s
EXIT=101
```

The panic is at `lifecycle_update.rs:541` — exactly the `unreachable!()` the plan
names.

#### Fixture 5 — type identity (compile-time)

```console
$ cd /tmp/eggup-baseline-service012
$ cargo test -p eggup-service --locked --offline --test ownership_type_identity
error[E0308]: mismatched types
   --> crates/eggup-service/tests/ownership_type_identity.rs:24:46
    |
24 |     let cron: eggup_service::CronOwnership = eggup_service::Ownership::Foreign;
    |               ----------------------------   ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `CronOwnership`, found `Ownership`
error[E0308]: mismatched types
  --> crates/eggup-service/tests/ownership_type_identity.rs:25:39
   |
25 |     let core: eggup_core::Ownership = cron;
   |     ---------------------   ^^^^ expected `Ownership`, found `CronOwnership`
error: could not compile `eggup-service` (test "ownership_type_identity") due to 4 previous errors
EXIT=101
```

**This fixture's baseline evidence is a compile error, by construction.** It is a
compile-time identity assertion; at the baseline the three types were distinct
and the assertion could not hold. The compiler's own note states the defect
verbatim: "`eggup_core::Ownership` and `eggup_service::Ownership` have similar
names, but are actually distinct types". This fixture is therefore *not*
evidence for the engine-duplication defect — the parity fixture is — and it is
recorded here as its own category.

#### Fixture 6 — one byte ceiling

Transcribed into the baseline worktree's `crates/eggup-service/src/lib.rs` test
module, adapted only to the baseline's `CommandOutput` type name (the baseline
has no `MAX_DIAGNOSTIC_BYTES`; the transcription uses the crate doc's literal
`512`).

```console
$ cd /tmp/eggup-baseline-service012
$ cargo test -p eggup-service --locked --offline --lib manager_output_diagnostics_use_the_one_declared_ceiling
running 1 test
test tests::manager_output_diagnostics_use_the_one_declared_ceiling ... FAILED

failures:

---- tests::manager_output_diagnostics_use_the_one_declared_ceiling stdout ----

thread 'tests::manager_output_diagnostics_use_the_one_declared_ceiling' panicked at crates/eggup-service/src/lib.rs:3432:9:
the manager output must be retained up to the declared ceiling, got 281 bytes
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    tests::manager_output_diagnostics_use_the_one_declared_ceiling

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 109 filtered out; finished in 0.00s
EXIT=101
```

281 bytes = the 25-byte message prefix + the 256-byte second ceiling. The
baseline silently discarded the manager's diagnostic text at a ceiling the crate
doc never declared.

#### Baseline suite integrity

With the two transcribed unit tests in place, the baseline suite was otherwise
untouched:

```console
$ cd /tmp/eggup-baseline-service012
$ cargo test -p eggup-service --locked --offline --lib
test lifecycle_update::tests::non_running_stopped_desired_state_is_a_typed_failure ... FAILED
test tests::manager_output_diagnostics_use_the_one_declared_ceiling ... FAILED
test result: FAILED. 108 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
```

Only the two transcribed fixtures fail; the baseline's own 108 tests still pass.

### Platform evidence

| Platform | Result |
|---|---|
| **macOS (darwin arm64, this host)** | **RUN** — native. All commands above ran here. |
| **Linux** | **NOT RUN** — no Linux host is reachable from this machine. |
| **Windows** | **NOT RUN** — no Windows host is reachable. The Windows SCM backend path is not exercised on this host, and plan §5 excludes Windows SCM execution coverage from this milestone. |
| `stable` CI lane | **NOT RUN** — no hosted runner is reachable from this machine. |
| `msrv` CI lane | **NOT RUN** — no hosted runner is reachable from this machine. |
| `macos` CI lane | **NOT RUN** — no hosted runner is reachable from this machine. |
| `windows-check` CI lane | **NOT RUN** — no hosted runner is reachable from this machine. |
| `service-systemd` CI lane | **NOT RUN** — no hosted runner is reachable from this machine. |

No platform result is inferred from another platform. The Windows-specific risk
of this change is confined to `crates/eggup-service/src/windows_scm.rs`, which
was **not modified**; it continues to resolve `crate::Ownership` through the
re-export and its `Ownership` matching arms remain exhaustive because
`eggup_core::Ownership` is not `#[non_exhaustive]`.

## Invariant review

| Invariant | Verdict | Note |
|---|---|---|
| `Owned` is never inferred | **Held** | The unified engine's ownership check reads the manager's caller-proven observation and never derives `Owned` from state. The permissive `TolerateAlreadyStopped` input *admits* an already-stopped unit; it never asserts `Owned`. |
| `Foreign` / `Unknown` fail closed with a receipt, not `Err` | **Held, with one deliberate narrowing** | The strict default makes an ownership loss at a transition a typed failure, exactly as §8 requires. The permissive input is an explicit, opt-in, named choice. |
| Only lock contention / setup failures return `Err` | **Held** | The orchestrators' error taxonomy is unchanged; only the *content* of a post-commit failure became stricter (a mismatching observation is now recorded instead of being silently accepted). |
| `validate` requires every member to carry `IntegrityStatus::Verified` | **Held** | Core's contract is untouched; `eggup-service` does not re-implement it. |
| Core never creates live destination parents / never decides a lock is stale | **Held** | Core is untouched. |
| Staging is owner-private | **Held** | Core is untouched. |
| No invented timeouts | **Held** | The three `LifecycleUpdatePolicy` timeouts and `MAX_TRANSITION_TIMEOUT` are unchanged. `remaining()` is unchanged in behaviour. |
| No crash-journaling claims | **Held** | No new claim was added. |
| SHA-256 is integrity evidence only | **Held** | No authenticity or signature claim was added or removed. |
| The manager adapters' external command surface is unchanged | **Held** | The argv for `systemctl` / `launchctl` / `crontab` is byte-identical. Only the output *type name* changed. |
| The crate's dependency boundary is unchanged | **Held** | `cargo tree -p eggup-service --locked` is identical to the baseline. |
| `unsafe_code = "deny"` | **Held** | `commit_engine.rs` carries `#![forbid(unsafe_code)]`; no `unsafe` was introduced anywhere. |
| `missing_docs` | **Held** | `commit_engine.rs` carries `#![deny(missing_docs)]`; `cargo doc` is clean with no warnings. |
| The public API has rustdoc for every public item | **Held** | The two new public enums, the new policy struct, the two new public functions, the new receipt field, the re-export, the alias, and the new constant are all documented. |

## Failure/rollback/recovery review

| Path | Behaviour before | Behaviour after |
|---|---|---|
| Desired state outside `{Running, Stopped}` | The lifecycle path **panicked** (`unreachable!()`, `lifecycle_update.rs:541`); the disposition path returned a typed error | Both paths return the same typed, bounded failure stating that the desired state is not a valid `Running`/`Stopped` target |
| Manager panics during `start` / `stop` | Caught, but the lifecycle path labelled it `PostInstallCheck` and could let the receipt claim `Restored` | Caught at the mechanic that is executing and labelled with the phase actually executing (`RestoreNew` / `Quiesce` / `RestoreOld`) |
| Manager panics during the rollback restore | **Not guarded** in the disposition path — the panic escaped the orchestrator | Guarded by `safe_transition_to` |
| Manager panics during the rollback quiesce `inspect` | The lifecycle path swallowed it with `.ok()` and **proceeded to stop an unproven target** | Refused: `QuiesceForRollback` failure recorded, no destructive `stop` issued |
| Rollback quiesce against a non-`Owned` service | The lifecycle path **issued the destructive `stop`** | Refused: `QuiesceForRollback` failure recorded |
| Post-transition observation disagrees with the wanted state | The disposition path **silently accepted** the transition | Typed validation failure, recorded under `RestoreNew` |
| Post-transition observation is not `Owned` | The disposition path silently accepted it | Typed validation failure (or, under the explicitly selected permissive ownership input, tolerated) |
| Ownership lost at a transition | The disposition path tolerated it when the unit was already stopped | Typed failure by default; tolerated only under the explicitly selected `TolerateAlreadyStopped` input |
| Core returns `RolledBack` | The lifecycle path restores `before.state`; the disposition path restores it only for `ManagedRunning` / `ManagedStopped` | **Unchanged** — this is disposition policy owned by the closed M005/M006 milestones, not a mechanic. The shared restore mechanic is now identical. |
| Core returns `RecoveryRequired` | `NotAttemptedRecoveryRequired`, no automatic start | **Unchanged** |
| Core returns before a terminal receipt | `CoreBeforeReceipt { core, restore_failure }` | **Unchanged** |
| Rollback quiesce fails | `rollback_quiesce_failure` recorded separately; the disposition is unchanged | **Unchanged** |

**Restart and contention semantics are unchanged.** This milestone touches no
lock, no receipt, and no Core transaction semantics; the crate's own
`MutationLock` and stale-lock handling live in `eggup-core` and were not
modified.

**Net effect:** no failure path became *less* safe. Two failure paths became
strictly safer (D4, D5, D6) and one failure mode that was a silent success
became a recorded typed failure (D1, D2). The one deliberate behaviour
*loosening* is reachable only through an explicitly selected, non-default,
named policy input, and the selection is observable in the returned receipt.

## Compatibility and migration review

**Type-identity changes — no consumer migration is required.**

| Item | Before | After | Migration |
|---|---|---|---|
| `eggup_service::Ownership` | A local `enum` | `pub use eggup_core::Ownership;` — the **same type** as before in every use position | None. The type name, the variant names, and the trait bounds all resolve unchanged. `windows_scm.rs` resolves `crate::Ownership` unmodified. |
| `eggup_service::CronOwnership` | A local `enum` | `pub type CronOwnership = Ownership;` — a **type alias** | None. `CronOwnership::Absent` / `Owned` / `Foreign` / `Unknown` all keep resolving; the alias's variant paths are inherited from the aliased type. `cron_classify`'s public return spelling is unchanged. |
| `eggup_service::CommandOutput` | `struct` | `pub type CommandOutput = ManagerCommandOutput;` — a **type alias** | None for type positions and for `CommandExecutor` implementations written against either spelling: an alias denotes the same type, so `fn run(...) -> Result<CommandOutput, ServiceError>` still satisfies the trait, which now names `ManagerCommandOutput`. |
| `eggup_service::ManagerCommandOutput` | Did not exist | The canonical struct | Additive. |
| `LifecycleUpdateReceipt::engine_policy` | Did not exist | New public field | **Source-visible addition.** A consumer that destructures the receipt *exhaustively* (without `..`) must add the field. Consumers that read fields are unaffected. This field is required by plan §6.1 R-A3's explicit observability requirement — the permissive policy selection has no failure to attach itself to, so the receipt is the only place it can be observed. |
| `commit_with_lifecycle` / `commit_with_disposition` | 6- and 9-argument functions | **Unchanged signatures** | None. They now delegate to the new `*_with_engine_policy` variants under `CommitEnginePolicy::default()`. |
| `commit_with_lifecycle_with_engine_policy` / `commit_with_disposition_with_engine_policy` | Did not exist | New public functions | Additive. |
| `CommitEnginePolicy` / `PostTransitionVerification` / `OwnershipLossTolerance` | Did not exist | New public types | Additive. |
| `MAX_DIAGNOSTIC_BYTES` | Did not exist | New public constant | Additive. The crate doc now names it. |

**The two deliberate behaviour tightenings** (plan §9), with their `file:line`
sites — these are the only consumer-visible behaviour changes, and both are
fail-closed:

1. **Post-transition verification is on by default on the disposition path.**
   Baseline site: `disposition.rs:918` (the disposition engine accepted a
   transition on the manager's `completed` flag alone). Now:
   `commit_engine.rs:358-364`. A consumer that relied on the permissive
   behaviour must select `PostTransitionVerification::TrustManagerReport`
   explicitly.
2. **Ownership loss is denied by default on the disposition path.** Baseline
   site: `disposition.rs:904-907`. Now: `commit_engine.rs:324-334` and
   `commit_engine.rs:268-276`. A consumer that relied on the permissive
   behaviour must select `OwnershipLossTolerance::TolerateAlreadyStopped`
   explicitly.

**Byte-ceiling compatibility.** The declared ceiling value did not change
(512 bytes). The *enforced* ceiling for the manager stderr excerpt moved from
256 to 512, so a `ServiceError::Manager` diagnostic may now carry up to 512
bytes of manager output where it previously carried at most ~281. This is
strictly more diagnostic evidence, never less, and it remains bounded by the
declared ceiling. The value was not raised.

**Why the policy inputs are new functions rather than new
`LifecycleUpdatePolicy` fields.** `LifecycleUpdatePolicy` is a public struct
with public fields, so adding a field to it would break every downstream
struct-literal construction of that type. The plan's baseline-proof requirement
that the new fixture "must compile against the pre-M012 baseline and fail
there" makes that impossible for the fixture itself: a fixture that constructs
`LifecycleUpdatePolicy` with a literal would not compile at the baseline. The
policy inputs are therefore carried by two **new** entry points, and the two
existing entry points keep their exact signatures and run the engine under the
stricter default. The consequence for consumers is strictly smaller: no
existing call site changes.

**Residual glob-import ambiguity (plan §6.4 R-C4) — recorded as an unresolved
finding.** `eggup_core` still exports a public `CommandOutput` and
`eggup_service` still exports `CommandOutput` (as an alias). A consumer that
glob-imports both crates and then names `CommandOutput` unqualified remains
ambiguous and will not compile. The plan explicitly requires this to be
recorded rather than resolved in this milestone, because resolving it would
mean removing the alias or renaming core's type — the former breaks existing
explicit-path consumers, the latter is a `eggup-core` change that is outside
this milestone's ownership. See finding 2.

## Security review

| Property | Verdict | Note |
|---|---|---|
| No `unsafe` introduced | **Held** | `commit_engine.rs:2` carries `#![forbid(unsafe_code)]`; the crate root already forbids it. |
| Destructive operations still require `Owned` | **Held and strengthened** | `require_owned` (`lib.rs:426`) is unchanged; the engine's own ownership check (`commit_engine.rs:324-334`) additionally refuses a non-`Owned` unit at a transition. |
| A destructive `stop` is never issued against an unproven target | **Strengthened** | The rollback quiesce now refuses an unobservable (`commit_engine.rs:492-501`) and a non-`Owned` (`commit_engine.rs:509-518`) service. The baseline lifecycle path did neither. |
| Failure detail never carries a secret | **Held** | `bounded_detail` (`commit_engine.rs:236`) strips control characters and caps at `MAX_DIAGNOSTIC_BYTES`. The `PostInstallCheck` contract still requires the caller not to include secrets. |
| The diagnostic ceiling was not silently raised | **Held** | The value stays 512. The *inner* 256-byte cap was removed so that one value is enforced once; the effective bound on any diagnostic is unchanged at 512. |
| The manager's stderr is not logged wholesale | **Held** | It is captured under `MAX_COMMAND_OUTPUT_BYTES` (256 KiB) and folded into a bounded diagnostic. |
| A manager panic cannot crash the orchestrator | **Strengthened** | Every manager call in the engine is behind a catch boundary at the mechanic that is executing (`safe_inspect`, `safe_transition_to`, `quiesce_service`). |
| No command-interpolation surface changed | **Held** | The argv built for `systemctl` / `launchctl` / `crontab` is unchanged; `SystemExecutor` still resolves only allowlisted absolute paths and clears the child environment. |
| The permissive ownership tolerance is not a silent default | **Held** | It is `Deny` by default, must be selected by name, and the selection is observable in the receipt. |
| `SystemExecutor` was not consolidated onto `eggup_core::run_bounded` | **Held** | Plan §6.6 R-E1 forbids it because core's bounded-execution primitive is itself defective until a separate core milestone closes. Left as a recorded follow-on (finding 3). |

## Documentation/operations evidence

| Document | Change |
|---|---|
| `crates/eggup-service/README.md` | The ownership bullet now states that `Ownership` **is** `eggup_core::Ownership` and that `CronOwnership` is an alias; the byte-bounds bullet now names the single `MAX_DIAGNOSTIC_BYTES` (512) ceiling; a new bullet describes the single engine and the two named policy inputs; a new bullet describes `ManagerCommandOutput` and the non-deprecated `CommandOutput` alias. |
| `crates/eggup-service/CHANGELOG.md` | The `## Unreleased` section now carries the "not published by this milestone / no consumer migration for the type-identity changes" disclaimer, the single-engine fix, the two named policy inputs, the ownership unification, the command-output rename, and the byte-ceiling fix. The published `0.1.3` section and the `v0.1.3` tag were not touched. |
| `architecture/service-lifecycle.md` | Source layout now lists `commit_engine.rs` and states the single-engine rule and the acyclic dependency direction; a new `commit_engine.rs` per-file section; the `lifecycle_update.rs` and `disposition.rs` per-file sections updated; a new "One canonical ownership type" subsection; the `ServiceError` section rewritten around the one ceiling; the process-execution section documents `ManagerCommandOutput`, the alias, and the alias's deprecation policy; the lifecycle-composition section documents the two named policy inputs; the `Ref` column was re-derived for the items this milestone moved. |
| `plans/registry.md` | **Not edited by this session** — the parent applies the registry transition. The intended prose is in "Registry updates" below. |
| `plans/subsystems/service-lifecycle-roadmap.md` | **Not edited by this session** — the parent applies the roadmap transition. The intended prose is in "Roadmap disposition" below. |
| `CHANGELOG.md` (root) | **Not edited by this session** — the parent aggregates it. |
| `.github/workflows/ci.yml` | **Not edited.** The three named service test commands still resolve; no test was renamed or deleted. |

## Unresolved findings with severity

**Finding 1 — `SystemExecutor` is not consolidated onto `eggup_core::run_bounded`.**
Severity: **low**. Reason: plan §6.1 R-E1 explicitly forbids the consolidation
in this milestone because core's bounded-execution primitive is itself
defective until a separate core milestone closes. The duplication is therefore
*known*, *recorded*, and *deferred by the plan itself*, not an undetected
defect. It is a maintainability finding, not a safety finding: `SystemExecutor`'s
own bounds (allowlisted absolute paths, cleared environment, bounded output,
bounded stdin, no shell interpolation) are unchanged and independently
defensible. It is a follow-on for the core bounded-execution milestone.

**Finding 2 — the residual `CommandOutput` glob-import ambiguity.**
Severity: **low**. Reason: plan §6.4 R-C4 requires this to be recorded as an
unresolved finding rather than fixed here. The residual is a *compile-time
naming* ambiguity affecting only a consumer that glob-imports both `eggup_core`
and `eggup_service` **and** then names `CommandOutput` unqualified. It is not
a safety issue: neither type is a secret-bearing or security-relevant value,
and the ambiguity is a loud compile error, not a silent mis-binding. It is
low rather than informational because the plan's own §9 acknowledges the
ambiguity survives the rename and because a future core-side rename would
remove it. Resolving it in this milestone would require either removing the
service-side alias (breaking existing explicit-path consumers, including the
`published-api-0.1.3` fixture) or renaming `eggup_core::CommandOutput` — the
latter is a `eggup-core` change, outside this milestone's ownership, and would
need its own republication.

**Finding 3 — `tests/commit_engine_policy.rs` has no baseline-failure proof.**
Severity: **low**. Reason: the file exercises the two *named* policy inputs,
`PostTransitionVerification` and `OwnershipLossTolerance`, which by definition
did not exist at the pre-M012 baseline, so the file cannot compile there. The
*baseline-observable half* of the same behaviour — "an ownership that is no
longer `Owned` is a typed failure by default" — is proved by
`tests/commit_engine_parity.rs`, which does compile and does fail at the
baseline. The permissive-input half is therefore new-API evidence, not
defect evidence, and is recorded as such rather than being presented as a
baseline failure.

**Finding 4 — the crate's byte-bound test name retains "256".**
Severity: **informational**. Reason:
`diagnostic_byte_bounds_preserve_utf8_at_256_and_512_edges` is named in
`.github/workflows/ci.yml:72` and in `architecture/tooling-governance.md:300`,
neither of which is in this milestone's ownership list. The name now describes a
generic boundary value exercised by the truncation helper rather than a crate
ceiling: the crate no longer declares a 256-byte ceiling. Renaming it requires
a coordinated CI and tooling-governance edit and is therefore recorded as a
follow-on. No behaviour depends on the name.

**Finding 5 — the `Ref` column of `architecture/service-lifecycle.md` is a
pointer layer.**
Severity: **informational**. Reason: the `Ref` values were re-derived for every
item this milestone moved, renamed, or added. The remaining refs are pointers
into files that other milestones are concurrently editing, so they will drift
again; the repo's `docs-hygiene` skill treats this table as an index layer over
the source, not as a second copy of the source.

**Finding 6 — no Linux or Windows native evidence, and no hosted CI lane
evidence.**
Severity: **medium**. Reason: the change is platform-neutral in its
implementation — the engine operates on the manager-neutral `ServiceManager`
trait and changes no adapter argv — but the Windows SCM backend is not
exercised on this host and plan §5 excludes Windows SCM execution coverage from
this milestone. The severity is medium rather than low because the two
Windows-relevant surfaces (`windows_scm.rs`'s `Ownership` usage and the
`Ownership` matching arms) were *not* natively exercised, and the crate's
`windows-check` lane is the only place that can prove them. The mitigation is
that `windows_scm.rs` was not modified at all and that
`eggup_core::Ownership` is not `#[non_exhaustive]`, so its existing exhaustive
matches remain exhaustive. The hosted lanes must still be run before this
milestone's work is relied upon outside this repository.

## Roadmap disposition

Service M012 is **complete on the source line** and is **ready for the parent's
workspace gate and hosted CI lanes**. It does not publish: the crate's
published version remains `0.1.3` and the `v0.1.3` tag is unmoved. The
roadmap status table row for M012 should read **closed** once the parent has
run the workspace gate and the hosted lanes, with the two behaviour
tightenings and the four follow-on findings carried forward.

Proposed status-table row for
`plans/subsystems/service-lifecycle-roadmap.md`:

> | M012 | Commit-engine deduplication corrective | closed | One engine (`commit_engine.rs`) for both orchestrators; canonical `Ownership` re-export + `CronOwnership` alias; `ManagerCommandOutput` rename with non-deprecated `CommandOutput` alias; one declared diagnostic ceiling (512 B). Two fail-closed tightenings (post-transition verification on by default; ownership-loss denial on by default). Recorded follow-ons: `SystemExecutor` not consolidated onto `eggup_core::run_bounded` (plan §6.6 R-E1); residual `CommandOutput` glob-import ambiguity; no Linux/Windows native or hosted-CI evidence. |

## Registry updates

The parent applies these edits; this session did not touch
`plans/registry.md` or any `plans/subsystems/*-roadmap.md`.

**Registry entry for the milestone.** Under the service-lifecycle subsystem
entries, add:

> - **M012 — commit-engine deduplication corrective (closed, source line; not
>   published).** `eggup-service` carried two divergent copies of the
>   destructive commit/rollback engine, one in `disposition.rs` and one in
>   `lifecycle_update.rs`. They are now one engine,
>   `crates/eggup-service/src/commit_engine.rs`, reached by both public
>   orchestrators. The two behavioural differences that used to separate the
>   copies are named, separately testable policy inputs on
>   `CommitEnginePolicy` (`PostTransitionVerification`, default `Verify`;
>   `OwnershipLossTolerance`, default `Deny`), both defaulting to the stricter
>   behaviour, with the selection observable in the returned receipt. The three
>   structurally identical ownership enums are now one type:
>   `eggup_service::Ownership` re-exports `eggup_core::Ownership` and
>   `eggup_service::CronOwnership` is a public alias of it. The service-side
>   `CommandOutput` is renamed `ManagerCommandOutput`, with the old spelling
>   retained as a non-deprecated type alias. Manager-output diagnostic text is
>   now bounded by one declared ceiling, `MAX_DIAGNOSTIC_BYTES` (512 bytes).
>   Two consumer-visible fail-closed tightenings: post-transition verification
>   is on by default, and an ownership that is no longer `Owned` is a typed
>   failure by default. No consumer migration is required for the type-identity
>   changes. Closure: `plans/closure/service-lifecycle/012-status.md`.

**Registry open-findings section.** Add the following entries to the
"Post-0.1.3 open findings" list (or extend the existing entries), leaving the
existing entries untouched:

> 6. **`SystemExecutor` is not consolidated onto `eggup_core::run_bounded`.**
>    Severity: low. The duplication is deliberate and plan-deferred (M012
>    §6.6 R-E1) because core's bounded-execution primitive is itself defective
>    until a separate core milestone closes. Follow-on for that milestone.
> 7. **Residual `CommandOutput` glob-import ambiguity between `eggup_core` and
>    `eggup_service`.** Severity: low. Required to be recorded by M012 §6.4
>    R-C4. Affects only a consumer that glob-imports both crates and then names
>    `CommandOutput` unqualified; it is a loud compile error, not a silent
>    mis-binding. Removing it requires either dropping the service-side alias
>    (breaking existing explicit-path consumers) or renaming the core type,
>    which needs its own republication.
> 8. **M012 has no Linux or Windows native evidence and no hosted CI lane
>    evidence.** Severity: medium. The change is platform-neutral in its
>    implementation and `windows_scm.rs` was not modified, but the Windows SCM
>    backend was not exercised on the authoring host and M012 §5 excludes
>    Windows SCM execution coverage from the milestone. The hosted `windows-check`
>    and `service-systemd` lanes must run before the change is relied upon
>    outside this repository.

**Registry "known API observations" section.** Extend the existing
`CleanupDisposition` entry's deprecation-policy note so it also covers the
`CommandOutput` alias: both aliases are deliberately **not** `#[deprecated]`,
because deprecation is a publication-policy decision owned by a later
republication milestone and a deprecated alias breaks downstream `-D warnings`
builds. Both dispositions are settled at the intentional 0.2/1.0 API boundary.

---

## Parent-session verification addendum (2026-10-23)

This record noted that the parent session owns the full workspace gate and that
`cargo test --workspace` and `./scripts/check-local.sh` were not run at this
stage. The parent session has since run both.

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 449 passed; 0 failed; 36 test binaries
```

The dependency surface is unchanged and still passes the boundary review:
`eggup-service` resolves to `eggup-core` and `windows-args` only, with
`windows-service` remaining a Windows-target dependency.

**Still not run:** Linux, Windows, and every hosted CI lane (`stable`, `msrv`,
`service-systemd`, `macos`, `windows-check`). No hosted runner is reachable from
this machine, so this milestone is qualified on the native macOS lane only.
