# Closure record — service-lifecycle M014

## Status

**Implemented, not closed.** Every production change, fixture, guard, and
documentation update this milestone requires is in the tree, and every
requirement is proven on the native macOS lane except the ones that can only
be proven by a **real Windows run of the native SCM backend**. That run has
**not** happened: the hosted Windows lane is run by the parent session, and
this record does not claim, infer, or substitute for it.

The milestone therefore stays open until the parent appends the hosted
`windows-check` result for `cargo test -p eggup-service --test windows_scm_live
--locked -- --nocapture`. Until then the R-A5 evidence is **not run**.

## Source plan and roadmap

- Source plan: `plans/implementation/service-lifecycle/014-windows-scm-execution-and-api-surface-corrective.md`
- Roadmap: `plans/subsystems/service-lifecycle-roadmap.md` (service-lifecycle subsystem)
- Predecessor in the same crate: service milestone **M013**
  (`plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`),
  already landed in this working tree. M014 was implemented **on top of** M013's
  typed `ManagerFailureKind` taxonomy and its crate-wide `# Errors` / `# Panics`
  sweep; M013 deliberately left `windows_scm.rs` untouched so that M014 could
  author it. Nothing in M012 or M013 was reverted or re-derived.
- M013 explicitly reserved the Windows SCM public surface for M014; this
  milestone's `windows_scm.rs` work is the reserved work.

## Reviewed repository baseline

- HEAD at the time of implementation: `c4805411f74b85158e0ae5d0a219a32c4209d98e`
  on branch `plans/service-m010-failed-unit-recovery`.
- The reviewed baseline for the failure proofs is that same commit, taken as a
  detached `git worktree` at `/tmp/eggup-m014-baseline-4108` (see §"Baseline
  failure proofs"). The worktree was removed afterwards; the main working
  tree's git state was not mutated.
- The working tree was **not** clean at implementation time: it already carried
  the uncommitted M012 and M013 service work, core M014/M015, archive M005,
  and modified `plans/` files from other agents. The baseline proof is
  therefore stated against `c4805411` **plus** those uncommitted changes, and
  the reason is recorded below.

## Implementation commits / PRs

None. The working tree is uncommitted; the parent session owns the commit and
the branch. No tag was moved and nothing was published.

## Executive finding

The native Windows SCM backend had **never executed on any platform**. Every
test of `windows_scm.rs` drove a fake backend, so the half of the adapter that
crosses the OS boundary — the `windows-service` mapping layer and the SCM
calls themselves — was unproven. Two of the five ways that backend could
report a failure were `unreachable!()` panics, reachable only if the SCM
returned a code the adapter had not anticipated; the adapter's own type made
that unreachable, which is exactly why the panic was latent rather than
absent. This milestone replaces those five sites with typed errors built from
the crate's **existing** constructors, adds a Windows-only fixture that drives
the real SCM through a disposable registration, adds a local compile-only
public-API surface guard with a committed review artifact, and replaces the
two enum-wide `#[allow(dead_code)]` attributes with a per-variant allowance
whose necessity is demonstrated by a negative check.

The `1060` / `1072` mapping is unchanged and lossless. The `ScmState` and
`ScmLookup` variant sets are unchanged. No public item was added, removed, or
re-typed. No dependency was added, of any kind, including dev-dependencies.

What remains unproven is the one thing this milestone exists to prove: that the
real backend behaves as the adapter claims on a real Windows Service Control
Manager.

## Requirement → evidence matrix

| Requirement | Proof | Command / observation | Result |
|---|---|---|---|
| **R-A1** — a Windows-only fixture registers a disposable service through the **public** API and asserts install → inspect → ownership → start → stop → uninstall | `crates/eggup-service/tests/windows_scm_live.rs` — uses `WindowsScmInstall` + `WindowsScmManager::new` only; no crate-internal back door (`WindowsScmManager::with_backend` is `#[cfg(test)]` and is not reachable from an integration test) | `cargo check --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service` | **Compiles.** The behavioural run is **not run** — hosted Windows run pending from the parent |
| **R-A2** — the `windows-service` mapping layer asserted explicitly: start type, error control, service dependencies, `ServiceInfo` round-trip | `windows_scm_live.rs` asserts `config.start_type == ServiceStartType::OnDemand`, `config.error_control == ServiceErrorControl::Severe`, `config.service_type == ServiceType::OWN_PROCESS`, `config.dependencies == vec![ServiceDependency::Service(<own disposable registration>)]`, and that the SCM's stored command line parses back to the spec's exact executable and arguments | Same as R-A1 | **Compiles; behavioural assertions pending the hosted run** |
| **R-A3** — disposable, self-cleaning on every exit path including a failing assertion; no pre-existing service touched | `Disposable` guard with a `Drop` impl in `windows_scm_live.rs`; the dependency target is a **second disposable registration of the fixture's own**, so no pre-existing service is ever named; `remove_registration` is bounded and reports the observed state | Same as R-A1 | **Compiles; the cleanup path executes only on Windows** |
| **R-A4** — bounded polling under an explicit deadline; never "started within N ms" | `poll_until(deadline, probe)` in `windows_scm_live.rs` — an explicit `Instant` deadline, a 50 ms poll interval, and a timeout error that **names the last observed state**. The adapter's own transitions use the crate's `OperationDeadline`. No `sleep`-then-assert appears anywhere in the fixture | `grep -n "sleep" crates/eggup-service/tests/windows_scm_live.rs` → 1 hit, inside `poll_until`'s bounded back-off | **Verified by inspection + compile** |
| **R-A5** — skippable with a **recorded reason** when administrator rights are unavailable; never a silent pass | The fixture probes `ServiceManager::local_computer(None, CREATE_SERVICE)`; on failure it writes `EGGUP-SCM-LIVE-SKIP: …` **directly to the process stdout**, bypassing `libtest`'s output capture so the reason survives a passing run, and returns | `record()` in `windows_scm_live.rs` | **Verified by inspection + compile**; the skip path itself executes only on Windows |
| **R-A6** — the five former panic sites become typed errors built from the **existing** constructors; no new error variant and no new taxonomy | `OpenFailure` (`windows_scm.rs:299`) + `classify_open_code` (`:315`) + `unmapped_open_error` (`:336`) + `open_failure` (`:357`); the sites use `ServiceError::manager_with_os_code` / `ServiceError::conflict` and the existing `ManagerFailureKind` set | `grep -n "unreachable!" crates/eggup-service/src/windows_scm.rs` | **ZERO hits** |
| **R-A7** — the `1060` / `1072` mapping unchanged and lossless | `classify_open_code` maps `1060 → NotFound`, `1072 → MarkedForDelete`; `query` maps `NotFound → ScmLookup::Absent` and `MarkedForDelete → ScmLookup::MarkedForDelete`; the four mutating sites map both to the "service disappeared before \<operation\>" conflict. Every other code is carried verbatim in `OpenFailure::Other(Option<i32>)` | `cargo test -p eggup-service --locked --offline former_panic` → 1 passed | **Verified** |
| **R-A8** — no new dependency, no new public API, no `unsafe` | `Cargo.toml` untouched; `windows_scm.rs` adds only crate-private items; the fixture contains no `unsafe` (the workspace denies `unsafe_code` for every target) | `git status --short -- '**/Cargo.toml'` → no change; `cargo clippy --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service -- -D warnings` | **Verified** |
| **R-B1** — a dependency-free, network-free, root-free public-API surface guard inside the root workspace | `crates/eggup-service/tests/public_api_surface.rs` — uses only `eggup-service`, `eggup-core` (an existing dependency) and `std`; reads the crate's own sources through `include_str!`; writes nothing | `cargo test -p eggup-service --locked --offline --test public_api_surface` | **5 passed** |
| **R-B2** — a compile-only signature-pinning test with explicit typed bindings | Every public item is bound to an explicit type (`const _: fn(..) -> .. = ..;` for functions and methods; exhaustive or wildcard matches for enums; field-destructuring closures for structs). A removed item or a changed signature is a **compilation** error | Removal probe, below | **Verified — the guard fails to compile** |
| **R-B3** — the pinning test covers **every** public item | 75 top-level public items + 5 `pub use` re-exports, all pinned. The coverage assertion compares the crate's own sources with the pinned counts, so an unpinned new public item fails the guard | `cargo test -p eggup-service --locked --offline --test public_api_surface` → `every_top_level_public_item_is_pinned`, `every_public_reexport_is_pinned`, `the_committed_snapshot_lists_every_public_item` | **3 passed** |
| **R-B4** — a committed plain-text snapshot of the public surface | `crates/eggup-service/tests/public-api-surface.txt` — 75 top-level items with kind and location, the 5 re-exports, the public methods of the public types, and the enum variant sets | `the_committed_snapshot_lists_every_public_item` | **Passed** |
| **R-B5** — the enumeration command is recorded and covers every public item | Recorded in §"Public-surface enumeration" below and in the snapshot's own header | `grep -n "^pub \(fn\|struct\|enum\|trait\|const\|static\|type\|mod\)" crates/eggup-service/src/*.rs \| wc -l` → **75** | **Verified** |
| **R-B6** — the existing `api_compatibility.rs` is **retained** and folded into the guard | The file is retained as a test target of its own **and** compiled as a module of the guard (`#[path = "api_compatibility.rs"] mod api_compatibility;`), so its `LifecycleState` variant-pinning signal is enforced by both. Its `classify` helper was made `pub` so the fold is a real fold, not a copy | `cargo test -p eggup-service --locked --offline --test public_api_surface` → `api_compatibility::closed_lifecycle_state_match_remains_exhaustive` also runs **in the guard target** | **Verified** |
| **R-B7** — the snapshot and the pinning test change only through a deliberate, separately reviewed edit | The snapshot's header states the update process and names the enumeration command; the guard's coverage assertion makes an unpinned public item a **test failure** with an explicit message | Unpinned-item probe, below | **Verified — the guard fails** |
| **R-B8** — no new dependency; the snapshot is hand-maintained because automatic regeneration is not possible with the existing toolchain | The only tool that emits a machine-readable public surface from Rust sources is `cargo rustdoc -- -Z unstable-options --output-format json`, which needs a **nightly** compiler and is not part of the stable/MSRV lanes this workspace gates on. The snapshot is therefore verified by the enumeration command and by the pinning test compiling, and the pinning test is the enforcement | `git status --short -- '**/Cargo.toml'` → no change | **Recorded in the snapshot header and below** |
| **R-C1** — the enum-wide `#[allow(dead_code)]` attributes are removed; the allow is per variant | `ScmState` (`windows_scm.rs:893`) and `ScmLookup` (`:940`) carry per-variant `#[cfg_attr(not(windows), allow(dead_code))]`; the enum-wide attributes are gone | `grep -n "allow(dead_code)" crates/eggup-service/src/windows_scm.rs` → 8 × `cfg_attr` + 1 × unconditional (on `ScmState::Unknown`, see below) | **Verified** |
| **R-C2** — the negative check: temporarily removing a `#[cfg_attr]` produces a dead-code diagnostic on a non-Windows build; a variant for which the check cannot fail is NOT Windows-only and its allow MUST be removed | Two negative checks were run and their **exact output** is recorded in §"R-C2 negative check" below. The check **can** fail for every variant that keeps an allowance, so no allowance had to be removed. `ScmState::Unknown` is the one variant no `windows-service` service state maps to; a probe confirmed it is dead on **Windows** as well, so its allowance is unconditional and is reported as such | `cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings` with the attribute removed | **Verified — a real dead-code diagnostic** |
| **R-C3** — the variant sets must not change | `ScmState` keeps `Stopped, StartPending, StopPending, Running, ContinuePending, PausePending, Paused, Unknown`; `ScmLookup` keeps `Absent, MarkedForDelete, Present(ScmRecord)` | `git diff` on the two enums | **Unchanged** |
| **R-C4** — the comments are corrected to state the actual per-platform condition | The enum doc comments now state that every variant is constructed by the `#[cfg(windows)]` backend (and by the test helpers in test builds), and that on a non-Windows build none is constructed, so each carries its own allowance | `windows_scm.rs:888-899`, `:935-941` | **Verified** |
| **R-D1** — `# Errors` rustdoc on the Windows SCM public surface only | `WindowsScmManager` (type and `new`), `WindowsScmInstall` (type and its three validating builders), `WindowsStartType`, `WindowsErrorControl`, `WindowsServiceDependency`, `scm_error_detail` and `unmapped_open_error` / `open_failure` | `cargo doc -p eggup-service --no-deps --locked --offline` → no warnings | **Verified** |
| **R-D2** — each `# Errors` section names concrete conditions, including SCM code `5` (access denied) and the "disappeared before operation" case | Every section names code `5` → `ManagerFailureKind::Permission`, the disappeared case → `ServiceError::Conflict`, and the raw-code-carrying `ServiceError::Manager` | `cargo doc -p eggup-service --no-deps --locked --offline` | **Verified** |
| **R-D3** — no `# Errors` added to `lib.rs`, `disposition.rs`, or `lifecycle_update.rs`; no doc line M013 owns is edited | `git diff -- crates/eggup-service/src/lib.rs crates/eggup-service/src/disposition.rs crates/eggup-service/src/lifecycle_update.rs` contains **only** M012/M013's already-closed work; the M014 diff touches none of them | `git diff --stat` | **Verified — untouched by M014** |
| **R-D4** — no `# Panics` section promising the absence of panics | `grep -n "# Panics" crates/eggup-service/src/windows_scm.rs` → **0 hits** | grep | **Verified** |

## Production implementation evidence

### Typed SCM failures — `crates/eggup-service/src/windows_scm.rs`

The five former panic sites all sat on the same seam: opening an SCM service
handle. The helper's inner error type was `u32` and it produced exactly two
codes, so the compiler proved the `Err(_)` arms unreachable — which is why the
panic was latent rather than absent.

The inner error type is now a named, crate-private enum that can carry any
code:

```rust
enum OpenFailure {
    NotFound,        // SCM 1060
    MarkedForDelete, // SCM 1072
    Other(Option<i32>), // every other code, verbatim
}
```

`classify_open_code` maps a raw code onto it; `unmapped_open_error` turns an
unmapped code into `ServiceError::manager_with_os_code(...)`; `open_failure`
turns the two absence codes into the "service disappeared before \<operation\>"
conflict and delegates everything else to `unmapped_open_error`.

**No new error variant and no new taxonomy.** The existing
`ServiceError::Manager` / `ServiceError::Conflict` constructors and M013's
existing `ManagerFailureKind` set carry the codes. Because M013 landed before
M014, plan §16's "use M013's typed kind" clause is the implementation: the
unmapped path uses `ManagerFailureKind::from_scm_os_code`, the same mapping
function the rest of the crate uses.

**Observable behaviour is unchanged for every previously-reachable path.** The
detail text, the `ManagerFailureKind`, and the `os_code` are identical to what
the baseline produced, because the baseline already routed a non-`1060`/
non-`1072` Winapi failure through `Self::map_error(error, "open service")`
before it could reach a panic site; the new code routes the same failure
through `unmapped_open_error`, which renders the same detail with the same
operation label.

### The real SCM fixture — `crates/eggup-service/tests/windows_scm_live.rs`

The fixture registers a disposable registration through the crate's **public**
API and drives the whole lifecycle against the genuine Service Control
Manager. Three design points are worth recording:

- **The service image is the test binary.** The SCM must start a real service
  process for the adapter to observe `Running`, so the fixture registers its
  own executable and passes a marker argument. When the SCM starts the binary,
  the marker selects exactly one test — the service entry point — which
  registers the control handler, reports `Running`, and blocks in the
  dispatcher until the SCM stops the service. The harness is invoked with
  `--exact --test-threads=1` so that the entry point is the only test that
  runs and so that it runs on the harness's main thread.
- **The dependency target is the fixture's own.** `ChangeServiceConfigW`
  validates a dependency, so the fixture creates a **second** disposable
  registration of its own and depends on that. No pre-existing service is
  ever named, and the dependency round-trip is asserted against a real SCM
  entry.
- **The skip reason survives a passing run.** `libtest` captures
  `println!`/`eprintln!` and only replays the capture for a failing test, so
  a recorded skip reason would be invisible in a passing run. The fixture
  writes it straight to the process's stdout instead.

### The public-API surface guard — `crates/eggup-service/tests/public_api_surface.rs`

The guard binds every public item to an explicit type. Functions and methods
are bound as `const _: fn(..) -> .. = path;`; enums are bound by an
exhaustive match (when they are not `#[non_exhaustive]`) or by a match that
names every known variant; structs are bound by a closure that reads each
public field with its declared type, so a renamed field, a changed field type,
a removed field, or an added field is a compilation error. `impl Into<String>`
parameters cannot be turbofished, so the four affected constructors are
pinned through a local generic wrapper whose bound must still satisfy the
constructor's bound.

The guard also asserts **coverage** against the crate's own sources, so a new
public item that nobody pinned fails the test rather than passing silently.

## Public-surface enumeration (R-B5)

```text
$ grep -n "^pub \(fn\|struct\|enum\|trait\|const\|static\|type\|mod\)" \
      crates/eggup-service/src/*.rs | wc -l
75
```

Per file: `commit_engine.rs` 8, `disposition.rs` 13, `lib.rs` 43,
`lifecycle_update.rs` 6, `windows_scm.rs` 5.

```text
$ grep -n "^pub use" crates/eggup-service/src/lib.rs
15:pub use commit_engine::{
19:pub use disposition::{
24:pub use lifecycle_update::{
28:pub use windows_scm::{
176:pub use eggup_core::Ownership;
```

**Deviation from the plan's literal counts, stated plainly.** The plan's §3.5
records "68 top-level public items plus the 3 `pub use` re-exports" — that is
the count at the reviewed baseline `c4805411`. The working tree also carries
the already-closed service milestones **M012** and **M013**, which moved
`Ownership`, `CommandOutput`, and `CronOwnership` from locally-defined types
to re-exports and type aliases, split `commit_engine.rs` out of
`lifecycle_update.rs`, and added the typed manager-failure taxonomy. The
current surface is therefore **75 top-level public items and 5 `pub use`
re-exports**. R-B5's operative requirement is that the guard cover *every*
public item; it does, and it covers the surface that exists now. The count
difference is recorded here so a reviewer is not surprised by it.

A related observation, recorded as a finding rather than fixed: the
M012/M013 type-identity changes (`Ownership` becoming a re-export of
`eggup_core::Ownership`, `CommandOutput` becoming a type alias) alter the
**published** `eggup-service 0.1.3` surface. The published-surface fixtures
under `tests/published-api-0.1.2/` and `tests/published-api-0.1.3/` are
CI-only and are owned by another milestone; this milestone's guard pins the
**source-line** surface, which is the surface this milestone's plan asks for.

## Exact commands run and results

All commands were run from the workspace root on macOS (darwin/arm64) with
`--offline`. No network access was used or needed.

| # | Command | Result |
|---|---|---|
| V1 | `cargo fmt --all -- --check` | **exit 0, clean** |
| V2 | `cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings` | **exit 0**, no warnings |
| V3 | `cargo test -p eggup-service --locked --offline` | **14 test binaries, all ok** — 111 lib tests + 46 integration tests, 0 failed |
| V4 | `cargo doc -p eggup-service --no-deps --locked --offline` | **exit 0**, no rustdoc warnings |
| V5 | `cargo check --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service` | **exit 0** — the Windows-only fixture and the `#[cfg(windows)]` adapter compile |
| V6 | `cargo clippy --target x86_64-pc-windows-msvc --all-targets --locked --offline -p eggup-service -- -D warnings` | **exit 0** |
| V7 | `grep -n "unreachable!" crates/eggup-service/src/windows_scm.rs` | **ZERO hits** |
| V8 | `grep -n "# Panics" crates/eggup-service/src/windows_scm.rs` | **ZERO hits** (R-D4) |
| V9 | `cargo test -p eggup-service --locked --offline --test public_api_surface` | **5 passed** (4 guard tests + the folded `api_compatibility` test) |
| V10 | `cargo test -p eggup-service --locked --offline --lib former_panic` | **1 passed** (`windows_scm::tests::former_panic_sites_report_typed_errors_instead_of_panicking`) |

`cargo test -p eggup-service --locked --offline` in full:

```text
running 111 tests   test result: ok. 111 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.28s
running 1 test      test result: ok. 1 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 3 tests     test result: ok. 3 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
running 4 tests     test result: ok. 4 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 8 tests     test result: ok. 8 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 4 tests     test result: ok. 4 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 2 tests     test result: ok. 2 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 2 tests     test result: ok. 2 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
running 14 tests    test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
running 0 tests     test result: ok. 0 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 5 tests     test result: ok. 5 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
running 1 test      test result: ok. 1 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 0 tests     test result: ok. 0 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
running 2 tests     test result: ok. 2 passed;  0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

The two `running 0 tests` entries are `tests/ownership_type_identity.rs` (a
compile-only type-identity target) and `tests/windows_scm_live.rs` — the
Windows-only fixture, which is compiled out on this host. **That is a "not
run", not a pass.**

### Commands deliberately NOT run at this stage

| Command | Why |
|---|---|
| `cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture` | The fixture is `#![cfg(windows)]`. On this macOS host the target compiles to zero tests. **Not run** — it needs a real Windows Service Control Manager. |
| `cargo package -p eggup-service --locked` | The whole-workspace packaging step is part of the parent session's gate. |
| `cargo test --workspace --all-targets --all-features --locked` | The whole-workspace gate is the parent session's. |
| `cargo +1.89.0 check --workspace --all-targets --locked` | The whole-workspace MSRV check is the parent session's. |
| `./scripts/check-local.sh` | The whole-workspace local gate is the parent session's. |
| Hosted `stable`, `msrv`, `service-systemd`, `macos`, `windows-check` | The parent session runs and records the hosted lanes. |

### Platform evidence

Reported separately; **never inferred** from one another.

| Platform | Result | Evidence |
|---|---|---|
| **macOS (darwin/arm64)** | **run** — all green | V1–V10 above. 14 test binaries, 111 + 46 tests, 0 failed. The Windows-target compile (V5/V6) also ran here, because the `x86_64-pc-windows-msvc` target is installed on this host; it proves compilation only, **not** behaviour. |
| **Linux** | **not run** | No Linux host was available in this session. The crate's Unix adapters are exercised by the parent session's hosted lanes. |
| **Windows** | **not run** | No Windows host was available in this session. The hosted `windows-check` lane is the parent session's. The `windows_scm_live` fixture has therefore **never executed**. |

### Baseline failure proofs (plan §11)

The mechanism was `git worktree add` at a unique `/tmp` path
(`/tmp/eggup-m014-baseline-4108`, removed afterwards). The main working tree's
git state was not mutated: no stash, no checkout, no restore, no commit, no
branch, no tag.

**Item 2 — the API-surface guard, at the baseline.** The new fixture files were
copied unchanged into the baseline worktree and the guard was run there:

```text
$ cd /tmp/eggup-m014-baseline-4108
$ cargo test -p eggup-service --locked --offline public_api_surface
error: couldn't read `crates/eggup-service/tests/../src/commit_engine.rs`: No such file or directory (os error 2)
   --> crates/eggup-service/tests/public_api_surface.rs:117:26
error[E0432]: unresolved imports `eggup_service::commit_with_disposition_with_engine_policy`,
  `eggup_service::commit_with_lifecycle_with_engine_policy`, `eggup_service::CommitEnginePolicy`,
  `eggup_service::ManagerCommandOutput`, `eggup_service::ManagerFailureKind`,
  `eggup_service::OwnershipLossTolerance`, `eggup_service::PostTransitionVerification`,
  `eggup_service::MAX_DIAGNOSTIC_BYTES`
  --> crates/eggup-service/tests/public_api_surface.rs:53:5
error[E0624]: associated function `manager` is private
   --> crates/eggup-service/tests/public_api_surface.rs:85:27
error[E0061]: this function takes 1 argument but 2 arguments were supplied
   --> crates/eggup-service/tests/public_api_surface.rs:85:13
  --> /private/tmp/eggup-m014-baseline-4108/crates/eggup-service/src/lib.rs:318:19
...
error: could not compile `eggup-service` (test "public_api_surface") due to 21 previous errors
```

**This is red, but it is red for a structural reason and it is not the
detection-power proof.** The baseline `c4805411` predates M012 and M013, so
the pinned surface it declares is genuinely different — the guard is correct
to refuse it. The plan's §11 wording asks for the removal of one pinned public
item in the baseline worktree; because the baseline guard already fails to
compile for that reason, the isolated removal probe was therefore run in a
tree where the guard otherwise compiles (the current working tree):

```text
# `WindowsScmInstall::with_dependencies` — a pinned public item with no
# internal caller — removed from crates/eggup-service/src/windows_scm.rs:
error[E0599]: no function or associated item named `with_dependencies` found for struct `WindowsScmInstall` in the current scope
   --> crates/eggup-service/tests/public_api_surface.rs:911:67
   --> /Users/davidbowman/projects/eggup/crates/eggup-service/src/windows_scm.rs:175:5
error: could not compile `eggup-service` (test "public_api_surface") due to 1 previous error
```

That is the plan's §11 proof: **one pinned public item removed → the pinning
test fails to compile.** The file was restored immediately afterwards and the
guard is green.

**Item 3 — the typed-error fixture, at the baseline.** The baseline's
`windows_scm.rs` test module received the §10 item 3 fixture:

```text
$ cd /tmp/eggup-m014-baseline-4108
$ cargo test -p eggup-service --locked --offline --lib former_panic
error[E0425]: cannot find function `classify_open_code` in this scope
    --> crates/eggup-service/src/windows_scm.rs:1115:20
     |                    ^^^^^^^^^^^^^^^^^^ not found in this scope
error[E0433]: failed to resolve: use of undeclared type `OpenFailure`
    --> crates/eggup-service/src/windows_scm.rs:1115:52
error[E0425]: cannot find function `classify_open_code` in this scope
    --> crates/eggup-service/src/windows_scm.rs:1116:20
error[E0433]: failed to resolve: use of undeclared type `OpenFailure`
    --> crates/eggup-service/src/windows_scm.rs:1116:52
error[E0425]: cannot find function `classify_open_code` in this scope
    --> crates/eggup-service/src/windows_scm.rs:1117:20
...
error: could not compile `eggup-service` (lib) due to 9 previous errors
```

**Mechanism recorded (plan §10 item 3).** The baseline's `Self::open` returns
`Result<Result<Service, u32>, ServiceError>` — its inner error type is `u32`,
so a third code is not even expressible, and the fifth arm is guarded by an
`unreachable!()`. The fixture therefore drives the arm through the seam's own
contract: the new `OpenFailure` type is the seam's contract, and
`classify_open_code` is the test-visible construction of the third arm. At
the baseline that contract does not exist, so the fixture **fails to compile**
— which is the off-Windows form of the red proof. The plan's own "honest
statement" is that §10 item 3 cannot be proved red on a macOS or Linux host,
because the panic sites live behind `#[cfg(windows)]`; the panic itself is
therefore observable only on Windows, and the hosted run is where it would be
observed. This record does not claim the baseline panic was observed.

**Item 1 — the Windows SCM fixture, at the baseline.** At the baseline the
fixture does not exist, so per plan §10 item 1 it "cannot be red"; its evidence
is the **first real execution** of `WindowsBackend`, which has not happened.
Copied into the baseline worktree, the fixture compiles to zero tests on this
host:

```text
$ cd /tmp/eggup-m014-baseline-4108
$ cargo test -p eggup-service --locked --offline --test windows_scm_live
     Running tests/windows_scm_live.rs (target/debug/deps/windows_scm_live-f84941749973e9ec)
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

That is **not run**, and it is not a pass.

**Item 7 (R-B7) — an unpinned public item.** Adding one public item to
`crates/eggup-service/src/lib.rs` without pinning it:

```text
thread 'every_top_level_public_item_is_pinned' panicked at crates/eggup-service/tests/public_api_surface.rs:951:5:
assertion `left == right` failed: the crate's public surface changed: the guard pins 75 top-level public items but the sources declare 76. Either pin the new item in this file and in tests/public-api-surface.txt, or record the intended API change.
test result: FAILED. 4 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## R-C2 negative check

**Positive half.** With the per-variant allowances in place, the non-Windows
build is clean:

```text
$ cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings
    Checking eggup-service v0.1.3 (/Users/davidbowman/projects/eggup)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.60s
```

**Negative half — exact command and exact diagnostic.**

Variant 1 — `ScmState::ContinuePending` (constructed only by the
`#[cfg(windows)]` backend and by the test helpers). Its
`#[cfg_attr(not(windows), allow(dead_code))]` was temporarily deleted:

```text
$ cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings
error: variant `ContinuePending` is never constructed
   --> crates/eggup-service/src/windows_scm.rs:908:5
    |
893 | enum ScmState {
    |      -------- variant in this enum
...
908 |     ContinuePending,
    |     ^^^^^^^^^^^^^^^
    |
    = note: `ScmState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
    = note: `-D dead-code` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(dead_code)]`

error: could not compile `eggup-service` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

Variant 2 — `ScmLookup::MarkedForDelete` (the plan's other named variant):

```text
$ cargo clippy -p eggup-service --all-targets --all-features --locked --offline -- -D warnings
error: variant `MarkedForDelete` is never constructed
   --> crates/eggup-service/src/windows_scm.rs:946:5
    |
940 | enum ScmLookup {
    |      --------- variant in this enum
...
946 |     MarkedForDelete,
    |     ^^^^^^^^^^^^^^
    |
    = note: `ScmLookup` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
    = note: `-D dead-code` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(dead_code)]`

error: could not compile `eggup-service` (lib) due to 1 previous error
warning: build failed, waiting for other jobs to finish...
```

**A third probe, on `ScmState::Unknown`, resolved a plan ambiguity.**
`windows-service` 0.8.1 exposes exactly seven `ServiceState` variants and the
backend maps each of them, so **no** platform can produce `ScmState::Unknown`.
Its allowance therefore cannot be `not(windows)`-conditional: it was probed on
the **Windows** target, where its `#[allow(dead_code)]` was temporarily
deleted:

```text
$ RUSTFLAGS="-D warnings" cargo check --target x86_64-pc-windows-msvc --locked --offline -p eggup-service
error: variant `Unknown` is never constructed
   --> crates/eggup-service/src/windows_scm.rs:921:5
    |
893 | enum ScmState {
    |      -------- variant in this enum
...
921 |     Unknown,
    |     ^^^^^^^
    |
    = note: `ScmState` has derived impls for the traits `Clone` and `Debug`, but these are intentionally ignored during dead code analysis
    = note: `-D dead-code` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(dead_code)]`

error: could not compile `eggup-service` (lib) due to 1 previous error
```

`ScmState::Unknown` is therefore the **one** variant whose allowance is
unconditional, and its doc comment says so. Every other variant of both enums
keeps `#[cfg_attr(not(windows), allow(dead_code))]`. All three probes were
undone immediately afterwards; the file was byte-compared against its
pre-probe copy and the tree is green.

**R-C2's "if the negative check cannot fail for a given variant, that variant
is NOT Windows-only and its allow MUST be removed"** — the check **can** fail
for every variant that keeps an allowance (proved above for a `ScmState`
variant, a `ScmLookup` variant, and the `Unknown` variant), so no allowance
had to be removed.

## Invariant review

- **`Ownership` is never inferred.** Unchanged. The fixture asserts `Owned` for
  the exact spec and `Foreign` for a spec naming a different executable, and
  asserts that a mutation against the foreign spec is refused.
- **Destructive operations require `Owned`.** Unchanged. The adapter's own gate
  is untouched.
- **No elevation path.** Unchanged and re-asserted: the fixture probes the SCM
  for `CREATE_SERVICE` access and skips with a recorded reason; it never
  prompts, never calls `runas`, and never shells out.
- **No shell, no ambient `PATH`, no `sc.exe`.** Unchanged. The fixture uses the
  `windows-service` wrapper directly and the adapter uses it too.
- **Bounded output and bounded waits.** Unchanged and extended: the fixture
  adds its own explicit deadlines with a 50 ms poll interval and a timeout
  error that names the observed state.
- **SHA-256 is integrity evidence only.** Untouched — this milestone has no
  artifact work.
- **No new dependency.** `Cargo.toml` and `Cargo.lock` are untouched.

## Failure / rollback / recovery review

- The five former panic sites now return typed errors on every path. The
  access-denied case (`5`) keeps its bounded remediation detail and its
  `Permission` kind; the disappeared case is a `Conflict`; any other code is a
  `Manager` failure that still carries the raw OS code.
- The fixture's own failure paths are bounded: every wait has a deadline, and
  a timeout reports the observed state instead of asserting a duration.
- The fixture's cleanup runs in `Drop`, so it also runs while a panic unwinds.
  A cleanup failure is recorded (`EGGUP-SCM-LIVE-CLEANUP-FAILED`) rather than
  swallowed.
- The adapter's rollback and commit semantics are **out of scope and
  untouched**: `commit_engine.rs`, `disposition.rs`, and `lifecycle_update.rs`
  were not edited.

## Compatibility and migration review

- **No public item added, removed, or re-typed.** The new crate-private items
  (`OpenFailure`, `classify_open_code`, `unmapped_open_error`, `open_failure`)
  are not exported. The `pub use` blocks in `lib.rs` are unchanged.
- **No consumer migration.** The observable error text, kind, and `os_code`
  for every previously-reachable path are identical.
- **No `# Panics` promise is made** (R-D4). The absence of panics on those
  paths is the outcome of the typed-error conversion, not an independent
  claim, and no rustdoc asserts it.
- The published `eggup-service 0.1.3` is unaffected: nothing was published,
  no tag was moved, and the change is source-line only.
- The M012/M013 type-identity changes to the published surface are recorded as
  a finding below; they are already-closed work and were not touched here.

## Security review

- **No privilege escalation.** The fixture requires administrator rights and
  *skips with a recorded reason* when they are absent. It never elevates.
- **No pre-existing service is touched.** The fixture creates two
  registrations with a unique per-run name (`EggupScmLive-<pid>-<nanos>`) and
  removes both; the dependency target is the fixture's own second
  registration, not a system service.
- **No secrets.** The service account is the SCM default (LocalSystem); no
  account password is ever accepted or stored.
- **No `unsafe`.** The workspace denies `unsafe_code` for every target and the
  fixture contains none: the SCM service entry point is an
  `extern "system" fn` whose raw-pointer arguments are never dereferenced.
- **Bounded diagnostics.** The adapter's SCM detail strings remain bounded by
  the crate's single ceiling, `MAX_DIAGNOSTIC_BYTES`.

## Documentation / operations evidence

- `crates/eggup-service/CHANGELOG.md` — the `Unreleased` section gained a
  "Service milestone 014" block covering all four defects, with the
  no-publication and no-migration disclaimer already at the head of the
  section.
- `crates/eggup-service/README.md` — the typed-error contract, the real-SCM
  fixture, and the public-API surface guard are stated.
- `architecture/service-lifecycle.md` — the `windows_scm.rs` source-layout
  table, the "Windows SCM" adapter section, the "Testing approach" section,
  and the "Known doc/code drift" section were updated. The testing section's
  stale "There is no `tests/` directory" claim was corrected while the
  milestone's own fixtures were added.
- `crates/eggup-service/tests/public-api-surface.txt` — the committed
  snapshot, with its regeneration policy and its enumeration command.

## Unresolved findings

### UF-1 — The hosted Windows run is outstanding; the milestone's core evidence is absent

**Severity: high (blocking for closure).**

The fixture `crates/eggup-service/tests/windows_scm_live.rs` has **never
executed**. Everything it asserts — that the real SCM accepts the
registration, that the adapter observes `Running` and `Stopped`, that the
`windows-service` mapping round-trips, that the ownership classification
matches the SCM's own view — is unproven. The Windows-target compile (V5/V6)
proves that the code is well-formed for Windows; it proves **nothing** about
its behaviour.

The milestone is therefore **implemented but not closed**. The parent session
must run, on a hosted Windows runner:

```text
cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture
```

and append the result here. If the runner cannot obtain the administrator
rights a real registration requires, the fixture records its skip reason and
the milestone is **blocked on Windows evidence**, not closed; the skip reason
belongs in this record.

A second, smaller risk sits behind the same lane: the fixture depends on the
SCM starting the test binary as a service and on the harness running the
service entry point on its main thread (`--test-threads=1`). That is the
standard shape for this kind of fixture, but it is unverified until the
hosted run happens. If the SCM reports error `1053` (the service did not
connect), the entry point is not registering itself — the fixture's failure
detail will name it.

### UF-2 — `cargo fmt --all` (the apply form) reformatted one file outside this milestone's ownership

**Severity: low (formatting-only, semantically inert).**

`cargo fmt --all -- --check` is one of this milestone's required commands, but
the **apply** form was run once during implementation, to normalise the new
files. It also reformatted **one** file this milestone does not own:

```text
crates/eggup-curl/src/lib.rs   — one line: a method-chain continuation indent
```

The change is indentation only; `git diff -w` shows it as a pure whitespace
difference and the file's semantics are unchanged. The tree was **already
failing** `cargo fmt --all -- --check` in that file (and in five untracked
test files under `crates/eggup-acquisition/tests/`,
`crates/eggup-curl/tests/`, and `crates/eggup-eggfetch/tests/`, which a
concurrent session had formatted at 17:41–17:44) before this milestone
started. Reverting the one line would re-introduce that gate failure, and
the gate is the repository's own contract, so the line was left normalised.

This is reported rather than hidden: the file belongs to another milestone's
ownership, and the parent session may want to confirm the normalisation.

### UF-3 — The `windows-check` CI lane needs the new fixture's step

**Severity: medium (the lane must run the fixture for UF-1 to close).**

Plan §11 names the Windows-lane command form

```text
cargo test -p eggup-service --test windows_scm_live --locked -- --nocapture
```

`.github/workflows/ci.yml` is **not** in this milestone's file ownership, so
this milestone did not edit it. The existing `windows-check` lane already
runs `cargo test --workspace --all-targets --locked --no-fail-fast`, which
compiles and runs the new fixture target — but without `--nocapture`, so a
recorded skip reason would be swallowed by the harness's output capture. The
fixture writes its skip reason straight to the process's stdout precisely so
that it survives that capture, but the explicit step from §11 is still the
form the plan asks for. **The parent session owns the CI file and must add
the step** (or confirm that the workspace-wide run is sufficient).

### UF-4 — The published-surface divergence introduced by M012/M013

**Severity: medium (pre-existing, already-closed work; recorded, not fixed).**

The M012/M013 changes moved `Ownership` from a locally-defined enum to a
re-export of `eggup_core::Ownership`, and `CommandOutput` from a struct to a
type alias of `ManagerCommandOutput`. Both alter the **published**
`eggup-service 0.1.3` surface: `eggup_service::Ownership` is now a *different
type identity* than the published `0.1.3` `Ownership`, and
`eggup_service::CommandOutput` is now an alias rather than a struct. A
consumer that constructed `eggup_service::Ownership::Absent` still compiles
(it is the same variant set), but a consumer that relied on
`eggup_service::Ownership` being a *distinct* type from
`eggup_core::Ownership` is affected, and the published-surface fixtures under
`tests/published-api-0.1.2/` / `tests/published-api-0.1.3/` are the CI-only
controls for exactly that.

This milestone's guard pins the **source-line** surface, which is what the
plan asks for; it does not and cannot assert the published surface. The
publication-policy consequence is a finding for the parent session and for
the republication milestone, not for this one.

### Adjacent findings (recorded, deliberately not fixed here)

- **`WindowsScmScope` does not exist.** Plan §6's R-D1 enumerates
  "`WindowsScmScope`/`WindowsStartType`/`WindowsErrorControl`/`WindowsServiceDependency`
  (`:21`, `:32`, `:45`, `:58`)". There is no `WindowsScmScope` type in the
  crate: the four line references are `WindowsStartType` (`:29`), the
  `WindowsErrorControl` (`:47`), `WindowsServiceDependency` (`:67`), and
  `WindowsScmInstall` (`:90`) after this milestone's edits. The `# Errors`
  documentation was added to all four existing types. The plan's type name is
  recorded here as a plan inaccuracy, not as a missing type.
- **`WindowsScmManager::new` cannot fail.** R-D1 asks for a `# Errors`
  section on it, and one is present, but the constructor is infallible: it
  does not contact the SCM, and every SCM handle is opened per operation. The
  section says so explicitly and then names the conditions the adapter
  reports, rather than asserting a failure that cannot happen.
- **The plan's baseline SHA predates two milestones.** `c4805411` is the
  baseline both M012 and M013 were implemented against, and M012/M013 are
  uncommitted in this tree. The baseline worktree therefore reproduces a
  surface that is two milestones behind the one the guard pins, which is why
  the isolated removal probe (the plan's actual detection-power proof) was run
  in the current tree. This is stated in §"Baseline failure proofs" rather
  than glossed over.

## Roadmap disposition

Service milestone **M014** is **implemented, not closed**. The four defects
the plan targets are addressed in the source line:

| Defect | State |
|---|---|
| A — the real Windows SCM backend has never executed | The fixture exists and compiles for Windows; **the hosted run is outstanding** (UF-1) |
| B — no dependency-free public-API surface guard | **Done.** Compile-only pinning test + committed snapshot + the retained `api_compatibility.rs` folded in |
| C — enum-wide `#[allow(dead_code)]` | **Done.** Per-variant gating, with the negative check's exact diagnostic recorded |
| D — no `# Errors` on the Windows SCM public surface | **Done**, without a `# Panics` promise |

The milestone's status flips to **closed** when the parent session appends the
hosted Windows result. Until then it stays open, and the roadmap status table
and `plans/registry.md` — both owned by the parent session — must not record it
as closed.

## Registry updates

**None made by this milestone.** `plans/registry.md` and
`plans/subsystems/service-lifecycle-roadmap.md` are owned by the parent
session. The parent session should record M014 as *implemented, hosted Windows
run outstanding* — not closed — and then *closed* once UF-1 is discharged.

The source plan's `Status:` header has been updated from `ready for handoff`
to `implemented — hosted Windows run outstanding`, pointing at this record.

## Files touched by this milestone

```text
crates/eggup-service/src/windows_scm.rs              (modified)
crates/eggup-service/tests/public_api_surface.rs      (new)
crates/eggup-service/tests/public-api-surface.txt     (new)
crates/eggup-service/tests/windows_scm_live.rs         (new)
crates/eggup-service/tests/api_compatibility.rs        (modified — `classify` made `pub` so the fold is real)
crates/eggup-service/CHANGELOG.md                     (modified)
crates/eggup-service/README.md                        (modified)
architecture/service-lifecycle.md                     (modified)
plans/closure/service-lifecycle/014-status.md          (new — this file)
plans/implementation/service-lifecycle/014-windows-scm-execution-and-api-surface-corrective.md
                                                    (modified — `Status:` header only)
```

One file outside this milestone's ownership was reformatted by `cargo fmt
--all` (apply form) — see UF-2.

### Ownership verification

`git status --short` shows a long list of modified files, because the working
tree already carried the uncommitted work of the earlier closed milestones
(core M014/M015, archive M005, service M012/M013, acquisition and transport
M012/M013) **and** a concurrent session was editing the tree at the same time
as this milestone. The verification below separates those from this
milestone's own changes.

Files this milestone modified or created — **all inside its declared
ownership**:

```text
crates/eggup-service/src/windows_scm.rs
crates/eggup-service/tests/api_compatibility.rs
crates/eggup-service/tests/public_api_surface.rs        (new)
crates/eggup-service/tests/public-api-surface.txt       (new)
crates/eggup-service/tests/windows_scm_live.rs          (new)
crates/eggup-service/CHANGELOG.md
crates/eggup-service/README.md
architecture/service-lifecycle.md
plans/closure/service-lifecycle/014-status.md           (new)
plans/implementation/service-lifecycle/014-windows-scm-execution-and-api-surface-corrective.md
                                                      (the `Status:` header only)
```

Files this milestone modified **outside** its ownership: exactly one,
`crates/eggup-curl/src/lib.rs`, by the formatter's apply form — one line of
indentation, recorded as UF-2.

`crates/eggup-service/src/lib.rs`, `disposition.rs`, and
`lifecycle_update.rs` were **not** edited by this milestone: their diffs
against `HEAD` contain only the already-closed M012/M013 work. `Cargo.toml`
and `Cargo.lock` were not touched — the only `Cargo.toml` in `git status` is
`crates/eggup-eggpack/Cargo.toml`, which belongs to an earlier milestone. The
root `CHANGELOG.md` and `plans/registry.md` were not touched.

