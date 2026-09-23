# Consumer Adoption Milestone 003 — Eggsearch Service-Aware Adoption

Status: closed

Eggup planning baseline: `4495df6241b3fac9e396553727cf8d3d497ff3cd`
Eggup implementation baseline: `cf5b3d3819c168eb2dbf841daa8332f3eb28c915`

Consumer evidence baseline: `eggstack/eggsearch@68ae2fa5457c3fb8fa335851d60d0dce3e98aa08` (refreshed before implementation; the originally captured `30d597f6b9eadff20e5569a1034312004c248de8` had advanced)

Closure record: `plans/closure/consumer-adoption/003-status.md`

Source roadmap:

- `plans/subsystems/consumer-adoption-roadmap.md`

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: capability / compatibility

## 1. Objective

Make eggsearch the first service-aware Eggup consumer by replacing duplicated updater mechanics with the corrected Eggup core/acquisition path and replacing duplicated Unix manager mechanics with `eggup-service`, while preserving eggsearch-owned release selection, Cargo fallback, application health policy, service definitions, cron watchdog behavior, CLI UX, and Windows-specific behavior that is not yet qualified in Eggup.

This milestone proves that the shared layers compose in a real daemon consumer. It is not the generic update-lifecycle orchestration milestone and must not pull eggsearch policy into Eggup.

## 2. Readiness and dependencies

Hard dependencies are closed:

- verified-update-core M006 qualification;
- acquisition M004 validation/promotion corrective;
- service M003 Unix adapter correctness/security corrective;
- two simple-consumer adoptions.

Current Eggup HEAD/plan baseline is `4495df6241b3fac9e396553727cf8d3d497ff3cd`.

Current eggsearch evidence baseline is `30d597f6b9eadff20e5569a1034312004c248de8`. Before editing the consumer, refresh against its actual HEAD if it has advanced and record the implementation baseline in the closure.

Operational dependency:

- corrected Eggup crates are not yet published as a new crates.io patch. Implementation MAY use workspace/path or an exact git revision for qualification, but the final dependency disposition must be explicit. Do not silently bind eggsearch to an unpublished moving branch.
- If a normal crates.io dependency is required for merge/release, use the separately directed lockstep Eggup patch release (expected 0.1.1) after qualification. Publication remains maintainer-directed, not an implicit action of this plan.

Soft/interface dependency:

- service M004 Windows SCM may execute in parallel. Eggsearch M003 does not depend on it because Windows manager migration is excluded here.

## 3. Current evidence

At consumer baseline `30d597f6b9eadff20e5569a1034312004c248de8`:

- `src/update.rs` owns registry lookup, release asset acquisition, checksum parsing/hashing, candidate `--version` execution, staging/replacement, Cargo fallback, and post-update service restart;
- Eggfetch is already the HTTP stack, so `eggup-eggfetch` does not introduce a second transport family;
- release policy is stable: latest stable version comes from crates.io; exact GitHub asset is preferred; Cargo fallback is allowed only for unsupported targets or exact asset HTTP 404;
- `src/startup.rs` owns systemd, launchd, cron, Windows SCM, health probing, service-definition rendering, cron watchdog/PID identity, manager selection, and CLI instructions;
- Unix manager mechanics overlap directly with `eggup-service`;
- health semantics are eggsearch-specific: `GET /healthz`, bounded response, exact `service=eggsearch`, `status=ready`;
- cron process supervision is application-specific and remains outside `CronManager`;
- Windows currently uses `windows-service` for the service entry point and `sc.exe` for registration/lifecycle; Eggup Windows SCM is service M004 and is not available at this milestone;
- current-executable replacement uses `self-replace` on Windows. Eggup core has not yet claimed native running-image replacement qualification there.

## 4. Invariants

- eggsearch remains owner of version authority, GitHub/crates.io endpoints, target selection, exact Cargo fallback conditions, CLI wording, service definitions, health semantics, and cron watchdog process policy;
- Eggup owns generic acquisition staging/bounds, checksum-integrity mechanics, candidate validation where representable, destination ownership revalidation, commit/rollback/receipt semantics, and reusable Unix manager mechanics;
- no second HTTP/TLS stack is added;
- 404 is the only release-download condition eligible for the existing binary-to-Cargo fallback; TLS, timeout, 5xx, redirect, checksum, candidate, ownership, and commit failures never become fallback;
- a stopped registered service remains stopped across update;
- ambiguous/multiple service registrations never authorize mutation/restart;
- Foreign/Unknown service registrations are never destructively changed;
- health failure never becomes manager ownership evidence;
- cron watchdog ownership/PID safety remains consumer-owned;
- no hidden sudo/UAC;
- Windows behavior is preserved, not falsely declared migrated;
- no fetched shell execution;
- no Eggup API is widened solely to reproduce eggsearch-specific policy.

## 5. Scope and non-scope

### In scope

Updater:

- add corrected Eggup core/acquisition/Eggfetch dependencies;
- translate exact eggsearch URLs into `AcquisitionRequest`;
- use bounded Eggup acquisition for release assets/checksum metadata;
- use Eggup SHA-256 sidecar/integrity machinery;
- use Eggup candidate-validation primitives for exact `eggsearch <version>` identity where their public surface fits;
- use Eggup prepared transaction/commit/rollback for supported replacement paths;
- add a thin consumer ownership verifier for the exact destination;
- preserve Cargo fallback as consumer policy;
- map Eggup errors/receipts into existing `UpdateError`/CLI semantics;
- delete superseded generic hashing/staging/replacement code where Eggup fully owns it.

Unix startup:

- construct `ServiceSpec` from eggsearch `RuntimeSpec`;
- use `SystemdManager` for systemd inspect/install/uninstall/start/stop/restart mechanics;
- use `LaunchdManager` for launchd mechanics;
- use `CronManager` for managed crontab block install/uninstall/ownership;
- retain eggsearch rendering of systemd/plist content;
- retain eggsearch manager-selection policy and explicit privilege messages;
- retain `probe_health` / `wait_for_health`;
- retain cron detached-process/PID/start-token behavior;
- update `startup_state` to combine shared manager state with eggsearch health.

Integration:

- preserve update-time lifecycle snapshot/restart behavior using the corrected shared Unix manager interface plus eggsearch health checks;
- measure before/after dependency graph, release binary size, updater/startup LOC ownership, and duplicated code removed.

### Out of scope

- Windows SCM migration (service M004);
- claiming Eggup core supports Windows running-image replacement if it is still unqualified;
- generic service/update orchestration (service M005);
- changing release hosts/version policy;
- changing target matrix;
- removing Cargo fallback;
- changing cron frequency/watchdog process policy;
- moving health JSON semantics into Eggup;
- adopting `eggup-dist` in eggsearch;
- release publication;
- changing optional Eggress provider-routing behavior.

## 6. Required production changes

### A. Dependency and adapter boundary

Prefer corrected published Eggup patch crates if available at implementation time.

If they are not yet published:

- qualify with exact local path patches or an immutable Eggup git revision;
- record that dependency mode in closure;
- do not commit a floating branch dependency.

Expected shared crates:

- `eggup-core`;
- `eggup-acquisition`;
- `eggup-eggfetch`;
- `eggup-service`.

After migration, direct `eggfetch-core` remains in eggsearch because it is used broadly outside self-update (fetch, health, providers). The acceptance criterion is no second HTTP/TLS stack, not removal of Eggfetch itself.

### B. Keep release discovery consumer-owned

Retain:

- crates.io `max_stable_version` lookup;
- SemVer/pre-release policy;
- `platform::current_target`;
- GitHub URL construction;
- unsupported-target classification;
- exact fallback classifier.

Registry metadata may continue using eggsearch's existing Eggfetch client or be moved through `AcquisitionTransport::fetch_metadata` if doing so removes duplication without weakening JSON/decompression requirements. Do not force all application HTTP through Eggup.

### C. Replace release artifact acquisition

For the exact selected release asset:

- create an `AcquisitionRequest`;
- use `EggfetchTransport` with eggsearch-equivalent strict timeout/redirect/proxy policy;
- set metadata/artifact caps no looser than current 64 KiB registry, 4 KiB checksum, 128 MiB asset limits where applicable;
- use a destination in an Eggup-controlled acquisition/staging area;
- treat `FetchOutcome::NotFound` as the existing exact-asset 404 signal and only then permit Cargo fallback;
- map other acquisition errors to hard updater failure.

Do not re-buffer the whole release binary in memory.

### D. Replace checksum/integrity duplication

Use Eggup SHA-256 parsing/verification instead of local `parse_checksum` + `sha256_file` where the exact current sidecar format is representable.

The checksum sidecar must bind to the selected asset filename. Malformed/mismatched sidecars fail before candidate execution and commit.

If Eggup's parser cannot represent an existing accepted eggsearch sidecar form without weakening strictness, stop and classify whether the missing behavior is generic. Do not maintain two security-sensitive parsers.

### E. Candidate identity

Use Eggup bounded candidate execution/validator primitives to prove:

- process exits successfully;
- output is bounded;
- exact program identity is `eggsearch`;
- exact version equals the selected release;
- timeout kills/reaps the child.

Preserve any eggsearch-specific output mapping outside Eggup.

### F. Core transaction and ownership

Construct a one-member `ArtifactSet` / `InstallPlan` for the destination.

The ownership verifier must fail closed and be grounded in the exact intended current executable path/identity. Do not treat mere path existence as sufficient if the current Eggup ownership surface can prove more.

Set executable permission intent on Unix.

Use the full preparation -> integrity -> candidate validation -> commit flow and interpret `TransactionReceipt`.

Map:

- committed -> updated outcome;
- rolled back -> hard failure with old installation preserved;
- recovery required -> actionable typed error containing real recovery evidence/path;
- ownership conflict -> no mutation.

Do not silently flatten recovery-required into a generic filesystem error.

### G. Windows running-image compatibility boundary

Before replacing the current executable on Windows, explicitly test whether the corrected Eggup core path can perform the operation safely.

If not:

- keep only the minimum Windows-specific `self-replace` commit shim after shared acquisition/integrity/candidate validation; or
- stop and write a core Windows replacement qualification/corrective.

Closure MUST state whether Windows commit is shared or remains consumer-local. It must not claim complete cross-platform transaction adoption without evidence.

### H. Cargo fallback

Retain exact existing command/policy:

```text
cargo install eggsearch --version =X.Y.Z --locked --root <temp>
```

The built candidate must pass the same shared candidate validation before replacement.

Fallback is allowed only when:

- current target has no supported release mapping; or
- exact release asset returns typed NotFound/404.

It is forbidden for all other failure categories.

### I. Unix service translation

Create consumer helpers that translate eggsearch `RuntimeSpec` into:

- `ServiceId`;
- `ServiceSpec` with exact executable, argv, and config identity;
- caller-owned `SystemdInstall` / `LaunchdInstall` / cron block configuration.

Do not move service names, paths, unit/plist bytes, cron marker, or hardening text into Eggup constants.

### J. systemd

Replace local command/file mechanics for install/uninstall/status/restart with `SystemdManager`.

Preserve eggsearch policy:

- system service scope;
- `/etc/systemd/system/eggsearch.service`;
- rendered definition bytes;
- explicit root-required remediation;
- health check after manager transition.

Do not call `systemctl` directly from the migrated production path except code proven outside the shared manager responsibility.

### K. launchd

Replace local launchctl/plist mutation mechanics with `LaunchdManager`.

Preserve:

- per-user LaunchAgent domain/target;
- exact `com.eggstack.eggsearch` label;
- eggsearch-rendered plist;
- log path policy;
- post-transition health.

### L. cron

Use `CronManager` only for crontab ownership/install/uninstall.

Retain:

- `croncheck`;
- PID record/start token;
- detached process spawn;
- stop-owned-process;
- health-before-spawn rules;
- cron restart semantics.

The managed block representation may change from one marked line to Eggup's BEGIN/END block only if migration can recognize and safely convert the existing eggsearch-owned line. Do not strand or duplicate an existing registration.

If safe automatic migration from the old marker cannot be proven, keep a narrow compatibility recognizer and document the transition.

### M. startup state and update restart

`startup_state` remains consumer-owned because it combines manager registration/state with application health and conflict policy.

Use shared manager inspection for Unix methods.

Update behavior must remain:

- no registration -> no restart;
- registered but stopped -> remain stopped;
- exactly one registered/running/healthy service -> restart same manager after successful commit;
- ambiguous/conflicting registration -> update may proceed only according to existing policy, but must not mutate/restart ambiguous services;
- restart failure after successful binary commit reports installed version + exact remediation; do not roll back a verified binary merely because application health restart failed unless a future service M005 policy explicitly changes this.

### N. Delete superseded code

After tests prove parity, delete local generic code superseded by Eggup. Candidates include:

- asset streaming temp mechanics;
- checksum hashing/parser;
- generic candidate bounded runner;
- generic non-Windows replacement staging;
- systemd/launchd command execution and definition write helpers;
- crontab merge/remove helpers now owned by Eggup.

Do not delete application-specific rendering, discovery, health, watchdog, fallback, or Windows compatibility code.

## 7. Ordered work packages

A. Refresh eggsearch HEAD and create a before-migration ownership/dependency/size inventory.

B. Add exact Eggup dependencies using the approved immutable source.

C. Migrate acquisition/checksum/candidate pipeline behind existing updater tests.

D. Migrate one-member core transaction and receipt/error mapping; qualify Unix replacement and Windows boundary.

E. Migrate systemd + launchd manager mechanics.

F. Migrate cron registration mechanics with legacy-marker compatibility.

G. Recompose `startup_state` and post-update restart around shared manager state + eggsearch health.

H. Delete superseded updater/startup helpers and remove dependencies that are no longer directly needed solely by those helpers.

I. Run full eggsearch verification, package/release-contract checks, feature combinations, dependency/size comparison.

J. Record Eggup-side closure and update both repositories' planning/maintenance docs as appropriate.

## 8. Failure, restart, and contention semantics

- acquisition/checksum/candidate failure -> live binary unchanged;
- ownership failure -> no commit;
- pre-commit service inspection failure -> no service mutation;
- core commit failure -> honor receipt; rollback/recovery evidence preserved;
- update succeeds while service was stopped -> do not start it;
- update succeeds while one owned service was running/healthy -> restart that manager and then health-check;
- restart manager succeeds but health fails -> installed binary remains; report restart/health remediation;
- service ownership Foreign/Unknown -> never mutate service;
- multiple registered managers -> conflict, never choose one silently;
- cron registration change races -> fail closed per Eggup cron reread semantics;
- Windows unsupported shared-commit behavior -> use explicit compatibility path or stop for core corrective, never silently fall back to unsafe rename;
- Cargo fallback only for the two allowed policy cases.

## 9. Compatibility and migration

Preserve user-visible commands:

- `eggsearch update [--check]`;
- startup install/uninstall/status/instructions/restart;
- croncheck;
- existing config path behavior;
- existing release asset names.

Service-definition bytes may differ only where Eggup writes the same consumer-rendered definition safely.

Legacy existing registrations must remain discoverable during migration.

Dependency release policy:

- do not require publication to begin local qualification;
- closure must state exact Eggup source/version;
- before an eggsearch release that uses crates.io dependencies, corrected Eggup package versions must exist.

## 10. Required tests

Updater:

- already current / local ahead / check-only;
- exact asset success;
- bounded metadata/checksum/artifact;
- asset 404 -> Cargo fallback;
- unsupported target -> Cargo fallback;
- 5xx/TLS/timeout/redirect/checksum/candidate failure -> no fallback;
- checksum filename mismatch;
- wrong candidate identity/version;
- candidate timeout/output overflow;
- ownership Foreign/Unknown;
- successful commit;
- rollback receipt mapping;
- recovery-required mapping;
- Unix executable permissions;
- Windows current-exe behavior explicitly qualified.

Service:

- systemd absent/owned/foreign/unknown;
- launchd absent/owned/foreign/unknown;
- config identity exact/different/ambiguous;
- start/stop/restart completion;
- no hidden privilege escalation;
- cron unrelated bytes preserved;
- legacy eggsearch cron marker migration/compatibility;
- cron watchdog/PID tests remain green;
- multiple manager conflict remains fail-closed;
- health states remain distinct from manager state.

Update + lifecycle:

- stopped service stays stopped;
- running healthy systemd/launchd restarts after commit;
- restart incomplete/failure surfaces installed-version remediation;
- health failure after restart is not mislabeled rollback;
- stdio/no-manager update does not restart;
- cron path retains consumer-owned process restart semantics.

Repository:

- default features;
- `--all-features`;
- static guards;
- packaging checks;
- docs;
- Rust 1.89;
- Linux/macOS/Windows CI.

## 11. Verification commands

At minimum in eggsearch:

```bash
cargo fmt --all -- --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all-features
make check
make packaging-check
cargo +1.89.0 check --locked --all-targets
```

Run the repo's documented mock/integration/static-guard suites and release smoke targets.

In Eggup, rerun workspace tests if any generic code changes are required.

Measure release binary size with the same build profile before/after and record direct/transitive dependency deltas.

## 12. Documentation updates

Eggsearch:

- `docs/update.md`;
- `docs/service.md`;
- `architecture/startup.md`;
- relevant maintenance/ownership docs;
- changelog;
- dependency/license docs if present.

Eggup:

- consumer adoption roadmap;
- registry;
- closure record.

Documentation must state which Windows mechanics remain consumer-owned after M003.

## 13. Acceptance criteria

M003 closes only when:

- eggsearch uses corrected Eggup acquisition/core on the qualified updater path;
- generic checksum/staging/candidate/transaction duplication is materially removed where shared APIs cover it;
- systemd/launchd/crontab registration mechanics use `eggup-service`;
- eggsearch still owns health, rendering, manager choice, cron watchdog, release and fallback policy;
- no second HTTP/TLS stack is introduced;
- all fallback cases exactly match existing policy;
- service ownership/conflict semantics remain fail-closed;
- dependency and binary-size impact are recorded;
- Windows replacement/service scope is stated truthfully;
- full eggsearch verification and Eggup compatibility tests pass;
- no medium-or-higher migration defect remains.

## 14. Stop conditions

Stop and write an Eggup corrective rather than a consumer workaround if:

- shared acquisition cannot preserve current bounds/fallback classification;
- core cannot preserve old installation/recovery semantics on a supported non-Windows path;
- Unix service ownership cannot represent current eggsearch registrations;
- cron migration would destroy/duplicate unrelated or legacy owned entries;
- eggsearch requires product-specific logic in Eggup;
- a generic Windows replacement flaw is exposed and cannot remain a narrow explicitly-owned compatibility shim.

## 15. Closure evidence required

Record:

- exact Eggup and eggsearch implementation SHAs;
- Eggup dependency source/version;
- before/after updater/startup module ownership;
- deleted duplicated LOC/helpers;
- release/fallback behavior matrix;
- core receipt/recovery matrix;
- Unix service ownership/transition matrix;
- legacy cron migration evidence;
- Windows scope/disposition;
- dependency tree and release binary-size delta;
- Rust 1.89 + hosted platform CI;
- full eggsearch test/packaging results;
- any generic Eggup finding and corrective disposition.

## 16. Handoff notes

This plan may run in parallel with service M004 Windows SCM and distribution M003 validators.

Do not expand eggsearch M003 to consume service M004 mid-pass unless M004 closes cleanly early and the additional Windows migration remains small enough to verify independently. Prefer a later narrow consumer follow-up over destabilizing this first service-aware adoption.
