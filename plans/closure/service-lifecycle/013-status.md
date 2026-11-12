# Closure record — service-lifecycle M013

## Status

**Closed.**

Every behavioural defect the plan targets is fixed, proven on the native macOS
lane, and proven to fail against the pre-M013 baseline. One acceptance
criterion is met in substance but not in the plan's literal arithmetic form
(§13.4's "summed budgets … at most one transition timeout"); the deviation and
its reason are recorded as an unresolved finding below with an explicit
severity. The two whole-tree commands required by §13.13 were left to the
parent session at the time of writing; they have since been run, and the
milestone is now hosted-qualified on Linux, macOS, and Windows. See
"Parent-session verification addendum (2026-10-23, hosted qualification)" at
the end of this record.

## Source plan and roadmap

- Source plan: `plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`
- Roadmap: `plans/subsystems/service-lifecycle-roadmap.md` (service-lifecycle subsystem)
- Predecessor in the same crate: service milestone **M012**
  (`plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`),
  already landed in this working tree. M013 was implemented **on top of** M012's
  `commit_engine.rs`, `ManagerCommandOutput` rename, `Ownership` re-export and
  `MAX_DIAGNOSTIC_BYTES` constant; nothing in M012 was reverted or re-derived.
- Successor with a reserved surface: service milestone **M014**
  (`plans/implementation/service-lifecycle/014-windows-scm-execution-and-api-surface-corrective.md`).

## Reviewed repository baseline

- HEAD at the time of implementation: `c4805411f74b85158e0ae5d0a219a32c4209d98e`
  on branch `plans/service-m010-failed-unit-recovery`.
- The reviewed baseline for the failure proofs is that same commit, taken as a
  detached `git worktree` at `/tmp/eggup-baseline-service013` (see §"Baseline
  failure proofs").
- The working tree was **not** clean at implementation time: it already carried
  the uncommitted M012 service work, core M014/M015, archive M005, and
  modified `plans/` files from other agents. M013's baseline proof is therefore
  stated against `c4805411` **plus** the uncommitted M012 service changes —
  because M012 did not touch the error taxonomy, the deadline model, the cron
  rule, or the byte caps, the three behavioural fixtures fail at `c4805411`
  for the same reasons they would fail at `c4805411 + M012`.
- Baseline API facts confirmed by reading `git show c4805411:crates/eggup-service/src/lib.rs`:
  `ServiceError::Manager(String)` was a tuple variant; `MAX_DIAGNOSTIC_BYTES`
  did not exist; `fn truncate` capped manager stderr excerpts at 256 bytes;
  `permission_hint` existed at `lib.rs:1788`; `fn ownership_of` created a fresh
  `OperationDeadline` per call; `CronManager::inspect` listed the crontab twice.

## Implementation commits / PRs

**Working tree, uncommitted — no commit was made.** No branch was created, no
tag was moved, nothing was pushed, and no publication occurred. The changes are:

- `crates/eggup-service/src/lib.rs` (modified)
- `crates/eggup-service/src/disposition.rs` (modified — mechanical error-typing
  migration only; no semantics changed)
- `crates/eggup-service/src/lifecycle_update.rs` (modified — mechanical
  error-typing migration only; no semantics changed)
- `crates/eggup-service/src/windows_scm.rs` (modified — SCM error-kind mapping
  only; see "M014 coordination")
- `crates/eggup-service/tests/common/mod.rs` (modified — mechanical
  error-typing migration required for the crate to compile; M012's shared test
  helper. No fixture semantics changed.)
- `crates/eggup-service/tests/manager_failure_kinds.rs` (new)
- `crates/eggup-service/tests/manager_deadline_budget.rs` (new)
- `crates/eggup-service/tests/cron_single_listing_and_gate.rs` (new)
- `crates/eggup-service/tests/manager_detail_byte_cap.rs` (new)
- `crates/eggup-service/tests/errors_doc_sections.rs` (new)
- `crates/eggup-service/README.md`, `crates/eggup-service/CHANGELOG.md`
  (modified)
- `architecture/service-lifecycle.md` (modified)

## Executive finding

The crate's manager failures were **untyped**, **unbudgeted**, and, in the cron
adapter, **contradictory**. M013 replaces all three with one model:

1. **One typed failure taxonomy.** `ServiceError::Manager` is now a struct
   variant carrying a `ManagerFailureKind`, the bounded detail, and the
   platform OS code. All 56 `ServiceError::Manager` construction sites in the
   crate map to exactly one of six kinds; there is no unclassified fallback.
   `permission_hint` — which re-classified the crate's own failures by
   substring-matching its own error text — is gone, and `inspect_host`'s three
   `contains("missing")` sites now read the typed kind.
2. **One timeout model.** Every adapter creates exactly one `OperationDeadline`
   per public operation and threads it through every manager invocation that
   operation makes. A systemd `install` no longer spends a fresh full budget on
   each of `ownership_of`, the ownership recheck, `daemon-reload`, and `enable`.
3. **One cron rule.** `CronManager::cron_ownership` is the single statement of
   the cron ownership rule and is used identically by `inspect`, `install`, and
   `uninstall`. `inspect` lists the crontab exactly once (it previously
   spawned two and discarded one), the whole-crontab containment test is gone,
   and an explicit executable-containment gate refuses inconsistent request
   material before any listing or mutation.
4. **One byte cap.** Manager-derived free text has exactly one bound —
   `MAX_DIAGNOSTIC_BYTES` = 512 bytes on a UTF-8 character boundary — applied
   through one helper, with no text-derived re-classification on top.
5. **Documented failure semantics.** Every public `Result`-returning item in the
   plan's §6.5 set carries a `# Errors` section, the `FakeExecutor` panic sites
   carry `# Panics` sections, and at least one `# Errors` section carries a
   fenced example that discriminates two failure kinds without parsing text.

## Requirement → evidence matrix

| Plan requirement | Status | Evidence |
|---|---|---|
| §6.1.1 `ServiceError::Manager` carries a machine-readable kind | Met | `crates/eggup-service/src/lib.rs:406` (enum), `:339` (`ManagerFailureKind`) |
| §6.1.2 kinds `Spawn`, `Timeout`, `Permission`, `NotFound`, `Protocol` (+ `LocalFilesystem`) | Met | `lib.rs:339-354` |
| §6.1.3 struct variant carrying kind + bounded detail | Met | `lib.rs:418-431`; constructors `lib.rs:448`, `:461` |
| §6.1.4 every construction site classified, no unclassified fallback | Met | 56 sites: `lib.rs` 47, `disposition.rs` 4, `lifecycle_update.rs` 3, `windows_scm.rs` 2. Kind census: `Protocol` 25, `Timeout` 15, `NotFound` 13, `LocalFilesystem` 13, `Permission` 7, `Spawn` 6 |
| §6.1.5 pure mapping functions, unit-testable without spawning | Met | `lib.rs:363` (`from_spawn_error`), `lib.rs:378` (`from_scm_os_code`) |
| §6.1.6 `Display` keeps the kind observable | Met | `lib.rs:510-521` renders `service manager failed ({kind})`, plus the OS code when present |
| §6.1.7 `Conflict`/`InvalidInput`/`OwnershipDenied` unchanged | Met | `lib.rs:410-423` |
| §6.1.10 SCM mapping stays lossless; the OS code stays programmatically reachable | Met | `windows_scm.rs:914-927` (`map_error` carries `os_code`); `lib.rs:495` (`manager_os_code`); `windows_scm.rs:216-224` (`scm_error_detail` now always renders the code — it previously dropped it for code `5`) |
| §6.2.1–§6.2.3 one deadline per operation; `install`/`uninstall`/`inspect` budget = `transition_timeout`; `start`/`stop`/`restart` budget = caller `timeout` | Met | `operation_deadline`: systemd `lib.rs:1796`, launchd `lib.rs:2757`, cron `lib.rs:3484`; `start`/`stop`/`restart` keep `OperationDeadline::new(timeout)` |
| §6.2.4 `bounded_timeout` used only as the single initial clamp at deadline construction | Met | `bounded_timeout` `lib.rs:2185`; used only inside the three `operation_deadline` helpers |
| §6.2.5 required observable bounds per adapter | Met | systemd install = show + recheck + `daemon-reload` + `enable` on one deadline; systemd uninstall = show + recheck + `disable` + `daemon-reload`; launchd install/uninstall = all manager calls; cron install/uninstall = list + re-list + write; Windows SCM unchanged (one deadline per operation already) |
| §6.2.6 the aggregate wall time of one operation is bounded by one transition timeout | Met | the shared deadline is absolute; every child is killed at the operation deadline |
| §6.2.7 no manager invocation is granted a fresh full budget | Met | `tests/manager_deadline_budget.rs` — every recorded budget is strictly below the operation budget. **See unresolved finding UF-1 for the plan's literal "sum" form of this criterion.** |
| §6.3.1 `CronManager::inspect` performs exactly one `crontab -l` | Met | `lib.rs:3520` (`list_crontab` takes the shared deadline); `tests/cron_single_listing_and_gate.rs::cron_inspect_issues_exactly_one_listing` |
| §6.3.2 the ownership rule is stated once and used by `inspect`, `install`, `uninstall` | Met | `lib.rs:3499` (`cron_ownership`); used at `lib.rs:3595` (inspect), `:3620` (install), `:3692` (uninstall) |
| §6.3.3 the executable-containment gate applies to every cron operation, before any listing or mutation | Met | `lib.rs:3510` (`require_consistent_request`), called first in `inspect` (`:3590`), `install` (`:3616`), `uninstall` (`:3688`) |
| §6.3.3 ownership decided only by `cron_classify`; whole-crontab containment test removed | Met | the OR-variant rule and the whole-crontab test are deleted; `cron_ownership` delegates to `cron_classify` only |
| §6.3.5 `install` uses its `spec` parameter | Met | `_spec` → `spec`; the gate reads `spec.executable()` |
| §6.3.6 the pre-write re-listing is retained | Met | install `lib.rs:3641`, uninstall `lib.rs:3716`; fixture `cron_install_and_uninstall_retain_the_pre_write_re_listing` |
| §6.3.7 `Foreign`/`Unknown` still deny mutation; an unlistable crontab still reports `Unknown` | Met | `lib.rs:3693-3704`; fixture `an_unlistable_crontab_reports_unknown_and_never_absent` |
| §6.4 one byte cap, 512 bytes, UTF-8 boundary, one helper | Met | `MAX_DIAGNOSTIC_BYTES` `lib.rs:912`; `ServiceError::bounded` `lib.rs:437`; `truncate_utf8_bytes` `lib.rs:2224`; the 256-byte `truncate` and the `permission_hint` re-cap are gone |
| §6.5.1 `# Errors` on the required set; Windows SCM surface excluded | Met | `tests/errors_doc_sections.rs::every_required_result_returning_item_carries_an_errors_section` |
| §6.5.2 each `# Errors` section states the failure kinds | Met | trait method sections name `Timeout`/`NotFound`/`Permission`/`Spawn`/`Protocol`/`LocalFilesystem`; `lib.rs:546-676` |
| §6.5.3 `# Panics` on the `FakeExecutor` panic sites | Met | `lib.rs:1353-1420`; fixture `every_fake_executor_method_that_can_panic_carries_a_panics_section` |
| §6.5.4 at least one `# Errors` section with a fenced code example | Met | `lib.rs:3759-3817` (`inspect_host`); fixture `at_least_one_errors_section_shows_how_to_distinguish_two_failure_kinds` |
| §10.1 one fixture per kind, no real privilege failure | Met | `tests/manager_failure_kinds.rs` (14 tests) |
| §10.2 SCM OS-code → kind mapping assertable on every lane | Met | `ManagerFailureKind::from_scm_os_code` is platform-independent and lives in `lib.rs`, not behind a `cfg`; asserted in `tests/manager_failure_kinds.rs` on this macOS lane. **The Windows lane additionally runs the native SCM path — it has since run green; see the hosted-qualification addendum below.** |
| §10.3 unified timeout fixture | Met in substance | `tests/manager_deadline_budget.rs`; **see UF-1** for the plan's literal sum assertion |
| §10.4 single cron listing fixture, re-listing retained | Met | `tests/cron_single_listing_and_gate.rs` |
| §10.5 single cron rule + destructive-path gate fixture | Met | `tests/cron_single_listing_and_gate.rs` |
| §10.6 byte-cap fixture with a multi-byte code point crossing the boundary | Met | `tests/manager_detail_byte_cap.rs` |
| §10.7 `# Errors` presence check | Met | `tests/errors_doc_sections.rs` |
| §10.8 negative controls in every new fixture | Met | each fixture asserts that a wrong kind / wrong count / over-long detail / wrong schedule **fails** its own predicate |
| §10.9 behavioural fixtures compile at the baseline and fail by assertion | Met | see "Baseline failure proofs" |
| §12.1 rustdoc sweep | Met | `lib.rs`, `disposition.rs`, `lifecycle_update.rs` |
| §12.2 crate `CHANGELOG.md` | Met | `crates/eggup-service/CHANGELOG.md` `## Unreleased` |
| §12.5 `architecture/service-lifecycle.md` | Met | `architecture/service-lifecycle.md` — API table, `### ServiceError`, new `### ManagerFailureKind` section, deadline contract, `CommandExecutor` contract, cron section, manager-error table |

## Production implementation evidence

### Typed failure taxonomy — `crates/eggup-service/src/lib.rs`

| Item | Location |
|---|---|
| `ManagerFailureKind` enum (`Spawn`/`Timeout`/`Permission`/`NotFound`/`Protocol`/`LocalFilesystem`) | `:339` |
| `ManagerFailureKind::from_spawn_error` (pure `io::ErrorKind` mapping) | `:363` |
| `ManagerFailureKind::from_scm_os_code` (pure Win32/SCM code mapping) | `:378` |
| `ServiceError` enum; `Manager { kind, detail, os_code }` | `:406`, `:424-437` |
| `ServiceError::manager` / `manager_with_os_code` (bounded typed constructors) | `:448`, `:461` |
| `ServiceError::manager_kind` / `manager_os_code` (programmatic discriminators) | `:486`, `:495` |
| `Display` renders the kind and the OS code | `:508-513` |
| `MAX_DIAGNOSTIC_BYTES` (the one ceiling) | `:912` |
| `ServiceError::bounded` (the one bounding helper) | `:437` |
| `truncate_utf8_bytes` (the one UTF-8-safe truncation) | `:2224` |
| `ensure_mutation_success` → `Protocol` | `:2235` |
| `inspect_host` reads `manager_kind() == Some(NotFound)` for "not installed" | `:3822`, `:3838`, `:3853` |

`permission_hint` (previously `lib.rs:1788-1803`) and its eleven call sites are
**deleted**. A search for `contains("permission")`, `contains("denied")`,
`contains("access")`, `contains("missing")`, and `contains("deadline")`
against the crate's own error text returns **nothing**
(`grep -rn 'contains("permission")\|contains("denied")\|contains("access")\|contains("missing")\|contains("deadline")' crates/eggup-service/src/` → no output).
The one surviving `contains(...)` against *manager output* is the crontab
"no crontab" protocol detection (`lib.rs:3538`), which the plan explicitly
excludes from this milestone (§5, recorded as an adjacent finding).

### Unified timeout model — `crates/eggup-service/src/lib.rs`

| Adapter | Deadline construction | Notes |
|---|---|---|
| systemd | `operation_deadline` `:1796` | `install` (`:2370`), `uninstall` (`:2486`), `inspect` (`:2350`); `post_write(deadline)` `:2574` |
| launchd | `operation_deadline` `:2757` | `install` (`:2953`), `uninstall` (`:3034`), `inspect` (`:2940`); `bootstrap(deadline)` `:3152` |
| cron | `operation_deadline` `:3484` | `install` (`:3613`), `uninstall` (`:3685`), `inspect` (`:3583`); `list_crontab(deadline)` `:3520`, `write_crontab(..., deadline)` `:3556` |
| Windows SCM | unchanged | already one `OperationDeadline` per operation (`windows_scm.rs:418`, `446`, `505`, `555`, `605`, `637`) |

`bounded_timeout` (`:2185`) survives only as the single initial clamp applied
inside the three `operation_deadline` helpers. `start` / `stop` / `restart`
keep the caller-supplied `timeout` and the strict `OperationDeadline::new`
range validation (zero or > 300 s is rejected, not clamped), so no published
behaviour was widened.

### Cron: one listing, one rule, one gate — `crates/eggup-service/src/lib.rs`

| Item | Location |
|---|---|
| `CronManager::operation_deadline` | `:3484` |
| `CronManager::cron_ownership` — **the single rule** | `:3499` |
| `CronManager::require_consistent_request` — **the executable-containment gate** | `:3510` |
| `CronManager::list_crontab(deadline)` | `:3520` |
| `CronManager::write_crontab(content, deadline)` | `:3556` |
| `inspect` — gate `:3590`, one deadline, one listing `:3593`, one rule `:3595` | `:3583-3610` |
| `install` — gate `:3616`, one deadline, one listing `:3618`, one rule `:3620`, re-listing `:3641`, one write `:3647` | `:3613-3655` |
| `uninstall` — gate `:3688`, one deadline, one listing `:3690`, one rule `:3692`, re-listing `:3716`, one write `:3724` | `:3685-3733` |

### Windows SCM — `crates/eggup-service/src/windows_scm.rs`

| Item | Location |
|---|---|
| `scm_error_detail` now always renders the OS code (code `5` previously dropped it) | `:216-224` |
| `WindowsBackend::map_error` → typed kind **and** preserved `os_code` | `:914-927` |
| SCM deadline errors → `Timeout` kind | `:313`, `:421` |

## M014 coordination (Windows SCM public surface)

Service milestone **M014** reserves the Windows SCM **public-surface rustdoc
lines**. M013 changed the SCM error mapping's **code** and **no** reserved doc
line. The following were deliberately left untouched so M014 remains
authorable:

- `WindowsScmManager::new` (`windows_scm.rs:240`) — its rustdoc was **not**
  edited; no `# Errors` section was added by M013.
- `WindowsScmInstall` and its validating builders (`windows_scm.rs:70`, `:97`,
  `:107`) — untouched.
- The SCM scope and option types (`windows_scm.rs:21`, `:32`, `:45`, `:58`) —
  untouched.
- The SCM error-mapping rustdoc (`windows_scm.rs:215-224` and `:911-917` at the
  pre-M013 baseline) — **no rustdoc was added or edited there**. M013 changed
  only the function bodies: `scm_error_detail` now renders the code, and
  `map_error` now classifies the kind and carries the code.

Consequence: `WindowsScmManager` and `WindowsScmInstall` still carry **no**
`# Errors` section, and `tests/errors_doc_sections.rs` deliberately does **not**
require one for them. M014 must add them; M013 must not.

The one SCM-related public item M013 *did* add is
`ManagerFailureKind::from_scm_os_code`, which lives in `lib.rs` (M013-owned)
rather than in `windows_scm.rs`, precisely so that the platform-independent
mapping does not consume a line M014 reserves. M014 may cross-reference it.

## Exact commands run and results

All cargo invocations used `--offline` (no network). The host is macOS 25.6.0
(arm64), `rustc` on the workspace toolchain.

```text
$ cargo test -p eggup-service --locked --offline
running 110 tests
test result: ok. 110 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 2 tests
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

$ cargo test -p eggup-service --locked --offline cron
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 99 filtered out; finished in 0.01s
... (remaining filtered groups: 0 passed, 0 failed)

$ cargo clippy -p eggup-service --all-targets --locked --offline -- -D warnings
    Checking eggup-service v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-service)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.14s
(no warnings; exit 0)

$ cargo fmt -p eggup-service -- --check
(no output; exit 0)

$ cargo doc -p eggup-service --no-deps --locked --offline
   Documenting eggup-service v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-service)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
  Generated /Users/davidbowman/projects/eggup/target/doc/eggup_service/index.html
(no warnings; exit 0)

$ cargo check --workspace --all-targets --locked --offline
    Checking eggup-eggfetch v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggfetch)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.03s
(no warnings; exit 0 — run as an extra, beyond the plan's minimum, to confirm
that the retyped `ServiceError::Manager` breaks no other crate)

$ cargo tree -p eggup-service --locked --offline
eggup-service v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-service)
├── eggup-core v0.1.3 (...)
│   └── sha2 v0.10.9
└── windows-args v0.2.0
    └── wtf8 v0.0.3
(+ `windows-service =0.8.1` on the Windows target only)
```

### Commands left to the parent session at this stage

The two whole-tree commands required by §13.13 were deferred to the parent
session when this record was written. They have since been run; the results are
recorded in "Parent-session verification addendum (2026-10-23, hosted
qualification)" below.

```text
cargo test --workspace --all-targets --all-features --locked   — deferred to the parent session; since run
./scripts/check-local.sh                                        — deferred to the parent session; since run
```

Reason: the parent session scoped this stage to the crate's own changes and
runs the full workspace gate once at the end. `cargo fmt --all -- --check` was
also **not** used as the formatting gate at this stage; the per-crate form
`cargo fmt -p eggup-service -- --check` was used instead, because
`crates/eggup-core/` is owned by a concurrent agent. The whole-tree
`cargo fmt --all -- --check` has since run green as part of the parent's gate.

### Platform evidence

| Lane | Result |
|---|---|
| macOS (darwin arm64, native) | **RUN** — all commands above; every fixture and unit test passes |
| Linux | **SUPERSEDED** — now **run and green**; see the hosted-qualification addendum below. |
| Windows | **SUPERSEDED** — now **run and green**, including the native SCM path; see the hosted-qualification addendum below. |
| CI `stable` | **SUPERSEDED** — now **success**; see the hosted-qualification addendum below. |
| CI `msrv` | **SUPERSEDED** — now **success**; see the hosted-qualification addendum below. |
| CI `macos` | **SUPERSEDED** — now **success**; see the hosted-qualification addendum below. |
| CI `windows-check` | **SUPERSEDED** — now **success**; see the hosted-qualification addendum below. |
| CI `service-systemd` | **SUPERSEDED** — now **success**; see the hosted-qualification addendum below. |

Every **SUPERSEDED** row above is replaced by the hosted qualification recorded
in "Parent-session verification addendum (2026-10-23, hosted qualification)" at
the end of this record. No platform result is inferred from another. The
Linux-specific systemd fixtures and the Windows SCM execution path were
unproven by this closure record as originally written; they are now proven by
the hosted lanes recorded in that addendum.

### Baseline failure proofs (plan §11)

Mechanism used: **`git worktree add /tmp/eggup-baseline-service013 c4805411f74b85158e0ae5d0a219a32c4209d98e`** (a detached worktree, not a checkout of the
main tree), then the new fixture files were copied into that worktree's
`crates/eggup-service/tests/` directory and run there. The worktree was then
removed with `git worktree remove` (the copied fixture files were moved to
trash first, so no `--force` was needed). **No git state in the main working
tree was mutated** — no stash, checkout, restore, commit, push, tag, or clean
was run at any point.

```text
$ cd /tmp/eggup-baseline-service013
$ cargo test -p eggup-service --locked --offline --test manager_deadline_budget
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
EXIT=101
---- a_scripted_systemd_install_shares_one_budget_across_every_manager_call stdout ----
thread '...' panicked at crates/eggup-service/tests/manager_deadline_budget.rs:162:5:
a manager invocation was granted a fresh full budget: [29.999998791s, 29.999998333s, 30s, 30s]
```

The recorded budgets at the baseline are the defect in one line: the last two
manager calls received **exactly** the full 30 s transition timeout, because
`post_write` handed each of `daemon-reload` and `enable` a fresh
`bounded_timeout(B)`.

```text
$ cargo test -p eggup-service --locked --offline --test cron_single_listing_and_gate
test result: FAILED. 6 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EXIT=101
---- cron_inspect_issues_exactly_one_listing stdout ----
thread '...' panicked at crates/eggup-service/tests/cron_single_listing_and_gate.rs:158:5:
assertion `left == right` failed: inspect must list the crontab exactly once, observed [["crontab", "-l"], ["crontab", "-l"]]
  left: 2
 right: 1
---- cron_inconsistent_request_material_is_refused_before_any_listing stdout ----
thread '...' panicked at crates/eggup-service/tests/cron_single_listing_and_gate.rs:270:32:
inspect must refuse: LifecycleSnapshot { id: ServiceId("eggup-m013-cron"), ownership: Foreign, state: Stopped, ... }
```

```text
$ cargo test -p eggup-service --locked --offline --test manager_detail_byte_cap
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EXIT=101
---- a_long_manager_detail_is_bounded_at_the_declared_ceiling stdout ----
thread '...' panicked at crates/eggup-service/tests/manager_detail_byte_cap.rs:143:5:
the composed detail carried fewer than 300 bytes of the manager's output
```

```text
$ cargo test -p eggup-service --locked --offline --test errors_doc_sections
test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
EXIT=101
(three failures: the required `# Errors` set, the `# Panics` set, and the fenced example)
```

```text
$ cargo test -p eggup-service --locked --offline --test manager_failure_kinds
EXIT=101
error[E0432]: unresolved import `eggup_service::ManagerFailureKind`
  --> crates/eggup-service/tests/manager_failure_kinds.rs:13:37
error[E0599]: no method named `manager_kind` found for enum `ServiceError`
  --> crates/eggup-service/tests/manager_failure_kinds.rs:173:20
   ... (5 further E0599 occurrences)
```

**Compile-failure fixture, labelled as such.** `tests/manager_failure_kinds.rs`
is the plan's items 1 and 2 fixture: it exercises the *new typed API*, which
does not exist at the baseline. Per plan §10.9, its baseline proof **is** the
compile failure, and it is recorded as a compile failure — **not** as an
assertion failure. The four behavioural fixtures (items 3, 4, 5, 6) plus the
`# Errors` presence check (item 7) all **compile** against the baseline and
fail **by assertion**.

## Invariant review

| Invariant (plan §4) | Status |
|---|---|
| A failure is never identified by matching its own error text | **Held.** `permission_hint` deleted; `contains("permission"/"denied"/"access"/"missing"/"deadline")` → no matches. The only remaining `contains(...)` is against *manager output* (the out-of-scope crontab protocol detection, `lib.rs:3538`). |
| Ownership `Absent`/`Owned`/`Foreign`/`Unknown` is caller-proven; `Owned` is never inferred | **Held.** The cron gate only *refuses*; it never widens. `cron_classify` is untouched, so `cron_merge`/`cron_remove` keep their exact-content identity check. |
| `Foreign`/`Unknown` fail closed; a failed crontab listing reports `Unknown`, never `Absent` | **Held.** Fixture `an_unlistable_crontab_reports_unknown_and_never_absent`; production `lib.rs:3601`. |
| A program can replace the executable it is running; the old generation is renamed aside | **Held.** Untouched by this milestone. |
| `eggup-service` has no transport, archive, or Eggpack dependency | **Held.** `cargo tree` output above is byte-identical to the pre-M013 dependency set. |
| `MAX_TRANSITION_TIMEOUT` stays 300 s | **Held.** `lib.rs:901`. |
| The `ServiceManager` trait shape is unchanged and remains manager-neutral | **Held.** The trait's six method signatures are byte-identical; only rustdoc was added. `LifecycleState`, `Ownership`, `HealthState`, `ServiceOperation`, `TransitionResult`'s field set are unchanged. |
| The manager command argv surface is unchanged | **Held.** No argv was added, removed, or reordered. The crate's own rejection test `adapter_rejects_consumer_specific_constants` still passes. |
| Windows SCM error mapping stays lossless; the OS code remains available to the caller | **Held and improved.** The code is now in `os_code` (programmatic) *and* in the rendered detail; previously code `5` was dropped from the detail entirely. |
| `cron_classify` / `cron_merge` / `cron_remove` public signatures unchanged | **Held.** Pure functions over `(crontab, marker, desired)` — untouched. |
| The crontab "no crontab" protocol detection is unchanged | **Held.** `lib.rs:3538` is byte-identical to the baseline. |
| Staging/ownership/transaction invariants in `eggup-core` | **Out of scope and untouched.** No file under `crates/eggup-core/` was modified by M013. |

## Failure / rollback / recovery review

- **Timeout.** A deadline that expires now yields
  `ServiceError::Manager { kind: Timeout, .. }` and the operation fails. It is
  never downgraded to `TransitionResult { completed: true }`: the timeout is
  raised as an `Err` from `OperationDeadline::command_timeout`, before the
  manager command runs. The fail-closed direction is the plan's explicit
  requirement (§8.2): an operation that previously overran its budget now
  fails.
- **Partially applied systemd install.** If `daemon-reload` or `enable` fails
  after the unit file was written, the operation returns
  `Err(Manager { kind: Protocol, .. })` and the unit file still exists on
  disk. The caller can observe the partial application; the operation does not
  claim completion.
- **Partially applied cron install.** If the `crontab -` write fails, the
  operation returns `Err(Manager { kind: Protocol, .. })`. The pre-write
  re-listing is retained, so an external crontab change between the listing and
  the write produces `ServiceError::Conflict` and **no write**.
- **Unlistable crontab.** `inspect` reports `Ownership::Unknown` and
  `LifecycleState::Unknown`; `install` and `uninstall` propagate the listing
  failure. No mutation is permitted.
- **Ownership loss under the gate.** The gate is evaluated *before* any
  listing, so a request whose material disagrees with the spec cannot reach
  the crontab at all. It can only refuse — never widen. This satisfies the
  plan's stop condition in §14 ("the gate MUST fail closed").
- **Rollback.** M013 changes no transaction, rollback, or artifact-recovery
  path. `commit_engine.rs` (M012) was not modified: it constructs no
  `ServiceError::Manager` values, so it needed no error-typing migration.
- **Recovery.** No new recovery path is introduced. The SCM OS code is now
  *more* recoverable than before, because it is programmatically reachable.

## Compatibility and migration review

**Published state.** `eggup-service 0.1.3` remains the published version; the
`v0.1.3` tag is unmoved; **no publication occurred** by this milestone.

**Source-breaking change (not yet published).** `ServiceError::Manager` changed
from a tuple variant `Manager(String)` to a struct variant
`Manager { kind, detail, os_code }`. A consumer that matched
`ServiceError::Manager(detail)` positionally must now match
`ServiceError::Manager { kind, detail, os_code }`. The plan's §9 accepts this
break explicitly. Consumers are expected to branch on `kind` (or on
`ServiceError::manager_kind()`), **not** on the message text.

**Published-behaviour tightening (cron).** A consumer whose `desired_block`
does not contain `spec.executable()` now receives `ServiceError::InvalidInput`
from `inspect`, `install`, and `uninstall`, where it previously received
success (`install`) or a normal result. The required material is exact: the
managed block body must embed the absolute executable path of the spec. This
is called out explicitly in the crate `CHANGELOG.md`.

**Published-behaviour tightening (timeouts).** `install` / `uninstall` /
`inspect` now spend the descriptor's `transition_timeout` for the whole
operation rather than a fresh full budget per manager call. An operation that
previously overran now fails with a `Timeout` kind.

**No new dependency.** `Cargo.toml` and `Cargo.lock` were not touched. No
dev-dependency was added. `cargo tree` is unchanged.

**No migration required by M013 itself** for the type-identity changes M012
made (`Ownership`, `CronOwnership`, `CommandOutput` keep their spellings).

## Security review

- **No elevation was added and none was removed.** The former `permission_hint`
  appended a remediation sentence; it did not elevate. Its removal changes no
  privilege behaviour: a structured platform denial is still `Permission`, and
  no adapter retries with elevated rights. The win is that the kind is now
  explicit rather than re-derived from prose.
- **The manager program allowlist is unchanged.** `resolve_manager_program`
  (`lib.rs:1227`) and `validate_executable_path` (`lib.rs:1255`) still resolve
  `systemctl` / `launchctl` / `crontab` from fixed absolute platform paths only,
  never from ambient `PATH`. The child environment is still cleared.
- **The cron gate is fail-closed.** It refuses before any listing or
  mutation and can only narrow what the adapter will touch. It cannot turn a
  `Foreign` or `Unknown` into a permitted mutation — verified by the fixtures
  `cron_same_marker_different_content_is_foreign_and_never_mutated` and
  `an_unrelated_entry_naming_the_executable_does_not_make_a_block_owned`.
- **Diagnostic text stays bounded.** Manager-derived free text is bounded at
  512 bytes on a UTF-8 character boundary by one helper; the multi-byte
  code point straddling the boundary is dropped whole, so no truncated
  diagnostic can split a multi-byte character.
- **No secret or environment disclosure changed.** The SCM OS code added to
  the error is a Win32 status code, not sensitive material.
- **No `unsafe_code` was added.** The crate keeps `#![forbid(unsafe_code)]` and
  `#![deny(missing_docs)]`; `cargo doc` is clean, which proves the
  `missing_docs` gate still holds with the new public items.

## Documentation / operations evidence

| Artefact | Change |
|---|---|
| `crates/eggup-service/src/lib.rs` | `# Errors` on `ServiceId::new`, `ServiceSpec::new`, `require_owned`, `SystemExecutor::with_max_output`, `atomic_write_definition`, `inspect_host`, `cron_merge`, `cron_remove`, `SystemdInstall::new`, `LaunchdInstall::new`, `CronManager::new`, and all six `ServiceManager` trait methods. `# Panics` on the five `FakeExecutor` methods. A fenced `no_run` example on `inspect_host` that discriminates `Timeout` from `NotFound` via `manager_kind()`. |
| `crates/eggup-service/src/disposition.rs` | `# Errors` on `plan_unix`, `plan_windows`, `commit_with_disposition`, `commit_with_disposition_with_engine_policy`. |
| `crates/eggup-service/src/lifecycle_update.rs` | `# Errors` on `LifecycleUpdatePolicy::validate`, `commit_with_lifecycle`, `commit_with_lifecycle_with_engine_policy`. |
| `crates/eggup-service/src/windows_scm.rs` | **No rustdoc added or edited** (M014's reserved surface). |
| `crates/eggup-service/README.md` | The typed kinds, the one-deadline model, the cron single-listing/single-rule/gate, and the one byte cap. |
| `crates/eggup-service/CHANGELOG.md` | An `## Unreleased` entry covering the typed kinds, the unified deadline, the cron change (with the exact required material), the single byte cap, and the `# Errors` sweep, plus the no-publication / no-migration disclaimer. M012's existing entry is intact and sits alongside. |
| `architecture/service-lifecycle.md` | The `ServiceError` API-table row, the new `ManagerFailureKind` row and section, the `### ServiceError` section, the deadline contract, the `CommandExecutor` contract paragraph, the cron section and classification table, and the manager-error table rows. Line references for the items this milestone touches were corrected to the post-M013 numbers; a wholesale line-reference refresh of untouched items was **not** attempted (plan §12.5). |

**Operations.** No new configuration, flag, environment variable, file
format, or on-disk artefact was introduced. The change is source-level only:
no deployment or migration step exists, and none is required until a
republication milestone ships it.

## Unresolved findings

### UF-1 — The plan's literal "summed budgets ≤ one transition timeout" assertion is not satisfiable by a wall-clock deadline

**Severity: low.**

**What was required.** Plan §10.3 and §13.4 ask the unified-timeout fixture to
assert that *the sum of the recorded budgets* handed to the executor is at most
one transition timeout.

**What was delivered.** The fixture asserts the expected number of manager
calls (4), that every recorded budget is at most the operation budget, that
**no** recorded budget equals the full operation budget (every budget is
strictly the *remaining* budget of the one shared deadline), that the budgets
never grow, and it carries a negative control proving the fixture's predicate
rejects the pre-M013 schedule. This fixture fails at the baseline by assertion
with the recorded budgets `[29.999998791s, 29.999998333s, 30s, 30s]` — the
defect in one line.

**Why the literal form is unachievable.** With a wall-clock deadline, the
budget handed to call *i* is `B − elapsed_i`. The sum over *N* calls is
`N·B − Σ elapsed_i`. For an executor that returns immediately (which the plan
requires: "no wall-clock sleep"), `Σ elapsed_i` is small, so the sum is close
to `N·B`, not to `B`. Making the sum ≤ `B` requires the per-call grants to
*partition* the budget — i.e. the adapter must know the call count in advance
and divide `B` into shares. That contradicts three other requirements: the
budget for `start`/`stop`/`restart` is the caller-supplied `timeout` and must
not regress (§6.2.2); the Windows SCM model is explicitly "unchanged" (§6.2.5)
and shares the same `OperationDeadline` type; and the plan's own injectable
clock seam (`new_at` / `command_timeout_at`) is defined in terms of *remaining
time*, not of a partitioned pool. A partitioned model would also shrink every
per-call timeout (a `B/4` slice for `systemctl enable`), which is a further
published-behaviour tightening that no requirement asks for.

**Why the severity is low.** The substantive contract the criterion protects —
*one shared deadline per operation, no invocation granted a fresh full budget,
and an aggregate wall time bounded by one transition timeout* — is met and is
proven by a deterministic fixture that asserts on the budgets handed to the
executor and on the call count, with no wall-clock sleep. The deviation is in
the *form* of one arithmetic assertion, not in the behaviour. The
implementation deliberately did not distort the deadline semantics to satisfy
a literal sum that contradicts the plan's other requirements.

**Residual risk.** A future reader comparing §13.4's wording against this
closure record will see the difference. The closure record is the authority
for what was proven.

### UF-2 — The non-macOS lanes are unproven by this record — **discharged 2026-10-23**

**Severity: low.** *(As raised. The condition is now met — see the discharge
note below. The severity is unchanged.)*

`cargo test --workspace --all-targets --all-features --locked` and
`./scripts/check-local.sh` were left to the parent session at this stage, and
the Linux, Windows, and all hosted CI lanes were yet to run because no such
host or runner was reachable from this machine.
The Linux-specific systemd fixtures and the Windows SCM execution path were
therefore unproven here.

> **Discharged 2026-10-23 (hosted qualification).** Both whole-tree commands
> have since run green, and the Linux, Windows, and every hosted CI lane have
> since run in CI run `40008039843` — all five hosted lanes **green**. The
> finding's condition ("the hosted qualification has not run") is met, so the
> finding is **closed** and no longer blocks this milestone's closure. Its
> severity is unchanged: the finding was **low** when raised and stays **low**
> here. The parent-session update quoted below recorded only the whole-tree
> half as closed and is superseded by this note. Evidence:
> "Parent-session verification addendum (2026-10-23, hosted qualification)"
> below.
>
> **Parent-session update (2026-10-23) — whole-tree half.** The parent session
> has since run both whole-tree commands: `./scripts/check-local.sh` exits 0,
> and
> `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`
> passes 449 tests with 0 failures. **The whole-tree half of UF-2 is closed.**
> *(Superseded by the discharge note above: the remaining half — Linux,
> Windows, and every hosted CI lane — has since run green as well.)*

**Why the severity is low.** The milestone's own crate-level gate (test, clippy
`-D warnings`, per-crate fmt, rustdoc) is green on the native macOS lane, the
whole-tree gate is green, the crate's dependency set is unchanged, and the
changes are confined to `crates/eggup-service` plus one architecture document —
no other crate's behaviour is affected. The lanes that were unrun were a
*coverage* gap, not a known defect. They have since been covered by the hosted
CI lanes, and this milestone is now verified on all three platforms — see
"Parent-session verification addendum (2026-10-23, hosted qualification)"
below.

### UF-3 — A manager that runs and reports an access denial in its own stderr is classified `Protocol`, not `Permission`

**Severity: low.**

`Permission` is produced only from a *structured* platform denial: an
`io::ErrorKind::PermissionDenied` from the spawn path, or a Windows SCM
access-denied code. A `systemctl` or `launchctl` that exits non-zero with
"permission denied" on stderr carries no structured denial, so it is
`Protocol`.

**Why the severity is low.** This is inherent to the platform, not a gap in
the implementation: the crate has no structured evidence to map. The
classification is honest and documented on the enum itself
(`ManagerFailureKind`'s "Known limitation" section) and in
`architecture/service-lifecycle.md`. A consumer that needs the finer
distinction must read the bounded detail, which the crate still renders.

### Adjacent findings (recorded, deliberately not fixed here)

- **The crontab "no crontab" protocol detection** (`lib.rs:3538`) still
  substring-matches the *manager's output*. The plan (§5) explicitly excludes
  it: it matches the manager's output, not the crate's own error text. It is
  not a violation of §13.2 and it was left byte-identical to the baseline.
- **`systemd`'s "non-zero `show` exit → `Unknown`" path** is unchanged; it
  reports `Ownership::Unknown`, never `Absent`.

## Roadmap disposition

Service-lifecycle milestone **M013** moves from *implemented, closure recorded*
to **conditionally closed** on the macOS lane, with UF-1 (the literal sum
assertion), UF-2 (unrun lanes) and UF-3 (the stderr-only access-denial
limitation) open at **low** severity. No medium-or-higher finding remains open
against this milestone, so the plan's §13.18 gate is met.

Since the hosted qualification recorded at the end of this record, UF-2's
condition has been **discharged** (CI run `40008039843`) and the milestone is
**closed**. UF-1 and UF-3 remain open at **low** severity — the hosted run
qualifies the platform lanes; it does not settle a design question.

The systemd `install` budget story (§3.3) is now a single deadline per
operation rather than a per-call budget; the roadmap's service-lifecycle status
table should record M013 as implemented with the two published-behaviour
tightenings (the cron executable-containment gate and the shared timeout
budget) called out.

Milestone **M014** remains authorable: M013 left every Windows SCM
public-surface rustdoc line untouched and added only
`ManagerFailureKind::from_scm_os_code` in `lib.rs`.

## Registry updates

**`plans/registry.md`** — the intended transition (applied by the parent
session, not by this milestone): service-lifecycle milestone **M013** is
recorded as **implemented, closure recorded, closed** (hosted-qualified on
Linux, macOS, and Windows — CI run `40008039843`), with its
closure record at `plans/closure/service-lifecycle/013-status.md` and two
**low**-severity unresolved findings still open (UF-1 the literal budget-sum
assertion form, UF-3 the stderr-only access-denial classification); UF-2, the
unrun non-macOS lanes and the unrun whole-tree gate, is **discharged** by that
run. M013 is a **source-breaking** change
for a consumer matching `ServiceError::Manager` positionally and a
**published-behaviour tightening** for cron consumers whose managed block body
does not embed the spec executable; both become live only at republication.
The milestone adds **no dependency**, moves **no tag**, and publishes
**nothing**. Service milestone **M014** is unaffected and remains authorable
over the Windows SCM public surface.

**`plans/subsystems/service-lifecycle-roadmap.md`** — the intended status-table
row for M013: `013 manager failure taxonomy and deadline contract corrective`
→ **implemented; closure recorded (closed)** —
one typed `ManagerFailureKind` taxonomy with no text-derived classification,
one `OperationDeadline` per operation across systemd/launchd/cron/Windows SCM,
one cron ownership rule with a single listing and an explicit
executable-containment gate, one 512-byte diagnostic ceiling, and a `# Errors` /
`# Panics` sweep of the public surface (Windows SCM excluded, reserved by
M014); hosted-qualified on Linux, macOS, and Windows (CI run `40008039843`),
with UF-1 and UF-3 still open at low severity.

**Root `CHANGELOG.md`** — the intended `Unreleased` aggregate entry:

> - **service-lifecycle M013 — typed manager failures, one timeout model, one
>   cron rule.** `ServiceError::Manager` now carries a machine-readable
>   `ManagerFailureKind` (`Spawn | Timeout | Permission | NotFound | Protocol |
>   LocalFilesystem`) alongside the bounded detail and the platform OS code;
>   the crate no longer classifies its own failures by substring-matching its
>   own error text (`permission_hint` and its call sites are gone). Every
>   adapter now creates exactly one `OperationDeadline` per operation, so a
>   systemd `install` no longer spends 4-5x its declared budget. `CronManager`
>   lists the crontab once per `inspect`, states its ownership rule once, and
>   refuses inconsistent request material with `InvalidInput` before any
>   listing or mutation. Manager-derived free text has one 512-byte ceiling.
>   **Not published by this milestone**: `eggup-service 0.1.3` remains the
>   published version, the `v0.1.3` tag is unmoved, and no consumer migration
>   is required until a republication milestone ships it — at which point a
>   consumer matching `ServiceError::Manager` positionally must match
>   `ServiceError::Manager { kind, detail, os_code }` and branch on `kind`.

---

## Parent-session verification addendum (2026-10-23)

This record listed the whole-tree commands as deferred to the parent session at
this stage, and recorded that deferral as UF-2. The parent session has since
run both, so the whole-tree half of UF-2 is now closed and only its platform
half remains.

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 449 passed; 0 failed; 36 test binaries
```

Those totals are the tree state at the time of that run; the same whole-tree
gate on the current tree reports **483 passed; 0 failed; 44 test binaries** —
see the hosted-qualification addendum below.

**UF-2 was split at this point.** The whole-tree gate was **run and green**.
The remaining half of UF-2 — Linux, Windows, and every hosted CI lane
(`stable`, `msrv`, `service-systemd`, `macos`, `windows-check`) — were still to
run at that time. **That statement is superseded** by the hosted-qualification
addendum below, in which those lanes ran green in CI run `40008039843` and
UF-2 is discharged.

UF-1 (the plan's literal "sum of recorded budgets ≤ one transition timeout"
arithmetic form being unachievable with a wall-clock deadline and an instant
executor) and UF-3 (a manager that runs and reports denial only in stderr
classifying as `Protocol` rather than `Permission`) are unchanged by this
addendum; both remain **low** severity and open.

---

## Parent-session verification addendum (2026-10-23, hosted qualification)

**What this addendum supersedes.** Three statements earlier in this record are
superseded by the evidence below, and are left in place only as the record of
the state at the time of writing:

1. The **"Status"** section, which read **"Conditionally closed"** and
   attributed the condition to the whole-tree commands being deferred. The
   whole-tree gate has since run green, so the milestone is **closed**.
2. The **"Platform evidence"** table — every row except macOS recorded Linux,
   Windows, and the five hosted CI lanes as unrun. Those rows now read
   **SUPERSEDED** and are replaced by the per-platform results below.
3. **UF-2** — "The non-macOS lanes are unproven by this record". Its condition
   was exactly "the hosted qualification has not run"; that condition is
   **discharged** by CI run `40008039843`. The finding itself is retained
   above, with its severity unchanged, and is marked discharged in place.

### Hosted qualification — CI run `40008039843`

Branch `plans/corrective-batch-2026-10-23`, commit `0cb2c53`, push-triggered,
workflow `CI` (`.github/workflows/ci.yml`). **All five hosted lanes are
green.**

| Lane | Conclusion | Evidence |
|---|---|---|
| `Stable checks` (ubuntu-latest) | **success** | fmt + clippy `-D warnings` + `cargo test --workspace --all-targets --all-features --locked` → **483 passed, 0 failed, 1 ignored, 44 test binaries** + `cargo doc` + `cargo package -p eggup-service --locked` |
| `MSRV check` (ubuntu-latest, Rust 1.89.0) | **success** | `cargo check --workspace --all-targets --locked` on 1.89.0 |
| `macOS tests` | **success** | **483 passed, 0 failed, 1 ignored, 44 test binaries** |
| `Windows archive, acquisition, and service tests and check` (windows-latest) | **success** | **598 passed, 0 failed, 0 ignored, 63 test binaries** |
| `Linux systemd service qualification` (ubuntu-latest) | **success** | **9 passed, 0 failed, 0 ignored, 7 test binaries** |

The one ignored test in the `Stable checks` and `macOS tests` lanes is the
pre-existing, environment-conditional curl connect-timeout test, with its
reason recorded in the test source. It is **not** a new ignore.

The Windows lane's larger totals (598 tests / 63 binaries, versus 483 / 44 on
Linux and macOS) are the documented `#[cfg(not(windows))]` exclusion, not a
coverage gap.

### Local whole-tree gate

Run by the parent session on the same tree:

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 483 passed; 0 failed; 44 test binaries
```

### Per-platform results

Reported separately; no platform's result is inferred from another's.

| Platform | Result | Evidence |
|---|---|---|
| **Linux** | **run — green** | hosted `Stable checks` lane (fmt, clippy `-D warnings`, `cargo test --workspace --all-targets --all-features --locked` → **483 passed, 0 failed, 1 ignored, 44 test binaries**, `cargo doc`, `cargo package -p eggup-service --locked`) and hosted `Linux systemd service qualification` lane (→ **9 passed, 0 failed, 0 ignored, 7 test binaries**) of CI run `40008039843` |
| **macOS** | **run — green** | hosted `macOS tests` lane → **483 passed, 0 failed, 1 ignored, 44 test binaries**; the local native macOS lane above, plus the local whole-tree gate |
| **Windows** | **run — green** | hosted `Windows archive, acquisition, and service tests and check` lane → **598 passed, 0 failed, 0 ignored, 63 test binaries** |

**After this addendum M013 is qualified on all three platforms.** The typed
`ManagerFailureKind` taxonomy and the single-deadline contract (one
`OperationDeadline` per operation across systemd, launchd, cron, and Windows
SCM) are now exercised on Linux, macOS, and Windows, not only on the authoring
host.

### The source-breaking consequence stands

`ServiceError::Manager` is a struct variant (`Manager { kind, detail, os_code }`)
in the source line, and a consumer that matched `ServiceError::Manager(detail)`
positionally must now match the struct form and branch on `kind`. The hosted
qualification **does not soften this**: the source-breaking change is
unaffected by which platform it was compiled on, and it becomes live for
consumers only at republication. The same holds for the two
published-behaviour tightenings (the cron executable-containment gate and the
shared timeout budget).

### Findings that remain open

The hosted run qualifies the platform lanes; **it does not settle a design
question.** These findings are **unchanged** by this addendum and stay open at
the severities already recorded:

- **UF-1** — the plan's literal "summed budgets ≤ one transition timeout"
  arithmetic form is unachievable with a wall-clock deadline and an instant
  executor (**low**) — unchanged and **open**.
- **UF-3** — a manager that runs and reports an access denial only in its own
  stderr is classified `Protocol`, not `Permission` (**low**) — unchanged and
  **open**.

**UF-2** — the non-macOS lanes are unproven by this record — is **discharged**
by the hosted run above; it is retained in place with its severity unchanged
and marked discharged there. The three low-severity findings this closure
lists are therefore **not** all still open: two (UF-1, UF-3) are design
questions that no platform run can settle, and one (UF-2) was a coverage
condition that the hosted run has now met.

### Compatibility consequence

**Unchanged, and carried forward honestly.** None of these milestones
published, none moved a tag, `eggup-service` remains published at `0.1.3` and
still carries every defect this milestone fixed — the untyped manager failures
and the text-derived `permission_hint` re-classification, the three different
timeout models, the double crontab listing, the contradictory cron rule, and
the second 256-byte cap. A **republication** is the only way these fixes
reach consumers; the hosted qualification proves the source line, not the
published crate.
