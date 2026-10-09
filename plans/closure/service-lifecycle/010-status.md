# Service Lifecycle M010 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/010-owned-failed-systemd-service-quiescence-corrective.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M010--owned-failed-systemd-service-quiescence-corrective`

Reviewed repository baseline (plan): `70ec4e63c52988bc6c82bea30d14f72eefda920a`

Implementation source: `753bc7c281a9b8bfa29dafea46805a7245060409`

Hosted qualification: GitHub Actions run [`37874476249`](https://github.com/eggstack/eggup/actions/runs/37874476249), exact head `753bc7c281a9b8bfa29dafea46805a7245060409`, all five jobs green.

## Executive finding

M010 found a required Eggup runtime correction. On Linux systemd 255, an exact-owned
unit whose process had exited remained `ActiveState=failed` after a successful
`systemctl stop`; its `is-active` result remained `failed` (exit status 3), and
the unit had no control group. The previous stop loop accepted only
`inactive`, so it could not report completion. The published `eggup-service
0.1.2` therefore does not supply this behavior, and downstream M011 is
applicable.

The correction keeps the public API unchanged. Only a recognized `failed` state
with exact proven ownership enters the failed-unit path. It rechecks unit
identity around mutation and proof, verifies stable manager state and no pending
job, requires zero MainPID/ControlPID, and validates cgroup v2 process/event
evidence whenever systemd supplies a control group. The final `is-active`
observation must agree with `failed` or `inactive`. Unknown or contradictory
observations and unconfirmed quiescence return incomplete/refusal; they never
authorize rollback. The adapter does not clear systemd failure history, restart
the service, kill processes directly, elevate privileges, or add dependencies.

Real-systemd qualification also demonstrated the negative case: a failed unit
with a surviving child in its cgroup returned `completed=false` at the bounded
deadline, while the process remained alive. A changed-ExecStart unit was denied
without stopping the foreign running service. Caller repair and a known-good
start/stop succeeded.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Exact Owned authority before mutation | Failed-specific path revalidates parsed `ExecStart`; real foreign/changed-ExecStart control remained active and returned ownership denial | passed |
| Failed diagnosis does not mean quiescent | Real failed fixture remained `ActiveState=failed`, `is-active=failed`/3, with MainPID=0, ControlPID=0, no Job, and no ControlGroup after successful `systemctl stop` | passed |
| Manager proof and independent task proof | Stable post-stop observation, manager-job check, zero process IDs, and validated cgroup-v2 `cgroup.procs`/`cgroup.events` when present; residual child fixture remained populated and incomplete | passed |
| Restart/start-limit case | Real fixture recorded two distinct failed ExecStart invocations, rejected a third start without changing invocation identity, and retained systemd's “Start request repeated too quickly” journal evidence; subsequent stop completed | passed |
| Ownership/state races and malformed data | Deterministic fake-executor tests cover identity flap, unknown state, active/restart race, inconsistent status, unsafe cgroup, remaining process, and bounded timeout; mutation refusals remain fail-closed | passed |
| Ordinary running stop and repair path | Real systemd disposable sleep unit started and stopped after caller repaired its unit definition | passed |
| Public API compatibility | No public signatures or enum variants changed; external fixture resolves registry `eggup-service =0.1.2` and compiles wildcard matching for the already-`#[non_exhaustive]` `LifecycleState` | passed |
| Platform/MSRV qualification | Exact-head Stable, Rust 1.89 MSRV, macOS, Windows, and Linux systemd jobs green in run `37874476249` | passed |
| Package integrity / dependency scope | `cargo package` verifies 10-file service package; dry-run upload succeeds; no dependency or workspace manifest changes | passed |
| Documentation and plan governance | Service README/changelog, root changelog, service architecture, source plan, roadmap, registry, and this closure updated | passed |

## Production and real-systemd evidence

The systemd observation now includes `SubState`, `MainPID`, `ControlPID`,
`ControlGroup`, and `Job` in addition to load/active state and executable
identity. The failed-specific path is private to the systemd adapter and
preserves `LifecycleState::Unknown` for the generic public observation. The
successful stop command is not treated as sufficient evidence: ownership and
the final state are reread, cgroup paths are validated before use, and
contradictory `is-active` results prevent completion.

The hosted fixture ran on Ubuntu 24.04.5 LTS, kernel `6.17.0-1022-azure`, systemd
`255 (255.4-1ubuntu8.17)`, cgroup v2. The failed `/usr/bin/false` fixture was
observed both before and after stop as `LoadState=loaded`, `ActiveState=failed`,
`SubState=failed`, `Result=exit-code`, `MainPID=0`, `ControlPID=0`, `Job=` and
`ControlGroup=`. `systemctl stop` exited 0; `is-active` returned `failed` with
status 3; Eggup returned `completed=true` only after that state was independently
shown to have no cgroup.

The residual-process control used `KillMode=process`: its failing wrapper left a
live `/usr/bin/sleep` child in
`/system.slice/eggup-m010-…-residual.service`; cgroup evidence was
`cgroup.procs=<child pid>` and `cgroup.events=populated 1`. Despite
`systemctl stop` exiting 0 and the manager reporting failed with zero MainPID,
Eggup returned `completed=false` at the 5-second deadline. This directly
discriminates process quiescence from the systemd failure diagnosis.

The restart fixture used a 60-second `StartLimitIntervalSec` and
`StartLimitBurst=2`. It observed different invocation PIDs for the first two
failed starts and the same `ExecStart` invocation after the rejected third
request; the journal recorded the start-limit message. This avoids depending on
the optional `NRestarts` show property or localized journal text as a test gate.

The test uses disposable system unit files under `/run/systemd/system`, removes
them, and reloads systemd on teardown. It does not alter unrelated units. The
real native lane qualifies system scope; user scope is not exercised by the
hosted VM, while deterministic adapter tests cover the shared proof logic.

## Exact verification commands and results

On the implementation tree:

```text
rtk cargo fmt --all
  passed
rtk cargo test -p eggup-service --all-targets --all-features --locked
  110 passed (3 suites)
rtk cargo package -p eggup-service --list --locked
  passed; 10 package files
rtk cargo package -p eggup-service --locked
  passed; 10 files, 395.7 KiB unpacked, 72.5 KiB compressed
rtk cargo publish -p eggup-service --dry-run --locked
  passed; upload verified, aborted due to dry-run
rtk cargo check --manifest-path crates/eggup-service/tests/published-api-0.1.2/Cargo.toml --locked
  passed; exact registry-only 0.1.2 compatibility fixture
rtk cargo info eggup-service@0.1.3 --registry crates-io
  confirmed absent from crates.io at qualification time
rtk git diff --check
  passed
```

Package qualification produced `target/package/eggup-service-0.1.3.crate`,
72.5 KiB compressed, SHA-256
`9d5f0246bf0e61b2b53c38049294c9b611c3205a5931f514e11719177e5d240c` at source
`753bc7c`. This is an M011 preflight artifact, not a release artifact: M011 must
rebuild and record the checksum from its final clean release-preparation source
commit. Crates.io reported no `eggup-service 0.1.3` version. No publish command
without `--dry-run` was issued.

Exact hosted run `37874476249` results:

| Lane | Result |
|---|---|
| Stable Linux: fmt, clippy, full workspace all-target/all-feature tests, docs | success |
| Rust 1.89 MSRV: workspace/all-target `cargo check` | success |
| macOS: full workspace/all-target/all-feature tests | success |
| Windows: named acquisition/archive/curl/eggpack/service/Core tests and full workspace test suite | success |
| Linux systemd: published 0.1.2 registry fixture plus privileged systemd integration matrix | success |

Earlier hosted attempts `37874216157` and `37874306959` exposed fixture
assumptions (localized journal wording, and a deliberately failing service's
start exit status). They were corrected in the test harness; final exact-head
run `37874476249` supersedes them and passed every lane.

## Invariant, failure, and recovery review

- Foreign, changed, missing, malformed, unknown, or inconsistent observations
  cannot authorize a stop or successful quiescence.
- `ActiveState=failed`, zero MainPID, a successful stop command, and a status-3
  `is-active` result are each insufficient alone. Completion requires the
  complete observation protocol and cgroup proof when a group exists.
- A stop deadline or live cgroup task returns an incomplete bounded result. The
  caller retains rollback/recovery authority; Eggup does not start a service or
  mutate artifacts in response.
- Restart and identity races fail closed; the real rate-limit fixture confirms
  the failed state can preserve manager diagnostics without `reset-failed`.
- No lock, artifact transaction, database compatibility, or consumer recovery
  policy was moved into the service adapter.

## Compatibility and security review

Public `LifecycleState`, `Ownership`, `ServiceManager`, and `TransitionResult`
surfaces are unchanged; `LifecycleState` was already `#[non_exhaustive]`. The
plan's initial “closed enum” wording was corrected before acceptance. The
registry-only consumer check used published `eggup-service 0.1.2` and its
registry dependency graph, with no workspace path or Git override. Rust 1.89 and
all supported hosted target lanes passed. No dependencies, unsafe code,
privilege escalation, arbitrary command construction, or service-manager
fallback were added.

The cgroup path is accepted only when rooted, component-safe, and accessed below
the validated cgroup v2 root. Symlink/unsafe/unreadable paths and conflicting
manager evidence do not become empty-cgroup success. Diagnostics remain bounded
through the existing executor and error paths.

## Documentation, unresolved findings, and roadmap disposition

Updated `architecture/service-lifecycle.md`,
`crates/eggup-service/README.md`, `crates/eggup-service/CHANGELOG.md`, and the
root `CHANGELOG.md`. Corrected an existing stale architecture statement that
said service 0.1.2 was unpublished. Updated the source plan, roadmap, registry,
and closure record. No user-facing guide needed a new code example.

No unresolved M010 finding at medium-or-higher severity. The hosted systemd
evidence is Linux system-scope only; this does not weaken the cross-platform
compile/test evidence and is stated rather than inferred for user scope.

Roadmap disposition: M010 is closed with a required runtime source change.
Service M011 is now dependency-ready and applicable; it owns publication of
the qualified service behavior if and only if explicit maintainer publication
authorization is obtained. The existing published 0.1.2 does not meet M010's
failed-owned-unit completion contract. The separate wg-basic C001a consumer
handoff remains pending the immutable registry release evidence from M011.

## Registry updates

- Service Lifecycle M010: `closed`; this record is authoritative.
- Service Lifecycle M011: applicable and dependency-ready after M010 runtime
  closure; release preparation can proceed, while registry publication remains
  blocked on explicit maintainer authorization.
- No other implementation plan's hard dependency changed as a consequence of
  M010.
