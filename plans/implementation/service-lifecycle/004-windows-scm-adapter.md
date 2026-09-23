# Service Lifecycle Milestone 004 — Windows SCM Adapter

Status: closed

Repository baseline: `4495df6241b3fac9e396553727cf8d3d497ff3cd`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: infrastructure / capability

## 1. Objective

Add a native Windows Service Control Manager adapter to `eggup-service` that implements the manager-neutral `ServiceManager` contract with the same ownership, deadline, fail-closed, no-hidden-elevation, and truthful-transition semantics established by Unix M002/M003.

The adapter must use a typed Windows SCM API rather than parsing localized `sc.exe` output or executing through ambient PATH.

## 2. Readiness and dependencies

Hard dependencies are closed:

- service M001 manager-neutral contract;
- service M002 Unix adapters/shared mechanics;
- service M003 deadline/execution/config-identity corrective.

Consumer evidence:

- eggsearch currently supports Windows SCM registration/lifecycle and depends on `windows-service 0.8.1`;
- its existing policy uses service name `Eggsearch`, automatic start, explicit binary path/args, description, failure actions, start/stop/delete, and a Windows service entry point.

M004 extracts only generic SCM mechanics. Product names, descriptions, failure policy, account policy, and service entrypoint remain consumer-owned.

## 3. Current evidence

At Eggup baseline:

- `ServiceSpec` identifies exact executable, argv, optional config;
- `Ownership::{Absent,Owned,Foreign,Unknown}` is canonical;
- `LifecycleState` and `TransitionResult` already define manager-neutral behavior;
- M003 introduced one monotonic deadline per transition, trusted execution semantics, and exact config-identity reconciliation;
- `eggup-service` currently has no runtime dependencies.

Windows-specific evidence from eggsearch shows the need for:

- SCM open/query/create/update/delete;
- start/stop/restart and pending-state polling;
- automatic-start configuration;
- service description/failure actions as caller policy;
- permission remediation without automatic UAC;
- exact executable/argument ownership.

The current eggsearch implementation uses `sc.exe` for management; Eggup should avoid carrying that parsing/executable surface into the shared layer.

## 4. Invariants

- Foreign/Unknown registrations are never destructively mutated;
- service name alone never proves ownership;
- exact executable + critical argv + optional config identity are required for Owned;
- malformed/unparseable SCM binary-path data yields Unknown, not Owned;
- all transition waits share the caller's single monotonic deadline;
- `completed=true` means the requested terminal state was confirmed;
- no ambient PATH, shell, PowerShell, `sc.exe`, WMI, or hidden UAC;
- permission errors return typed/bounded remediation only;
- delete/marked-for-delete state is represented truthfully;
- caller service configuration remains caller-owned;
- no service password/credential is logged;
- first-party code remains `unsafe_code = deny`; any unsafe Windows FFI must be encapsulated in an audited dependency rather than new Eggup unsafe blocks;
- non-Windows builds do not pull unnecessary Windows runtime surface.

## 5. Scope and non-scope

### In scope

- target-specific Windows SCM dependency;
- `WindowsScmManager` implementing `ServiceManager`;
- caller-owned Windows install descriptor;
- inspect/create/refresh/start/stop/restart/uninstall;
- exact ownership from queried SCM configuration;
- lifecycle-state mapping and bounded pending-state polling;
- access/permission mapping;
- deterministic fake/backend seam for non-Windows unit tests;
- Windows-native CI/runtime tests where the environment safely permits;
- docs/package/dependency qualification.

### Out of scope

- Windows service process entrypoint/dispatcher implementation;
- installing application service handler code;
- automatic elevation/UAC prompt;
- custom service-account passwords unless existing typed dependency can support them without storing/logging secrets and real consumer evidence requires them;
- recovery/failure action policy defaults;
- application health;
- update transaction orchestration (M005);
- consumer migration;
- `sc.exe` compatibility parser;
- remote SCM;
- drivers/kernel services.

## 6. Required production changes

### A. Dependency choice

Prefer a safe Rust SCM wrapper already proven by a consumer, currently `windows-service 0.8.1`, as a target-specific dependency:

```toml
[target.'cfg(windows)'.dependencies]
windows-service = ...
```

Before coding, inspect its exact current API/features and dependency tree.

Do not add direct unsafe Win32 FFI unless an ADR explicitly authorizes a narrowly isolated exception. If the selected wrapper cannot expose the required exact configuration/state safely, stop and evaluate another safe wrapper or a dedicated small platform crate.

Non-Windows `cargo tree` should not gain Windows SCM runtime dependencies.

### B. Backend seam

Separate manager-neutral logic from concrete SCM calls enough to permit deterministic tests on non-Windows hosts.

Use a narrow private/public-testable backend trait representing operations such as:

- open service manager;
- query service config/status;
- create service;
- change owned service config;
- start;
- request stop;
- delete.

Do not create a generic Windows API abstraction.

Production backend is Windows-only. Fake backend scripts exact states/errors.

### C. Windows install descriptor

Add a caller-owned descriptor such as `WindowsServiceInstall` containing only generic mechanics needed to register a service.

At minimum:

- service name;
- display name if SCM requires/provides it;
- exact executable/launch args or a representation derived from `ServiceSpec`;
- start type;
- error control where supported;
- optional dependencies;
- optional description only if cleanly supported;
- bounded transition timeout/default mechanics.

Do not hardcode:

- Eggstack product names;
- descriptions;
- failure restart schedules;
- account identities;
- config paths.

Advanced policy fields not required by two consumers should remain out of the initial adapter.

### D. Service name validation

Validate SCM identifiers before API calls:

- non-empty/bounded;
- reject NUL/control characters;
- reject path-like names;
- keep display name separate from service key name.

Do not normalize two distinct names into one silently.

### E. Ownership observation

Query SCM configuration and produce a `RegistrationSnapshot`.

Rules:

- service missing -> Absent;
- exact executable + args + optional config identity -> Owned;
- well-formed but different executable/args/config -> Foreign;
- malformed/ambiguous/unparseable command line -> Unknown;
- permission/query failure -> error/Unknown according the existing inspect contract, never Absent.

Do not use display name/description/start type alone as ownership evidence.

### F. Windows command-line identity

SCM stores executable/arguments in Windows command-line form. This is security-sensitive.

Use the selected safe wrapper's structured executable/launch-argument representation if it round-trips query/create reliably.

If query returns only an opaque command line:

- use a Windows-compatible parser with documented `CreateProcess` semantics from a safe dependency;
- reject ambiguous/unrepresentable quoting as Unknown;
- do not implement an ad-hoc whitespace split;
- add spaces/quotes/backslashes/non-ASCII path tests.

The exact expected executable must be absolute.

Config identity follows the M003 rule: exact configured path must appear exactly once in expected/observed canonical argv; missing -> Foreign; ambiguous -> Unknown.

### G. Lifecycle state mapping

Map SCM states deliberately:

- Stopped -> Stopped;
- Running -> Running;
- StartPending / StopPending / ContinuePending / PausePending -> Transitioning;
- Paused or unsupported states -> document mapping (prefer Stopped or Unknown based on semantic evidence, not convenience);
- query failure -> Unknown/error.

Preserve raw status details only as bounded diagnostics if useful.

### H. Deadlines and polling

Reuse the M003 monotonic deadline semantics.

Start/stop/restart:

- one deadline at API entry;
- all SCM calls/polls consume remaining budget;
- pending-state polling cannot reset the timeout;
- sleeps are capped by remaining budget;
- use SCM wait-hint/checkpoint information only as advisory and always within caller deadline;
- restart stop+start share one deadline;
- zero timeout remains rejected.

### I. Install / refresh

Absent:

- create using exact caller configuration;
- no automatic start unless caller explicitly requests it through the chosen install semantics.

Owned:

- re-query immediately before change;
- update only the fields M004 explicitly owns;
- preserve unsupported/unmanaged SCM fields instead of resetting them accidentally.

Foreign/Unknown:

- deny.

If the safe wrapper cannot update configuration without clobbering unmanaged fields, stop and narrow refresh behavior rather than resetting policy.

### J. Start / stop / restart

Start:

- require Owned;
- already Running -> completed true;
- call SCM start;
- poll to Running.

Stop:

- require Owned;
- already Stopped -> completed true;
- send Stop control;
- poll to Stopped.

Restart:

- require Owned;
- one shared deadline;
- if stop incomplete/timeout, do not start;
- start only after confirmed stopped;
- completed true only after Running confirmed.

Nonzero Win32/SCM errors map to typed/bounded `ServiceError` with no sensitive environment dump.

### K. Uninstall/delete

Require Owned and re-inspect immediately before delete.

If running:

- stop using the same owned/deadline rules as appropriate before delete, or return incomplete if policy says caller must stop first. Choose one behavior and document it.

SCM deletion may mark a service for deletion until all handles/processes release.

The result MUST distinguish:

- delete request accepted but service still observable/marked pending -> `completed=false` or explicit transitional detail;
- confirmed absent -> completed true.

Never claim immediate deletion solely because the API call succeeded.

### L. Permissions/no elevation

Access denied:

- return `ServiceError` with remediation such as requiring an appropriately elevated process;
- do not invoke `runas`, PowerShell, UAC COM, or `sc.exe`;
- do not guess whether Administrator is required before attempting the requested scope.

### M. Failure actions and descriptions

Do not hardcode recovery/failure actions.

If the dependency exposes descriptions safely, an optional caller-supplied description may be supported.

Failure-action configuration should be deferred unless needed for generic parity. Eggsearch can retain its product-specific failure-action step until a later evidence-driven extension.

## 7. Ordered work packages

A. Qualify the safe SCM dependency/API and record target-specific dependency impact.

B. Add Windows backend seam + fake backend tests.

C. Add install descriptor/name validation/config observation + command-line identity tests.

D. Implement inspect/ownership/state mapping.

E. Implement install/refresh with reinspection and unmanaged-field preservation.

F. Implement start/stop/restart with shared deadlines.

G. Implement uninstall/delete truthfully around marked-for-delete semantics.

H. Add Windows-native tests where CI permissions permit; otherwise separate compile/deterministic evidence honestly.

I. Update docs/changelog/package/roadmap/registry.

## 8. Failure, restart, and contention semantics

- service absent on inspect -> Absent;
- access denied/query failure -> never Absent by assumption;
- Foreign/Unknown -> no mutation;
- external config change between inspect/mutate -> reinspection catches and fails closed;
- start pending beyond deadline -> completed false/timeout;
- stop pending beyond deadline -> completed false/timeout;
- restart stop incomplete -> start not attempted;
- delete accepted but still marked/present -> not completed;
- service disappears concurrently before a non-destructive query -> classify according to exact API evidence;
- service disappears between Owned reinspection and mutation -> return manager/conflict outcome, do not recreate unless operation is explicit install-from-Absent;
- no retry loop beyond bounded state polling.

## 9. Compatibility and migration

All Windows-specific types should be `cfg(windows)` where appropriate while keeping cross-platform public docs/builds coherent.

Do not break existing Unix APIs.

If `ServiceManager` itself cannot express a necessary truthful Windows outcome, stop and decide whether a small generic additive change is justified. Any such change requires regression tests across TestDouble/systemd/launchd/cron.

The initial M004 adapter does not automatically migrate eggsearch.

## 10. Required tests

Cross-platform fake/backend tests:

- absent/owned/foreign/unknown ownership;
- executable mismatch;
- argv mismatch;
- config exact/missing/ambiguous;
- quoted executable path with spaces;
- quoted/backslash arguments;
- malformed opaque command line -> Unknown;
- state mapping for all SCM states;
- start already running;
- start pending -> running;
- start deadline;
- stop already stopped;
- stop pending -> stopped;
- stop deadline;
- restart stop incomplete -> no start;
- restart successful one-budget;
- install absent;
- refresh owned after reinspection;
- foreign/unknown install denied;
- uninstall owned;
- delete marked/pending -> not complete;
- access denied diagnostics;
- race/change between inspection and mutation.

Windows-native where feasible:

- create/query/start/stop/delete a uniquely named inert test service only if CI privilege/sandbox policy explicitly permits;
- otherwise do not mutate host SCM; record native lane as compile/API-link evidence only.

Regression:

- Unix M001-M003 suites unchanged;
- Rust 1.89;
- docs/package.

## 11. Verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo package -p eggup-service --locked --allow-dirty
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh
```

Hosted Windows must at least compile/check the real backend. Run deterministic SCM logic tests on all platforms where possible.

If a privileged Windows integration lane exists, record its exact operations and cleanup.

## 12. Documentation updates

Update:

- `crates/eggup-service/README.md`;
- Windows adapter rustdoc/examples;
- dependency/feature notes;
- `CHANGELOG.md`;
- service roadmap;
- registry.

Document unsupported Windows features explicitly, especially custom service credentials/failure actions if deferred.

## 13. Acceptance criteria

M004 closes only when:

- Windows SCM inspect/install/start/stop/restart/uninstall implement the neutral ownership contract;
- service name alone cannot authorize mutation;
- executable/argv/config identity is exact and ambiguity yields Unknown;
- transitions use one caller deadline and report completion truthfully;
- deletion semantics do not claim absence before confirmed;
- no `sc.exe`, shell, PATH lookup, or hidden elevation is used;
- no new first-party unsafe code is introduced;
- non-Windows dependency/runtime impact is appropriately isolated;
- Unix regressions remain green;
- Windows hosted compile/check passes and runtime evidence is recorded honestly;
- no medium-or-higher SCM correctness/security issue remains.

## 14. Stop conditions

Stop and write an ADR/corrective if:

- exact SCM command identity cannot be recovered safely from the chosen dependency;
- configuration refresh would clobber unmanaged fields;
- truthful deletion/marked-for-delete state cannot fit the current result contract;
- implementation requires first-party unsafe FFI;
- generic support would require storing custom service account passwords;
- the adapter begins absorbing application service-entrypoint or health policy.

## 15. Closure evidence required

Record:

- implementation SHA;
- exact Windows dependency/version/features;
- dependency-tree delta on Windows and non-Windows;
- ownership matrix;
- command-line quoting/config identity matrix;
- state/deadline/restart matrix;
- install/refresh/delete race matrix;
- access-denied behavior;
- Windows-native evidence or explicit environment limitation;
- Unix regression/MSRV/hosted CI;
- unsupported/deferred SCM capabilities;
- unresolved findings.

## 16. Handoff notes

M004 may run in parallel with eggsearch M003 and distribution M003.

Its closure unlocks service M005 prepared-transaction lifecycle integration and enables a later narrow eggsearch Windows-service migration. Do not fold either downstream milestone into this adapter implementation.
