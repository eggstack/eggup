# Service Lifecycle Milestone 014 — Windows SCM execution coverage and published API surface corrective

Status: closed; see `plans/closure/service-lifecycle/014-status.md` (the real Windows SCM fixture ran green on the hosted Windows lane, run `40008039843`; hosted-qualified on Linux, macOS, and Windows)

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Original closed work affected:

- `plans/implementation/service-lifecycle/004-windows-scm-adapter.md`
- `plans/closure/service-lifecycle/004-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **6**: "`eggup-service`'s real Windows SCM backend has never been executed on any platform, and the published API surface has no local-gate guard." Registered at baseline `c480541` alongside this plan. The finding is assessed **high** severity and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: infrastructure

> **Classification note.** `.opencode/skills/planning-workflow/SKILL.md` §"Classification" states that *"Infrastructure alone is not a completed capability until a real consumer proves the path."* M014 is classified `infrastructure` because it adds **execution evidence for a path that already exists** — the `WindowsBackend` SCM adapter was written and closed under M004 — not a new capability. Closing M014 therefore establishes evidence for an existing capability; it does not create one. A real consumer proof of the SCM path is **not** supplied by M014 and remains outstanding.

## 1. Objective

1. **Execution evidence for the real Windows SCM backend.** Produce a recorded, hosted-run execution of `WindowsBackend` against the genuine Windows Service Control Manager, exercising the `windows-service` API-mapping layer that today has never executed on any platform.
2. **A local-gate guard on the published API surface.** Provide a lightweight, network-free, root-free check that runs inside the root workspace, so that removing or changing a public item of `eggup-service` fails a gate a developer runs locally — not only a CI lane that requires root on Linux.
3. **Per-variant dead-code gating** so that a genuinely-unreachable `ScmState` / `ScmLookup` variant is still reported.
4. **`# Errors` documentation for the Windows SCM public surface only**, without duplicating the crate-wide sweep owned by milestone `013`.

## 2. Why this milestone is ready

- Every defect below is **fully characterized by reading the source at the baseline SHA** `c4805411f74b85158e0ae5d0a219a32c4209d98e`. Each claim in §3 carries a `file:line` citation; none depends on an unobserved runtime behavior.
- The service-lifecycle subsystem has **no open milestone**: the roadmap status table (`plans/subsystems/service-lifecycle-roadmap.md` §12) records M001–M011 closed, and `eggup-service 0.1.3` was published 2026-10-09 (roadmap §11, M011).
- Scope is bounded to **one crate** (`eggup-service`), its own `tests/` tree, its `README.md`/`CHANGELOG.md`, and `architecture/service-lifecycle.md`. No other crate's production code is modified.
- **Severity, stated explicitly** per the planning-workflow severity gate:
  - **Defect A (the real SCM backend has never executed) — HIGH.** It is the only code path in the whole workspace that talks to a privileged OS service manager, it contains five `unreachable!()` panic sites (`crates/eggup-service/src/windows_scm.rs:977`, `:1051`, `:1077`, `:1089`, `:1100`), and it has never run against the real SCM API on any platform.
  - **Defect B (no local-gate API-surface guard) — MEDIUM.** The crate is published with a wide public surface and the only guard on it is two CI-only fixtures in one root-requiring lane.
  - **Defect C (whole-enum `#[allow(dead_code)]`) — LOW.** It suppresses a diagnostic the workspace otherwise relies on, but it does not change behavior.
  - **Defect D (no `# Errors` on the SCM public surface) — MEDIUM.** `#![deny(missing_docs)]` is on and `clippy::missing_errors_doc` lives in `clippy::pedantic`, not `clippy::all`, so the `-D warnings` gate does not catch it.
- **Honest detection statement:** there is **no failing production report** for any of these defects. They were found by static reading at the baseline SHA. The `FakeBackend` seam (§3.2) is precisely why the gap was invisible: the seam made an unexecuted backend look tested, because the test module is large, green, and names the SCM surface in its assertions.
- **Honest statement of what already works:** the `windows-check` CI lane compiles and runs the Windows-specific code on a real Windows runner (`.github/workflows/ci.yml:64-84`). The defect is therefore **not** "never compiled". It is **"never executed against the real SCM API"**. This distinction must survive into the closure.

## 3. Current implementation evidence

### 3.1 Defect A — the real SCM backend has no call site outside its own constructor

`crates/eggup-service/src/windows_scm.rs` is 1,604 lines and contains a `ScmBackend` trait seam with two implementations.

| Site | Content |
|---|---|
| `windows_scm.rs:748` | `trait ScmBackend: fmt::Debug` — methods `query` (`:749`), `create` (`:750`), `refresh` (`:756`), `start` (`:762`), `stop` (`:763`), `delete` (`:764`), plus provided `test_calls` (`:765-768`, `#[cfg(test)]`) and `sleep` (`:769-771`) |
| `windows_scm.rs:907` | `struct WindowsBackend;` — `#[cfg(windows)]` (`:905`) |
| `windows_scm.rs:910` | `impl WindowsBackend` — `#[cfg(windows)]` (`:909`) |
| `windows_scm.rs:968` | `impl ScmBackend for WindowsBackend` — `#[cfg(windows)]` (`:967`) |
| `windows_scm.rs:239-245` | `WindowsScmManager::new` — `#[cfg(windows)]` (`:239`); hard-wires `backend: Box::<WindowsBackend>::default()` at `:242` |

`grep -n "WindowsBackend" crates/eggup-service/src/windows_scm.rs` returns **exactly four** hits — `:242`, `:907`, `:910`, `:968` — and every one is inside a `#[cfg(windows)]` region. **There is no test call site for `WindowsBackend` anywhere in the repository**, and no other file in the workspace names the type.

The only production constructor of the real backend is `WindowsScmManager::new` (`:240`), which is `#[cfg(windows)]` and is the public entry point a Windows consumer uses.

### 3.2 The test module drives only the fake

`windows_scm.rs:1105-1604` is the entire test module. It contains **15** `#[test]` functions (attributes at `:1210`, `:1251`, `:1273`, `:1285`, `:1292`, `:1320`, `:1336`, `:1372`, `:1383`, `:1395`, `:1421`, `:1518`, `:1546`, `:1574`, `:1590`). **Nine** of them drive the fake backend:

- `inspect_maps_absent_and_requires_exact_command_identity` (`:1337`),
- `validate_spec_rejects_identity_mismatch_and_oversized_command` (`:1373`),
- `install_creates_absent_and_denies_foreign` (`:1384`),
- `refresh_reinspects_owned_registration` (`:1396`),
- `start_stop_restart_use_confirmed_states_and_one_deadline` (`:1422`),
- `restart_does_not_start_if_stop_does_not_complete` (`:1519`),
- `uninstall_reports_marked_for_delete_until_absent` (`:1547`),
- `uninstall_does_not_claim_completion_while_marked_for_delete` (`:1575`),
- `stop_of_already_stopped_registration_is_complete_without_a_control` (`:1591`).

The remaining six are pure and platform-independent: the descriptor validator (`:1211`), the access-denied diagnostic bound (`:1252`), the two command-line parser tests (`:1274`, `:1286`), the config-identity test (`:1293`), and the state-mapping test (`:1321`).

The fake is `struct FakeBackend` (`:1111`), constructed by `FakeBackend::new` (`:1118`) and `FakeBackend::scripted` (`:1126`), and implements `ScmBackend` at `:1135`. It is reached through the `#[cfg(test)]` constructor `WindowsScmManager::with_backend` (`:252-258`) and the local helper `fn manager(backend: FakeBackend)` (`:1269`).

**The seam is the point.** `FakeBackend` implements the same six methods, so the entire orchestration layer — `observe` (`:305`), `require_owned_observation` (`:327`), `wait_for_state` (`:340`), `stop_phase_for_restart` (`:367`) — is genuinely exercised. What is *not* exercised is the half that crosses the OS boundary.

### 3.3 The untested surface: `windows-service` API mapping

| Mapping | Site | Notes |
|---|---|---|
| `start_type` | `windows_scm.rs:867-875` | `WindowsStartType::{Automatic, Manual, Disabled}` → `ServiceStartType::{AutoStart, OnDemand, Disabled}` |
| `error_control` | `windows_scm.rs:877-886` | `WindowsErrorControl::{Ignore, Normal, Severe, Critical}` → `ServiceErrorControl` |
| `service_dependencies` | `windows_scm.rs:888-903` | `WindowsServiceDependency::{Service, Group}` → `ServiceDependency` |
| `ServiceInfo` construction | `windows_scm.rs:946-964` (`WindowsBackend::info`) | builds the `windows_service::service::ServiceInfo` written by `create_service` (`:1022-1027`) |
| config read-back | `windows_scm.rs:979-981`, `:985` | `query_config()` → `config.executable_path.to_str()` → `command_line` |
| `ServiceState` → `ScmState` | `windows_scm.rs:988-1004` | seven `windows_service::service::ServiceState` arms → seven `ScmState` variants |
| `1060` / `1072` mapping | `windows_scm.rs:936-941` | `Self::open` maps `Err(1060)` → `Ok(Err(1060))` (Absent) and `Err(1072)` → `Ok(Err(1072))` (MarkedForDelete) |
| error mapping | `windows_scm.rs:911-917` (`WindowsBackend::map_error`), `:215-224` (`scm_error_detail`) | raw OS code → `ServiceError::manager`; code `5` becomes an access-denied sentence |

### 3.4 The five panic sites

`grep -n "unreachable!" crates/eggup-service/src/windows_scm.rs` returns exactly five hits, all of the form `Err(_) => unreachable!()` over `Self::open`'s `1060 | 1072` contract (`:927-944`):

| Site | Enclosing method | Preceding arm |
|---|---|---|
| `:977` | `query` (`:969`) | `Err(1060)` → `Absent` (`:975`), `Err(1072)` → `MarkedForDelete` (`:976`) |
| `:1051` | `refresh` (`:1031`) | `Err(1060 \| 1072)` → `ServiceError::conflict("service disappeared before refresh")` (`:1048-1050`) |
| `:1077` | `start` (`:1070`) | `Err(1060 \| 1072)` → `ServiceError::conflict("service disappeared before start")` (`:1076`) |
| `:1089` | `stop` (`:1081`) | `Err(1060 \| 1072)` → `ServiceError::conflict("service disappeared before stop")` (`:1088`) |
| `:1100` | `delete` (`:1093`) | `Err(1060 \| 1072)` → `ServiceError::conflict("service disappeared before delete")` (`:1099`) |

Today these arms are unreachable **by construction of `Self::open`**, which can only produce `1060` or `1072` in its inner `Result`. They are therefore not live panics at the baseline SHA. They are a latent panic: the arm asserts an invariant that lives in a different function, across a `windows-service` version boundary, with no compile-time link between the two. If `windows-service` changed its error surface, or if `Self::open` were extended to surface another code, the process would panic inside a privileged operation with no typed error and no receipt.

### 3.5 Defect B — the published API surface has no local-gate guard

- `crates/eggup-service/tests/published-api-0.1.2/Cargo.toml` and `crates/eggup-service/tests/published-api-0.1.3/Cargo.toml` each contain an empty `[workspace]` table, making each its **own workspace root**. Neither is a member of the root `Cargo.toml` (which lists 8 members, `crates/eggup-core` … `crates/eggup-transport-footprint`). Therefore `scripts/check-local.sh` — which runs `cargo fmt/clippy/test/doc/tree --workspace` from the root (`scripts/check-local.sh:4-8`) — **cannot reach either fixture**.
- `.github/workflows/ci.yml` runs the fixtures only in the `service-systemd` lane: `ci.yml:40-41` (0.1.2 fmt/check), `ci.yml:44` (0.1.2 test), `ci.yml:49-50` (0.1.3 fmt/check), `ci.yml:51` (0.1.3 test). Those steps require a Linux runner, `sudo`, and `EGGUP_SYSTEMD_INTEGRATION=1`.
- `crates/eggup-service/tests/api_compatibility.rs` (16 lines, one test) is the **only** in-workspace API-compat check. It defines a local helper `classify` at `:3-11` and asserts `classify(LifecycleState::Unknown) == "unknown"` at `:13-16`. It exercises the test's own match arms rather than crate behavior — but it does carry a **weak but real** signal: the helper names every `LifecycleState` variant, so removing a variant would break **compilation**, which fails `cargo test`.
- The crate's public surface is wide: a `^pub (fn|struct|enum|trait|const|static|type|mod)` count over `crates/eggup-service/src/*.rs` yields **68** top-level public items (53 types, 11 free functions, 4 constants), plus 3 `pub use` re-exports at `crates/eggup-service/src/lib.rs`.
- Packaging **is** covered: `cargo package -p eggup-service --locked` runs in `stable` (`ci.yml:23`), `msrv` (`ci.yml:32`), `macos` (`ci.yml:62`), and `windows-check` (`ci.yml:84`). **API-surface drift is not.**

**Consequence.** A developer on macOS or Windows gets no API-compat signal from `./scripts/check-local.sh` at all. The only guard on the published surface of a published crate is a lane that a contributor without a Linux root runner never sees.

### 3.6 Defect C — whole-enum `#[allow(dead_code)]`

`windows_scm.rs:719-730`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Several SCM states are only constructible on Windows, but tested cross-platform.
enum ScmState {
    Stopped, StartPending, StopPending, Running,
    ContinuePending, PausePending, Paused, Unknown,
}
```

`windows_scm.rs:740-746`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Production variants are constructed by the Windows backend.
enum ScmLookup {
    Absent,
    MarkedForDelete,
    Present(ScmRecord),
}
```

The attribute is **unconditional and enum-wide**. On a non-Windows build it suppresses dead-code diagnostics for every variant of both enums, including any variant that is unreachable on *every* platform. The comment describes a per-platform condition that the attribute does not express.

The reachable-variant map is known by reading: `ScmState::ContinuePending` and `ScmState::PausePending` are constructed only in the `#[cfg(windows)]` `WindowsBackend::query` arm at `:997-1002`; on a non-Windows build they are produced only by the test helper `record` (`:1189`). `ScmLookup::MarkedForDelete` is produced only by the `#[cfg(windows)]` arm at `:976`.

### 3.7 Defect D — zero `# Errors` on the SCM surface

Across `crates/eggup-service/src/lib.rs`, `disposition.rs`, `lifecycle_update.rs`, and `windows_scm.rs`: zero `/// # Errors` sections, zero `/// # Panics` sections, zero doc code fences. The crate is `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]` and exposes many public `Result`-returning items, including `WindowsScmManager::new` (`:240`), `WindowsScmInstall::with_transition_timeout` (`:107`), and the `ServiceManager` trait methods. `clippy::missing_errors_doc` is in `clippy::pedantic`, not `clippy::all`, so `cargo clippy -- -D warnings` does not catch it.

**Scope boundary.** The crate-wide `# Errors` sweep and the typed manager failure taxonomy are owned by milestone `013` (`plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`). M014's `# Errors` scope is **limited to the Windows SCM public surface** (§6.4). The two milestones must not edit the same doc lines.

## 4. Invariants that must not regress

- The `ServiceManager` trait (`crates/eggup-service/src/lib.rs:364-398`) stays **manager-neutral**. A test backend or a fixture is an implementation detail; the trait's method set, signatures, and contract are unchanged.
- The Windows SCM adapter stays **manager-neutral at the trait boundary**. The `ScmBackend` seam (`:748`) stays crate-private; no SCM-specific type is added to any public trait.
- Ownership remains **fail-closed**: `Foreign` and `Unknown` never permit a mutation; `require_owned` remains the single gate (`:327-339`).
- The SCM error mapping stays **lossless**: `windows_scm.rs:936-941` must continue to map `1060` → `Absent` and `1072` → `MarkedForDelete`, and the raw OS code must remain reachable (this is M013's invariant too — M014 must not consume it).
- `ScmState` and `ScmLookup` keep their **exact variant sets** (`:721-730`, `:742-746`); no variant is added, removed, or renamed.
- `LifecycleState` and its four variants are unchanged.
- **No automatic privilege elevation** is introduced anywhere, and the fixture must not introduce one either.
- The `windows-service` dependency stays `=0.8.1` and stays Windows-target-only (`crates/eggup-service/Cargo.toml:28-29`); `windows-args` stays `=0.2.0` (`:27`).
- `eggup-service` gains **no dependency** of any kind, including dev-dependencies.
- `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` stay on; the workspace's `unsafe_code = "deny"` and `clippy all = warn` lints are unchanged.
- The bounded-poll discipline of `wait_for_state` (`:340-365`, `DEFAULT_POLL_INTERVAL` 50 ms at `:15`, `MAX_POLL_INTERVAL` 500 ms at `:16`) is preserved: one monotonic `OperationDeadline` per transition, no unbounded wait.
- No live destination parent is created; the service crate makes no artifact-level decision.

## 5. Scope

### In scope

- A hosted Windows fixture that registers a **disposable** service through `WindowsScmManager` and drives the full observable lifecycle against the real SCM: install → inspect → ownership → start → stop → uninstall.
- Conversion of the five `unreachable!()` sites (`:977`, `:1051`, `:1077`, `:1089`, `:1100`) into typed errors.
- A dependency-free, network-free, root-free **public-API surface guard** that runs inside the root workspace and fails `cargo test --workspace` when a pinned public item is removed or its signature changes.
- Per-variant dead-code gating for `ScmState` and `ScmLookup`.
- `# Errors` rustdoc for the Windows SCM public surface only.
- Crate `README.md`, crate `CHANGELOG.md` `Unreleased`, and `architecture/service-lifecycle.md` updates, plus the new fixtures.

### Explicitly out of scope

- **No commit-engine consolidation.** That is milestone `012` (`plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`). M014 does not touch the commit/rollback engine, `disposition.rs`, or `lifecycle_update.rs` semantics.
- **No manager failure taxonomy and no crate-wide `# Errors` sweep.** That is milestone `013`. M014 adds `# Errors` only to the Windows SCM public surface and must not edit a doc line M013 owns.
- **No new dependencies.** No `winapi`, no `windows-sys`, no `serde`/`serde_json`, no `cargo-public-api`, no dev-dependency of any kind. If a snapshot mechanism would require one, the plan requires a dependency-free alternative (§6.2) or deferral (§14).
- **No republication.** `eggup-service 0.1.3` is published (2026-10-09, source `feb6ae5`, checksum `9f7f7ea8…32124dc`). M014 must not publish, must not run `cargo publish`, and must not move the `v0.1.3` tag.
- **No change to the `ServiceManager` trait contract** — no new method, no changed signature, no manager-specific type in the trait.
- **No change to the SCM command surface.** No `sc.exe`, no shell, no ambient `PATH`, no change to the argv the adapter hands to `windows-service`.
- **No change to the `ScmState` / `ScmLookup` variant sets.**
- **No change to the `windows-service` version.** It stays `=0.8.1`.
- **No removal or modification of the two published-API fixtures.** `crates/eggup-service/tests/published-api-0.1.2/` and `.../published-api-0.1.3/` stay as they are, still CI-only. M014 adds a **local** guard; it does not replace the registry-only fixtures.
- **No change to `eggup-core` or any other crate's production code.**
- **No change to the `ServiceError` taxonomy shape.** M014 uses the existing `ServiceError::manager` / `ServiceError::conflict` constructors. M013 owns the retyping.

## 6. Required production changes

Requirements are observable outcomes, not an implementation sketch.

### 6.1 Defect A — the Windows SCM fixture and the panic sites

- **R-A1.** A Windows-only fixture registers a **disposable** service with a unique per-run name through the crate's public API (`WindowsScmInstall` + `WindowsScmManager::new`, `windows_scm.rs:240`) and asserts each observable in sequence:
  1. **install** — the service is created and is **not started** by creation (`windows_scm.rs:1011-1029` creates only);
  2. **inspect** — the registration is observed as `Present` with the exact parsed command line (`registration_ownership`, `:774-792`);
  3. **ownership** — the registration classifies as `Owned` for the exact spec and as `Foreign` for a spec naming a different executable;
  4. **start** — the service reaches `Running`;
  5. **stop** — the service reaches `Stopped`;
  6. **uninstall** — the registration is deleted and the adapter reports completion only after confirmed absence (`MarkedForDelete` → `Unknown` mapping, `:319`).
- **R-A2.** The fixture MUST assert the **`windows-service` mapping layer** explicitly, not only the end state: `start_type` (`:867-875`), `error_control` (`:877-886`), `service_dependencies` (`:888-903`), and the `ServiceInfo` round-trip (`:946-964` → `:985`) — i.e. the configuration written at install is read back at inspect and is equal on the mapped fields.
- **R-A3.** The fixture MUST be **disposable and self-cleaning**: it MUST uninstall the service it created on every exit path, including a failing assertion, so a CI runner is left without an orphaned registration. It MUST NOT touch, disable, stop, or reconfigure any pre-existing service.
- **R-A4.** The fixture MUST wait on service state transitions using the crate's own **bounded** discipline: a polling loop with an explicit deadline, never a fixed sleep asserted as a fact. The repository has a documented precedent against scheduler-sensitive wall-clock assertions (`plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`; the same defect class in this crate is `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`). A fixture that must wait MUST poll to a target state under an explicit deadline and MUST fail with the observed state on timeout; it MUST NOT assert "the service started within N milliseconds".
- **R-A5.** The fixture MUST be **skippable with a recorded reason, never a silent pass.** When the process cannot obtain the administrator rights a real SCM registration requires, the fixture MUST record the skip **and its reason** in the test output and in the closure, and the closure MUST record the Windows lane's real result. A lane not run is **not run**, never inferred from the Linux or macOS lanes.
- **R-A6.** **The five `unreachable!()` sites (`:977`, `:1051`, `:1077`, `:1089`, `:1100`) MUST become typed errors.** A `windows-service` behavior change — a new error code, a changed error variant, an extended `Self::open` contract — MUST degrade to a typed `Err` with a bounded detail, never a panic, on every path that holds a privileged SCM handle. This is required, not optional: these are the only panic sites in the workspace's only privileged-OS code path.
- **R-A7.** The typed error MUST be built from the crate's **existing** constructors (`ServiceError::manager`, `ServiceError::conflict`) and MUST NOT introduce a new error variant, a new error type, or a new taxonomy. The `1060`/`1072` mapping at `:936-941` MUST remain lossless and MUST remain the source of the `Absent` / `MarkedForDelete` distinction.
- **R-A8.** The `ScmBackend` seam (`:748`) MUST remain crate-private and MUST NOT gain a public method. The fixture MUST reach the real backend through `WindowsScmManager::new` (`:240`), i.e. through the same public path a Windows consumer uses — not through a new crate-internal back door.

### 6.2 Defect B — the local-gate API-surface guard

- **R-B1.** The required observable property: **removing a public item of `eggup-service`, or changing the signature of a pinned public item, MUST fail a gate a developer runs locally** — specifically, it MUST fail `cargo test --workspace --all-targets --all-features --locked`, which `scripts/check-local.sh:6` already runs.
- **R-B2.** The guard MUST be **dependency-free, network-free, and root-free**. It MUST run inside the root workspace. It MUST NOT require the two `published-api-*` fixtures, a registry fetch, or administrator rights.
- **R-B3.** The enforcement mechanism MUST be a **compile-only signature-pinning test** in `crates/eggup-service/tests/`: for each pinned public item, the test binds the item to an explicit type (a `const _: fn(..) -> .. = ..;` binding or an equivalent explicit typed binding), so a removed item or a changed signature is a **compilation error**, which fails `cargo test`. A runtime assertion over a hand-written list is **not** sufficient on its own: it can be edited to agree with a broken crate. The compile-time pinning is what makes the guard unforgeable without a reviewer noticing.
- **R-B4.** A **committed plain-text snapshot** of the public surface (e.g. `crates/eggup-service/tests/public-api-surface.txt`) MUST exist alongside the pinning test, listing every pinned public item with its kind and, where meaningful, its signature. It is the **reviewable artifact**: a change to the published surface shows up as a diff in it.
- **R-B5.** The snapshot and the pinning test MUST cover **every** public item of `eggup-service` — all 68 top-level public items plus the 3 `pub use` re-exports at `crates/eggup-service/src/lib.rs`. The closure MUST record the exact enumeration command used to confirm coverage.
- **R-B6.** The existing `crates/eggup-service/tests/api_compatibility.rs` MUST be **retained and folded into** the guard: its `classify` helper (`:3-11`) already pins the `LifecycleState` variants by name and is a genuine compile-time signal. It MUST NOT be deleted as redundant.
- **R-B7.** **The update process must be explicit and reviewed.** Any change to the snapshot or the pinning test MUST be a deliberate, separately-reviewed edit that states, in the closure, which public item changed and why — because the guard's purpose is to make an intentional API change *visible*, not to make it impossible. A snapshot edit that is not accompanied by a stated API change is a defect, and the closure MUST treat it as one.
- **R-B8.** The snapshot MUST be regenerable **without** a new dependency. If an automatic regeneration cannot be produced from the crate's own build output with the existing toolchain, the pinning test is the enforcement and the snapshot is the review artifact; the closure MUST record that the snapshot was verified by the enumeration command in R-B5 and by the pinning test compiling.

### 6.3 Defect C — per-variant dead-code gating

- **R-C1.** The enum-wide `#[allow(dead_code)]` at `:720` and `:741` MUST be **removed**. The allow MUST be expressed **per variant** — e.g. `#[cfg_attr(not(windows), allow(dead_code))]` on the individual variants whose construction is Windows-only — so a variant that is genuinely unreachable on **every** platform is still reported by the compiler.
- **R-C2.** The per-variant gating MUST be **verified**, not assumed. The verification is: (a) `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` stays green on a non-Windows host, which proves the Windows-only variants are correctly allowed rather than accidentally reachable; and (b) a **negative check** proves the allow is still effective — temporarily deleting a `#[cfg_attr]` from a variant that is genuinely Windows-only MUST produce a dead-code diagnostic on a non-Windows build. The negative check's exact command and exact diagnostic output MUST be recorded in the closure. If the negative check cannot be made to fail for a given variant, that variant is NOT Windows-only and its allow MUST be removed.
- **R-C3.** The `ScmState` and `ScmLookup` variant sets MUST be unchanged (`:721-730`, `:742-746`). Gating changes which variants are *reported*, never which variants *exist*.
- **R-C4.** The existing comment (`:720`, `:741`) MUST be corrected to state the actual per-platform condition it describes.

### 6.4 Defect D — `# Errors` on the Windows SCM public surface only

- **R-D1.** `# Errors` rustdoc MUST be added to the **Windows SCM public surface only**: `WindowsScmManager::new` (`:240`), `WindowsScmInstall` and its validating builders (`:70`, `:97`, `:107`), `WindowsScmScope`/`WindowsStartType`/`WindowsErrorControl`/`WindowsServiceDependency` (`:21`, `:32`, `:45`, `:58`), and the SCM error mapping described at `windows_scm.rs:911-917` and `:215-224`.
- **R-D2.** Each `# Errors` section MUST name the concrete conditions, including the access-denied case (SCM code `5` → `scm_error_detail`, `:215-218`) and the disappeared-before-operation case (`:1049`, `:1076`, `:1088`, `:1099`).
- **R-D3.** **The crate-wide sweep belongs to M013.** M014 MUST NOT add `# Errors` to a `lib.rs`, `disposition.rs`, or `lifecycle_update.rs` item, and MUST NOT edit a doc line M013 owns. M013 has reciprocally excluded `windows_scm.rs` from its own §6.5 required set (`plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`), so the two surfaces are disjoint: M014 owns `windows_scm.rs`, M013 owns the other three files. Where the `ServiceManager` trait's own `# Errors` section (`lib.rs:364-398`) must state which failure kinds each adapter can produce, M013 owns that trait method and M014 does not touch it — the SCM adapter's contribution to it is recorded in M014's closure instead.
- **R-D4.** M014 MUST NOT add a `# Panics` section that promises the absence of panics at the five sites, because R-A6 converts them to typed errors — the promise is the outcome of R-A6, not an independent claim.

## 7. Ordered work packages

Each work package is independently verifiable and leaves the tree green.

- **WP1 — Typed-error conversion for the five panic sites.** Apply R-A6 and R-A7 at `:977`, `:1051`, `:1077`, `:1089`, `:1100`. Use the existing `ServiceError` constructors; keep the `1060`/`1072` mapping at `:936-941` lossless. *Verify:* `cargo test -p eggup-service --locked` green; `grep -n "unreachable!" crates/eggup-service/src/windows_scm.rs` returns zero hits; the §10 item 3 fixture returns a typed error where the baseline panicked.
- **WP2 — The Windows end-to-end SCM fixture.** Apply R-A1…R-A5 and R-A8: a Windows-only integration test under `crates/eggup-service/tests/` that registers a disposable service through the public API and asserts install → inspect → ownership → start → stop → uninstall, the `windows-service` mapping round-trip, bounded polling, self-cleanup, and a recorded skip reason when administrator rights are unavailable. *Verify:* the fixture is **not run** on macOS/Linux (recorded as not run); it is run in the `windows-check` lane with real output recorded.
- **WP3 — The local-gate API-surface guard.** Apply R-B1…R-B8: the compile-only signature-pinning test plus the committed plain-text snapshot, folded together with the existing `api_compatibility.rs`. *Verify:* `cargo test --workspace --all-targets --all-features --locked` is green at the implementation SHA, and **fails** against the baseline worktree (§11) when the pinning test is copied in unchanged.
- **WP4 — Per-variant dead-code gating.** Apply R-C1…R-C4. *Verify:* the non-Windows clippy run is green and the R-C2 negative check produces the expected dead-code diagnostic, with exact output recorded.
- **WP5 — SCM-surface `# Errors` docs.** Apply R-D1…R-D4. *Verify:* `cargo doc -p eggup-service --no-deps --locked` is clean and the required set is present; a check that no `lib.rs`/`disposition.rs`/`lifecycle_update.rs` doc line was added by this milestone.
- **WP6 — Tests and changelog.** Add every fixture listed in §10, each proved to fail against the pre-M014 baseline where the platform permits (§11); update `crates/eggup-service/CHANGELOG.md` `Unreleased` and `crates/eggup-service/README.md` (§12).

Order: WP1 → WP2 → WP3 → WP4 → WP5 → WP6. WP3, WP4 and WP5 are independent of WP1/WP2 and may be developed in any order, but all precede WP6.

## 8. Failure, cancellation, restart, and contention semantics

- **Administrator rights unavailable.** The fixture MUST record **"not run" with a reason** and MUST NOT report a pass. It MUST NOT attempt an automatic elevation, a UAC prompt, or a `runas` invocation. The closure MUST record the Windows lane's real result: a fixture that skipped is a **not run**, and a closure that does not contain a **real Windows run** is **not a closure** of this milestone (§15).
- **Fixture assertion fails mid-sequence.** The fixture MUST still attempt uninstall of the service it created (R-A3) before propagating the failure, so a failed run does not leave an orphaned registration that makes the next run fail for the wrong reason.
- **The SCM returns an unmapped code.** A code that is neither `1060` nor `1072` MUST be surfaced as a typed error carrying the raw code (lossless, R-A7) — never as `Absent`, never as `MarkedForDelete`, never as a panic. The `1060`/`1072` arms at `:936-941` are the **only** codes that may take the `Absent` / `MarkedForDelete` paths.
- **A `windows-service` behavior change makes `Self::open` return a third code.** This MUST degrade to a typed `Err` at every one of the five sites (R-A6), not a panic, even though the arm is unreachable by construction today.
- **The service is in a pending state.** `ScmState::StartPending` / `StopPending` / `ContinuePending` / `PausePending` map to `LifecycleState::Transitioning` (`:794-804`). The fixture MUST treat a pending state as **not yet converged** and MUST keep polling to its explicit deadline (R-A4); it MUST NOT assert a transition on a pending observation.
- **The registration is marked for delete.** `ScmLookup::MarkedForDelete` → `Ownership::Unknown` (`:319`). The fixture MUST observe that the adapter reports an **incomplete** uninstall while the entry is marked for delete, and MUST NOT assert completion on that observation.
- **Ownership is not `Owned`.** A `Foreign` or `Unknown` observation MUST deny the mutation (fail-closed). The fixture MUST include that negative control.
- **The API-surface guard detects an intentional change.** The process MUST be explicit and reviewed: the snapshot and the pinning test are edited in the same commit as the API change, the change is stated in the `CHANGELOG.md` `Unreleased` entry, and the closure names the changed items (R-B7). A snapshot edit **not** accompanied by a stated API change is treated as a defect and is recorded as such.
- **Contention.** If another process mutates the disposable registration mid-sequence, the fixture MUST fail with the observed state rather than retry indefinitely. No unbounded retry loop is permitted.
- **Cancellation.** The fixture MUST honor the crate's own transition deadline: a transition that does not converge inside its deadline is a **failure with the observed state**, never a hang. The fixture MUST NOT extend a production deadline to make the test pass.
- **The five panic sites.** Under every condition above, they degrade to typed errors (R-A6). No panic in the SCM adapter may remain.

## 9. Compatibility and migration

- **Published baseline.** `eggup-service 0.1.3` is **published** (2026-10-09, source `feb6ae5`, checksum `9f7f7ea8…32124dc`). M014 does **not** publish, does not run `cargo publish`, and does **not** move the `v0.1.3` tag. The changes land on the source line and reach consumers only through a separate, later republication milestone (§16).
- **The SCM changes are internal.** R-A1…R-A5, R-A6, and R-C1…R-C4 change **no** public API: the fixture is a test, the typed-error conversion reuses the existing published `ServiceError` constructors, and the dead-code gating changes diagnostics only. No public item is added, removed, or re-typed.
- **Exception, stated explicitly.** If the typed-error conversion in R-A6 can only be expressed by adding a public `ServiceError` variant or by re-typing a published variant, that is a **public API change** and it must be sequenced at the 0.2/1.0 boundary instead of landing here (§14). The expected outcome is no public change, because `ServiceError::manager` and `ServiceError::conflict` already exist and already carry the needed detail.
- **`# Errors` documentation is additive and non-breaking.** It changes no signature. A consumer sees no behavior difference; it gains the ability to read what the SCM surface can return.
- **The API-surface guard is additive tooling.** It adds no public item, no dependency, and no consumer-visible behavior. A consumer of `eggup-service` is unaffected. The only effect is on the **contributor**: a deliberate public-API change now requires a stated, reviewed snapshot update.
- **The two `published-api-*` fixtures are unchanged** and remain CI-only. Their continued registry-only nature is stated in the crate README (§12).
- **No migration** is required for any consumer of any published crate.

## 10. Required tests

Each new fixture MUST be proved to **fail** against the pre-M014 implementation where the platform permits (§11 gives the exact command and states what cannot be proved where).

1. **The Windows end-to-end SCM fixture** (R-A1, R-A2). Windows-only. Registers a disposable service through `WindowsScmManager::new` and asserts install → inspect → ownership → start → stop → uninstall, plus the `windows-service` mapping round-trip. At the baseline this fixture **does not exist**, so it cannot be "red"; its evidence is the **first real execution** of `WindowsBackend` and the recorded output. This is the milestone's core deliverable and is not a regression test against a known-bad implementation.
2. **The API-surface guard test** (R-B1, R-B3, R-B5). The compile-only pinning test, copied unchanged into the baseline worktree, MUST **fail to compile** there if any pinned item is absent at the baseline — or, stated in the direction that is actually provable: the pinning test MUST fail to compile in a worktree in which one pinned public item has been **removed**, which demonstrates the guard's detection power. The proof command is in §11.
3. **A fixture proving the previously-panicking SCM paths now return typed errors** (R-A6). A crate-internal unit test in `windows_scm.rs` that reaches each of the five former `unreachable!()` arms and asserts a **typed** `Err` rather than a panic. Because `Self::open` currently cannot produce a third code, the fixture MUST drive the arm through the seam's own contract — e.g. a test-visible construction of the third arm — and the mechanism used MUST be recorded. At the baseline the same input **panics**. This fixture is the red proof for WP1.
4. **A per-variant dead-code check** (R-C2). Two parts: the positive check (non-Windows clippy stays green with the per-variant allows) and the **negative check** (temporarily removing one `#[cfg_attr]` produces a dead-code diagnostic on a non-Windows build). The negative check's exact command and exact diagnostic MUST be recorded. It is a recorded observation, not a committed test, because a committed test cannot easily assert a compiler diagnostic.
5. **The existing suites stay green.** The full `eggup-service` suite, including all 15 `windows_scm.rs` tests, stays green. **No existing test may be deleted or weakened** to accommodate the change. If an existing test encoded a behavior that R-A6 deliberately changes, the closure MUST name that test and record the behavior change.
6. **The existing `api_compatibility.rs` remains** (R-B6) and is part of the guard.

**Honest statement of what can and cannot be proved where.** Items 1 and 3 (and the negative part of item 4's Windows-specific behavior) **cannot be proved red on a macOS or Linux host**, because the fixture is Windows-only and the panic sites live behind `#[cfg(windows)]`. On those hosts, the provable red proofs are item 2 (the guard's compile-time detection) and the positive half of item 4 (clippy stays green). The closure MUST record the Windows results from a real hosted run; a Windows result that was not obtained is **not run**.

## 11. Required verification commands

Exact strings, runnable from the workspace root:

```text
cargo fmt --all -- --check
cargo clippy -p eggup-service --all-targets --locked -- -D warnings
cargo test -p eggup-service --locked
cargo test -p eggup-service --locked windows_scm
cargo test -p eggup-service --locked public_api_surface
cargo doc -p eggup-service --no-deps --locked
cargo package -p eggup-service --locked
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh
```

`./scripts/check-local.sh` is the local gate: `cargo fmt --all -- --check`,
`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
`cargo test --workspace --all-targets --all-features --locked`,
`cargo doc --workspace --no-deps --locked`, plus `cargo tree --workspace --locked`
as a human review step (`scripts/check-local.sh:4-8`).

**The Windows-lane command form used by CI** (`.github/workflows/ci.yml:64-84`), which is where this milestone's evidence comes from:

```text
cargo test -p eggup-service --lib windows_scm::tests --locked
cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture
cargo test --workspace --all-targets --locked --no-fail-fast
```

The first of these is the existing CI step verbatim (`ci.yml:75`). The second is the
form the new fixture uses; the fixture's own target name MUST match the file name
under `crates/eggup-service/tests/`.

**Proving the new fixtures fail against the pre-M014 baseline**, verbatim:

```text
git worktree add ../eggup-m014-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
(cd ../eggup-m014-baseline && cargo test -p eggup-service --locked <fixture_name>)
git worktree remove ../eggup-m014-baseline
```

The baseline worktree then receives the new fixture files (`crates/eggup-service/tests/*.rs`
and, for the crate-internal fixture of §10 item 3, the edited `windows_scm.rs` test module),
and each fixture MUST be shown to **fail** — or to **panic**, for §10 item 3 — in the
baseline worktree. For the API-surface guard, the baseline worktree additionally has one
pinned public item removed, and the pinning test MUST be shown to **fail to compile**:

```text
(cd ../eggup-m014-baseline && cargo test -p eggup-service --locked public_api_surface)
```

**Commands that CANNOT run on macOS or Linux and MUST be recorded as "not run" locally:**

```text
cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture
```

The Windows-only fixture, and any hosted `windows-check` lane result, is **not run**
on a macOS or Linux host. It is **never inferred** from the Linux or macOS lanes.

Hosted qualification, run and recorded separately: `stable`, `msrv`,
`service-systemd`, `macos`, `windows-check`. A lane that was not run is recorded as
**not run**, never inferred.

## 12. Documentation updates

- **rustdoc `# Errors`** on the Windows SCM public surface only (R-D1, R-D2): `WindowsScmManager::new` (`:240`), `WindowsScmInstall` and its builders (`:70`, `:97`, `:107`), the SCM configuration enums (`:21`, `:32`, `:45`), and the SCM error mapping (`:911-917`, `:215-224`).
- **`crates/eggup-service/CHANGELOG.md`** — an `Unreleased` entry carrying the crate's standard disclaimer: the changes are **not published** by this milestone, **no consumer migration** is required, and republication is a separate milestone.
- **`crates/eggup-service/README.md`** — MUST state that the two `published-api-0.1.2` / `published-api-0.1.3` fixtures are **CI-only** (they are their own workspaces and run only in the `service-systemd` lane) and are **not** part of the local gate; and, after WP3, that a **local** public-API guard now exists inside the root workspace. The README's existing Windows-SCM bullets (the `WindowsScmManager` / `WindowsScmInstall` descriptions) MUST be reconciled with R-A1's recorded evidence — in particular, any claim that the adapter is "tested" MUST be qualified to say which parts are exercised against the real SCM.
- **`architecture/service-lifecycle.md`** — the `windows_scm.rs` section (`:165-180`), the "Platform adapters → Windows SCM" section (`:866-...`), and the "Source layout" table row for `windows_scm.rs` (`:22`) MUST reflect: the real backend has now been executed against the SCM, the five former panic sites are typed errors, the local API-surface guard exists, and the `published-api-*` fixtures remain CI-only.
- **`docs/` is non-normative.** If a consumer-facing page names the SCM surface or the published-API fixtures, it points at the contracts; it does not restate them and does not introduce a rule there.
- `plans/registry.md` § "Post-0.1.3 open findings" (entry **6**, registered at baseline `c480541` alongside this plan) and the service-lifecycle roadmap status table are updated by the **closure**, not by this plan.

## 13. Acceptance criteria

M014 closes only when all of the following are checkable:

1. A **real hosted Windows run** of the SCM fixture exists, with the recorded output showing the disposable service registered, observed, started, stopped, and uninstalled against the genuine SCM — and with the `windows-service` mapping round-trip asserted (R-A1, R-A2).
2. The fixture is **disposable and self-cleaning** (R-A3) and no pre-existing service is touched.
3. The fixture's state waits are **bounded and deadline-driven**, with no wall-clock assertion of a fixed transition duration (R-A4).
4. `grep -n "unreachable!" crates/eggup-service/src/windows_scm.rs` returns **zero** hits, and each of the five former sites returns a **typed** error instead of a panic (R-A6), with the §10 item 3 fixture red against the baseline.
5. The `windows-service` version stays `=0.8.1`, the `1060`/`1072` mapping at `:936-941` stays lossless, and `ScmState`/`ScmLookup` variant sets are unchanged.
6. Removing or changing a pinned public item **fails `cargo test --workspace --all-targets --all-features --locked`**, demonstrated against the baseline worktree (§11) (R-B1, R-B3).
7. The API-surface guard is **dependency-free, network-free, and root-free**, covers every public item (R-B5), retains `api_compatibility.rs` (R-B6), and has an explicit reviewed update process (R-B7).
8. The per-variant dead-code gating is in place, and the R-C2 **negative check's exact diagnostic output** is recorded.
9. `# Errors` rustdoc exists on the Windows SCM public surface, and **no** crate-wide doc line outside that surface was edited (R-D3).
10. `cargo test -p eggup-service --locked`, `cargo test -p eggup-service --locked windows_scm`, `cargo clippy -p eggup-service --all-targets --locked -- -D warnings`, `cargo doc -p eggup-service --no-deps --locked`, `cargo package -p eggup-service --locked`, `cargo test --workspace --all-targets --all-features --locked`, and `./scripts/check-local.sh` are green, with exact outputs recorded.
11. Hosted `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` lanes are recorded with **real** results; any lane not run is recorded as **not run**.
12. `crates/eggup-service/Cargo.toml` gains **no dependency**; `cargo tree -p eggup-service --locked` still shows only `eggup-core`, `windows-args`, and (Windows target only) `windows-service`.
13. `eggup-service 0.1.3` remains the published version; no publication occurred; the `v0.1.3` tag is unmoved.
14. **No medium-or-higher finding** remains open against the service-lifecycle subsystem as a result of this milestone.

## 14. Stop conditions

Halt and escalate rather than proceed when:

- **The Windows runner cannot obtain the administrator rights** a real SCM registration requires, and no available CI configuration provides them. In that case the milestone is **blocked** on Windows evidence, not closed; the closure records the block and the fixture's skip reason. A closure with no real Windows run is **not** a closure of this milestone.
- **The API-surface guard would require a new dependency** — `cargo-public-api`, `serde`, `serde_json`, `winapi`, `windows-sys`, or any dev-dependency. The guard MUST be implemented with the existing toolchain (R-B3, R-B8) or **deferred** with a recorded finding; it MUST NOT be bought with a dependency.
- **The typed-error conversion would require a breaking public API change** — adding a `ServiceError` variant, re-typing a published variant, or otherwise altering the published error surface. That change must instead be sequenced at the 0.2/1.0 boundary (with M013's taxonomy work) and M014 stops before it.
- **The real SCM fixture would require touching a pre-existing service**, a host-global service-manager configuration, an automatic elevation, or `sc.exe`/shell invocation. The fixture must operate only on a disposable registration it creates.
- **The fixture cannot be made deterministic** — i.e. it can only pass by asserting a wall-clock transition duration. It MUST then use the bounded-poll discipline of R-A4 or be withdrawn; it must not assert a fixed duration.
- **Any change requires a new dependency**, a transport/archive/Eggpack dependency, or a change to the `windows-service` version.
- **Any change alters the `ServiceManager` trait contract**, the `ScmBackend` seam's visibility, the SCM argv/command surface, the `ScmState`/`ScmLookup` variant sets, or the `1060`/`1072` mapping.
- **The per-variant dead-code gating cannot be verified** — i.e. the R-C2 negative check cannot be made to fail for the Windows-only variants. In that case those variants are not Windows-only and the allow MUST be removed instead; if the resulting diagnostic cannot be resolved without changing behavior, stop.
- **M014 and M013 are scheduled to edit the same doc lines.** Stop and sequence; M014's `# Errors` scope is the SCM surface only (§6.4, R-D3).
- A hosted lane fails for a reason attributable to this milestone.

## 15. Closure evidence required

`plans/closure/service-lifecycle/014-status.md` must contain:

- **Status**: closed / conditionally closed / corrective pass required / blocked; source plan and roadmap; reviewed repository baseline; implementation commits.
- **Executive finding**: that the real SCM backend had never executed, that it now has, and what the first execution showed.
- **Requirement→evidence matrix**: one row per requirement R-A1…R-A8, R-B1…R-B8, R-C1…R-C4, R-D1…R-D4, each with the exact command and the exact observed output that satisfies it.
- **Production implementation evidence**: `file:line` for each changed site — the five former panic sites now typed, the per-variant `cfg_attr` placement, the `# Errors` sections added, the guard test and snapshot file.
- **The Windows fixture's recorded output**: the hosted run identifier, the runner image, the exact test output showing the disposable service's install → inspect → ownership → start → stop → uninstall sequence, the mapping round-trip assertions, and the self-cleanup result. If the fixture was **skipped**, the skip and its reason — a skipped fixture is **not run**, and the milestone cannot close on it.
- **Baseline-failure evidence**: per fixture, the exact baseline worktree SHA, the exact command, and the exact observed failure or panic output. A fixture that does not fail at the baseline is not evidence and must be corrected or withdrawn. For the Windows-only fixture, the closure states explicitly that no baseline-red proof is possible off Windows and that the Windows lane's first execution is the evidence.
- **API-surface guard evidence**: the enumeration command used to confirm coverage of every public item, the baseline-worktree compile-failure output proving detection, and the stated API change for any snapshot edit.
- **Dead-code gating evidence**: the positive clippy result and the negative check's exact diagnostic output.
- **Exact commands and results**, including §11's list, with real output.
- **Separate platform results**: Linux, macOS, and Windows reported **individually**. A lane that was not run is recorded as **not run**, never inferred from another platform. **A closure without a real Windows run is not a closure of this milestone** — this is the whole point of M014.
- **Invariant review**: §4's invariants, each marked preserved / changed / violated, with evidence.
- **Failure, rollback, and recovery review**: §8's semantics, each with the fixture or observation that proves it, including the skip-with-reason path and the unmapped-code path.
- **Compatibility and migration review**: §9's items, including the explicit statement that the SCM changes are internal and that the guard is additive tooling.
- **Security review**: the fail-closed ownership direction, the `Owned`-never-inferred rule, the absence of automatic elevation (in the adapter **and** in the fixture), the unchanged allowlist/environment/output bounds, and the confirmation that no panic path remains in the SCM adapter.
- **Documentation/operations evidence**: the rustdoc, `CHANGELOG.md` `Unreleased`, `README.md` (including the CI-only statement about the published-API fixtures), and `architecture/service-lifecycle.md` updates.
- **Unresolved findings with severity**: each with an explicit severity and reason, including any residual Windows-only evidence gap.
- **Roadmap disposition** and **registry updates**, including the new numbered entry **5** in `plans/registry.md` § "Post-0.1.3 open findings".

## 16. Handoff notes

- **Sequencing relative to `012` and `013`.**
  - Milestone `012` (commit-engine deduplication and ownership-type unification) is **independent** of M014: M014 touches no commit-engine code, no `Ownership` type, and no `CommandOutput` type. Either may land first. If `012` lands first, M014's references to `Ownership` must be re-read against the unified type.
  - Milestone `013` (manager failure taxonomy, unified deadline, crate-wide `# Errors`) **overlaps M014 at exactly two points**: the SCM error taxonomy and the `# Errors` doc surface. The boundary is fixed by §6.4/R-D3 (SCM surface only) and §5 (M014 uses the existing `ServiceError` constructors and introduces no taxonomy). **If M013 lands first**, M014 must use M013's typed kind for the five former panic sites instead of `ServiceError::manager`, and the closure must record that substitution. **If M014 lands first**, M013 must mechanically migrate M014's SCM error sites when it retypes `ServiceError::Manager` — M014's closure MUST list those sites by `file:line` so the migration is bounded.
- **What a separate republication milestone would require.** M014 lands on the source line only. A republication milestone would need to: bump `eggup-service` past `0.1.3`, re-run the published-API fixtures against the new version, re-tag or add a tag, and decide the version boundary (`0.1.4` vs `0.2.0`) in light of M013's error-shape change and the `CleanupDisposition` / `Error`-extensibility finding already recorded at `plans/registry.md` § "Post-0.1.3 open findings" item 2. **M014 does not authorize any of that.**
- **What this plan does NOT authorize.** Publication of `eggup-service` or any other crate; moving the `v0.1.3` tag; a new dependency of any kind, including dev-dependencies; a `ServiceError` taxonomy change; a change to the `ServiceManager` trait contract; a change to the `ScmBackend` seam's visibility; a change to the SCM command surface; a change to the `ScmState`/`ScmLookup` variant sets; a change to the `windows-service` version; removal or modification of the `published-api-0.1.2` / `published-api-0.1.3` fixtures; a crate-wide `# Errors` sweep; commit-engine consolidation; or any change to `eggup-core` production code.
- **Reuse warning for the next author.** The `ScmBackend` seam is genuinely useful and must not be widened. The five former panic sites asserted an invariant that lived in a different function across a dependency boundary; if a later change adds a third code to `Self::open`, the typed errors from WP1 must absorb it. Do not reintroduce an `unreachable!` on any code path.
