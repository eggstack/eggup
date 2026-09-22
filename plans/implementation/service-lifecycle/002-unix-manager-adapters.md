# Service Lifecycle Milestone 002 — Unix Manager Adapters

Status: ready for handoff

Repository baseline: `8f6ce48cda5bdeb593939077bcca452cdd5f2800`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Source closure:

- `plans/closure/service-lifecycle/001-status.md`

Consumer evidence:

- `eggstack/eggsearch/src/startup.rs`
- `eggstack/gregg/crates/greggd/src/startup/{systemd,launchd,cron,install}.rs`

Long-term requirements:

- `plans/000-long-term-specification.md#10-ownership-model`
- `plans/000-long-term-specification.md#13-service-lifecycle`
- `plans/000-long-term-specification.md#20-platform-scope`

Applicable ADR:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: capability / infrastructure

## 1. Objective

Implement reusable Unix service-manager mechanics for systemd, launchd, and user crontab supervision on top of the closed manager-neutral M001 contract, without absorbing consumer-specific unit/plist/cron content or privilege policy.

The milestone should be usable by both eggsearch and Gregg but must not migrate either consumer yet.

## 2. Readiness and dependencies

Service M001 is closed and established:

- `ServiceSpec`;
- `Absent | Owned | Foreign | Unknown`;
- lifecycle and health separation;
- `ServiceManager`;
- bounded transition result/error vocabulary.

Current eggsearch and Gregg implementations provide two independent examples of all three Unix manager families.

No acquisition/core dependency is required for manager adapters themselves.

## 3. Current evidence

Eggsearch currently centralizes all startup managers in `src/startup.rs`, including:

- platform auto-selection;
- systemd/launchd/cron registration detection;
- exact executable/config/argv construction;
- bounded health probing;
- conflict detection;
- cron-managed process state.

Gregg has more decomposed implementations:

- bounded manager command execution;
- narrow `ExecStart` parsing that returns Unknown for ambiguous units;
- exact executable ownership checks;
- atomic definition writes;
- launchd/systemd install/restart;
- managed cron-block merge/remove that preserves unrelated crontab content;
- privilege remediation text rather than hidden elevation.

Those common mechanics are the extraction target. App-specific hardening text, usernames, paths, health JSON, cron command content, and CLI messages remain consumer-owned.

## 4. Invariants that must not regress

- Foreign/Unknown registration is never destructively mutated;
- ownership cannot be inferred from service/unit name alone;
- executable plus critical argv/config participates in ownership;
- ambiguous native-manager output is Unknown, not Owned;
- stopped services remain stopped unless caller explicitly starts/restores them;
- all subprocess manager calls are bounded in time/output;
- no shell interpolation for manager commands;
- no internal sudo/elevation/UAC;
- cron mutation preserves unrelated bytes/entries;
- health remains a separate consumer probe;
- no release/update/network policy enters `eggup-service`.

## 5. Scope

### In scope

- host manager detection helpers for Linux/macOS/other Unix;
- bounded command runner internal to service crate;
- `SystemdManager`;
- `LaunchdManager`;
- `CronManager`/managed-block mechanics;
- manager-specific configuration types;
- registration inspection;
- ownership classification;
- install/update owned definition;
- uninstall;
- start/stop/restart where manager supports it;
- bounded state transition polling;
- test seams/fake command executor;
- native smoke tests where environment permits;
- package documentation.

### Explicitly out of scope

- Windows SCM;
- service-update orchestration around an Eggup transaction;
- health HTTP implementation;
- rendering app-specific unit/plist/cron bodies;
- creating system users/groups;
- creating app configs;
- changing file ownership to application users;
- automatic elevation;
- package-manager integration;
- eggsearch/Gregg migration.

## 6. Required production changes

### A. Adapter boundary

Keep `ServiceSpec` as runtime identity.

Add manager-specific install descriptors that supply only manager mechanics and caller-owned definition material.

The service crate may know:

- unit/label identity;
- definition path/scope;
- exact definition bytes or normalized fields required for ownership;
- managed cron marker/block;
- transition timeout.

It must not know product names, fixed paths, daemon users, or hardening directives.

### B. Bounded manager command runner

Centralize literal argv process execution with:

- null/controlled stdin unless an operation intentionally streams crontab content;
- cleared or documented minimal environment;
- bounded stdout/stderr;
- deadline;
- kill/reap;
- stable exit classification;
- no shell.

Allow injection of a command executor for deterministic tests.

### C. systemd adapter

Support explicit system/user scope without guessing privilege.

Inspection must determine:

- whether the registration exists;
- actual executable and critical args/config when parseable;
- active/stopped/transitioning state.

Use narrow parsing for the supported `ExecStart` shape. Ambiguous directives, multiple commands, unsupported escaping/specifiers, or unparseable manager output return Unknown.

Install/update:

- only Absent creation with explicit caller authorization or Owned refresh;
- definition write must be atomic/no-clobber-safe and private/appropriate for the supplied path;
- call daemon-reload/enable only when explicitly requested by descriptor policy;
- never invoke sudo.

Start/stop/restart use bounded `systemctl` and confirm resulting state.

### D. launchd adapter

Support explicit user-agent versus system-daemon domain.

Ownership must include exact label + executable + critical args/config.

Prefer parsing structured plist data or deterministic caller-owned fields rather than ad-hoc text matching.

Install/bootstrap, bootout, kickstart/start/stop operations must be bounded and explicit.

Foreign label with different program/args fails closed.

No automatic choice between user/system domain based solely on EUID.

### E. cron adapter

Model cron as a managed block, not a whole-crontab file.

Require caller-provided unique marker and exact desired block.

Operations must:

- list existing crontab with bounded command;
- identify zero/one/multiple managed blocks;
- classify an exact single block as Owned;
- classify same marker with different content as Foreign or Unknown according to documented rule;
- preserve unrelated content byte-for-byte where practical;
- install exactly one block idempotently;
- remove only the owned block;
- avoid a redundant `crontab -` write when nothing changes.

Cron has no native start/stop process semantics; represent unsupported operations explicitly rather than pretending success.

### F. Definition file safety

Reusable atomic text writes must use:

- same-directory temp;
- create-new;
- owner-safe initial mode;
- fsync where documented;
- rename/persist with clear overwrite authority;
- cleanup only of owned temp.

Do not create missing privileged parents or change permissions/ownership implicitly.

### G. Platform detection

Provide host fact detection separately from policy.

A consumer may ask for Auto, but manager selection policy should be a small explicit helper and overridable.

Linux with inactive/unavailable systemd may return cron as a candidate; macOS may return launchd; other Unix may return cron. Detection failure must remain distinguishable from "not installed".

## 7. Ordered work packages

A. Shared bounded command/test executor.

B. systemd inspection + ownership tests.

C. systemd mutation/state transitions.

D. launchd inspection + ownership tests.

E. launchd mutation/state transitions.

F. cron pure merge/remove ownership helpers.

G. cron command adapter.

H. host detection + docs + native smoke lanes.

## 8. Failure, cancellation, restart, and contention semantics

- manager command timeout -> typed Manager/Conflict error; never assume desired state;
- definition write failure -> old definition preserved where possible;
- reload/bootstrap failure after definition update -> return incomplete transition; do not silently delete previous evidence;
- Foreign/Unknown -> no mutation;
- concurrent external manager changes -> re-inspect before destructive definition overwrite/removal;
- cron changed between list and install cannot be perfectly CASed through standard `crontab`; detect/re-read where practical and document remaining race;
- no background worker or implicit retries.

## 9. Compatibility and migration

No consumer is migrated here.

Public M001 types should remain source-compatible unless a real adapter requirement proves a defect. Any M001 public API change must carry migration notes because `eggup-service 0.1.0` is published.

Manager-specific types may be new 0.1.x API.

Do not copy eggsearch/Gregg application-specific constants into Eggup.

## 10. Required tests

### Shared

- timeout/kill/reap;
- output bound;
- environment behavior;
- nonzero exit;
- missing manager binary.

### systemd

- absent;
- exact Owned;
- same unit name/different executable Foreign;
- argv/config drift Foreign;
- ambiguous ExecStart Unknown;
- active/stopped/transitioning;
- permission denial returns remediation-capable error without sudo;
- atomic definition write fault;
- start/stop/restart confirmation.

### launchd

- absent/exact/foreign/unknown;
- user vs system domain distinction;
- malformed plist/manager output Unknown;
- bootstrap/bootout/kickstart success/failure;
- no label-only ownership.

### cron

- empty crontab;
- unrelated entries preserved;
- exact managed block idempotent;
- duplicate marker -> conflict/Unknown;
- modified same-marker block -> Foreign/Unknown, no destructive overwrite;
- uninstall only owned block;
- no-op uninstall does not rewrite;
- command timeout and missing `crontab`.

## 11. Required verification commands

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo tree -p eggup-service --locked
cargo package -p eggup-service --locked --allow-dirty
./scripts/check-local.sh
```

Run Rust 1.89 variants.

Hosted Linux should exercise systemd parsing/fixture tests even if the runner is not booted with systemd. macOS should run launchd parser/fixture tests. Native manager smoke tests must be clearly marked/skipped when environment cannot safely create registrations.

## 12. Documentation updates

Update `crates/eggup-service/README.md`, rustdoc, roadmap, registry, and changelog.

Document:

- what ownership proves for each manager;
- unsupported/ambiguous forms;
- privilege expectations;
- user/system scope;
- cron preservation contract;
- absence of health/release/update policy.

## 13. Acceptance criteria

M002 closes when:

- all three Unix manager families implement the M001 contract truthfully;
- Foreign/Unknown mutation is prevented;
- manager calls are bounded;
- cron preserves unrelated entries;
- no implicit elevation exists;
- consumer-specific definitions remain outside;
- test seams cover partial failure;
- package/MSRV/CI are green;
- no medium-or-higher Unix adapter issue remains.

## 14. Stop conditions

Stop and write an ADR/corrective if:

- systemd/launchd require incompatible ownership vocabulary;
- generic ownership requires a full parser for arbitrary systemd shell grammar;
- safe cron mutation requires stronger external locking than the host interface can provide;
- privilege policy cannot remain caller-owned;
- M001 must be broken substantially to support real adapters.

## 15. Closure evidence required

- adapter public API;
- systemd/launchd/cron ownership matrices;
- command-bound tests;
- definition-write fault evidence;
- cron byte-preservation fixtures;
- native/platform test matrix;
- dependency/package results;
- consumer-specific code explicitly rejected from extraction;
- unresolved findings.

## 16. Handoff notes

This milestone makes eggsearch's service portion dependency-ready but does not migrate eggsearch.

Do not implement Windows SCM or update-lifecycle orchestration here.
