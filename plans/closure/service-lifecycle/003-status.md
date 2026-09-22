# Service Lifecycle M003 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md`

Source roadmap: `plans/subsystems/service-lifecycle-roadmap.md`

Reviewed repository baseline: `01554361be11ea4d851610eb8aee8840d3048e01`

## Implementation commits/PRs

- Implementation: `01554361be11ea4d851610eb8aee8840d3048e01` (`fix: harden Unix service adapters`).
- No PR was required. Push to `main` triggered hosted CI. No publication or consumer migration was performed.

## Executive finding

M003 is complete. systemd and launchd start/stop/restart operations use a
single caller deadline for ownership checks, commands, polling, and composed
restart phases. launchd restart cannot continue after an incomplete stop or
report completion after an incomplete start. Production manager commands use
validated absolute executables and a cleared environment with only documented
user-systemd session variables retained. systemd and launchd ownership
reconcile optional config identity against exact argv. Mutation exit statuses
fail explicitly. No medium-or-higher M003 finding remains open.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| launchd incomplete stop | `launchd_restart_does_not_start_after_incomplete_stop`; result is incomplete or a typed deadline error, and no bootstrap/kickstart occurs | passed |
| launchd incomplete start | `LaunchdManager::restart` propagates the start phase `completed` value | passed |
| systemd restart confirmation | `systemd_start_stop_restart_confirm_state`; `poll_active` reports incomplete or timeout | passed |
| TestDouble composition | `TestDoubleManager::restart` propagates stop/start results | passed |
| One deadline per transition | `OperationDeadline`, remaining-time command budgets, bounded sleeps, launchd elapsed-budget regression | passed |
| Zero timeout | `transition_deadline_rejects_zero_and_only_shrinks` | passed |
| Trusted executable resolution | fixed absolute paths for allowlisted bare names; validated absolute overrides; fake PATH candidate regression | passed |
| Minimal environment | system scope clears all variables; user systemd retains only three documented session variables; sentinel test and empty child `env` output | passed |
| Config identity | systemd and launchd matrices: exact → Owned, missing/different → Foreign, repeated → Unknown | passed |
| Failed systemd observation | nonzero `show` status maps to Unknown in ownership and inspection | passed |
| Mutation statuses | systemd lifecycle/enable/disable/reload, launchd bootstrap/bootout/kickstart/stop, cron write all check status | passed |
| M002 regressions | full workspace suite, including ownership and cron byte-preservation fixtures | passed |
| Docs/package | README, rustdocs, changelog; workspace docs and service package verification | passed |

## Production implementation evidence

- `OperationDeadline` validates `(0, MAX_TRANSITION_TIMEOUT]`; each command
  receives only remaining monotonic time. Start/stop/restart construct one
  deadline at entry, and launchd restart shares it across both phases.
- `SystemExecutor` uses absolute program paths and clears the child environment.
  Absolute overrides must be executable regular files. Bare names are limited
  to `systemctl`, `launchctl`, and `crontab` in fixed platform directories.
- User-scoped systemd retains only `DBUS_SESSION_BUS_ADDRESS`,
  `XDG_RUNTIME_DIR`, and `SYSTEMD_BUS_ADDRESS` when present. System scope
  retains no inherited variables.
- Config is observed only when the exact configured path appears once in
  expected and observed argv. Missing identity is Foreign; repeated or
  non-UTF-8 identity is Unknown.
- Mutation commands reject nonzero statuses. Expected nonzero query codes are
  interpreted only in the corresponding state-query logic.

## Exact commands and results

Environment: Darwin 25.6.0 arm64; stable `rustc 1.98.1`; Rust `1.89.0`.

Passed locally:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo test -p eggup-service --locked                         # 55 passed
cargo test --workspace --all-targets --all-features --locked # 172 passed
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo doc --workspace --no-deps --locked
cargo package -p eggup-service --locked --allow-dirty
cargo +1.89.0 check --workspace --all-targets --locked
cargo +1.89.0 test -p eggup-service --locked                 # 55 passed
./scripts/check-local.sh
```

Hosted CI run `35787502494` passed stable fmt/clippy/workspace tests/docs,
macOS workspace tests, Windows workspace check, and Rust 1.89 workspace
check. Native tests do not create, boot, stop, or remove services.

Native Darwin read-only observations:

- `/bin/launchctl version` succeeded and reported Darwin Bootstrapper 7.0.0.
- `/usr/bin/crontab -l` reported no user crontab; no write was attempted.
- systemd is unavailable on this Darwin host. Hosted Linux tests cover
  deterministic systemd fixtures, not a live systemd manager.
- Windows evidence is compile/check only; Unix manager runtime behavior is
  not inferred from that lane.

## Invariant, recovery, and compatibility review

- Foreign/Unknown registrations remain denied before mutation.
- An exhausted deadline never grants a later phase a fresh full timeout;
  launchd does not attempt start after an unconfirmed stop.
- Nonzero mutations cannot become successful transitions.
- Ambient PATH is not consulted. Child credentials, proxy, loader, and
  arbitrary application variables are absent.
- Missing config identity remains Foreign; ambiguous identity remains Unknown.
- Cron preserves unrelated bytes; prior M002 fixtures remain green.
- Health remains consumer-owned. No elevation, shell, network, release policy,
  or service content was added.
- No consumer has migrated; `ServiceSpec` and ownership vocabulary are
  unchanged. No ADR or consumer migration was required.

## Unresolved findings

- Informational: no live Linux systemd host was available; deterministic
  fixtures and hosted Linux CI cover parsing and command behavior.
- Informational: Windows lane is a workspace compile/check, not SCM runtime
  evidence; SCM remains a future milestone.
- None: no medium-or-higher Unix adapter issue remains open for M003.

## Disposition and roadmap transition

M003 is closed. Service M004 Windows SCM is dependency-ready for plan
authoring. Consumer M003 eggsearch is dependency-ready because acquisition
M004 and service M003 are closed. Distribution M003 validators remain
dependency-ready. Gregg M004 remains blocked only on corrected-path footprint
evidence; optional acquisition M005 remains deferred on that measurement.
Service M005 update-lifecycle integration remains blocked on M004 plus core.
No service-aware consumer migration was performed.

## Registry updates

- Service M003 → closed.
- Service M004 Windows SCM → dependency-ready for plan authoring.
- Service M005 update-lifecycle integration → remains blocked on M004 + core.
- Consumer M003 eggsearch → dependency-ready for plan authoring.
- Consumer M004 Gregg → remains blocked on corrected-path footprint evidence.
- Distribution M003 validators → remains dependency-ready for plan authoring.
- Acquisition M005 lightweight adapter → remains deferred/evidence-driven.
