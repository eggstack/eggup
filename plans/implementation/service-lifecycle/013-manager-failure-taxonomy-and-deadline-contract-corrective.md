# Service Lifecycle Milestone 013 — manager failure taxonomy and deadline contract corrective

Status: conditionally closed 2026-10-23; implemented and green on the native macOS lane. Closure record: `plans/closure/service-lifecycle/013-status.md`. The remaining condition is the outstanding hosted qualification: Linux, Windows, and the hosted CI lanes were **not run**.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Original closed work affected:

- `plans/implementation/service-lifecycle/003-unix-adapter-correctness-security-corrective.md`
- `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`
- `plans/closure/service-lifecycle/003-status.md`
- `plans/closure/service-lifecycle/008-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **7**: "`eggup-service` collapses manager failures into untyped strings and applies three different timeout models across its adapters." Registered at baseline `c480541` alongside this plan. The finding is assessed **medium** severity and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: invariant

## 1. Objective

Four defects in `crates/eggup-service`, all characterized by reading the source at baseline `c4805411`.

1. **Typed manager failures.** `ServiceError::Manager` carries an untyped `String`, so a consumer cannot distinguish "the manager timed out" from "the manager binary is missing" from "you lack privileges". The crate compensates by substring-matching its own error text. Replace that with a typed failure classification carried on the `Manager` variant, and remove the text-sniffing compensation.
2. **One timeout model.** Three adapters apply three different timeout models. Apply a single per-operation deadline threaded through every manager invocation the operation makes.
3. **One cron listing and one cron rule.** `CronManager::inspect` spawns `crontab -l` twice and discards one result; the two copies of the ownership rule disagree; the documented executable-containment gate is not applied on the destructive paths.
4. **Documented failure semantics.** The crate has zero `# Errors` rustdoc sections, zero `# Panics` sections, and zero doc code fences, while `#![deny(missing_docs)]` is on.

Defect D (two byte caps for the same data class) is low-severity and folded into the same work.

## 2. Why this milestone is ready

- Every defect is fully characterized by reading the code at baseline `c4805411`. No part of the diagnosis depends on a runtime observation that has not been made.
- The service-lifecycle subsystem has no open milestone: M001–M011 are closed (`plans/subsystems/service-lifecycle-roadmap.md` §12), and M011 published `eggup-service 0.1.3` on 2026-10-09.
- The scope is bounded to one crate, `eggup-service`, and touches no other crate's API.
- Severity: **A, B, C, and E are medium. D is low.** Reasons are stated per defect below.

Honest limitations:

- **No failing production report exists.** No consumer has filed a defect caused by any of A–E. The severity ratings reflect the size of the behavioral surface a consumer cannot rely on, not an observed outage.
- **The current test suite passes**, because it encodes the *implementation*, not the contract: `lib.rs:3409-3412` asserts that `permission_hint` appends the literal `"permission denied;"` text, and `windows_scm.rs:1541` asserts on `detail.contains("deadline")`. A test asserting the text-sniffing behavior cannot detect that the behavior is the defect.
- Defect A's severity is raised by a fact established by reading: the substring-sniffing surface is **larger than `permission_hint` alone** (§3.2).

## 3. Current implementation evidence

### 3.1 The error taxonomy has four variants and no failure kind

`crates/eggup-service/src/lib.rs:287-303`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ServiceError {
    InvalidInput(String),
    OwnershipDenied { id: String, ownership: Ownership },
    Conflict(String),
    Manager(String),
}
```

`ServiceError::manager` (`lib.rs:318-320`) is the single constructor and applies only a 512-byte bound (`lib.rs:310-312`), no classification. The `Manager` variant is also overloaded to carry failures that are **not** manager-process failures: local definition-file writes (`lib.rs:1107`, `1160`, `1167`, `1186-1197`, `1216`, `1231`), unit/plist removal (`lib.rs:2090`, `2302`, `2626`), systemd observation parsing (`lib.rs:1606`, `1609`, `1616`, `1621`, `1624`), and injected test failures (`disposition.rs:1652-1703`, `lifecycle_update.rs:771-808`).

### 3.2 The crate sniffs its own error text in five places

`crates/eggup-service/src/lib.rs:1788-1803`:

```rust
fn permission_hint(e: ServiceError) -> ServiceError {
    match e {
        ServiceError::Manager(d) => {
            let lower = d.to_lowercase();
            if lower.contains("permission") || lower.contains("denied") || lower.contains("access")
            {
                const HINT: &str = " (permission denied; re-run with appropriate user/system scope; no automatic elevation)";
                let detail = truncate_utf8_bytes(d, 512 - HINT.len());
                ServiceError::Manager(format!("{detail}{HINT}"))
            } else { ServiceError::Manager(d) }
        }
        other => other,
    }
}
```

Applied at eleven production sites: `lib.rs:1413`, `1415`, `1439`, `1575`, `1809`, `2147`, `2164`, `2318`, `2688`, `2712`, `2726`. The cron path does **not** apply it (`lib.rs:3021`, `3044`), so the hint is already inconsistent across adapters. Four further self-sniffs exist beyond `permission_hint`:

- `lib.rs:3279`, `3292`, `3304` — `inspect_host` matches `Err(ServiceError::Manager(d)) if d.contains("missing")` to report "manager not installed" instead of a failure. These three sites would not compile after the `Manager` retyping and must be migrated to the typed kind.
- `windows_scm.rs:1541` — a test asserts `Err(ServiceError::Manager(detail)) if detail.contains("deadline")`.
- `lib.rs:4548` and `lib.rs:4923` — tests assert on `format!("{e}").contains("deadline exhausted")` and `format!("{e}").contains("missing")`.

The only place in the crate where a failure carries a machine-readable kind is a local, per-adapter hack: `crates/eggup-service/src/windows_scm.rs:911-917` re-derives the distinction from a raw OS code (`io_error.raw_os_error()`), and `scm_error_detail` (`windows_scm.rs:215-224`) special-cases code `5` into an access-denied *sentence*.

### 3.3 Three timeout models across three adapters

- Ceiling: `lib.rs:617` — `pub const MAX_TRANSITION_TIMEOUT: Duration = Duration::from_secs(300);`
- Per-call clamp: `lib.rs:1734-1736` — `fn bounded_timeout(t: Duration) -> Duration { min(t, MAX_TRANSITION_TIMEOUT) }`
- Shared per-operation deadline: `lib.rs:1694-1732` — `struct OperationDeadline(Instant)` with `new` / `new_at` / `remaining` / `remaining_at` / `command_timeout` / `command_timeout_at`.

| Adapter | Model | Evidence |
|---|---|---|
| systemd | **per-call** `bounded_timeout` for read-only calls and `daemon-reload` / `enable` / `disable`; shared `OperationDeadline` only for `start` / `stop` / `restart` | per-call sites `lib.rs:1921`, `2068`, `2100`, `2145`, `2162` |
| launchd | **shared** deadline inside `bootstrap` / `kickstart` / `bootout` (`lib.rs:2725` et al.), but a fresh `bounded_timeout` for `launchctl bootout` in `uninstall` (`lib.rs:2603`) and a new `OperationDeadline` per phase in `install` (`lib.rs:2509`, `2515`) and `uninstall` (`2587`, `2589`) | — |
| Windows SCM | **one** `OperationDeadline` for the whole operation, `deadline.command_timeout()?` for every backend call | `windows_scm.rs:418`, `446`, `505`, `555`, `605`, `637` |

A systemd `install` on an `Owned` unit chains `ownership_of` (`lib.rs:1939`), a second `ownership_of` recheck (`lib.rs:1952`), `daemon-reload` (`2145`), and `enable` (`2162`) — four manager invocations, each with a fresh full budget. Worst-case wall time is roughly 4x the declared budget, and 5x on the `Absent` path where a recheck is skipped. A systemd `uninstall` chains up to four as well (`2051`, `2055`-adjacent recheck, `2068`, `2100`).

The `ServiceManager` trait (`lib.rs:364-398`) documents `start` / `stop` / `restart` as "within `timeout`" (`lib.rs:371`, `389`), and `architecture/service-lifecycle.md` §"Deadline handling" states that every subsequent command, state poll, and observation uses the *remaining* budget. The systemd per-call model contradicts that contract. `install`, `uninstall`, and `inspect` (`lib.rs:366`, `369`, `397`) take **no** timeout parameter; their only declared budget is `install.transition_timeout`, validated at `lib.rs:1298` (systemd), `2240` (launchd), `2984` (cron).

### 3.4 `CronManager::inspect` lists the crontab twice and the two rules disagree

`crates/eggup-service/src/lib.rs:3089-3128`:

```rust
fn inspect(&self, spec: &ServiceSpec) -> Result<LifecycleSnapshot, ServiceError> {
    let (ownership, _) = self
        .cron_ownership(spec)              // -> list_crontab() at :3070
        .unwrap_or((Ownership::Unknown, CronOwnership::Unknown));
    let current = self.list_crontab();     // -> list_crontab() a SECOND time
    let (ownership, state, present) = match current {
        Ok(text) => {
            let block_state = cron_classify(&text, &self.marker, &self.desired_block);
            let ownership = match block_state {
                CronOwnership::Owned => {
                    let exe = spec.executable().to_string_lossy().into_owned();
                    if text.contains(&exe) { Ownership::Owned } else { Ownership::Foreign }
                }
                // ... Absent / Foreign / Unknown arms
```

- Every `inspect` spawns **two** `crontab -l` subprocesses and discards one result.
- The `ownership` computed at `lib.rs:3094` is shadowed on the success path by the inline re-computation at `lib.rs:3102-3114`; it survives only in the `Err(_)` arm at `3118`.
- The two copies of the same rule disagree. `cron_ownership` at `lib.rs:3076` uses OR: `if current.contains(&exe) || self.desired_block.contains(&exe)`; the live path at `lib.rs:3106` uses current-only AND semantics: `if text.contains(&exe)`.
- The documented intent at `lib.rs:3057-3069` — "the block must contain the spec executable path; otherwise even an exact block for another binary would look Owned" — is **not enforced on the destructive paths**. `install` (`lib.rs:3130`, parameter named `_spec`, i.e. deliberately unused) and `uninstall` (`lib.rs:3194`) rely solely on `cron_merge` / `cron_remove`'s exact-content match and never consult the executable.

Two consequences follow directly from the code as written:

- The live check at `lib.rs:3106` tests containment in the **whole crontab**, not in the managed block. An unrelated user crontab entry mentioning the executable path satisfies it. It is weaker than the documented rule, not stronger.
- `install` does not consult `spec`, and `inspect` does. A caller supplying a `desired_block` naming executable `Y` while `spec.executable()` is `X` gets a **successful install** from `lib.rs:3130-3161` followed by an `Ownership::Foreign` verdict from `lib.rs:3106`. The crate contradicts itself.

`install` and `uninstall` also each list the crontab twice — at `3131` and `3150`, and at `3195` and `3221` respectively — but that second listing is an intentional pre-write re-validation (`if fresh != current` at `3151` and `3222`) that detects an external crontab change. It is a safety check, not a duplicate read, and M013 must not remove it.

### 3.5 Two byte caps for the same data class

`lib.rs:1784-1786` caps manager stderr excerpts at **256** bytes (`fn truncate`); `lib.rs:310-312` caps the composed manager detail at **512** bytes (`ServiceError::bounded`); `lib.rs:1795` re-caps to `512 - HINT.len()`; and `lib.rs:286` states the cap as 512 "without qualification". The same class of data is therefore bounded at 256, at 512, and at `512 - HINT.len()` depending on where it is measured.

### 3.6 No `# Errors`, no `# Panics`, no doc code fences

Across `crates/eggup-service/src/lib.rs` (4,959 lines), `disposition.rs` (2,502), `lifecycle_update.rs` (1,538), and `windows_scm.rs` (1,604), a search for `/// # Errors`, `/// # Panics`, and doc code fences returns **zero** matches in all four files. The crate is `#![deny(missing_docs)]` and has many public `Result`-returning functions: `ServiceId::new` (`lib.rs:41`), `ServiceSpec::new` (`lib.rs:79`), `require_owned` (`lib.rs:401`), `SystemExecutor::with_max_output` (`lib.rs:691`), `atomic_write_definition` (`lib.rs:1090`), `inspect_host` (`lib.rs:3268`), `WindowsScmInstall::with_transition_timeout` (`windows_scm.rs:107`), `disposition::plan_unix` (`disposition.rs:125`), `disposition::plan_windows` (`disposition.rs:224`), `LifecycleUpdatePolicy::validate` (`lifecycle_update.rs:96`), and all six `ServiceManager` trait methods. `clippy::missing_errors_doc` lives in `clippy::pedantic`, not `clippy::all`, so the workspace's `-D warnings` gate does not catch this. The crate's only panic paths are the `expect("fake lock")` calls in `FakeExecutor` (`lib.rs:1030`, `1035`, `1040`, `1052`, `1056`, `1059`).

## 4. Invariants that must not regress

- The manager program allowlist (`resolve_manager_program`, `lib.rs:879-904`) and executable validation (`validate_executable_path`, `lib.rs:906-927`) are not weakened. The allowlist remains `systemctl` / `launchctl` / `crontab` at fixed absolute paths only.
- The minimal filtered environment (`filtered_manager_environment`, `lib.rs:929`) is not widened. The child environment stays cleared (`lib.rs:727`).
- No automatic privilege elevation is introduced. The crate's existing "no automatic elevation" statement remains the contract.
- Ownership remains fail-closed: `Foreign` and `Unknown` never permit mutation, and `require_owned` (`lib.rs:401`) remains the single gate.
- The `ServiceManager` trait stays manager-neutral. Adding a typed failure kind to the error type is in scope; adding manager-specific types, methods, or contracts to the trait is not.
- The `MAX_TRANSITION_TIMEOUT` ceiling of 300 s (`lib.rs:617`) is **not raised**. The unified model lowers the aggregate bound; it never lifts the ceiling.
- Diagnostics remain finitely bounded. No unbounded string, buffer, or subprocess capture is introduced.
- Windows SCM error mapping stays lossless: the OS code must remain available to the caller, not merely a sentence.
- No live destination parent is created. `atomic_write_definition` keeps its no-parent-creation behavior (`lib.rs:1088-1089`).
- `LifecycleState` and its variants are unchanged.
- The crate gains no dependency: `eggup-core`, `windows-args =0.2.0`, and `windows-service =0.8.1` (Windows target only) — `crates/eggup-service/Cargo.toml:23-28`.

## 5. Scope

### In scope

- A typed manager failure classification carried on `ServiceError::Manager`, with the kind decided at the boundaries where the information exists.
- Removal of `permission_hint` and of every self-sniff of the crate's own error text, including the three `contains("missing")` sites in `inspect_host`.
- One per-operation deadline model applied uniformly across systemd, launchd, cron, and Windows SCM.
- One `crontab -l` per `CronManager::inspect`; one ownership rule stated once and applied by `inspect`, `install`, and `uninstall` alike; an explicit executable-containment gate on the destructive paths.
- One byte cap for manager-derived free text.
- `# Errors` rustdoc for the crate's public `Result`-returning API, and `# Panics` for the existing `FakeExecutor` panic sites.
- Crate and architecture documentation, changelog, tests, and the closure record.

### Explicitly out of scope

- **No commit-engine consolidation.** That is service milestone `012`. M013 does not touch `eggup-core`'s transaction engine or the service crate's integration with it.
- **No Windows SCM backend EXECUTION coverage.** That is service milestone `014`. M013 does not add a real-SCM execution fixture; the SCM error-kind mapping fixture required here is a pure-function fixture.
- **No transport, archive, or Eggpack dependencies** in `eggup-service`, and no changes to `eggup-acquisition`, `eggup-eggfetch`, `eggup-curl`, `eggup-archive`, or `eggup-eggpack`.
- **No republication.** M013 does not publish, does not run `cargo publish`, does not create a GitHub release, and does not move the `v0.1.3` tag.
- **No new dependencies** of any kind, including dev-dependencies.
- **No change to the manager command argv surface.** The argv handed to `systemctl`, `launchctl`, `crontab`, and the SCM backend is unchanged.
- **No change to `LifecycleState`**, `Ownership`, `HealthState`, `ServiceOperation`, or `TransitionResult`'s field set.
- No change to `cron_classify` / `cron_merge` / `cron_remove`'s public signatures (`lib.rs:2842`, `2870`, `2915`); they remain pure functions over `(crontab, marker, desired)`.
- No change to the crontab "no crontab" protocol detection at `lib.rs:3018`. That is a match against the *manager's output*, not against this crate's own error text; it is recorded as an adjacent finding rather than fixed here.
- No change to `disposition.rs` or `lifecycle_update.rs` semantics beyond the mechanical error-typing migration required by Defect A.

## 6. Required production changes

The requirements below are observable outcomes, not an implementation sketch.

### 6.1 Defect A — typed manager failure classification

1. A new public enum `ManagerFailureKind` MUST exist, `#[non_exhaustive]`, `Copy`, `Debug`, `PartialEq`, `Eq`.
2. It MUST contain at least: `Spawn` (the manager child could not be created or driven), `Timeout` (a deadline expired), `Permission` (the platform reported an access denial), `NotFound` (the manager program or a referenced manager resource does not exist), and `Protocol` (the manager ran and produced a result the crate cannot accept — non-zero exit on a mutation, malformed manager output, or an output bound violation). Additional kinds are permitted only if every `ServiceError::Manager` construction site maps to exactly one of them. There MUST be no "unclassified" fallback.
3. `ServiceError::Manager` MUST become a struct variant carrying the kind and a bounded detail, e.g. `Manager { kind: ManagerFailureKind, detail: String }`. `detail` MUST remain bounded on a UTF-8 character boundary.
4. Every `ServiceError::Manager` construction site MUST be classified:

   | Site | Kind |
   |---|---|
   | `lib.rs:738-743` spawn: `NotFound` / `PermissionDenied` / other `io::ErrorKind` | `NotFound` / `Permission` / `Spawn` |
   | `lib.rs:818`, `830`, `837`, `856` deadline expiry; `832`, `839` output-collection expiry | `Timeout` |
   | `lib.rs:804`, `822`, `870` stdin / wait failure; `lib.rs:842` output bound violation | `Spawn`; `Protocol` |
   | `lib.rs:901`, `913` manager program missing; `lib.rs:1724-1726` deadline exhausted | `NotFound`; `Timeout` |
   | `lib.rs:1809-1814` manager ran, non-zero status; `lib.rs:1606`, `1609`, `1616`, `1621`, `1624` observation parse | `Protocol` |
   | `windows_scm.rs:911-917` SCM code `5` / `1060` / other | `Permission` / `NotFound` / `Protocol` |
   | `lib.rs:1107`, `1160`, `1167`, `1186-1197`, `1216`, `1231`, `2090`, `2302`, `2626` local filesystem | a distinct local-filesystem kind |
   | `lib.rs:3010`, `3021`, `3044` cron; timeout inside `list_crontab` | `Protocol`; `Timeout` |
   | `disposition.rs:1652-1703`, `lifecycle_update.rs:771-808` injected test failures | an explicit kind; never identified by matching text |

5. The classification logic MUST be factored into small pure mapping functions unit-testable without spawning a real process. A fixture MUST be able to produce a `Permission` kind from a constructed `std::io::Error` on every platform, including non-Windows lanes.
6. `permission_hint` (`lib.rs:1788-1803`) MUST be **removed**. No replacement function may classify a failure by substring-matching error text. The operator-facing remediation wording the hint carried MUST either be derived from the typed kind at `Display` time or be dropped; in either case the typed kind MUST be the carrier of the classification, and the detail text MUST continue to carry the manager's own bounded stderr so no operator-visible information is lost.
7. `inspect_host`'s three `contains("missing")` tests (`lib.rs:3279`, `3292`, `3304`) MUST be migrated to the typed kind. "Manager not installed" MUST be derived from `kind == NotFound`, never from the detail text.
8. `windows_scm.rs:1541`, `lib.rs:3409-3412`, `4548`, `4923` MUST be migrated to the typed kind.
9. `Display` for `ServiceError` (`lib.rs:331-342`) MUST render the kind, so a consumer that logs only the message still sees a machine-distinguishable failure class.
10. The Windows SCM error mapping MUST remain lossless: the OS code MUST stay reachable programmatically, not only inside a rendered sentence.

### 6.2 Defect B — one timeout model

1. Every adapter MUST create exactly **one** `OperationDeadline` at the start of each public operation and thread it through **every** manager invocation that operation makes. No manager invocation may be granted a fresh full budget.
2. The budget for `start` / `stop` / `restart` is the caller-supplied `timeout` parameter. This is already the behavior for launchd and Windows SCM and MUST not regress.
3. The budget for `install` / `uninstall` / `inspect` — which take no timeout parameter — is `install.transition_timeout` for the whole operation. The `ServiceManager` trait signature MUST NOT change to accommodate this.
4. `bounded_timeout` (`lib.rs:1734-1736`) MUST NOT be used to hand a fresh full timeout to an individual manager call. Any remaining use MUST be the single initial clamp when the operation's deadline is constructed.
5. Required observable bounds after M013: systemd `install` shares one budget across `ownership_of`, the ownership recheck, `daemon-reload`, and `enable`; systemd `uninstall` across `ownership_of`, the recheck, `disable`, and `daemon-reload`; launchd `install` / `uninstall` across all manager calls; launchd `start` / `stop` / `restart` across all manager calls (unchanged); Windows SCM across all backend calls per operation (unchanged); cron `install` / `uninstall` across `list_crontab`, the pre-write re-listing, and `write_crontab`.
6. The currently-permitted aggregate overrun MUST NOT remain permitted. Before M013 a systemd `install` could consume roughly 4-5x the declared budget; after M013 the aggregate wall time of one operation is bounded by one transition timeout. This is a fail-closed tightening: an operation that previously succeeded by exceeding its budget MUST now fail with a `Timeout` kind rather than overrun.
7. The `ServiceManager` trait's documented contract ("within `timeout`") and `architecture/service-lifecycle.md`'s "remaining budget" statement MUST hold for every adapter after M013.

### 6.3 Defect C — one listing, one rule, an explicit destructive-path gate

1. `CronManager::inspect` MUST perform exactly **one** `crontab -l` invocation. The shadowed ownership computed at `lib.rs:3094-3096` MUST be removed.
2. The ownership rule MUST be stated **once**, in one place, and used by `inspect`, `install`, and `uninstall` alike. The OR variant at `lib.rs:3076` and the duplicated inline rule at `lib.rs:3102-3114` MUST NOT both survive.
3. **Decision: the executable-containment gate applies to the destructive paths.** The single rule is:
   - The caller's request material must be internally consistent: the managed block body (`desired_block`) MUST contain `spec.executable()`. If it does not, every cron operation (`inspect`, `install`, `uninstall`) MUST refuse with `ServiceError::InvalidInput` before any listing or mutation.
   - Given consistent material, ownership is decided **only** by `cron_classify`: exactly one block with the marker whose body byte-equals `desired_block` is `Owned`; a same-marker block with different content is `Foreign`; multiple or unterminated blocks are `Unknown`; no block is `Absent`.
   - The containment check MUST be applied to the **managed block body**, not to the whole crontab. The current whole-crontab test at `lib.rs:3106` MUST be removed.
4. Why this direction wins:
   - The documented intent at `lib.rs:3057-3069` states the gate explicitly. The alternative — correcting the documentation down to "exact-content identity only" — would leave the spec/block disagreement undetected and preserve the contradiction where `install` succeeds and the subsequent `inspect` reports `Ownership::Foreign`.
   - The current live check is weaker than the documented rule (whole-crontab containment), so "exact-content identity only" is not what the code already does; it would be a second, different weakening.
   - Applying the gate to the destructive paths is the fail-closed direction: a caller whose material disagrees with the spec gets a refusal, not a write.
5. `install` MUST stop ignoring its `spec` parameter (`lib.rs:3130`, `_spec`). `uninstall` MUST apply the same gate (`lib.rs:3194`).
6. The pre-write re-listing in `install` (`lib.rs:3150`) and `uninstall` (`lib.rs:3221`) MUST be retained. Only `inspect`'s duplicate listing is removed.
7. `Foreign` and `Unknown` MUST continue to deny mutation, and a crontab that cannot be listed MUST continue to report `Unknown` rather than `Absent` (`lib.rs:3118`).

### 6.4 Defect D — one byte cap

1. Manager-derived free text has **one** cap: **512 bytes on a UTF-8 character boundary**, matching the crate doc at `lib.rs:286` and `ServiceError::bounded` (`lib.rs:310-312`).
2. The 256-byte cap in `truncate` (`lib.rs:1784-1786`) and the `512 - HINT.len()` re-cap at `lib.rs:1795` MUST NOT remain as separate caps for the same data class. Because the composed detail is bounded at 512 either way, removing the 256-byte excerpt cap raises no final information bound.
3. Exactly one bounding helper MUST be used for manager-derived free text, and the crate doc, the rustdoc, and `architecture/service-lifecycle.md` MUST state the same number.

### 6.5 Defect E — documented failure semantics

1. Every public `Result`-returning item MUST carry a `# Errors` rustdoc section naming the concrete error variants it can return and the condition for each. The required set: `ServiceId::new` (`lib.rs:41`), `ServiceSpec::new` (`lib.rs:79`), `require_owned` (`lib.rs:401`), `SystemExecutor::with_max_output` (`lib.rs:691`), `atomic_write_definition` (`lib.rs:1090`), `inspect_host` (`lib.rs:3268`), `disposition::plan_unix` (`disposition.rs:125`), `disposition::plan_windows` (`disposition.rs:224`), `LifecycleUpdatePolicy::validate` (`lifecycle_update.rs:96`), all six `ServiceManager` trait methods (`lib.rs:364-398`), and the `ServiceManager` implementors' public constructors. **The Windows SCM public surface is excluded from this set**: `windows_scm.rs` is owned by milestone `014` (`plans/implementation/service-lifecycle/014-windows-scm-execution-and-api-surface-corrective.md`), whose R-D1/R-D3 reserve `WindowsScmManager::new` (`windows_scm.rs:240`), `WindowsScmInstall` and its validating builders (`:70`, `:97`, `:107`), the SCM scope and option types (`:21`, `:32`, `:45`, `:58`), and the SCM error mapping (`:911-917`, `:215-224`). The two milestones MUST NOT edit the same doc line; M013's closure records the coordination.
2. Each `# Errors` section MUST be consistent with the typed kinds from §6.1 — in particular, each `ServiceManager` method's section MUST state which failure kinds its adapter can produce.
3. The `FakeExecutor` `expect("fake lock")` panic sites are the crate's only panic paths. Each public `FakeExecutor` method that can panic MUST carry a `# Panics` section stating the condition. M013 MUST NOT change the panic behavior; documenting it is sufficient.
4. At least one `# Errors` section MUST include a fenced code example showing how a consumer distinguishes two failure kinds without parsing text.

## 7. Ordered work packages

Each work package is independently verifiable and leaves the tree green.

- **WP1 — Failure-kind taxonomy and mapping.** Add `ManagerFailureKind`; add the pure mapping functions; classify every construction site listed in §6.1; retype `ServiceError::Manager` to a struct variant carrying the kind and the bounded detail; update `Display`; keep the SCM OS code reachable. Verify: `cargo test -p eggup-service --locked` is green apart from the fixtures WP7 adds, and the trait shape is unchanged.
- **WP2 — Remove `permission_hint` and migrate consumers.** Delete `permission_hint` (`lib.rs:1788-1803`) and all eleven call sites; migrate `inspect_host`'s three `contains("missing")` tests to the typed kind; update the text-asserting tests at `lib.rs:3409-3412`, `4548`, `4923`, and `windows_scm.rs:1541`. Verify: a search for `contains("permission")`, `contains("denied")`, `contains("access")`, `contains("missing")`, and `contains("deadline")` against the crate's own error text returns nothing.
- **WP3 — Unified timeout model.** Introduce one `OperationDeadline` per operation in systemd, launchd, cron, and Windows SCM; remove every per-call fresh full budget; keep the 300 s ceiling; keep the trait signatures. Verify: the recording-executor fixture from WP7 shows the aggregate budget of a chained systemd install is at most one transition timeout.
- **WP4 — Cron listing collapse, rule unification, destructive-path gate.** Reduce `inspect` to one listing; state the rule once; apply the executable-containment gate to `install` and `uninstall`; remove the whole-crontab containment test at `lib.rs:3106`; retain the pre-write re-listing. Verify: the cron fixtures from WP7.
- **WP5 — Byte-cap unification.** Apply the single 512-byte bound to manager-derived free text; remove the 256-byte excerpt cap and the `512 - HINT.len()` re-cap; keep every bound UTF-8 safe.
- **WP6 — `# Errors` / `# Panics` doc sweep.** Add the sections enumerated in §6.5 across `lib.rs`, `disposition.rs`, and `lifecycle_update.rs`. `windows_scm.rs` is **excluded** — milestone `014` owns that surface, and the two milestones must not edit the same doc line. Verify: `cargo doc -p eggup-service --no-deps --locked` is clean and the required set is present.
- **WP7 — Tests.** Add the fixtures enumerated in §10, including the baseline-failing proofs.
- **WP8 — Changelog and documentation.** Update the crate `CHANGELOG.md` `Unreleased` section, the root `CHANGELOG.md`, the crate `README.md`, and `architecture/service-lifecycle.md`.

Order: WP1 → WP2 → WP3 → WP4 → WP5 → WP6 → WP7 → WP8. WP3 and WP4 are independent of each other and may be developed in either order, but both precede WP7.

## 8. Failure, cancellation, restart, and contention semantics

1. **No failure may be downgraded to success.** A `Timeout`, `Spawn`, `Permission`, `NotFound`, or `Protocol` failure MUST NOT yield `TransitionResult { completed: true }`, a `LifecycleSnapshot` claiming `Absent`, or any other success signal.
2. **The kind is the carrier of meaning.** The remediation text MUST NOT be the only carrier. Two failures with different kinds MUST be programmatically distinguishable without inspecting `Display` output.
3. **Timeout vs. spawn failure.** A timeout is a deadline that expired while the operation was in flight; a spawn failure is a child that was never created. They MUST be different kinds. A deadline that had already expired before a manager command was issued MUST be `Timeout` (`lib.rs:1724-1726`), not a spawn failure.
4. **Timeout vs. permission failure.** A timeout is bounded by the operation's deadline; a permission failure is a platform access denial. A manager that runs and reports an access denial in its stderr is `Protocol` unless the platform supplied a structured denial (Windows SCM code `5`, or a Unix `io::ErrorKind::PermissionDenied` from the spawn path). M013 MUST document this limitation rather than reintroduce text matching.
5. **Partially applied systemd install.** If `daemon-reload` or `enable` fails or times out after the unit file was written, the operation MUST report the failure with its kind, and the caller MUST be able to observe that the unit file exists. The crate MUST NOT report the install as complete. The existing non-complete `TransitionResult` shape (`lib.rs:2072-2079`, `2104-2111`) is the model.
6. **Cron path when the crontab cannot be listed.** A failed listing MUST yield `Ownership::Unknown` and `LifecycleState::Unknown` (`lib.rs:3118`), MUST NOT yield `Absent`, and MUST NOT permit any mutation. The `install` and `uninstall` paths MUST propagate the listing failure.
7. **Deadline semantics under the unified model.** When the shared deadline is exhausted before a manager command is issued, the command MUST NOT be issued and the failure MUST be `Timeout`. When the deadline expires during a command, the child MUST be killed and reaped (existing behavior, `lib.rs:852-858`).
8. **Everything fails closed.** `Foreign` and `Unknown` never permit mutation. A cron request whose material disagrees with the spec is refused before any mutation.
9. **Contention.** A crontab that changes between the listing and the pre-write re-listing MUST produce `ServiceError::Conflict` and no write (`lib.rs:3151-3155`, `3222-3226`).

## 9. Compatibility and migration

**Published baseline.** `eggup-service 0.1.3` is published (2026-10-09, source `feb6ae5`, checksum `9f7f7ea8…32124dc`). M013 does not publish and does not move the `v0.1.3` tag. Republication is a separate milestone (§16).

**The `ServiceError::Manager` shape change.** `ServiceError` is `#[non_exhaustive]` (`lib.rs:288`), so external consumers already carry a wildcard arm and *adding* variants would have been source-compatible. The unavoidable break is different: changing `Manager(String)` to `Manager { kind, detail }` breaks any consumer that constructs `ServiceError::Manager(s)` or matches `ServiceError::Manager(_)` (a tuple pattern does not match a struct variant). `ServiceError::Manager { .. }` matches the new shape. The `ServiceManager` trait's method signatures, the trait itself, and every other public type in the crate are unchanged.

**Rejected alternative.** Keeping `Manager(String)` and adding a parallel `ManagerFailed { kind, detail }` variant would be source-compatible, but every existing consumer pattern `ServiceError::Manager(_)` would then silently stop matching the new failures — a silent misclassification, which is worse than a compile error. M013 rejects it.

**Precedent: core M012.** `eggup-core` solved a structurally similar problem in M012 (`plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`) by introducing an additive `RecoveryError` type instead of retyping the published `Error` enum. M013 does **not** follow that mechanism: core M012's problem was *adding* variants to a non-`#[non_exhaustive]` enum, which broke exhaustive matches. Here the enum is already `#[non_exhaustive]`, and the unavoidable break is a *field-shape* change on an existing variant. A parallel error type would not avoid the break — it would require changing every published signature that returns `ServiceError`, strictly larger than retyping one variant. M013 follows core M012's *decision discipline* instead: prefer compatibility, never publish a known source-breaking patch inside 0.1.x, and sequence the break to a later publication. M013 lands the break in the unpublished tree; the separate republication milestone decides whether the next published version is `0.1.4` (Cargo-compatible, source-breaking for consumers) or `0.2.0` (the intentional API boundary that registry finding 2 already defers to). The `CleanupDisposition` / `Error`-extensibility question recorded at `plans/registry.md` § "Post-0.1.3 open findings" item 2 belongs to that same boundary and MUST be decided together with this change.

**Consumer migration.** A consumer that needs to distinguish failure kinds replaces `ServiceError::Manager(_)` with `ServiceError::Manager { kind, .. }` and matches on `kind`. A consumer that only logs the error needs no change: `Display` renders the kind.

**Timeout-model change.** A caller that relied on a systemd `install` or `uninstall` consuming several times its declared transition timeout will now observe that operation bounded by one budget. Operations that previously succeeded by overrunning now fail with `Timeout`. This is a fail-closed tightening and is the intended behavior; it must be stated in the changelog.

**Cron executable-gate change.** A consumer whose `desired_block` does not contain `spec.executable()` will now receive `ServiceError::InvalidInput` from `inspect`, `install`, and `uninstall` where it previously received success (`install`) or a normal result. This is a published-behavior tightening for `CronManager` consumers and MUST be called out explicitly in the changelog with the exact required material: the managed block body must embed the absolute executable path of the spec.

**Byte-cap change.** The composed manager detail keeps the documented 512-byte bound; only the internal 256-byte excerpt cap is removed. A consumer that observed manager stderr excerpts truncated at 256 bytes will now observe up to 512 bytes inside the same 512-byte detail bound.

**Secondary public surface.** `FakeCall` (`lib.rs:984-991`) is a public test-support struct. Extending it to record the per-command timeout would be a source-breaking addition for any consumer constructing `FakeCall` literally. The preferred implementation is a dedicated `CommandExecutor` implementation inside the test module that records the timeout itself; `CommandExecutor` is public (`lib.rs:648`), so this requires no public API change. M013 MUST prefer the non-breaking option.

## 10. Required tests

All fixtures MUST be deterministic. The repository has a documented precedent: `plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md` exists because sub-second deadline tests are load-sensitive. A new timeout-related fixture MUST NOT depend on host scheduling pressure and MUST NOT sleep on the wall clock to prove a deadline. The proof MUST be made on the **budgets handed to the executor** and on **call counts**, which are exact and machine-independent.

1. **Failure-kind fixtures.** One fixture per kind, driving the pure mapping functions: `Spawn`, `Timeout`, `Permission`, `NotFound`, `Protocol`. `Permission` MUST be constructible from a constructed `std::io::Error` on every lane, with no real privilege failure. `NotFound` MUST be reachable through the allowlist path (`resolve_manager_program` / `validate_executable_path`). `Timeout` MUST be reachable through `OperationDeadline::command_timeout_at` using the crate's existing injectable-clock seam (`new_at` / `command_timeout_at`, `lib.rs:1702`, `1723`) — no wall-clock sleep.
2. **Windows SCM mapping fixture.** The SCM OS-code → kind mapping MUST be factored into a platform-independent function (the pattern already used by `scm_error_detail`, `windows_scm.rs:214-215`, guarded `#[cfg(any(windows, test))]`) so that code `5` → `Permission`, code `1060` → `NotFound`, and other codes → `Protocol` are asserted on **every** lane, including macOS and Linux. The Windows lane additionally runs the native `WindowsBackend` path.
3. **Unified timeout fixture.** A recording `CommandExecutor` implementation that captures the `timeout` argument of every `run` call MUST drive a scripted systemd `install` on an `Owned` unit. The fixture MUST assert the expected number of manager calls, that every recorded budget is at most the operation budget, and that the **sum** of the recorded budgets is at most one transition timeout. The sum assertion is the one that fails against the pre-M013 implementation, where each call received a fresh full budget.
4. **Single cron listing fixture.** A recording executor MUST assert that `CronManager::inspect` issues exactly one `crontab -l` invocation. It MUST also assert that `install` and `uninstall` still issue the expected re-listing before their write, so the safety check is proven retained.
5. **Single cron rule and destructive-path gate fixture.** For one managed block, fixtures MUST assert: the block is `Owned` in `inspect` when the block body contains the spec executable; the same block is refused by `inspect`, `install`, and `uninstall` with `InvalidInput` when the spec executable and the block body disagree; a same-marker block with different content is `Foreign` and is never mutated; an unrelated crontab entry mentioning the executable path does NOT make a non-matching block look `Owned` (the `lib.rs:3106` whole-crontab weakness).
6. **Byte-cap fixture.** A manager error detail longer than 512 bytes MUST be bounded at 512 bytes on a UTF-8 character boundary, including a multi-byte code point crossing the boundary.
7. **`# Errors` presence check.** A test or review checklist item MUST assert that each public `Result`-returning item listed in §6.5 carries a `# Errors` section.
8. **Negative controls.** Every new fixture MUST include a control that proves it can fail: a wrong expected kind, an extra expected manager call, or an over-long detail must make the fixture fail.
9. **Pre-M013 baseline proof.** The behavioral fixtures — items 3, 4, 5, and 6 — MUST be written so they **compile against the pre-M013 public API** and then **fail by assertion** at the baseline. The exact command is in §11. The fixtures that require the new typed API (items 1 and 2) cannot compile at the baseline; that compile failure IS their baseline proof and MUST be recorded as a compile failure, not reported as an assertion failure.

## 11. Required verification commands

```text
cargo test -p eggup-service --locked
cargo test -p eggup-service --locked cron
cargo test -p eggup-service --locked timeout
cargo clippy -p eggup-service --all-targets --locked -- -D warnings
cargo doc -p eggup-service --no-deps --locked
cargo test --workspace --all-targets --all-features --locked
./scripts/check-local.sh
```

Plus the deterministic-test lane:

```text
cargo test -p eggup-service --locked -- --test-threads=1
```

### Baseline-failure proof

```text
git worktree add /tmp/eggup-m013-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
```

Copy the new behavioral test files into the worktree's `crates/eggup-service/tests/` directory and run, in the worktree:

```text
cd /tmp/eggup-m013-baseline && cargo test -p eggup-service --locked <fixture-name>
```

Expected results, recorded verbatim in the closure record:

- The cron single-listing fixture: **fails by assertion** at the baseline (two `crontab -l` calls, not one).
- The unified-timeout fixture: **fails by assertion** at the baseline (the summed budgets exceed one transition timeout).
- The cron destructive-path gate fixture: **fails by assertion** at the baseline (no gate is applied).
- The byte-cap fixture: **fails by assertion** at the baseline if it asserts the single 512-byte bound for the composed detail.
- The typed-kind fixtures: **fail to compile** at the baseline, because `ManagerFailureKind` does not exist there.

```text
git worktree remove /tmp/eggup-m013-baseline
```

## 12. Documentation updates

1. **Rustdoc.** `# Errors` sections for every public `Result`-returning item enumerated in §6.5, and `# Panics` sections for the `FakeExecutor` methods that can panic. The `ServiceManager` trait methods' `# Errors` sections MUST state the failure kinds each adapter can produce.
2. **Crate `CHANGELOG.md`.** An `Unreleased` entry covering: the typed failure kinds and the removal of `permission_hint`; the unified per-operation deadline and the resulting fail-closed tightening; the cron single-listing and single-rule change and the new destructive-path executable gate; the single 512-byte cap; the `# Errors` sweep. The entry MUST carry the repository's no-publication / no-migration disclaimer: this work is **not** published by M013, and no consumer migration is required until a republication milestone ships it.
3. **Root `CHANGELOG.md`.** The same entry aggregated into the root `Unreleased` section, which does not replace the per-crate changelog.
4. **Crate `README.md`.** The failure-kind taxonomy and the one-budget contract, if the README states either.
5. **`architecture/service-lifecycle.md`.** The `ServiceError` table row (`architecture/service-lifecycle.md:109`), the `### ServiceError` section (`:255-261`), the `CommandExecutor` contract paragraph (`:370-374`), the deadline-handling contract (`:345-347`), the manager-error table rows (`:926-929`), and the cron classification table (`:853-858`) MUST be updated to the post-M013 facts. The line references in that file for the items this milestone touches MUST be corrected to the post-M013 line numbers; this milestone does not attempt a wholesale line-reference refresh of untouched items.
6. **`docs/` is non-normative.** If a consumer-facing how-to under `docs/` mentions service failure handling or service timeouts, it MAY be updated to point at the contract, but no rule may be introduced there. `docs/` never becomes the place a rule lives.
7. **`plans/registry.md` and the roadmap** are updated by the implementing milestone's closure step, not by this plan.

## 13. Acceptance criteria

M013 closes only when all of the following are checkable:

1. `ServiceError::Manager` carries a machine-readable `ManagerFailureKind`, and every `ServiceError::Manager` construction site maps to exactly one kind, with no unclassified fallback.
2. `permission_hint` is gone, and no function in the crate classifies a failure by substring-matching its own error text. A search for `contains("permission")`, `contains("denied")`, `contains("access")`, `contains("missing")`, and `contains("deadline")` against crate error text returns nothing.
3. `inspect_host` derives "manager not installed" from the typed kind, not from detail text.
4. Every adapter creates exactly one `OperationDeadline` per operation; the summed budgets of a chained systemd `install` are at most one transition timeout, proven by a deterministic fixture that asserts on the budgets handed to the executor and performs no wall-clock sleep.
5. `MAX_TRANSITION_TIMEOUT` remains 300 s and is not raised.
6. `CronManager::inspect` issues exactly one `crontab -l` invocation.
7. One cron ownership rule is stated once and applied identically by `inspect`, `install`, and `uninstall`; the executable-containment gate applies to the destructive paths; the pre-write re-listing is retained.
8. A cron request whose block body does not name the spec executable is refused with `InvalidInput` before any mutation, identically across `inspect`, `install`, and `uninstall`.
9. `Foreign` and `Unknown` still deny mutation, and a failed crontab listing still reports `Unknown`, never `Absent`.
10. Manager-derived free text has exactly one bound: 512 bytes on a UTF-8 character boundary, stated identically in the crate doc, the rustdoc, and `architecture/service-lifecycle.md`.
11. Every public `Result`-returning item in §6.5 carries a `# Errors` section, and the `FakeExecutor` panic sites carry `# Panics` sections.
12. Each new behavioral fixture is proven to fail against the baseline `c4805411f74b85158e0ae5d0a219a32c4209d98e` by the command in §11, with the result recorded verbatim.
13. `cargo test -p eggup-service --locked`, `cargo clippy -p eggup-service --all-targets --locked -- -D warnings`, `cargo doc -p eggup-service --no-deps --locked`, `cargo test --workspace --all-targets --all-features --locked`, and `./scripts/check-local.sh` are green.
14. The crate's dependency set is unchanged: `eggup-core`, `windows-args =0.2.0`, `windows-service =0.8.1` (Windows target only). No transport, archive, or Eggpack dependency is present.
15. The `ServiceManager` trait's shape is unchanged and remains manager-neutral; `LifecycleState` is unchanged.
16. The crate `CHANGELOG.md` `Unreleased` entry and the root `CHANGELOG.md` carry the change with the no-publication / no-migration disclaimer.
17. No publication occurred and the `v0.1.3` tag was not moved.
18. No medium-or-higher finding remains open against this milestone.

## 14. Stop conditions

Stop, do not improvise, and escalate for a design decision if any of the following occurs:

- The typed failure taxonomy cannot be introduced without a breaking public API change larger than retyping the `Manager` variant — in particular, if any published `ServiceManager` signature, any other crate's public API, or `LifecycleState` would have to change. The break MUST then be sequenced at the 0.2 / 1.0 boundary instead, and this milestone stops.
- The unified timeout model cannot be applied to one adapter without breaking its published contract — specifically, if removing a per-call fresh budget would make a documented, published behavior unreachable, that adapter's change MUST be split into its own milestone.
- The cron executable-containment gate cannot be applied to the destructive path without changing ownership semantics — specifically, if enforcing the gate would ever permit a mutation the current code denies, or would turn a `Foreign` / `Unknown` into a permitted mutation. The gate MUST fail closed; if it cannot, this milestone stops.
- The single cron rule cannot be stated once without weakening the exact-content identity check in `cron_classify` / `cron_merge` / `cron_remove`.
- Implementing the typed kinds would require the crate to gain a dependency, would require `unsafe_code`, or would require relaxing `#![deny(missing_docs)]`.
- A new timeout fixture cannot be made deterministic without a wall-clock sleep. A load-sensitive fixture MUST NOT be added; the fixture must assert on budgets and call counts instead.
- The SCM error-kind mapping cannot be factored into a platform-independent function, in which case its fixture MUST be recorded as a Windows-lane-only result and MUST NOT be reported as evidence on other lanes.
- Any lane was skipped. A lane that was not run is **not run**, never inferred.

## 15. Closure evidence required

The closure record is written at `plans/closure/service-lifecycle/013-status.md` and MUST contain, per `plans/closure/README.md`:

1. **Status**: closed, conditionally closed, corrective pass required, or blocked.
2. **Source plan and roadmap**: this plan and `plans/subsystems/service-lifecycle-roadmap.md`.
3. **Reviewed repository baseline**: `c4805411f74b85158e0ae5d0a219a32c4209d98e`.
4. **Implementation commits / PRs**.
5. **Executive finding**: whether the four defects are closed, and what remains.
6. **Requirement-to-evidence matrix**: one row per §6 requirement, with the fixture or command that proves it.
7. **Production implementation evidence**: the classified construction-site list for §6.1; the per-adapter deadline construction sites for §6.2; the cron rule statement and its three call sites for §6.3; the single bounding helper for §6.4; the documented-function list for §6.5.
8. **Exact commands with results**, including the baseline-failure proof from §11, recorded verbatim with the outcome and the observed failure message for each baseline-failing fixture.
9. **Invariant review**: §4 item by item, with the file:line proving each one still holds after the change.
10. **Failure / rollback / recovery review**: the §8 semantics, including the partially applied systemd install and the unlistable crontab.
11. **Compatibility and migration review**: the §9 consequences, the consumer migration steps, and an explicit statement that M013 published nothing and did not move `v0.1.3`.
12. **Security review**: the allowlist, the minimal environment, the absence of elevation, the lossless SCM mapping, and the single bounded detail.
13. **Documentation / operations evidence**: the rustdoc sweep, the crate and root changelog entries, the `architecture/service-lifecycle.md` updates.
14. **Unresolved findings with severity**, stated explicitly with the reason for the severity, per the planning workflow's severity gate.
15. **Separate platform results**: Linux, macOS, and Windows MUST be reported separately, from the `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` lanes as applicable. **A lane that was not run is not run, never inferred.** The Windows-lane-only nature of the native SCM path MUST be stated.
16. **Roadmap disposition** and **registry updates**, including the new `plans/registry.md` § "Post-0.1.3 open findings" entry 4.

Happy-path-only evidence is insufficient for closure.

## 16. Handoff notes

**Sequencing.** M013 is independent of service milestone `012` (commit-engine consolidation) and service milestone `014` (Windows SCM backend EXECUTION coverage): it touches neither. It may land before, between, or after them. If `012` lands first, M013's implementation MUST be rebased onto it and the closure record MUST state the rebased baseline. M013's cron and deadline changes are in `eggup-service` only and do not conflict with `012`'s scope by construction; `014`'s real-SCM execution fixture is the natural place to observe the typed SCM kinds against a live SCM, and M013's platform-independent mapping fixture is its prerequisite, not its substitute.

**What a separate republication milestone would require.** If M013 closes, a separate milestone — not M013 — owns the next `eggup-service` publication. That milestone must: choose the next version (`0.1.4` versus the intentional `0.2.0` API boundary) and record the decision together with registry finding 2 (`CleanupDisposition` / `Error` extensibility); publish from a reviewed source commit with a recorded checksum; run the registry-only fixture against the published package; extend the GitHub release **without** moving the `v0.1.3` tag; update the crate `CHANGELOG.md` from `Unreleased` to the released version; and record the consumer migration steps from §9 in the release notes. M013 does none of this.

**What this plan does NOT authorize.**

- It does not authorize publishing, releasing, or tagging.
- It does not authorize a commit-engine consolidation (`012`) or a Windows SCM backend execution fixture (`014`).
- It does not authorize a new dependency, an `unsafe` block, an automatic privilege elevation, a widened manager allowlist, or a widened child environment.
- It does not authorize a change to the `ServiceManager` trait shape, to `LifecycleState`, or to the manager command argv surface.
- It does not authorize changing the ownership model, the fail-closed direction, or the pre-write re-listing safety checks.
- It does not authorize weakening the 300 s transition ceiling or any diagnostic bound.
- It does not authorize editing `plans/registry.md`, the roadmap, any closure record, or any other plan.
