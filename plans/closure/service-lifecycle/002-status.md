# Service Lifecycle M002 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/002-unix-manager-adapters.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md#M002--unix-manager-adapters`

Reviewed repository baseline: `8f6ce48cda5bdeb593939077bcca452cdd5f2800` (plan baseline; implementation ran at `892d6cc` plus adapter changes)

## Implementation commits/PRs

- M002 service commit (this pass): `feat: add Unix manager adapters (M002)` (pending SHA; see git log)
- Prior: `892d6cc` (`planning: register post-adoption implementation wave`)
- No PR was required for this local implementation pass. No publication was performed. No consumer migrated.

## Executive finding

M002 is complete. All three Unix manager families implement the closed M001
contract truthfully with bounded execution, exact ownership, caller-owned
definitions, and no implicit elevation. Foreign/Unknown mutation is
prevented, cron preserves unrelated entries, and consumer-specific
definitions remain outside. Test seams cover partial failure; package/MSRV/CI
inputs are green. No medium-or-higher Unix adapter issue remains.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| `ServiceSpec` remains identity; manager descriptors carry mechanics only | `SystemdInstall`, `LaunchdInstall`, `CronManager{marker, desired_block}` (no product/user/hardening fields) | passed |
| Bounded command runner, no shell | `SystemExecutor` (literal argv, null/controlled stdin, bounded output, deadline kill/reap) + `FakeExecutor` injection | passed |
| Null/controlled stdin, minimal env, bounded output, deadline, kill/reap, stable exit | `command_timeout_kills_and_reaps`, `command_output_bound_fails_closed`, `command_passes_argv_literally_without_shell`, `command_nonzero_exit_is_ok_with_status`, `command_missing_binary_fails_closed`, `command_environment_adds_nothing` | passed |
| systemd explicit system/user scope, no guessing | `SystemdScope::{System, User}` + `--system/--user` flags; scope in install descriptor | passed |
| systemd inspection (existence, exe/args, state) | `systemd_ownership` over `show -p LoadState,ActiveState,ExecStart` | passed |
| Narrow `ExecStart` parsing, ambiguous → Unknown | `parse_exec_start` (single `{path;argv[]}`, `argv[0]==path`, no `%$"'\\\`, absolute); `systemd_ambiguous_execstart_is_unknown`, `systemd_specifier_execstart_is_unknown` | passed |
| systemd Absent / Owned / Foreign / drift / Unknown | `systemd_absent_allows_install`, `systemd_exact_owned`, `systemd_same_unit_different_executable_is_foreign`, `systemd_argv_drift_is_foreign`, ambiguous tests | passed |
| Active/stopped/transitioning | `systemd_active_stopped_transitioning_mapped` + `systemd_start_stop_restart_confirm_state` | passed |
| Permission denial without sudo | `systemd_permission_denial_has_no_sudo` (hint mentions scope, never sudo) | passed |
| Atomic definition write fault | `systemd_atomic_write_fault_preserves_old`, `definition_write_*` | passed |
| Start/stop/restart confirmation | `systemd_start_stop_restart_confirm_state` (poll `is-active`, incomplete when unconfirmed) | passed |
| launchd explicit user-agent vs system-daemon | `LaunchdDomain`, `target` (`gui/<uid>` vs `system`), `launchd_user_vs_system_domain_distinct` | passed |
| launchd ownership incl. exe + args, no label-only | `parse_launchd_plist` (structured `ProgramArguments` scan); `launchd_absent_exact_foreign_unknown`, `launchd_no_label_only_ownership` | passed |
| Malformed plist → Unknown | `launchd_malformed_plist_is_unknown_and_denies_mutation` | passed |
| Bootstrap/bootout/kickstart bounded explicit | `launchd_bootstrap_bootout_kickstart_flow`; `bootstrap_on_install` explicit | passed |
| Foreign label fails closed | Foreign tests deny start/uninstall | passed |
| No EUID-based domain choice | Domain + target caller-supplied; wrong target rejected | passed |
| Cron managed block (marker + exact block) | `CronManager::new(marker, desired_block)`; pure `cron_merge`/`cron_remove`/`cron_classify` | passed |
| Cron list zero/one/multiple; exact Owned; drift Foreign/Unknown; preserve bytes; idempotent; owned-only remove; no redundant write | `cron_empty_absent_then_install`, `cron_preserves_unrelated_entries_byte_for_byte`, `cron_exact_block_is_idempotent`, `cron_duplicate_marker_is_unknown_conflict`, `cron_modified_block_is_foreign_no_overwrite`, `cron_uninstall_only_owned_block`, `cron_command_adapter_skips_redundant_write` | passed |
| Cron command timeout + missing binary | `cron_command_timeout_and_missing_binary` (Unknown, never assumed) | passed |
| Cron no start/stop semantics explicit | `cron_has_no_start_stop_semantics` (InvalidInput, not fake success) | passed |
| Definition file safety | `atomic_write_definition` + `definition_write_is_atomic_private_and_no_clobber`, `definition_write_rejects_missing_parent` | passed |
| Host detection vs policy | `inspect_host` + `candidate_managers`; `host_detection_*`, `candidate_policy_*` | passed |
| Native smoke lanes marked/skipped | `native_*_smoke_*` (systemd gated on `/run/systemd/system`, launchd on macOS, crontab list-only) | passed |
| No consumer-specific constants | `adapter_rejects_consumer_specific_constants` | passed |

## Adapter public API

```rust
CommandOutput::{status, stdout, stderr}
CommandExecutor::run(argv, stdin, timeout)
SystemExecutor::{new, with_max_output}
FakeExecutor::{expect, expect_error, calls, call_count, is_exhausted}
atomic_write_definition(path, bytes, allow_overwrite)
SystemdScope::{System, User}
SystemdInstall::new(unit_name, scope, unit_path, definition, enable, reload, transition_timeout)
SystemdManager::new(executor, install)
LaunchdDomain::{UserAgent, SystemDaemon}
LaunchdInstall::new(label, domain, target, plist_path, definition, bootstrap_on_install, transition_timeout)
LaunchdManager::new(executor, install)
CronOwnership::{Absent, Owned, Foreign, Unknown}
cron_classify / cron_merge / cron_remove
CronManager::new(executor, marker, desired_block, transition_timeout)
HostFacts / inspect_host / CandidateManager / candidate_managers
```

M001 types unchanged (`ServiceSpec`, `Ownership`, `LifecycleState`,
`ServiceManager`, `TestDoubleManager` all source-compatible).

## systemd/launchd/cron ownership matrices

systemd (`show` → spec with exe/args, config must be in args):

| Observation | Result |
|---|---|
| `LoadState=not-found` | `Absent` |
| `loaded` + exact path + argv | `Owned` |
| Same unit, different path | `Foreign` |
| Same path, different argv | `Foreign` |
| Zero/multiple `ExecStart`, specifiers, quotes, relative path | `Unknown` |
| `loaded` with other `LoadState` | `Unknown` |

launchd (plist file + `list` for loaded/PID):

| Observation | Result |
|---|---|
| No plist file | `Absent` (label alone never `Owned`) |
| Exact `ProgramArguments` | `Owned` |
| Same label, different program/args | `Foreign` |
| Missing/duplicate `ProgramArguments`, bad XML, relative exe | `Unknown` |
| User vs system target mismatch | constructor rejects (no silent choice) |

cron (managed block):

| Observation | Result |
|---|---|
| Zero blocks | `Absent` |
| One exact block | `Owned` |
| Same marker, different content | `Foreign` (no overwrite) |
| Multiple/unterminated blocks | `Unknown`/`Conflict` (fail closed) |

## Command-bound tests

- Timeout kills and reaps (`/bin/sleep 5` with 200 ms → `Manager(timed out)` well under 4 s).
- Output bound fails closed (1 KiB via 64-byte bound → `Manager(bound)`).
- Literal argv (semicolons/pipes/dollars/`id` passed through `/bin/echo` unchanged).
- Nonzero exit is `Ok(status)` for caller classification.
- Missing binary is `Manager(missing)`.
- Environment adds nothing (no injected secrets).

## Definition-write fault evidence

- New files land `0644` (temp `0600` then mode set); overwrite preserves existing mode.
- `create_new` temp with 32-collision bound; same-dir (same filesystem); `fsync` file + best-effort parent.
- No-replace (`hard_link`) for creation vs `rename` for owned refresh (Windows `AlreadyExists` → remove-owned + rename, authorized by re-inspection).
- Missing/symlinked parent fails without creating; only the owned temp is ever removed.
- systemd fault test uses a file-as-parent; launchd/cron paths share the helper.

## Cron byte-preservation fixtures

- Empty → append with single trailing newline.
- Existing content preserved verbatim as prefix; removal restores exact original bytes.
- Idempotent reinstall changes nothing and skips `crontab -`.
- Duplicate markers and foreign content fail without touching bytes.

## Native/platform test matrix

| Lane | Evidence |
|---|---|
| Linux systemd parsing/fixtures | Deterministic `FakeExecutor` show/is-active tests (no booted systemd required) |
| Native systemd smoke | `native_systemctl_parsing_smoke_where_booted` (skips without `/run/systemd/system`) |
| macOS launchd parsing/fixtures | Deterministic plist + `list` tests |
| Native launchd smoke | `native_launchd_smoke_on_macos_only` (skips off macOS) |
| Cron pure + command | Pure merge/remove + `FakeExecutor` `-l`/`-` tests; native list-only smoke never mutates |
| Windows SCM | Explicitly out of scope (M003); no Windows-only behavior inferred |

Hosted Linux/macOS runners exercise the fixture tests; native registration
creation is never attempted in tests.

## Dependency/package results

- `cargo tree -p eggup-service`: 1 line, zero dependencies (no transport/update-policy coupling preserved).
- `cargo package -p eggup-service --allow-dirty`: green (6 files, 139.4 KiB, 27.8 KiB compressed).
- `cargo doc` clean on stable and 1.89 with `#![deny(missing_docs)]`.

## Consumer-specific code explicitly rejected from extraction

- No product names, fixed paths, daemon users, hardening directives, health JSON, cron command content, or CLI messages in the crate (proven by `adapter_rejects_consumer_specific_constants`).
- Install descriptors carry only unit/label identity, definition path/scope, exact caller bytes, marker/block, and transition timeout.

## Exact tests/commands actually run

Environment: `Darwin 25.6.0 arm64`, stable `1.98.1`, MSRV `1.89.0`.

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-service --locked
cargo package -p eggup-service --locked --allow-dirty
./scripts/check-local.sh
```

MSRV variants (`cargo +1.89.0 …`) run for check/clippy/test/doc: green.

Results at closure: 49 `eggup-service` (10 M001 + 39 M002) + 26 acquisition +
37 core + 23 eggfetch + 18 dist unit + 4 dist fixtures = 157 passed, 0 failed,
on stable and 1.89.

## Invariant review

- Foreign/Unknown never destructively mutated (all destructive paths `require_owned` or deny; re-inspection before overwrite/removal).
- Ownership never from name/label alone (exe + args required; plist `ProgramArguments` required; cron block content required).
- Stopped stays stopped (install leaves stopped; no auto-start; `Preserve` default untouched).
- All manager calls bounded (deadline + kill/reap + output bound; polling never assumes on timeout).
- No shell interpolation (literal argv everywhere; proven by injection test).
- No sudo/elevation/UAC (permission errors return remediation text).
- Cron preserves unrelated bytes; no redundant writes; races exposed via re-read + `Conflict`.
- Health remains a separate consumer probe (adapters report `Unknown` health, never mutate it).
- No release/update/network policy in `eggup-service` (zero deps preserved).

## Failure/recovery review

- Command timeout → `Manager`, never assumed state; transitions report `completed: false` when unconfirmed.
- Definition write failure → old definition preserved; no partial promotion.
- Reload/bootstrap failure after write → incomplete transition, evidence preserved.
- Foreign/Unknown → no mutation, ever.
- Concurrent external changes → re-inspect before overwrite/removal; crontab races return `Conflict`.
- Cron list/install races cannot be perfectly CASed via `crontab`; documented with re-read + conflict.
- No background workers or implicit retries.

## Compatibility/migration review

No consumer migrated. M001 public types source-compatible; manager-specific
types are new 0.1.x API. Any future M001 break would carry migration notes
(not triggered). No eggsearch/Gregg constants copied.

## Security review

- No symlink/privilege escalation: parents must be real dirs; temps exclusive + private; no `create_dir_all` on live paths; no `chown`/mode widening.
- Bounded inputs throughout (ids, labels, markers, blocks, definitions, outputs, diagnostics at 512).
- No secrets in specs (exe/args/config only); diagnostics never embed definition bytes.
- `#![forbid(unsafe_code)]`.

## Docs/operations evidence

- `crates/eggup-service/README.md`: M002 mechanics, ownership rules, scope/domain/cron contracts, no-health/no-policy notes.
- Rustdoc on every new public type/method with ownership/failure/privilege semantics.
- `CHANGELOG.md`: M002 entry.
- `cargo doc` clean on stable and 1.89.

## Unresolved findings with severity

- None (M002 scope). Windows SCM (M003) and update-lifecycle orchestration (M004) explicitly deferred.
- Informational: Hosted CI not run locally; native smoke lanes skip safely where the manager is unavailable.

## Disposition and roadmap transition

M002 is closed. Unblocks service M003 (Windows SCM) detailed planning against
M002 adapter evidence, and makes eggsearch's service portion dependency-ready
(without migrating eggsearch). M004 orchestration still waits for M003 + core
per the roadmap. No ADR was required: no incompatible ownership vocabulary,
no full systemd grammar, no external locking beyond `crontab` re-read, no
caller-owned privilege violation, and no substantial M001 break.

## Registry updates

- Service M002 → closed.
- Service M003 → planning-ready (detailed plan waits for this M002 evidence; may now be authored).
- eggsearch service portion → dependency-ready (migration still out of scope).
