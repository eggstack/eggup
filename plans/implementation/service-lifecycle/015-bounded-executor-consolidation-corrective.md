# Service Lifecycle Milestone 015 — Bounded executor consolidation corrective

Status: implemented

> **Implementation note (added at implementation time; the plan's decisions are
> unchanged).** Two sentences of §10 could not be satisfied literally, and both
> were resolved in the direction the plan's own hard constraints point:
>
> 1. **The portable child fixture does not use a `[[test]]` target with
>    `harness = false`.** §10 prescribes one, but a `[[test]]` entry is a
>    `crates/eggup-service/Cargo.toml` edit, and R-C4 and AC9 require that
>    manifest — and `Cargo.lock` — to be **byte-identical** after the
>    milestone; §5's "no manifest change" and §14's stop condition 9 say the
>    same. The conflict is resolved in favour of the manifest constraint: the
>    fixture child is **the test binary itself**, re-executed with a mode
>    argument (`crates/eggup-service/tests/common/fixture_child.rs`). That
>    keeps the fixture portable — `current_exe()` is an absolute path to a
>    real executable on every platform — and needs no shell, no interpreter,
>    and no manifest change. The mode is selected from argv, never from the
>    environment, because the environment is the property under test.
> 2. **§10's T9 "systemctl `--user`" case is Unix-only.** The service's
>    environment filter selects the user-scoped case by the program's file
>    name, so the child must be named `systemctl`; only Unix can execute a
>    program whose name has no extension. The case is therefore
>    `#[cfg(unix)]`-gated, and its **portable counterpart** is the
>    crate-private assertion `the_session_bus_filter_is_exactly_three_variables`
>    in `crates/eggup-service/src/lib.rs`'s `mod tests`, which pins the same
>    filter to exactly the three session-bus names on every lane.
>
> Both resolutions are recorded in
> `plans/closure/service-lifecycle/015-status.md` § "Plan deviations".
>
> 3. **§8's P3 premise is factually wrong, and an eighth difference exists.**
>    §8.1's P3 states that the broken-pipe and the still-running-at-the-deadline
>    cases are the *two ordinary cases* and that both implementations agree on
>    them. The baseline evidence contradicts the first half of that: the
>    replaced runner's wait loop polled its writer channel with `try_recv()`,
>    and `try_recv` **removes** the message, so a verdict of `Ok(())` was
>    consumed and discarded by a check that only looks for `Err(_)`. The
>    post-exit check then found an empty, disconnected channel and reported
>    `Manager(Spawn, "manager stdin failed for {program}")` for a write that
>    had **succeeded**. The **ordinary success case** was therefore *not*
>    identical: the replaced runner could not report a successful stdin write
>    at all. The difference is recorded as **D8** in
>    `architecture/service-lifecycle.md` and in the closure record, with the
>    baseline and published-crate evidence. D8 is a **defect fix**, not a cost:
>    the consolidation makes the `crontab -` install path work. The plan's §8
>    structure is unchanged; only its P3 claim is corrected by this note.

Repository baseline: `20ac770dacfdcd3d82e734c22613bfccd36f9c09`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Finding of record:

- `plans/closure/service-lifecycle/012-status.md` § "Unresolved findings with severity" — **Finding 1**: "`SystemExecutor` is not consolidated onto `eggup_core::run_bounded`." Assessed **low** severity and recorded as a follow-on. This milestone discharges that recorded finding.

Prerequisite milestone (not owned by this plan):

- `plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md` — Verified Update Core M017, landing in parallel. It supplies the bounded-stdin capability this consolidation depends on. Its closure record will be `plans/closure/verified-update-core/017-status.md`. M017's own §16 names this milestone as its downstream consumer.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: polish

> **Classification note.** `.opencode/skills/planning-workflow/SKILL.md` §"Classification" states that *"Infrastructure alone is not a completed capability until a real consumer proves the path."* M015 is classified `polish` because it adds **no capability**, creates **no new code path**, and changes **no public item**. It discharges a finding that the M012 closure itself recorded as *"a maintainability finding, not a safety finding"* (`plans/closure/service-lifecycle/012-status.md`, Finding 1). Its acceptance is nevertheless stated as a **behaviour-preservation invariant**: the consolidation is only correct if every observable outcome of the current runner is preserved, and §8 names the seven places where it is *not* bit-identical, each with an explicit, documented, fail-closed or narrowing direction.

## 1. Objective

1. **One bounded process runner.** `eggup-service`'s hand-rolled bounded process runner (`SystemExecutor`'s `impl CommandExecutor`) is replaced by a call into `eggup_core::run_bounded`, so the workspace holds **one** implementation of the bounded-subprocess mechanic rather than two.
2. **No loss of the service's own safety properties.** The consolidation must **not** hand the raw `argv[0]` to core, must **not** relax the executable allowlist, must **not** relax the cleared-child-environment policy, must **not** relax the stdin bound, and must **not** relax the timeout range check.
3. **No change to the failure taxonomy.** Every current `ServiceError::Manager` outcome keeps its `ManagerFailureKind` — `NotFound`, `Permission`, `Timeout`, `Spawn`, `Protocol` — under the explicit mapping in §6.2. A consumer matching on `ServiceError::manager_kind()` observes no difference.
4. **No change to any manager's argv.** The argv built for `systemctl` / `launchctl` / `crontab` is byte-identical before and after the change. Only the *execution mechanism* changes.
5. **No public API change.** `CommandExecutor`, `SystemExecutor`, `ManagerCommandOutput`, `CommandOutput`, and `ServiceError` keep their exact published shapes. `SystemExecutor`'s **public** field `max_output_bytes` stays.

## 2. Why this milestone is ready

- **The deferring condition is discharged.** Service M012 deliberately refused to consolidate `SystemExecutor` onto `eggup_core::run_bounded`. Its plan requirement **§6.6 R-E1** states: *"`SystemExecutor`'s execution behavior is **not** changed in M012; no reuse of `eggup_core::run_bounded` occurs, so M012 does not depend on core's currently-defective deadline behavior."* Its companion **§6.6 R-E2** made the deferral conditional: *"The deferral is recorded as a named follow-up obligation in §16 and as a closure finding, with the prerequisite that core's bounded-execution deadline corrective closes first and that bounded-stdin support is added to the core primitive before any service-side reuse."* Both prerequisites are now met:
  - **Core M014 closed the defect.** `plans/closure/verified-update-core/014-status.md` records the closure of the high-severity finding that *"`eggup-core` bounded candidate execution can exceed its documented wall-clock deadline"* (`plans/registry.md` § "Post-0.1.3 open findings" entry 3). Its executive finding states that `run_bounded` *"evaluated its wall-clock deadline only while the direct child was still alive"*, and that the correction makes the deadline *"an **unconditional maximum bound on the whole bounded execution**"*. The M012 closure's own addendum records the consequence: *"the follow-on is still **authorable** now that core M014 has closed. The hosted run does not discharge it."*
  - **Core M017 supplies bounded stdin.** M017 adds `MAX_STDIN_BYTES` and `CommandSpec::stdin_data` to the core primitive, which is the second of M012 §6.6 R-E2's two named prerequisites, and is the capability the `crontab -` path needs (see §3.2, step 10).
- **The finding is recorded, not detected.** This milestone discharges a **known, recorded, plan-deferred** finding. There is **no failing production report** for it, and none is claimed. The M012 closure's Finding 1 words it as follows, reproduced verbatim and without inflation:

  > **Finding 1 — `SystemExecutor` is not consolidated onto `eggup_core::run_bounded`.**
  > Severity: **low**. Reason: plan §6.1 R-E1 explicitly forbids the consolidation
  > in this milestone because core's bounded-execution primitive is itself
  > defective until a separate core milestone closes. The duplication is therefore
  > *known*, *recorded*, and *deferred by the plan itself*, not an undetected
  > defect. It is a maintainability finding, not a safety finding: `SystemExecutor`'s
  > own bounds (allowlisted absolute paths, cleared environment, bounded output,
  > bounded stdin, no shell interpolation) are unchanged and independently
  > defensible. It is a follow-on for the core bounded-execution milestone.

  (The requirement the finding names lives at plan §6.6 **R-E1**; the finding's prose cites it as "§6.1 R-E1". Both refer to the same requirement. This milestone discharges **§6.6 R-E1**, and closes Finding 1.)
- **Scope is bounded to one crate.** Only `crates/eggup-service` production code, its own `tests/` tree, its `README.md` / `CHANGELOG.md`, and `architecture/service-lifecycle.md` are touched. `eggup-core`'s production code is **read-only** here — M017 owns it.
- **No new dependency.** `crates/eggup-service/Cargo.toml:26` already declares `eggup-core = { version = "0.1.0", path = "../eggup-core" }` — a **path** dependency. The workspace's core source, including M017's new capability, is therefore consumed with **no version change, no `Cargo.lock` change, and no new dependency**. §4 states this as a hard invariant.
- **Severity, stated explicitly** per the planning-workflow severity gate: **low**. This is below the "medium-or-higher" blocking threshold, so it does not force another corrective before the service-lifecycle roadmap advances. It is nevertheless **not** optional: it is a recorded follow-on obligation, and the M012 closure attaches an explicit prerequisite to it.

## 3. Current implementation evidence

All anchors below are at the baseline SHA `20ac770dacfdcd3d82e734c22613bfccd36f9c09`.

### 3.1 The service-side runner to be replaced

`crates/eggup-service/src/lib.rs`:

| Element | Site | Notes |
|---|---|---|
| `MAX_COMMAND_OUTPUT_BYTES` = 256 KiB | `:895` | Default per-stream capture bound |
| `MAX_CRONTAB_BYTES` = 1 MiB | `:897` | `crontab` content / stdin bound |
| `MAX_TRANSITION_TIMEOUT` = 300 s | `:901` | Callers supply smaller values |
| `MAX_DIAGNOSTIC_BYTES` = 512 | `:912` | The one ceiling for manager-output diagnostic text |
| `ManagerCommandOutput` | `:921` | Public fields `status: Option<i32>` (`:923`), `stdout: Vec<u8>` (`:925`), `stderr: Vec<u8>` (`:927`) |
| `stdout_text()` / `stderr_text()` | `:932`, `:937` | Lossy-UTF-8 helpers |
| `CommandOutput` type alias | `:953` | Compatibility alias; not deprecated |
| `CommandExecutor` trait | `:961` | `fn run(&self, argv: &[String], stdin_data: Option<&[u8]>, timeout: Duration) -> Result<ManagerCommandOutput, ServiceError>` at `:968-973` |
| `SystemExecutor` struct | `:984` | `#[derive(Debug, Clone)]`; **one public field** `max_output_bytes: usize` at `:986` |
| `Default for SystemExecutor` | `:989-995` | Sets `max_output_bytes = MAX_COMMAND_OUTPUT_BYTES` |
| `SystemExecutor::new()` | `:999` | |
| `SystemExecutor::with_max_output(n)` | `:1008-1013` | Rejects `0` and `> 16 MiB` with `ServiceError::invalid("output bound out of range")` |
| `impl CommandExecutor for SystemExecutor` | `:1016` | The hand-rolled runner. `fn run` at `:1017` |

The current `run` sequence, step by step:

1. `:1023-1025` — reject empty `argv` or empty `argv[0]` → `ServiceError::invalid("manager command has no program")`.
2. `:1026-1028` — reject any argv element containing `\0` → `ServiceError::invalid("manager command contains nul bytes")`.
3. `:1029-1031` — reject a zero timeout or one above `MAX_TRANSITION_TIMEOUT + 60s` → `ServiceError::invalid("manager timeout out of range")`.
4. `:1032-1036` — reject a stdin payload longer than `MAX_CRONTAB_BYTES` → `ServiceError::invalid("manager stdin exceeds bound")`.
5. `:1037` — `let command_deadline = Instant::now() + timeout;` — **the deadline starts before the program is resolved and before the spawn**.
6. `:1038` — `resolve_manager_program(&argv[0])?` → an **absolute, allowlisted** path.
7. `:1039` — `let program = program_path.to_string_lossy().into_owned();` — the **resolved** program string used in every failure detail.
8. `:1040-1043` — build the child: `Command::new(&program_path)`, then `cmd.args(&argv[1..])` (literal argv, no shell).
9. `:1044-1047` — `cmd.env_clear()`, then re-add only `filtered_manager_environment(argv, std::env::vars_os())`.
10. `:1048-1052` — `Stdio::piped()` for stdin **only** when a payload is supplied; otherwise `Stdio::null()`. `:1053-1054` — stdout and stderr are always piped.
11. `:1055-1064` — spawn; on failure classify with `ManagerFailureKind::from_spawn_error(&e)` (`:1058`) and build the detail as `format!("manager binary missing: {program}")` when the kind is `NotFound` (`:1060`), else `format!("manager spawn failed for {program}: {e}")` (`:1061`).
12. `:1067-1085` — take both pipes and spawn two reader threads running `read_bounded_generic(pipe, max)`.
13. `:1094-1118` — spawn a writer thread that does `write_all` + `flush` and reports the result over a channel; the thread's `stdin` handle is dropped when it finishes, closing the pipe for the child.
14. `:1119` — the wait loop:
    - `:1122-1131` — a writer failure → kill + reap → `Err(manager(Spawn, format!("manager stdin failed for {program}")))`.
    - `:1133` — `Ok(Some(status))` → the child has exited. `:1137-1155` collect the writer's verdict within the remaining budget; a verdict that is not `Ok(Ok(()))` → kill + reap → the same `manager stdin failed` error.
    - `:1156-1185` — collect stdout, then stderr, each within the remaining budget; a collection timeout → `Err(manager(Timeout, "manager output collection timed out"))`.
    - `:1186-1191` — an overflow flag on either stream → `Err(manager(Protocol, format!("manager output exceeded bound for {program}")))`.
    - `:1192-1196` — otherwise `Ok(ManagerCommandOutput { status: status.code(), stdout, stderr })`.
    - `:1198-1206` — `Ok(None)` (still running): if the deadline has passed → kill + reap → `Err(manager(Timeout, format!("manager command timed out for {program}")))`.
    - `:1214-1221` — a wait error → kill + reap → `Err(manager(Spawn, format!("manager wait failed for {program}: {e}")))`.

### 3.2 The three helpers the consolidation must not bypass

| Helper | Site | Property it carries |
|---|---|---|
| `resolve_manager_program` | `:1227-1253` | The **only** path from a program name to a binary. An absolute path is validated directly (`:1230`); a bare name resolves **only** through the platform allowlist — `systemctl` → `/usr/bin/systemctl`, `/bin/systemctl`; `launchctl` → `/bin/launchctl`, `/usr/bin/launchctl`; `crontab` → `/usr/bin/crontab`, `/bin/crontab` (`:1233-1235`). Anything else → `Err(invalid("manager program must be absolute or allowlisted"))` (`:1237-1239`). All candidates failing validation → `Err(manager(NotFound, format!("trusted manager binary missing: {program}")))` (`:1249-1252`). **There is no `PATH` lookup at all.** |
| `validate_executable_path` | `:1255-1280` | Requires an absolute path (`:1256`), an existing regular file (`:1261-1271`, otherwise `Err(manager(NotFound, format!("manager binary missing: {display}")))` where `{display}` is the **requested** program string, not the resolved one), and — under `#[cfg(unix)]` — at least one execute bit (`:1272-1278`). |
| `filtered_manager_environment` | `:1282-1303` | Returns `(OsString, OsString)` pairs. The allowlist is **empty** unless the argv is `systemctl` **with** `--user` (`:1286-1298`); in that case exactly `DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS` are passed through from the parent environment. Every other argv shape gets an **empty** environment. |

`read_bounded_generic` (`:1305-1328`) is the service's own bounded reader: it accumulates into a buffer and returns `(buf, true)` once `buf.len() > max` (`:1320-1322`); on a **read error** it `break`s and returns `(buf, false)` (`:1324`) — that is, a reader error is currently **silently treated as success** with partial output. Core does not do this (§8, difference D4).

### 3.3 The target primitive in `eggup-core`

`crates/eggup-core/src/candidate.rs`:

| Element | Site | Notes |
|---|---|---|
| `CommandSpec` | `:19` | Private fields `program: PathBuf` (`:20`), `args: Vec<String>` (`:21`), `current_dir: Option<PathBuf>` (`:22`), `timeout: Duration` (`:23`), `max_output_bytes: usize` (`:24`), `environment: Vec<(String, String)>` (`:25`) |
| `CommandSpec::new` / `arg` / `args` | `:30`, `:42`, `:48` | Literal argv; never a shell |
| `current_dir` / `timeout` / `max_output_bytes` / `environment` | `:58`, `:72`, `:84`, `:90` | `environment` takes `impl Into<String>` for **both** name and value |
| `CommandOutput` | `:98` | Private fields `exit_code` (`:99`), `stdout` (`:100`), `stderr` (`:101`), `timed_out` (`:102`), `output_limited` (`:103`) |
| `exit_code()` / `stdout()` / `stderr()` | `:108`, `:113`, `:118` | Accessors |
| `timed_out()` / `output_limited()` / `success()` | `:129`, `:134`, `:139` | |
| `run_bounded` | `:192` | The bounded runner |
| — environment cleared | `:196` | `env_clear()` |
| — stdin nulled | `:197` | `Stdio::null()` — **this is what M017 changes** |
| — stdout/stderr piped | `:198-199` | |
| — explicit environment re-added | `:203-205` | |
| — spawn failure | `:206-208` | `Error::io("spawning bounded candidate", source)` |
| — missing pipe | `:209-216` | `Error::CandidateExecution("missing stdout pipe")` / `"missing stderr pipe"` |
| — two bounded reader threads | `:217-225` | `read_limited(stdout, limit)` |
| — deadline | `:227` | `Instant::now() + spec.timeout`, **after** the spawn returned |
| — poll loop | `:235-273` | `try_wait` failure → `Error::io("polling bounded candidate", source)` (`:245`); `output_limited` → kill + reap, break (`:247-255`); completion → break (`:256-260`); **unconditional deadline** → `timed_out = true`, kill + reap if still running, break (`:261-271`) |
| — final drain + join | `:274-285` | A reader still blocked at the bound is **abandoned explicitly** |
| — success result | `:286-296` | `exit_code: status.and_then(|s| s.code())` |
| `poll_reader` / `read_limited` | `:304`, `:323` | `read_limited` returns `Err` once `output.len() + count > limit` (`:331-333`) |

`crates/eggup-core/src/error.rs`:

| Element | Site | Notes |
|---|---|---|
| `Error` enum | `:28` | Exactly **seven** published variants; must not grow |
| `Error::InvalidInput(String)` | `:30` | |
| `Error::Io { operation: &'static str, source: io::Error }` | `:34-39` | The variant's **fields are public** |
| `Error::CandidateExecution(String)` | `:53` | |
| `Error::io` | `:157` | `pub(crate)` — a downstream crate **cannot construct** `Error::Io`, but it **can destructure** it |
| `impl std::error::Error for Error` | `:216-230` | `source()` returns the inner `io::Error` for the `Io` variant (`:219`), so a downstream crate can recover and re-classify it |

**The three `operation` strings are a stable classification surface** under M017: `"spawning bounded candidate"`, `"polling bounded candidate"`, `"writing bounded stdin"`. Branching on them from `eggup-service` is safe **only** because M017 documents them as such.

### 3.4 The core capability this plan depends on (Verified Update Core M017)

Core M017 lands in parallel and supplies bounded stdin to the core primitive, with exactly this contract:

1. `pub const MAX_STDIN_BYTES: usize = 1024 * 1024;` in `crates/eggup-core/src/candidate.rs`.
2. `CommandSpec` gains a private field `stdin_data: Option<Vec<u8>>` (default `None`) and one **infallible** public builder `pub fn stdin_data(mut self, data: impl Into<Vec<u8>>) -> Self`.
3. `run_bounded` with a payload attached:
   - a payload longer than `MAX_STDIN_BYTES` → `Err(Error::InvalidInput("stdin data exceeds the bounded stdin bound"))`, **before spawning anything**;
   - no payload → stdin stays `Stdio::null()` (unchanged);
   - a payload attached → stdin is piped and a dedicated writer thread performs `write_all` + `flush`;
   - a writer failure (including the broken pipe from a child that exits without reading) → kill + reap the child → `Err(Error::Io { operation: "writing bounded stdin", source })`;
   - the deadline stays unconditional and bounds the writer exactly as it bounds the readers.
4. The three `operation` strings — `"spawning bounded candidate"`, `"polling bounded candidate"`, `"writing bounded stdin"` — are a **stable classification surface** documented in core's rustdoc.

**Observed at authoring time.** At the time this plan was written, M017's core-side implementation was already present **uncommitted in the working tree** (`git diff -- crates/eggup-core/src/candidate.rs`), and it matches the contract above point for point: the bound check `data.len() > MAX_STDIN_BYTES` returns `Err(Error::InvalidInput("stdin data exceeds the bounded stdin bound"))` before the child is built; stdin is piped only when a payload is attached; the writer thread does `write_all` + `flush`; a writer failure kills and reaps the child and returns `Error::io("writing bounded stdin", source)`; and the three `operation` strings are documented as a stable classification surface. Because M017's diff shifts line numbers in `crates/eggup-core/src/candidate.rs`, **§3.3's anchors are baseline anchors** — the implementer must re-derive them from the working tree (that is what R-D1 requires).

**If this contract is not present in the workspace source when the implementation starts, this milestone cannot be implemented** and §14's first stop condition fires.

### 3.5 The recorded finding this milestone discharges

`plans/closure/service-lifecycle/012-status.md` § "Unresolved findings with severity" → **Finding 1** (reproduced verbatim in §2). The registry's own narrative at `plans/registry.md` records the same consequence: *"service M012 deliberately deferred the `SystemExecutor` consolidation because core's bounded-execution primitive was defective; core M014 has since closed, so that consolidation can be authored."* The registry's "Active subsystem roadmaps" table row for **Service lifecycle** likewise reads: *"the deliberately deferred `SystemExecutor` consolidation is authorable now that core M014 has closed; it is not yet written."*

## 4. Invariants that must not regress

- **`eggup-service` gains no dependency of any kind**, including dev-dependencies. `crates/eggup-service/Cargo.toml:26` already declares `eggup-core = { version = "0.1.0", path = "../eggup-core" }` — a **path** dependency — so core M017's new capability is reachable with **no manifest change, no version bump, and no `Cargo.lock` change**. `Cargo.toml` and `Cargo.lock` MUST be byte-identical after this milestone.
- **The executable allowlist is not bypassed.** `resolve_manager_program` (`:1227`) stays the only path from a program name to a binary. The **resolved absolute path** — never the raw `argv[0]` — is what is handed to core. A bare name that is not `systemctl` / `launchctl` / `crontab` still fails with `InvalidInput`.
- **The child environment stays cleared.** `env_clear()` plus the three explicit session-bus variables for user-scoped systemd only. No ambient variable reaches the child.
- **The stdin bound stays at `MAX_CRONTAB_BYTES` (1 MiB)** and the timeout range check stays at `non-zero && <= MAX_TRANSITION_TIMEOUT + 60s`. Both remain the **service's own pre-flight**, ahead of anything core sees.
- **The argv stays literal.** No shell, no stringification, no `PATH` mediation.
- **The failure taxonomy is unchanged.** `ServiceError`'s variant set (`InvalidInput`, `OwnershipDenied`, `Conflict`, `Manager`) and `ManagerFailureKind`'s variant set are untouched. No new variant, no re-typing, no change to `ServiceError::manager_kind()`'s meaning.
- **The public surface is unchanged.** `CommandExecutor` (`:961`), `SystemExecutor` (`:984`) and its **public field** `max_output_bytes` (`:986`), `SystemExecutor::new` (`:999`), `SystemExecutor::with_max_output` (`:1008`), `ManagerCommandOutput` (`:921`), `CommandOutput` (`:953`), `ManagerFailureKind::from_spawn_error` (`:363`), `ServiceError::manager` (`:448`), `ServiceError::manager_kind` (`:486`), `ServiceError::manager_os_code` (`:496`) all keep their exact shapes. The M014 public-API guard (`crates/eggup-service/tests/public_api_surface.rs` and its snapshot `crates/eggup-service/tests/public-api-surface.txt`) MUST still pass **without editing the snapshot**.
- **`#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` stay on** in `eggup-service`; the workspace's `unsafe_code = "deny"` and `clippy all = warn` are unchanged.
- **No manager's argv changes.** The argv built for `systemctl` (`:1801`, `:1812`, `:1843`, `:1871`, `:2007`, `:2498`, `:2534`, `:2576`; the `"systemctl"` literal at `:2588`), for `launchctl` (`:2765` list, `:3045` stop, `:3131` bootstrap, `:3153` bootstrap-args, `:3166-3167` kickstart), and for `crontab` (`:3521` list, `:3564` `-`) is byte-identical.
- **`eggup-core`'s production code is not modified by this milestone.** M017 owns it. `run_bounded`, `CommandSpec`, `CommandOutput`, and `Error` are read-only here.
- **No publication.** `eggup-service 0.1.3` is published (2026-10-09, source `feb6ae5`, checksum `9f7f7ea8…32124dc`). This milestone must not publish, must not run `cargo publish`, and must not move the `v0.1.3` tag. The published crate keeps the hand-rolled runner until a separate republication milestone.

## 5. Scope

### In scope

- Replacing the body of `impl CommandExecutor for SystemExecutor::run` (`crates/eggup-service/src/lib.rs:1017`) with a call into `eggup_core::run_bounded`, keeping every pre-flight check and every classification described in §6.
- The explicit core-result → service-result mapping in §6.2, including the exhaustive fallback arm.
- The crate-private `read_bounded_generic` helper (`:1305`) becoming dead once the consolidation lands — it is deleted **if and only if** it has no other caller. `grep -n "read_bounded_generic" crates/eggup-service/src/` must be re-run before deletion.
- New tests in `crates/eggup-service/tests/` proving the equivalence claims in §8, plus the portable fixture they need (§10).
- `crates/eggup-service/README.md`, `crates/eggup-service/CHANGELOG.md` (`Unreleased`), and `architecture/service-lifecycle.md` § "Process execution: `SystemExecutor`".

### Explicitly out of scope

- **No change to `eggup-core`'s production code.** M017 owns the core-side capability. `run_bounded`, `CommandSpec`, `CommandOutput`, and `Error` are read-only here.
- **No change to the `CommandExecutor` trait**, to `ManagerCommandOutput`, to `CommandOutput`, or to the `ServiceError` / `ManagerFailureKind` taxonomies. M013 owns the taxonomy.
- **No change to any manager's argv**, to the executable allowlist, to `validate_executable_path`, to `filtered_manager_environment`, or to `resolve_manager_program`.
- **No change to the `CommandSpec::environment` contract.** If a non-UTF-8 environment value cannot be passed through byte-exactly, the resolution is the service-side rule in §6.1 R-B3, **not** a change to core.
- **No new dependency**, no manifest change, no `Cargo.lock` change, no version bump.
- **No republication.** The published `eggup-service 0.1.3` keeps the hand-rolled runner. No tag movement, no `cargo publish`, no release.
- **No change to the Windows SCM adapter**, to the commit engine, to `disposition.rs`, or to `lifecycle_update.rs`. M014 and M012 own those.
- **No change to `MAX_CRONTAB_BYTES`, `MAX_COMMAND_OUTPUT_BYTES`, `MAX_TRANSITION_TIMEOUT`, or `MAX_DIAGNOSTIC_BYTES`.**
- **No change to `architecture/core-transaction.md`** or to core's contract docs. M017 owns those.
- **No change to `.github/workflows/ci.yml`, `scripts/**`, or `AGENTS.md`.**
- **No new public item.** The consolidation is internal.

## 6. Required production changes

Requirements are observable outcomes, not an implementation sketch.

### 6.1 The consolidation itself

- **R-A1.** `SystemExecutor::run` (`crates/eggup-service/src/lib.rs:1017`) keeps **its own** pre-flight, in this order, ahead of every call into core:
  1. the argv emptiness check and the `argv[0]` emptiness check → `ServiceError::invalid("manager command has no program")` (today `:1023-1025`);
  2. the NUL-byte check over every argv element → `ServiceError::invalid("manager command contains nul bytes")` (today `:1026-1028`);
  3. the timeout range check → `ServiceError::invalid("manager timeout out of range")` (today `:1029-1031`);
  4. the stdin bound check against `MAX_CRONTAB_BYTES` → `ServiceError::invalid("manager stdin exceeds bound")` (today `:1032-1036`);
  5. `resolve_manager_program(&argv[0])?` (today `:1038`) — **the resolved absolute path**, never the raw `argv[0]`;
  6. `filtered_manager_environment(argv, std::env::vars_os())` (today `:1045`) — **the filtered pairs**, never the ambient environment.

  These six checks are the service's **policy**, not core's. They MUST NOT be delegated to core, and core MUST NOT be given the raw `argv[0]`.
- **R-A2.** The implementation builds one `eggup_core::CommandSpec` and calls `eggup_core::run_bounded(&spec)`:

  | `CommandSpec` field | Value | Source |
  |---|---|---|
  | `program` | the **resolved absolute path** returned by `resolve_manager_program` | R-A1.5 |
  | `args` | `argv[1..]`, in order, each element as one literal argument | today `:1041-1043` |
  | `environment` | the filtered pairs from `filtered_manager_environment` | R-A1.6, R-B3 |
  | `timeout` | the caller's timeout, **less the elapsed pre-flight** | R-B2 |
  | `max_output_bytes` | `self.max_output_bytes` | today `:1075` |
  | `stdin_data` | the payload when `stdin_data.is_some()`, otherwise unset | today `:1048-1052`, M017 |
  | `current_dir` | unset | today: unset |

- **R-A3.** The `program` string used in every failure detail is the **resolved** program path, produced from `program_path.to_string_lossy()` exactly as today (`:1039`), so the details remain byte-identical to the current implementation.
- **R-A4.** The consolidation is **internal**. `SystemExecutor`'s **public field** `max_output_bytes` stays public, `Default` keeps setting it to `MAX_COMMAND_OUTPUT_BYTES`, `SystemExecutor::new()` and `SystemExecutor::with_max_output()` keep their exact signatures and their exact error text, and the `SystemExecutor` rustdoc keeps stating the same guarantees (allowlisted absolute paths, cleared environment, bounded output, bounded stdin, kill-and-reap deadlines). The rustdoc gains **one** sentence naming `eggup_core::run_bounded` as the bounded-execution primitive, so the crate's own documentation is truthful about where the mechanic lives.
- **R-A5.** `read_bounded_generic` (`:1305-1328`) is deleted **iff** `grep -n "read_bounded_generic" crates/eggup-service/src/` shows no other caller after the consolidation. If it has another caller, it stays and the consolidation is re-examined. The crate MUST NOT carry two bounded readers.

### 6.2 The result mapping

- **R-B1.** The result of `eggup_core::run_bounded(&spec)` is mapped to the service's taxonomy by exactly this table. The table is normative; the implementation MUST match it exhaustively.

  | core result | service result |
  |---|---|
  | `Ok(out)`, `out.timed_out()` | `Err(ServiceError::manager(ManagerFailureKind::Timeout, format!("manager command timed out for {program}")))` |
  | `Ok(out)`, `out.output_limited()` | `Err(ServiceError::manager(ManagerFailureKind::Protocol, format!("manager output exceeded bound for {program}")))` |
  | `Ok(out)` otherwise | `Ok(ManagerCommandOutput { status: out.exit_code(), stdout: out.stdout().to_vec(), stderr: out.stderr().to_vec() })` |
  | `Err(Error::Io { operation, source })`, `operation == "spawning bounded candidate"` | classify with `ManagerFailureKind::from_spawn_error(&source)`; detail `format!("manager binary missing: {program}")` when the kind is `NotFound`, else `format!("manager spawn failed for {program}: {source}")` — byte-identical to the current detail |
  | `Err(Error::Io { operation, .. })`, `operation == "polling bounded candidate"` | `Err(ServiceError::manager(ManagerFailureKind::Spawn, format!("manager wait failed for {program}: {source}")))` |
  | `Err(Error::Io { operation, .. })`, `operation == "writing bounded stdin"` | `Err(ServiceError::manager(ManagerFailureKind::Spawn, format!("manager stdin failed for {program}")))` |
  | any other `operation` | `Err(ServiceError::manager(ManagerFailureKind::Spawn, <bounded detail>))` — the fallback MUST NOT silently swallow the distinction; the detail MUST name the unrecognised operation |
  | `Err(Error::CandidateExecution(msg))` | `Err(ServiceError::manager(ManagerFailureKind::Protocol, <bounded detail>))` |
  | `Err(Error::InvalidInput(msg))` | `Err(ServiceError::invalid(msg))` — core's stdin-bound rejection |

  Notes that bind the table:
  - The `operation` strings are matched **by value**, and the match MUST be **exhaustive over the three documented strings plus a fallback arm**. The fallback arm MUST name the unrecognised `operation` in its bounded detail so that a future core change is attributable rather than silently folded into "spawn failed".
  - Branching on the `operation` string is safe **only because core M017 documents those strings as a stable classification surface**. If M017's contract is absent or its strings differ, this milestone cannot be implemented as specified and §14's stop conditions fire.
  - `Err(Error::InvalidInput(msg))` → `ServiceError::invalid(msg)` is a **defensive** arm: the service's own stdin bound (R-A1.4) uses the same 1 MiB threshold as core's `MAX_STDIN_BYTES` and is checked **first**, so core's stdin rejection is unreachable in practice. The arm exists so that a future divergence is a typed `InvalidInput` rather than a panic or a swallowed error.
  - `Err(Error::InvalidInput(msg))` and the fallback arm's detail are passed through `ServiceError::invalid` / `ServiceError::manager`, which bound the detail at `MAX_DIAGNOSTIC_BYTES` (512, UTF-8-safe). No diagnostic produced by this path may exceed that ceiling.
- **R-B2.** The timeout handed to core is the caller's timeout **less the elapsed service-side pre-flight**. The service measures `Instant::now()` before `resolve_manager_program` (today's `:1037`) and computes the remaining budget before building the `CommandSpec`. If the remaining budget is zero, the call returns `Err(ServiceError::manager(ManagerFailureKind::Timeout, format!("manager command timed out for {program}")))` **without spawning anything**. Reason: today's deadline starts before the resolve and the spawn (`:1037`), while core's deadline starts only **after** the spawn returns (`crates/eggup-core/src/candidate.rs:227`, and its rustdoc at `:147-149`: *"The deadline starts once the spawn has returned, so a blocking spawn is not covered by it."*). Without R-B2 the consolidated caller's wall-clock budget would grow by the pre-flight time. The pre-flight is a bounded `stat` plus an allowlist scan, so the practical difference is small — but the service's documented contract is that the caller's timeout bounds the **whole** call, and R-B2 keeps it that way.
- **R-B3.** The environment handed to core is the filtered pair list. Because `filtered_manager_environment` (`:1282`) yields `(OsString, OsString)` and core's `CommandSpec::environment` (`crates/eggup-core/src/candidate.rs:90`) accepts `impl Into<String>` for both name and value, the conversion MUST be stated explicitly and MUST NOT be silently lossy:
  - a pair whose key **and** value are both valid UTF-8 is converted with `to_string_lossy()` and is then **byte-identical** to what the current implementation passes to `cmd.env(key, value)`;
  - a pair that is **not** valid UTF-8 is **dropped**. The implementation MUST NOT pass a lossy-converted value to the child.

  The reason for the drop rule is the fail-closed direction: a lossy conversion could substitute a *mangled* session-bus address for the operator's real one, whereas dropping the malformed pair leaves the child with no such variable — the same state every non-`--user` command is in. The three allowlisted names (`DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS`) carry path- and address-shaped values that are valid UTF-8 in practice, so the dropped case is a corner. §8 records it as difference **D5** and §15 requires the closure to record it as a residual finding.

### 6.3 What must not change

- **R-C1.** The manager argv builders are untouched, and **no existing `fake.expect(...)` argv vector in the adapter tests is edited**. A test that must be edited to accommodate the consolidation is evidence of a behaviour change, not of a test update — see §14.
- **R-C2.** `ServiceError`'s variant set and `ManagerFailureKind`'s variant set are unchanged. No new variant is added to either, and no existing variant is re-typed.
- **R-C3.** The M014 public-API guard passes **without editing `crates/eggup-service/tests/public-api-surface.txt`**. If the snapshot must change, the consolidation changed the public surface and §14's stop condition fires.
- **R-C4.** `crates/eggup-service/Cargo.toml` and `Cargo.lock` are byte-identical after the milestone. `cargo tree -p eggup-service --locked` shows exactly `eggup-core` (+ `sha2`), `windows-args` (+ `wtf8`), and `windows-service` on Windows — unchanged from the baseline.

### 6.4 The core-side contract this plan depends on

- **R-D1.** The implementation MUST confirm, by reading the workspace source before writing any service code, that core exposes `MAX_STDIN_BYTES`, `CommandSpec::stdin_data`, and the three documented `operation` strings. The confirmation is recorded in the closure record with `file:line` citations.
- **R-D2.** The implementation MUST NOT modify any file under `crates/eggup-core/`. If core's contract does not match §3.4, the milestone stops (§14) — it does not extend core.
- **R-D3.** If core's `run_bounded` cannot express the service's requirements — bounded stdin **and** an unconditional deadline that bounds the writer — the consolidation does not happen and the finding stays open. The plan's §14 covers this.

## 7. Ordered work packages

Each work package is independently verifiable and leaves the tree green.

- **WP1 — Confirm the core-side contract.** Read `crates/eggup-core/src/candidate.rs` and `crates/eggup-core/src/error.rs` in the working tree. Confirm `MAX_STDIN_BYTES`, `CommandSpec::stdin_data`, the three `operation` strings, and the public field visibility of `Error::Io`. Record `file:line` for each. **If any is absent, stop** (§14) — do not write service code against a contract that is not there.
- **WP2 — Implement the consolidation.** Replace the body of `SystemExecutor::run` with: the six pre-flight checks (R-A1), the resolved path and the `program` string (R-A3), the `CommandSpec` construction (R-A2), the deadline accounting (R-B2), the environment conversion (R-B3), the `run_bounded` call, and the mapping table (R-B1) with its exhaustive fallback arm. Delete `read_bounded_generic` if and only if it has no other caller (R-A5). Keep the crate's rustdoc truthful (R-A4).
- **WP3 — Prove the equivalence with the new fixtures.** Write the portable fixture and the test targets named in §10. Every §8 difference (D1–D7) gets a test that pins the *new* documented behaviour, so the differences are recorded in code rather than only in prose — except **D7**, which has no portable fixture and is therefore recorded in the closure record instead (and whose two ordinary cases are covered by T2 and T5).
- **WP4 — Guard the untouched surfaces.** Add the argv byte-identity regression guard (R-C1) and the environment-policy guard (§10, T8). Verify the M014 public-API guard still passes with the snapshot unmodified (R-C3).
- **WP5 — Update the documentation.** `crates/eggup-service/README.md`, `crates/eggup-service/CHANGELOG.md` (`Unreleased`), and `architecture/service-lifecycle.md` § "Process execution: `SystemExecutor`" (§12).
- **WP6 — Run the verification commands and write the closure record.** Run §11's crate-scoped commands, then write `plans/closure/service-lifecycle/015-status.md` per §15. **The registry, the roadmap, and the root `CHANGELOG.md` are the parent's edits**; the drafted prose is in §16.

## 8. Failure, cancellation, restart, and contention semantics

This section is the behaviour-equivalence analysis. It is the substance of the milestone: a consolidation that silently changes a bound, a deadline, or a classification is a regression, not a refactor.

### 8.1 What is preserved exactly

| # | Property | Current evidence | Post-consolidation |
|---|---|---|---|
| P1 | **The deadline is unconditional** — the call always returns within the budget. | Core M014's correction (`plans/closure/verified-update-core/014-status.md`); core's rustdoc at `crates/eggup-core/src/candidate.rs:146-150`. | Core's deadline is unconditional and is inherited unchanged. The service's own budget is preserved by R-B2. |
| P2 | **The deadline still bounds the writer.** | Today the write runs on its own thread and is bounded by the same deadline-checked wait loop (`crates/eggup-service/src/lib.rs:1086-1093`, `:1199-1206`). | M017's contract states the deadline "stays unconditional and bounds the writer exactly as it bounds the readers" (§3.4, item 3). |
| P3 | **A broken pipe (a child that exits without reading stdin) still fails closed.** | Today: the writer's `Err` verdict is observed at `:1122-1131` → `manager(Spawn, "manager stdin failed for {program}")`, and again at `:1147-1154` after the child exits. | M017 kills + reaps the child and returns `Err(Error::Io { operation: "writing bounded stdin", .. })`; R-B1 maps it to `manager(Spawn, "manager stdin failed for {program}")` — the **same kind and the same detail text**. The two ordinary cases are identical; only the deep corner differs (D7). |
| P4 | **An output overflow still fails closed and discards the stream.** | Today: `read_bounded_generic` sets the overflow flag (`:1320-1322`), the runner returns `manager(Protocol, format!("manager output exceeded bound for {program}"))` (`:1186-1191`) and the buffer is **not** returned. | Core's `read_limited` errors on overflow (`crates/eggup-core/src/candidate.rs:331-333`), `output_limited` is set, and the stream's bytes are discarded; R-B1 maps it to the **same kind and the same detail text**. |
| P5 | **A nonzero exit is still `Ok` with the observed status.** | Today `:1192-1196` — `status.code()` (which is `None` when the child was signalled). | Core's `exit_code: status.and_then(|status| status.code())` (`crates/eggup-core/src/candidate.rs:287`) is identical; R-B1 passes it through unchanged. The adapter's job to reject a nonzero status (e.g. `ensure_mutation_success`) is untouched. |
| P6 | **The `crontab -` path still receives its payload.** | Today `:1048-1052` pipes stdin when a payload is supplied; `:1094-1118` writes it on a dedicated thread; `crates/eggup-service/src/lib.rs:3564-3567` is the only production caller (`crontab -` with `Some(content.as_bytes())`). | R-A2 attaches the payload through core's `stdin_data`, and M017 pipes stdin and writes it on a dedicated writer thread. The payload bytes are unchanged. |
| P7 | **No manager's argv changes.** | The argv builders at `:1801`, `:1812`, `:1843`, `:1871`, `:2007`, `:2498`, `:2534`, `:2576` (systemd), `:2765` list, `:3045` stop, `:3131` bootstrap, `:3153` bootstrap-args, `:3166-3167` kickstart (launchd), `:3521` list, `:3564` `-` (cron). | Untouched. R-C1 forbids it and §10's T7 guards it. |
| P8 | **The child environment is still cleared, and the session-bus variables are still the only ones injected.** | `env_clear()` at `:1044`; `filtered_manager_environment` at `:1282-1303` allows exactly `DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS` and only for `systemctl --user`. | R-A1.6 and R-B3 preserve this; §10's T8 guards it. |
| P9 | **The allowlist is still the only path to a binary, and there is still no `PATH` lookup.** | `resolve_manager_program` at `:1227-1253`; `validate_executable_path` at `:1255-1280`. | R-A1.5 and R-A2 pass the **resolved** path to core. Core never resolves a program name. |
| P10 | **The pre-flight rejections are unchanged.** | `:1023-1036` — four distinct `ServiceError::invalid` messages. | R-A1 keeps all four, ahead of core. |

### 8.2 The places where the behaviour is **not** bit-identical

These **seven** differences are the honest cost of the consolidation. Each is stated with its direction, and each must be pinned by a test (§10) or, where no portable fixture exists, recorded in the closure (§15).

- **D1 — A reader error is now fail-closed instead of silently tolerated.** Today `read_bounded_generic` `break`s on a read error and returns `(buf, false)` (`crates/eggup-service/src/lib.rs:1324`), so a mid-stream read failure is reported as a **successful** capture of partial output. Core surfaces a reader error as `output_limited` (`crates/eggup-core/src/candidate.rs:191`, `:314`), which R-B1 maps to `manager(Protocol, "manager output exceeded bound for {program}")`. **Direction: fail-closed (stronger).** The detail text says "exceeded bound" for what is really a read failure — a cosmetic imprecision inherited from the mandated mapping. The closure MUST record this.
- **D2 — Core kills the child on an output overflow; the current runner does not.** Today the overflow is detected only after the child has already exited (`:1133` precedes `:1186`), so no kill happens. Core detects the overflow **during** the poll loop and kills + reaps a still-running child (`crates/eggup-core/src/candidate.rs:247-255`). **Direction: strictly safer (a runaway child is now terminated).** The observable result — a `Protocol` manager failure — is unchanged.
- **D3 — The timeout detail text is normalised.** Today the runner can produce three different timeout details: `"manager command timed out for {program}"` (`:1204`, the child still running at the deadline), `"manager output collection timed out"` (`:1168`, `:1183`, the post-exit collection deadline), and `"manager command timed out"` (`:1144`, `:1161`, `:1176`, the remaining-budget computation). R-B1 normalises **all** of them to `"manager command timed out for {program}"`. **Direction: detail-text change only.** The machine-readable discriminator — `ServiceError::manager_kind() == Some(ManagerFailureKind::Timeout)` — is **unchanged**, and M013's taxonomy is the contract a consumer matches on. The closure MUST record the text change.
- **D4 — A descendant that holds the stdout/stderr pipe open now ends at the deadline with the *timeout* detail.** Today the post-exit `recv_timeout` on the reader channel expires at the deadline and yields `manager(Timeout, "manager output collection timed out")` (`:1165-1170`). Post-consolidation, core's unconditional deadline returns `Ok` with `timed_out` set, and R-B1 maps it to `manager(Timeout, "manager command timed out for {program}")`. **Direction: same kind, different detail text** (D3 covers the text). Core also **abandons** the still-blocked reader explicitly rather than joining it (`crates/eggup-core/src/candidate.rs:163-169`, `:278-285`): the call ends at the deadline and the service MUST NOT then wait on that reader. This is the defect core M014 closed.
- **D5 — A non-UTF-8 environment pair is dropped rather than passed through byte-exactly.** See R-B3. Today `cmd.env(key, value)` takes `OsString` and passes the bytes through verbatim (`crates/eggup-service/src/lib.rs:1045-1047`); core's setter takes `impl Into<String>` (`crates/eggup-core/src/candidate.rs:90`), so a byte-exact pass-through is not expressible. **Direction: narrowing, fail-closed.** A mangled value is never substituted. The closure MUST record it as a residual low-severity finding, because a *valid* UTF-8 value is unaffected and the three allowlisted names carry path- and address-shaped values.
- **D6 — The spawn-failure `NotFound` detail names the *resolved* path.** Today the spawn-failure branch formats `format!("manager binary missing: {program}")` with `{program}` = the **resolved** path (`:1039`, `:1060`), while the *pre-flight* `validate_executable_path` branch formats `format!("manager binary missing: {display}")` with `{display}` = the **requested** `argv[0]` string (`:1264`). Both survive the consolidation unchanged: the pre-flight still runs first and still names the requested program; the spawn-failure arm still names the resolved program, per R-A3 and R-B1. **Direction: no change.** The distinction is subtle enough that §10's T4 requires **both** paths to be asserted separately.
- **D7 — A stdin writer that is still blocked when the deadline passes is now reported as a timeout, not as a stdin failure.** Today the writer's verdict is collected **only after the child has exited** (`:1137-1155`): if the verdict never arrives within the remaining budget, the runner reports `manager(Spawn, "manager stdin failed for {program}")` (`:1147-1154`). Post-consolidation, core M017's unconditional deadline decides the outcome, the service maps `timed_out` to `manager(Timeout, "manager command timed out for {program}")`, and the still-blocked writer is **abandoned** rather than waited on (`crates/eggup-core/src/candidate.rs:426-441`). **Direction: kind change, in a deep corner.** Reaching it requires a descendant to hold the child's stdin pipe open after the child itself exits; in the two ordinary cases — the child exits without reading (the broken pipe, which both implementations report as `Spawn`) and the child is still running at the deadline (which both report as `Timeout`) — the kinds are identical. M017's rustdoc states the same precedence explicitly: a write failure that is live **at the same time** as an output overflow wins over the overflow, and a write failure that is detected **after** the deadline has already been reached does not retroactively turn the deadline outcome into an `Err`. **This difference has no portable fixture** (it needs a descendant holding a pipe open), so it is recorded rather than tested; §15 requires the closure to state it and to confirm that the two ordinary cases above are covered by T2 and T5.

### 8.3 Cancellation, restart, and contention

- **Cancellation.** The crate has no cancellation token; the mechanism is the deadline. The deadline is unconditional after core M014 (§8.1 P1), and the service's own budget is preserved by R-B2. A caller that aborts a manager operation still observes `ManagerFailureKind::Timeout`, never a hang and never a partial success.
- **Restart.** `restart` is composed of `stop` + `start`, each its own `OperationDeadline` under M013's one-deadline-per-operation model. This milestone changes neither. A restart whose `stop` times out fails closed exactly as before.
- **Contention.** Two concurrent manager operations contend at the OS level (the service manager's own locks), not in this crate. The consolidation does not change any lock, does not change the number of spawned children, and does not change the kill-and-reap discipline. A concurrent caller's failure is still a typed `Manager` error, never a panic.
- **Zombie discipline.** Today the runner kills **and reaps** the child on every failure path (`:1098-1099`, `:1124-1125`, `:1148-1149`, `:1200-1201`, `:1215-1216`). Core does the same (`crates/eggup-core/src/candidate.rs:250-253`, `:266-269`). The consolidation therefore **keeps** the reap and MUST NOT introduce a path that kills without reaping or that leaves the child unreaped.
- **No shell, ever.** Core's `run_bounded` builds the child with `Command::new` + `args` (`crates/eggup-core/src/candidate.rs:193-199`) — no `sh -c`, no stringification. The service's argv is therefore still literal, and the `crontab` content — which *is* a shell line — remains caller-owned content written to the crontab, never interpolated into a command this crate builds.

## 9. Compatibility and migration

- **The public API of `eggup-service` is unchanged.** `CommandExecutor` (`:961`), `SystemExecutor` (`:984`), `SystemExecutor::new` (`:999`), `SystemExecutor::with_max_output` (`:1008`), `ManagerCommandOutput` (`:921`), `CommandOutput` (`:953), `ManagerFailureKind` (`:339`), `ServiceError` and its four variants, `ServiceError::manager` (`:448`), `ServiceError::manager_kind` (`:486`), and `ServiceError::manager_os_code` (`:496`) all keep their exact shapes and their exact semantics. A consumer of the published `0.1.3` compiles against the consolidated crate unchanged.
- **`SystemExecutor`'s public field `max_output_bytes` stays public.** It is not made `pub(crate)`, not renamed, and not wrapped. The field's meaning is unchanged: the per-stream capture bound, defaulting to `MAX_COMMAND_OUTPUT_BYTES` and settable through `with_max_output` for `1..=16 MiB`.
- **The consolidation is internal.** A consumer that constructs a `SystemExecutor`, calls `CommandExecutor::run`, or implements `CommandExecutor` themselves observes no difference. A consumer that matches on `ServiceError::manager_kind()` observes **no difference in kind**: `NotFound`, `Permission`, `Timeout`, `Spawn`, and `Protocol` keep their meanings.
- **The detail text of a timeout is normalised** (§8.2 D3, D4). This is a text change inside `ServiceError::Manager`'s `detail` field. M013's contract is that the **kind** is the programmatic discriminator and that consumers MUST NOT match on the detail text; a consumer that matched on `"manager output collection timed out"` was already outside the contract and is affected. The closure MUST record the change so the republication milestone's notes can state it.
- **The published `eggup-service 0.1.3` keeps the hand-rolled runner.** The consolidation lands in the workspace source only. `eggup-service 0.1.3` was published 2026-10-09 (source `feb6ae5`, checksum `9f7f7ea8…32124dc`) and is not republished by this milestone; no tag moves. A consumer of the published crate therefore continues to receive the old runner's exact behaviour until a separate republication milestone (§8.2's differences reach consumers only then).
- **No dependency change, so no downstream pin cascade.** `eggup-service` already depends on `eggup-core` by path (`crates/eggup-service/Cargo.toml:26`) at version `0.1.0`. `eggup-eggpack` pins its Eggup dependencies exactly (`=0.1.3`), so the core-side capability reaches it only through a separate adapter republication; this milestone does not touch that cascade.
- **No migration step is required of any consumer.** There is nothing to change in consumer code.

## 10. Required tests

The tests live in `crates/eggup-service/tests/` as new integration targets, plus a small number of crate-private assertions in `crates/eggup-service/src/lib.rs`'s `mod tests` where a crate-private helper is the subject.

**The portable-fixture requirement (applies to every test below).** The Windows CI lane runs the whole workspace suite — `cargo test --workspace --all-targets --locked --no-fail-fast` at `.github/workflows/ci.yml:90` — so a fixture that only works on Unix is a lane failure, not a skip. The existing executor tests (`crates/eggup-service/src/lib.rs:4205-4296`) use POSIX absolute paths (`/bin/sleep`, `/bin/echo`, `/bin/sh`, `/usr/bin/env`); on Windows `Path::new("/bin/sleep").is_absolute()` is **false**, so `resolve_manager_program` rejects it with `InvalidInput` rather than resolving it. The new fixtures MUST NOT repeat that pattern.

The plan therefore requires a **portable child fixture**: a dedicated `[[test]]` target with `harness = false` and its own `main`, whose `current_exe()` is the child program. The fixture child selects its mode from **argv** (not from the environment — the environment is cleared, which is the property under test), and can:

- echo its stdin to stdout (proves the payload is delivered);
- exit with a chosen code (proves a nonzero exit is `Ok` with the status);
- sleep (proves the timeout path);
- emit more than a configured number of bytes (proves the overflow path);
- print its environment (proves the environment is cleared and that only the session-bus variables are injected).

The fixture path is absolute, so it passes `resolve_manager_program` → `validate_executable_path` on every platform (on Unix it needs one execute bit, which the built test binary has).

| # | Test | Where | What it proves |
|---|---|---|---|
| **T1** | `runs_a_command_and_returns_its_status_and_bounded_output` | `crates/eggup-service/tests/bounded_executor_consolidation.rs` | A platform-neutral run of the fixture child returns `Ok` with `status == Some(0)` and the child's stdout bytes, within the output bound. Replaces the current Unix-only shape (`crates/eggup-service/src/lib.rs:4238-4251`). |
| **T2** | `delivers_the_stdin_payload_to_the_child` | same file | The `crontab -` shape: `run(&["<fixture>".into(), "-".into()], Some(payload), timeout)` — the child echoes its stdin and the test asserts the payload arrives **byte-for-byte**. The `"-"` argument mirrors the real `crontab -` argv (`:3564`). |
| **T3** | `rejects_a_zero_timeout_an_over_range_timeout_and_an_over_bound_stdin_payload` | same file | All three pre-flight rejections still return `ServiceError::InvalidInput` with the current messages: `"manager timeout out of range"` (zero, and above `MAX_TRANSITION_TIMEOUT + 60s`), `"manager stdin exceeds bound"` (a payload above `MAX_CRONTAB_BYTES`), plus the existing `"manager command has no program"` and `"manager command contains nul bytes"`. |
| **T4** | `classifies_a_missing_program_as_not_found_with_the_current_detail` | same file | A missing program still yields `ServiceError::manager_kind() == Some(ManagerFailureKind::NotFound)`. The test asserts **both** pre-flight paths separately: (a) a program that does not exist → the pre-flight `NotFound` with the detail naming the **requested** program (`validate_executable_path`, `:1264`), and (b) the spawn-failure arm's `NotFound` detail naming the **resolved** program (`:1060`) — see §8.2 D6. |
| **T5** | `a_timeout_yields_the_timeout_kind` | same file | A child that sleeps past the budget yields `ManagerFailureKind::Timeout` and the call returns within a bounded wall time. |
| **T6** | `an_output_overflow_yields_the_protocol_kind_and_discards_the_stream` | same file | A child that emits more than `with_max_output(n)` yields `ManagerFailureKind::Protocol` with the detail `format!("manager output exceeded bound for {program}")`. |
| **T7** | `a_nonzero_exit_is_ok_with_the_observed_status` | same file | A child that exits `3` yields `Ok` with `status == Some(3)`. |
| **T8** | `the_managers_argv_is_byte_identical` | `crates/eggup-service/tests/bounded_executor_consolidation.rs` | A golden regression guard over the argv the adapters build. It asserts the exact vectors for one representative spec of each manager — `systemctl` (`show`, `is-active`, `start`, `stop`, `enable`, `disable`, `daemon-reload`), `launchctl` (`list`, `bootstrap`, `bootout`, `kickstart`, `stop`), and `crontab` (`-l`, `-`) — against the exact `Vec<String>` the current builders produce. In addition, **no existing `fake.expect(...)` argv vector in any adapter test is edited**; if one must be edited, §14's stop condition fires. |
| **T9** | `the_child_environment_is_cleared_and_only_the_session_bus_variables_are_injected` | `crates/eggup-service/tests/bounded_executor_environment.rs` | A dedicated test binary (its own process, so `std::env::set_var` cannot race another test) sets `DBUS_SESSION_BUS_ADDRESS`, `XDG_RUNTIME_DIR`, `SYSTEMD_BUS_ADDRESS` **and** a sentinel such as `EGGUP_EXECUTOR_SENTINEL` in the parent environment, then runs (a) a non-`--user` command and (b) a `systemctl --user` command against a fixture binary copied to a temporary path named `systemctl` (a copy, not a symlink, so the fixture is portable). The child prints its environment. The test asserts (a) the environment is **empty**, (b) the child sees **exactly** the three session-bus variables and **not** the sentinel, and (c) the sentinel never appears in either case. |
| **T10** | `the_pre_flight_rejections_are_service_errors_not_core_errors` | `crates/eggup-service/tests/bounded_executor_consolidation.rs` | The four pre-flight rejections happen **before** any spawn: the test asserts `ServiceError::InvalidInput` (not a manager failure) and that no child was started. |
| **T11** | `the_public_api_surface_is_unchanged` | `crates/eggup-service/tests/public_api_surface.rs` (existing) | The M014 guard still passes with `crates/eggup-service/tests/public-api-surface.txt` **unmodified** (R-C3). No new file, no new assertion — the guard is run, not changed. |

**Fixture and test-hygiene rules.**

- A Unix-only fixture **must** be `#[cfg(unix)]`-gated **and** have a portable counterpart; a Unix-only fixture on its own is not acceptable, because the Windows lane runs the suite.
- No test may assert on a wall-clock duration tighter than the budget it exercises plus a generous allowance; the workspace has already been bitten by load-sensitive sub-second budgets (registry finding 1).
- No test may require root, a real service manager, or network access. The fixtures are the crate's own binaries.
- **One difference has no portable fixture.** **D7** (a stdin writer still blocked at the deadline) requires a descendant holding the child's stdin pipe open after the child exits. The implementer MUST NOT fake it with a timing assertion. D7 is **recorded** in `architecture/service-lifecycle.md` and in the closure record instead, and the closure MUST confirm that D7's two ordinary cases — the broken pipe (T2) and the still-running child at the deadline (T5) — are covered.
- The existing executor tests in `crates/eggup-service/src/lib.rs:4205-4296` are **kept** and continue to run on Unix. The implementer may re-home them into the new integration targets; if they are re-homed, the Unix coverage must not be lost. The implementer MUST NOT delete an existing test without replacing it with an equivalent.

## 11. Required verification commands

### 11.1 Crate-scoped (the implementer runs these)

These are scoped to `eggup-service` and use `--offline`. `cargo fmt` is invoked with `-p eggup-service` and **not** `--all`, so a parallel agent's in-flight edits elsewhere in the workspace cannot make this command fail spuriously.

```sh
cargo fmt -p eggup-service -- --check
cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings
cargo test  -p eggup-service --all-targets --all-features --locked --offline
cargo doc    -p eggup-service --no-deps --locked --offline
cargo tree   -p eggup-service --locked
```

Targeted runs:

```sh
cargo test -p eggup-service --locked --offline --test bounded_executor_consolidation
cargo test -p eggup-service --locked --offline --test bounded_executor_environment
cargo test -p eggup-service --locked --offline --test public_api_surface
```

Windows-target compile checks (the fixture and the adapter must compile for the Windows lane):

```sh
cargo check  --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service
cargo clippy --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service -- -D warnings
```

Dependency-boundary review step (not a gate; read it):

```sh
cargo tree -p eggup-service --locked
```

### 11.2 Workspace-wide (the **parent** runs these)

```sh
./scripts/check-local.sh
```

which runs, in order: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-targets --all-features --locked`; `cargo doc --workspace --no-deps --locked`; `cargo tree --workspace --locked`; `./scripts/check-docs-truth.sh`.

CI lanes affected by this milestone (the parent's hosted qualification, if one is run): `stable`, `macos`, `windows-check` (which executes the Windows lane's workspace suite at `.github/workflows/ci.yml:90`), and `msrv` (compile-only; `msrv` and `windows-check` do not pass `--all-features`).

## 12. Documentation updates

| Document | Change |
|---|---|
| `crates/eggup-service/README.md` | The process-execution bullet names `eggup_core::run_bounded` as the bounded-execution primitive the crate uses, and states that the crate's own pre-flight (allowlisted absolute paths, cleared environment, bounded stdin, timeout range) is unchanged. No public-surface bullet changes. |
| `crates/eggup-service/CHANGELOG.md` | The `Unreleased` section gains one entry stating that `SystemExecutor`'s bounded runner now delegates to `eggup_core::run_bounded`, that the observable outcomes are preserved under the documented mapping, and that the changes are **unpublished** — the published `0.1.3` crate is unaffected — with **no consumer migration required**. |
| `architecture/service-lifecycle.md` | § "Process execution: `SystemExecutor`" (`:507`) is updated: the runner is now a thin policy layer over `eggup_core::run_bounded`; the "Executable allowlisting and path resolution", "Environment clearing", "Output and timeout bounds", and "The stdin deadlock fix" subsections are re-anchored to the new code; a new subsection records the **seven** documented behaviour differences D1–D7 from §8.2 with their direction (fail-closed / narrowing / kind change in a deep corner / detail-text only); the "No shell interpolation" subsection keeps its statement. The `Ref` column entries that move are re-derived. The crate's test-distribution table (`:1173`) is updated to include the new fixture targets. |
| `architecture/core-transaction.md` | **Not edited.** Core's contract docs are M017's ownership. |
| `crates/eggup-core/docs/*.md`, `crates/eggup-core/README.md`, `crates/eggup-core/CHANGELOG.md` | **Not edited.** M017's ownership. |
| `plans/registry.md`, `plans/subsystems/service-lifecycle-roadmap.md`, root `CHANGELOG.md` | **Not edited by the implementer.** The drafted prose is in §16 for the parent to apply. |

## 13. Acceptance criteria

- **AC1.** `impl CommandExecutor for SystemExecutor` calls `eggup_core::run_bounded` and contains **no** hand-rolled spawn, reader-thread, writer-thread, or wait loop. `grep -n "std::process::Command" crates/eggup-service/src/lib.rs` returns **zero** hits for the service crate's own runner.
- **AC2.** All six pre-flight checks (R-A1) are present, in order, ahead of the core call, with the current error messages.
- **AC3.** The **resolved** absolute path — never the raw `argv[0]` — is what is handed to core. `resolve_manager_program` and `validate_executable_path` are unmodified.
- **AC4.** The mapping table (R-B1) is implemented exactly, including the fallback arm that names an unrecognised `operation`.
- **AC5.** Every observable outcome keeps its `ManagerFailureKind`. `ServiceError::manager_kind()` returns the same value as before the change for every reachable failure.
- **AC6.** The `ManagerCommandOutput` returned on success carries the same `status`, `stdout`, and `stderr` as before.
- **AC7.** The timeout handed to core is the caller's timeout less the elapsed pre-flight (R-B2); a zero remaining budget spawns nothing.
- **AC8.** The environment conversion follows R-B3 exactly: valid-UTF-8 pairs are byte-identical, non-UTF-8 pairs are dropped, and no lossy-converted value reaches the child.
- **AC9.** `crates/eggup-service/Cargo.toml` and `Cargo.lock` are byte-identical to the baseline. `cargo tree -p eggup-service --locked` shows no new dependency.
- **AC10.** The M014 public-API guard passes with `crates/eggup-service/tests/public-api-surface.txt` **unmodified** (R-C3).
- **AC11.** Every test in §10 exists, passes, and is not Unix-only without a portable counterpart.
- **AC12.** No existing test was deleted without an equivalent replacement, and **no existing `fake.expect(...)` argv vector was edited** (R-C1).
- **AC13.** The seven differences D1–D7 are each pinned by a test — except **D7**, which has no portable fixture and is therefore recorded in `architecture/service-lifecycle.md` and in the closure record with its two ordinary cases identified — and each is recorded in `architecture/service-lifecycle.md` and in the closure record.
- **AC14.** The crate's `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` are still in force, and no `unsafe` was introduced.
- **AC15.** Nothing was published, no tag moved, and no version was bumped.
- **AC16.** `plans/closure/service-lifecycle/015-status.md` exists and satisfies §15.

## 14. Stop conditions

The implementer **stops** and reports — without improvising a substitute — if any of the following holds. Each is checkable.

1. **Core M017's contract is not present.** `MAX_STDIN_BYTES`, `CommandSpec::stdin_data`, or the documented `operation` strings are absent from `crates/eggup-core/src/candidate.rs`. The consolidation cannot be implemented; the finding stays open. **Report which of the three is missing, with `file:line`.**
2. **Core M017's `operation` strings do not match the contract in §3.4.** The strings `"spawning bounded candidate"`, `"polling bounded candidate"`, `"writing bounded stdin"` differ in any way. **Stop and report the observed strings**; do not branch on a string the contract does not document.
3. **Core's `run_bounded` cannot express bounded stdin *and* an unconditional deadline that bounds the writer.** If a payload cannot be delivered under the same deadline, the consolidation does not happen and the finding stays open.
4. **Core's `Error` variant set has grown** beyond the seven published variants, or `Error::Io`'s fields are no longer public, so the mapping table cannot be written. **Stop and report**; core's error surface is not this milestone's to change.
5. **Any test shows a behaviour difference from the current executor beyond the seven documented differences D1–D7.** A difference outside that set is a regression in the consolidation, not a finding. **Stop, report the exact difference, and leave the tree green.**
6. **The pre-flight rejections cannot be preserved.** If any of the four `ServiceError::invalid` messages must change, or if the allowlist, the cleared environment, the stdin bound, or the timeout range check must be relaxed to make the consolidation work, **stop**.
7. **An existing `fake.expect(...)` argv vector must be edited**, or any adapter's argv changes. **Stop** — the argv surface is fixed.
8. **The M014 public-API guard fails, or its snapshot must be edited.** **Stop** — the consolidation was supposed to be internal.
9. **A new dependency, a manifest change, or a `Cargo.lock` change appears to be necessary.** **Stop** — `eggup-core` is already a path dependency; no addition is permitted.
10. **No portable fixture can be built** for the Windows lane (the `harness = false` fixture child). **Stop and report** rather than shipping a Unix-only fixture that would break `.github/workflows/ci.yml:90`.
11. **Core M017's `stdin_data` bound differs from the service's `MAX_CRONTAB_BYTES`** in a way that changes which of the two rejections a caller observes. **Stop and report both bounds.**
12. **A publication, a tag move, or a version bump appears to be required.** **Stop** — this milestone publishes nothing.

## 15. Closure evidence required

`plans/closure/service-lifecycle/015-status.md` must contain:

- **Status**: closed / conditionally closed / corrective pass required / blocked; the source plan and roadmap; the reviewed repository baseline; the implementation commits or the explicit statement that the working tree is uncommitted.
- **Executive finding**: what the duplication was, what the consolidation changed, and what it did **not** change. It must state plainly that this milestone discharges **Finding 1** of `plans/closure/service-lifecycle/012-status.md` and quote that finding's severity (**low**) and wording.
- **Requirement → evidence matrix**: one row per requirement — R-A1, R-A2, R-A3, R-A4, R-A5, R-B1, R-B2, R-B3, R-C1, R-C2, R-C3, R-C4, R-D1, R-D2, R-D3 — each with the exact `file:line` evidence and the exact command and observed output that satisfies it. The mapping table (R-B1) is reproduced **in the closure**, with a row per core result, so the mapping is reviewable without reading this plan.
- **Core-contract confirmation** (R-D1): the `file:line` for `MAX_STDIN_BYTES`, `CommandSpec::stdin_data`, and each of the three `operation` strings, quoted from the working tree.
- **Production implementation evidence**: the new `SystemExecutor::run` with `file:line` anchors for each pre-flight check, the `CommandSpec` construction, the deadline computation, the environment conversion, the `run_bounded` call, and each arm of the mapping; the deletion site of `read_bounded_generic` (or the reason it stayed).
- **The seven documented differences D1–D7**, each with its direction (fail-closed / narrowing / kind change in a deep corner / detail-text only), the test that pins it — with **D7** explicitly marked as *recorded, not tested, no portable fixture* — and — for **D5** — a **residual finding** with an explicit severity and reason.
- **Exact commands and results**: §11.1's commands with their real output, and the exact `cargo tree -p eggup-service --locked` output, showing no new dependency and an unchanged `Cargo.lock`.
- **Per-platform results**: **Linux, macOS, and Windows reported individually.** A platform that was **not run** is reported as **"not run"** and is **never inferred** from another platform's result. A CI lane's result is reported with its run identifier. The Windows lane's result names the fixture targets that executed.
- **Baseline-failure evidence**: per new fixture, the exact baseline command, the baseline SHA, and the exact observed failure or panic output. A fixture that does not fail at the baseline is not evidence and must be corrected or withdrawn. The mechanism used (a `git worktree` at the baseline SHA, for example) is recorded; no mutating git command is used in the primary working tree.
- **Documentation/operations evidence**: the `README.md`, `CHANGELOG.md` (`Unreleased`), and `architecture/service-lifecycle.md` updates, each with the exact changed lines.
- **Unresolved findings with severity**: at minimum the D5 residual; any other difference discovered during implementation, each with an explicit severity and reason.
- **Roadmap disposition** and **registry updates**: the intended rows for `plans/subsystems/service-lifecycle-roadmap.md` and `plans/registry.md`, noting that the parent applies them (§16 drafts them).
- **A statement that nothing was published and no tag moved.**

## 16. Handoff notes

### 16.1 Work packages (WP1–WP6), in order

1. **WP1 — Confirm the core-side contract** (read-only). `file:line` for `MAX_STDIN_BYTES`, `CommandSpec::stdin_data`, the three `operation` strings, and `Error::Io`'s public fields. Stop if anything is missing (§14.1, §14.2).
2. **WP2 — Implement the consolidation** in `crates/eggup-service/src/lib.rs`. Pre-flight → `CommandSpec` → `run_bounded` → mapping table.
3. **WP3 — Prove the equivalence** with the portable fixture and the §10 test targets.
4. **WP4 — Guard the untouched surfaces** (argv byte-identity, environment policy, the M014 API guard).
5. **WP5 — Update the documentation** (crate `README.md`, crate `CHANGELOG.md`, `architecture/service-lifecycle.md`).
6. **WP6 — Verify and close.** Run §11.1, then write `plans/closure/service-lifecycle/015-status.md` per §15.

### 16.2 Intended registry row

For `plans/registry.md` § "Dependency-ready implementation work" (the parent applies it), one new row:

> | Service lifecycle | M015 bounded executor consolidation corrective | closed \<date\> | `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md`; `plans/closure/service-lifecycle/015-status.md` | Closes M012 closure **Finding 1** (**low**): `SystemExecutor` now delegates to `eggup_core::run_bounded`, so the workspace holds one bounded-process-runner implementation instead of two. The service's own pre-flight is unchanged — allowlisted absolute paths, cleared child environment, bounded stdin, timeout range — and the `ManagerFailureKind` taxonomy, the public API, and every manager's argv are unchanged. Seven behaviour differences are recorded in the closure: a reader error is now fail-closed, core kills a still-running child on output overflow, the timeout detail text is normalised, the descendant-holds-pipe case ends at the deadline with the timeout detail, a non-UTF-8 environment pair is dropped rather than passed through, the spawn-failure `NotFound` detail names the resolved path, and a stdin writer still blocked at the deadline is reported as a timeout rather than a stdin failure. No new dependency (`eggup-core` was already a path dependency), no republication, `v0.1.3` unmoved |

And the matching update to the "Post-0.1.3 open findings" list, if the parent records the discharge there:

> The M012 closure's **Finding 1** (`SystemExecutor` is not consolidated onto `eggup_core::run_bounded`, **low**) is **closed** by Service M015: `plans/closure/service-lifecycle/015-status.md`. The duplication was plan-deferred pending core M014's deadline corrective and core's bounded-stdin capability; both are in place.

### 16.3 Intended roadmap row

For `plans/subsystems/service-lifecycle-roadmap.md` § 12 milestone status table:

> | M015 | closed \<date\> | `plans/implementation/service-lifecycle/015-bounded-executor-consolidation-corrective.md` | `plans/closure/service-lifecycle/015-status.md` | Closes M012 closure Finding 1 (**low**). `SystemExecutor` delegates to `eggup_core::run_bounded`: one bounded-process-runner implementation in the workspace. The service's pre-flight policy (allowlisted absolute paths, cleared environment, bounded stdin, timeout range), the `ManagerFailureKind` taxonomy, the public API, and every manager's argv are unchanged; seven documented behaviour differences are recorded in the closure. No new dependency, no republication, `v0.1.3` must not move |

The roadmap's header status line (`plans/subsystems/service-lifecycle-roadmap.md:3`) advances from *"M001–M011 closed"* to *"M001–M015 closed"*, and the "Next milestone" cell of the registry's "Active subsystem roadmaps" row for **Service lifecycle** becomes *"no further service-lifecycle milestone is registered"* — unless the parent registers another.

### 16.4 Intended root `CHANGELOG.md` bullet

The root `CHANGELOG.md` aggregates but does not replace the per-crate changelogs. For its `Unreleased` section:

> - `eggup-service`: `SystemExecutor` now runs manager commands through `eggup_core::run_bounded`, so the workspace holds one bounded-process-runner implementation. The crate's own safety policy — allowlisted absolute program paths, a cleared child environment with only the explicit user-scoped systemd session-bus variables, bounded stdin, and the timeout range check — is unchanged, as are the `ManagerFailureKind` taxonomy, the public API, and every manager's argv. **Unpublished**: the published `eggup-service 0.1.3` keeps the previous runner until a republication. No consumer migration is required.

### 16.5 Closure record

The closure record will be written to `plans/closure/service-lifecycle/015-status.md` **after** implementation, per §15. It is **not** part of this plan's deliverable.

### 16.6 Sequencing and boundary notes for the parent

- **Dependency**: this milestone is **blocked on Verified Update Core M017** (`plans/implementation/verified-update-core/017-bounded-stdin-execution-corrective.md`, landing in parallel). Its closure record will be `plans/closure/verified-update-core/017-status.md`. M017's own §16 names Service M015 as its downstream consumer, so the two milestones name each other consistently. M015's WP1 is the gate.
- **Ownership boundary**: this plan's implementation touches `crates/eggup-service/**` and `architecture/service-lifecycle.md` only. `crates/eggup-core/**` is M017's. `plans/registry.md`, `plans/subsystems/service-lifecycle-roadmap.md`, and the root `CHANGELOG.md` are the parent's.
- **No commit was made and no git state was mutated** by the authoring of this plan. The baseline is `20ac770dacfdcd3d82e734c22613bfccd36f9c09` on branch `plans/corrective-batch-2026-10-23`.
