# Service Lifecycle Milestone 003 — Unix Adapter Correctness and Execution Hardening Corrective

Status: ready for handoff

Repository baseline: `889a234cbe7f461d92def3df45c83c06a7d257e5`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Corrects post-closure findings from:

- `plans/implementation/service-lifecycle/002-unix-manager-adapters.md`
- `plans/closure/service-lifecycle/002-status.md`

Primary class: invariant / corrective

## 1. Objective

Correct the Unix service adapters before any real service-bearing consumer migrates.

The corrective must address four related contract gaps:

1. launchd restart must not report success when stop/start completion is unconfirmed;
2. caller transition deadlines must bound the complete requested operation rather than individual substeps using independent/fixed timeouts;
3. production manager execution must not trust ambient PATH or inherit an unrestricted process environment;
4. systemd/launchd ownership must reconcile the manager-neutral `ServiceSpec.config` identity instead of treating any non-`None` config as effectively unrepresentable.

## 2. Readiness and dependencies

Service M001 and M002 are closed.

No consumer has migrated to the Unix adapters yet, so this is the correct point to tighten behavior before eggsearch/Gregg adoption.

Windows SCM and update-lifecycle orchestration are not implemented and must remain blocked until this corrective closes.

## 3. Current evidence and detection gaps

### A. launchd restart completion

`LaunchdManager::restart` currently:

- calls `stop(spec, timeout)?`;
- calls `start(spec, timeout)?`;
- discards each returned `TransitionResult.completed`;
- returns `Restart { completed: true }`.

Both stop/start can legitimately return `Ok(... completed: false)` when state confirmation times out.

Detection gap: M002 tested the happy restart flow but not an `Ok + incomplete` subtransition.

### B. End-to-end deadline drift

Examples at baseline:

- systemd `poll_active` executes each `is-active` command with a fixed 5 s command timeout;
- launchd confirmation calls `loaded_and_running`, which uses the install descriptor transition timeout;
- restart may spend the caller timeout separately in stop and again in start;
- `bounded_timeout(0)` substitutes a nonzero default rather than preserving a clear caller contract.

The API describes operations as occurring "within timeout"; the implementation can exceed that wall-clock budget.

Detection gap: M002 bounded individual commands but did not test end-to-end elapsed budget.

### C. Manager program/environment trust

`SystemExecutor` launches bare names such as `systemctl`, `launchctl`, and `crontab` through inherited PATH and inherits the full parent environment.

This creates avoidable executable-resolution and environment-influence risk, especially when a caller runs a system-scoped operation with elevated privileges.

Detection gap: M002 tested "no shell" and "adds nothing" but treated full environment inheritance as acceptable without separately qualifying PATH/manager control.

### D. `ServiceSpec.config` identity

The neutral model includes a separate optional config path.

Systemd and launchd observations currently populate `RegistrationSnapshot.config = None`; the systemd source comments explicitly note that a spec with `Some(config)` becomes Foreign unless the caller avoids the field.

This is narrower than the M001 identity contract and likely to surface immediately in eggsearch/Gregg.

Detection gap: M002 adapter tests encoded config only in argv and did not exercise a neutral spec with `config = Some(...)`.

## 4. Invariants that must not regress

- Foreign/Unknown registrations are never mutated;
- manager name alone never proves ownership;
- exact executable + critical argv + configured config identity participate in ownership;
- every `TransitionResult.completed=true` means the requested terminal state was actually confirmed where confirmation is part of that operation;
- a caller-supplied transition timeout is an end-to-end wall-clock budget;
- restart shares one budget across all required stop/start/confirmation work;
- manager programs are resolved through trusted absolute paths or an equally explicit caller-owned mechanism, never arbitrary ambient PATH;
- manager execution uses a minimal, documented environment;
- no shell;
- no implicit sudo/elevation;
- user-scoped systemd retains only environment needed for the native manager connection;
- health remains consumer-owned.

## 5. Scope and non-scope

### In scope

- launchd restart completion propagation;
- similar composition audit for test double/systemd;
- deadline/deadline-remaining helper;
- command timeout derived from remaining operation budget;
- trusted manager binary resolution/configuration;
- explicit manager environment policy;
- tests against PATH substitution;
- config identity reconciliation for systemd/launchd;
- documentation and compatibility evidence.

### Out of scope

- Windows SCM;
- update transaction/service orchestration;
- health probes;
- consumer migration;
- rendering consumer unit/plist/cron content;
- privilege escalation;
- service user/group creation;
- publication.

## 6. Required production changes

### A. Truthful composed transition results

For launchd restart:

- use one restart deadline;
- execute stop phase;
- if stop returns `completed=false`, do not claim restart success;
- only continue to start if doing so is semantically safe and the plan documents why;
- if start returns `completed=false`, restart returns `completed=false`;
- `completed=true` only after final Running confirmation.

Audit TestDoubleManager and systemd for the same rule.

A helper for composing `TransitionResult` is acceptable if it does not erase phase detail.

### B. End-to-end deadline object

Introduce a private monotonic deadline/remaining-budget helper.

For every start/stop/restart operation:

- validate the caller timeout;
- compute one deadline at operation entry;
- each manager command receives at most the remaining time;
- each polling loop sleeps only within remaining budget;
- no subcommand uses a fixed timeout greater than remaining budget;
- restart stop+start share the same deadline.

Zero timeout semantics must be explicit. Preferred: reject zero for operations requiring a bounded wait rather than silently substituting 10 s. If preserving existing zero behavior is required for compatibility, document and test the normalization.

### C. Trusted manager program resolution

Do not invoke production service-manager tools by an unqualified ambient-PATH name.

Define a small manager-program configuration/resolver that produces absolute paths.

Acceptable approaches:

- adapter constructor requires explicit absolute manager path; or
- platform resolver searches a small hard-coded trusted directory set and verifies a regular executable; or
- default trusted path plus explicit caller override.

Requirements:

- absolute path in production `SystemExecutor` calls;
- no current-directory resolution;
- no arbitrary inherited PATH lookup;
- caller override must itself be absolute and validated;
- fake executor remains deterministic.

Do not over-generalize into a general executable-discovery framework.

### D. Minimal environment policy

Change `SystemExecutor` from "inherit everything" to an explicit environment policy.

System scope should normally run with a cleared environment plus only documented required variables.

User-scoped systemd may require session-bus context. Preserve only explicitly required keys such as platform/session manager variables after evidence, not the entire parent environment.

Do not preserve:

- PATH for manager resolution;
- credential/token variables;
- arbitrary proxy variables;
- dynamic-loader injection variables.

Tests should inject sentinel environment variables and prove they are absent from the child-visible environment except allowlisted keys.

If a native manager demonstrably requires another variable, add it explicitly with rationale.

### E. Config identity reconciliation

Support the neutral `ServiceSpec.config` contract without hardcoding product-specific flags.

Preferred generic rule:

- `ServiceSpec.args` remains the exact canonical argv;
- if `ServiceSpec.config = Some(path)`, adapter observation may mark config as observed only when the exact path string is provably present in the observed canonical argv in the expected identity;
- zero matches -> Foreign;
- ambiguous/multiple unsupported matches -> Unknown;
- exact match -> populate observed config and allow neutral ownership comparison.

Alternatively introduce a small caller-supplied config-argument descriptor if exact argv matching cannot distinguish the config safely.

Do not hardcode `--config` into Eggup.

### F. Manager exit semantics audit

While touching the adapters, verify manager helpers do not interpret a command's nonzero `CommandOutput` as successful unless that specific command documents the code as an expected state query.

Mutation commands such as bootstrap, bootout, kickstart, crontab install, systemctl start/stop/restart/reload/enable/disable must have explicit exit-status handling.

## 7. Ordered work packages

A. Add failing tests for launchd incomplete restart and elapsed-budget overrun.

B. Implement monotonic operation deadlines and truthful composition.

C. Add trusted manager program-path configuration/resolution.

D. Implement minimal environment execution policy.

E. Reconcile `ServiceSpec.config` observation/ownership.

F. Audit mutation command exit status.

G. Run M002 ownership/cron/platform fixtures and native smoke lanes.

H. Update docs/changelog/roadmap/registry.

## 8. Failure, restart, and contention semantics

- exhausted deadline -> `completed=false` or typed timeout according existing contract; never start a fresh full timeout;
- incomplete stop during restart -> final restart not complete;
- incomplete start -> final restart not complete;
- manager binary missing/untrusted -> fail before mutation;
- required user-session environment missing -> manager error with remediation, no fallback to system scope;
- config identity absent/ambiguous -> Foreign/Unknown, no mutation;
- external definition change between inspect/mutate -> existing re-inspection rules remain;
- no automatic retries beyond bounded state polling.

## 9. Compatibility and migration

No consumer uses M002 adapters yet, so limited API adjustment is acceptable before first adoption.

Preserve M001 public model where possible.

If manager constructors gain a program-path/environment policy argument, provide safe defaults that use trusted platform paths. Avoid forcing every consumer to duplicate path constants.

No product-specific environment or config flag enters Eggup.

## 10. Required tests

### Transition truthfulness

- launchd stop incomplete -> restart not completed;
- launchd start incomplete -> restart not completed;
- launchd fully confirmed -> restart completed;
- systemd restart unconfirmed -> completed false;
- TestDouble remains coherent.

### Deadlines

- start end-to-end elapsed time <= requested budget + small scheduler tolerance;
- stop same;
- restart stop+start share one budget;
- systemd state query cannot consume fixed 5 s beyond a shorter caller budget;
- launchd internal status query cannot reset to descriptor timeout;
- zero timeout behavior explicitly tested.

### Program/environment trust

- production path resolver returns absolute paths;
- malicious temp PATH containing fake `systemctl` is never selected;
- relative override rejected;
- environment sentinel/token absent in child;
- only documented user-systemd session variables preserved when required;
- no shell remains.

### Config identity

- spec with Some(config) exact argv/config -> Owned;
- different config -> Foreign;
- absent expected config -> Foreign;
- ambiguous occurrence -> Unknown if ambiguity cannot be safely resolved;
- `config=None` existing M002 behavior preserved.

### Exit status

- each mutation command nonzero -> incomplete/error, never completed=true.

## 11. Verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo package -p eggup-service --locked --allow-dirty
./scripts/check-local.sh
```

Run Rust 1.89.

Hosted Linux/macOS/Windows-check must pass. Native systemd/launchd smoke evidence should be recorded where safely available.

## 12. Documentation updates

Update:

- `crates/eggup-service/README.md`;
- public rustdoc for timeout semantics;
- manager program resolution/environment policy;
- config identity semantics;
- `CHANGELOG.md`;
- service roadmap;
- registry.

Document exact scope differences between system/user manager execution.

## 13. Acceptance criteria

M003 closes only when:

- composed restart cannot convert incomplete subtransitions into success;
- caller timeout is an end-to-end budget;
- production manager commands use trusted absolute program paths;
- environment inheritance is minimal and explicit;
- `ServiceSpec.config` is faithfully representable in systemd/launchd ownership;
- Foreign/Unknown fail-closed behavior remains;
- cron byte preservation remains;
- all mutation exit codes are handled truthfully;
- CI/MSRV pass;
- no medium-or-higher Unix adapter issue remains.

## 14. Stop conditions

Stop and write an ADR if:

- user-scoped native manager operation fundamentally requires broad ambient environment inheritance;
- config identity cannot be represented without changing the neutral `ServiceSpec` model;
- trusted program discovery requires a platform package-manager policy;
- end-to-end deadline semantics would require a breaking ServiceManager result redesign.

## 15. Closure evidence required

Record:

- exact implementation SHA;
- launchd incomplete restart regression;
- elapsed-budget measurements/tests;
- manager path-resolution matrix;
- environment allowlist and sentinel test;
- config ownership matrix;
- mutation exit-status audit;
- M002 regression suite;
- native/hosted platform evidence;
- unresolved findings.

## 16. Handoff notes

This corrective is the hard service gate for eggsearch/Gregg adoption.

Future milestones are renumbered:

- Windows SCM -> service M004;
- prepared-transaction lifecycle integration -> service M005.
