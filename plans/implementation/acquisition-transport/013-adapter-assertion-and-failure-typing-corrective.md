# Acquisition Transport Milestone 013 — adapter assertion strength and failure typing corrective

Status: closed 2026-10-23; implemented and green on the native macOS lane, with every new fixture proven to fail against the pre-M013 baseline, and hosted-qualified on Linux, macOS, and Windows by CI run `40008039843` (commit `0cb2c53`, all five lanes green; local whole-tree gate exit 0). Closure record: `plans/closure/acquisition-transport/013-status.md`. The adapter-capability-parity finding M013 deferred stays **open** as acquisition M014, and the source-breaking `AcquisitionError::Io { operation, source }` retype (derives `Debug` only) still awaits a `0.2.0` republication of `eggup-acquisition`.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Original closed work affected:

- `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`
- `plans/implementation/acquisition-transport/007-boundary-safety-hardening-corrective.md`
- `plans/closure/acquisition-transport/005-status.md`
- `plans/closure/acquisition-transport/007-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **8**: "Nine `eggup-curl` tests pass
  vacuously when their fixture is unavailable, and `AcquisitionError::Io` flattens the underlying
  `io::Error`." Registered at baseline `c480541` alongside this plan. The finding is assessed
  **medium** severity and is closed when this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant

## 1. Objective

Close five characterized defects in the acquisition transport subsystem, and record the scope decisions for the two that
admit more than one correct answer.

1. **Nine `eggup-curl` tests report success while asserting nothing** when their fixture cannot be constructed.
2. **`AcquisitionError::Io(String)` flattens the underlying `io::Error`**, so a consumer cannot match on `std::io::ErrorKind` and the `source()` chain is empty.
3. **The two adapters do not offer an equivalent capability surface** — the eggfetch adapter exposes a caller-settable `User-Agent`; the curl adapter exposes none.
4. **The UTF-8-safe truncation helper is copied eight times across four crates**, including two inlined copies inside `eggup-acquisition` that do not call that crate's own helper.
5. **`eggup-curl` implements its own bounded subprocess runner** — justified by the dependency rules, but undocumented, so a future reader may "fix" it by adding an illegal dependency.

Required end state: no adapter test can report a pass without asserting its behavior; a filesystem failure carries a
matchable, source-linked I/O error; the adapter-parity question has one stated answer with a named owner; the truncation
primitive is consolidated inside each reachable dependency group; and the curl runner's duplication of `run_bounded` is
recorded as deliberate.

## 2. Why this milestone is ready

- All five defects are **fully characterized by reading the code** at baseline `c4805411f74b85158e0ae5d0a219a32c4209d98e`; each is cited with `file:line` in §3.
- The subsystem has exactly **one sibling milestone**, M012 (`plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`, `ready for handoff`), which is test-only and addresses host-sensitivity in the sub-second deadline tests. M013 must not duplicate M012's work; the one overlapping site is `:1394`, where the two milestones touch different lines of the same test (§6.1).
- Scope is bounded: three crates (`eggup-acquisition`, `eggup-curl`, `eggup-eggfetch`), no new dependency in any of them, no change to the seam contract, no change to release/fallback policy.
- **Severity is medium for Defects A, B, C, and D**, and medium for Defect E as a recorded-justification item. Per this repository's gate, "medium-or-higher" keeps a milestone open and forces a corrective; these are not polish items.

### 2.1 Why no failing production report exists

There is **no failing production report** for any of the five defects, and for Defect A that is the mechanism of the
defect, not an accident of history. A test that returns early before asserting anything is **indistinguishable from a
test that passed**: the harness records one passed test either way. The nine sites cannot fail, cannot alert, and cannot
be caught by a reviewer reading a green CI run. The vacuity is exactly why the gap went undetected.

Defects B, C, and D are latent rather than active: no reported consumer has mis-classified an I/O failure, no consumer
has reported a lost `User-Agent`, and no copy of the truncation helper has produced a wrong byte cap. They are medium
because each is a **truthfulness** gap in a published contract a consumer is entitled to rely on, not a cosmetic
difference.

## 3. Current implementation evidence

### 3.1 Defect A — nine vacuous `eggup-curl` tests

`grep -n "else { return }" crates/eggup-curl/src/lib.rs` returns exactly nine hits; `crates/eggup-eggfetch/src/lib.rs`
returns none. Eight sites use `let Some(t) = transport() else { return };`; the ninth (`:1485`) uses `let Some(_) =
real_curl() else { return };`.

| Line | Test | `#[cfg]` gate | Fixture needed |
|---|---|---|---|
| `:1036` | `metadata_success_not_found_and_server_error` | `not(windows)` (`:1034`) | system `curl` + loopback server |
| `:1060` | `artifact_success_not_found_and_server_error` | `not(windows)` (`:1058`) | system `curl` + loopback server |
| `:1122` | `truncated_transfer_is_hard_failure_without_promotion` | `not(windows)` (`:1120`) | system `curl` + loopback server |
| `:1149` | `metadata_and_artifact_size_overflow` | `not(windows)` (`:1147`) | system `curl` + loopback server |
| `:1183` | `connect_and_total_timeouts_are_typed` | `not(windows)` (`:1181`) | system `curl` + unroutable host |
| `:1394` | `sub_second_total_timeout_kills_child_promptly` | `not(windows)` (`:1392`) | system `curl` + loopback server |
| `:1432` | `cancellation_before_spawn_cleans_temp` | **none** — runs on Windows too | system `curl` |
| `:1454` | `cancellation_during_body_stream_kills_and_reaps_child` | `not(windows)` (`:1452`) | system `curl` + loopback server |
| `:1485` | `redirect_follow_and_reject` | `not(windows)` (`:1483`) | system `curl` that follows HTTP redirects |

The fixture is **a discoverable system `curl` executable**, not a fake script: `transport()`
(`crates/eggup-curl/src/lib.rs:884-888`) → `real_curl()` (`:880-882`) → `discover_curl_executable()` (`:378-380`), which
searches `PATH` for `curl` (`curl.exe` on Windows, `:386-389`) and returns `Unavailable` when nothing is found. The
vacuity trigger is therefore **"no `curl` on `PATH`"**. The crate's `sh`-based fake-curl fixture (`:955`,
`#[cfg(unix)]`) cannot substitute: it writes a fixed body and exit code and cannot serve the loopback HTTP fixtures
(`serve_once`, `:890`, `#[cfg(not(windows))]`) the nine tests assert against.

Consequences: on a host without `curl`, all nine tests report **passed** while asserting nothing; on Windows the eight
`#[cfg(not(windows))]` sites are **excluded entirely**, so the Windows lane runs a strictly smaller `eggup-curl` test
set; `:1432` is the one vacuous site that does run on Windows and needs `curl.exe` from `System32`. Separately,
`connect_and_total_timeouts_are_typed` is **weakly asserted**: its connect branch asserts only `matches!(err,
AcquisitionError::Timeout { .. })` (`:1198-1201`), which accepts a `total` phase, so the `phase: "connect"` label is
never asserted; the `total` branch (`:1224`) is exact.

### 3.2 Defect B — `AcquisitionError::Io(String)` flattens the `io::Error`

`crates/eggup-acquisition/src/lib.rs:270-271` declares `Io(String)`; `:324` declares `impl std::error::Error for
AcquisitionError {}` with **no `source()` override**, so the `std::error::Error` chain is empty for every variant. The
private constructor `:300-302` flattens the `io::Error` into a bounded string:

```rust
pub(crate) fn io(operation: &'static str, source: std::io::Error) -> Self {
    Self::Io(bound_detail(format!("{operation}: {source}")))
}
```

The seam constructs `Io` at eight sites (`:492`, `:500`, `:507`, `:510`, `:554`, `:787`, `:874`, `:878`); the two
adapters construct it **directly**, bypassing the seam's constructor, at eleven sites — `eggup-curl` `:254`, `:288`,
`:339`, `:592`, `:684`, `:693`, `:694`, `:701`, `:704`, `:705` and `eggup-eggfetch` `:352`, `:433`, `:437` — because
`AcquisitionError::io` is `pub(crate)`.

A consumer therefore cannot distinguish `NotFound` from `PermissionDenied` from `AlreadyExists` on a staging or
promotion failure. For a transaction-owned filesystem operation those are materially different events, and
`architecture/acquisition.md` §6 (`:287-289`) already states that `Io` is **always terminal** — precisely the
classification a consumer cannot make.

The in-repo typed-I/O precedent exists: `eggup-core`'s published `Error::Io { operation: &'static str, source: io::Error
}` (`crates/eggup-core/src/error.rs:15-20`), whose `Error` impl returns `Some(source)` for that variant (`:192-205`);
core's `Error` derives **only** `Debug` (`:8`).

Two verified facts constrain any fix: `AcquisitionError` derives `Debug, Clone, PartialEq, Eq`
(`crates/eggup-acquisition/src/lib.rs:250`), and `std::io::Error` implements **neither `Clone` nor `PartialEq`/`Eq`**
(verified with `rustc 1.89.0`: `the trait bound std::io::Error: Clone is not satisfied`, `the trait bound
std::io::Error: Eq is not satisfied`). Embedding an `io::Error` therefore forces the enum's derives to change. Also,
`AcquisitionError` **is** `#[non_exhaustive]` (`:251`), so *adding* a variant is source-compatible for consumers, while
*changing the shape of the published `Io(String)` variant* is not — `AcquisitionError::Io(_)` stops compiling against a
struct-shaped variant.

### 3.3 Defect C — the adapters do not offer an equivalent capability surface

`crates/eggup-eggfetch/src/lib.rs:50-63` — `pub struct EggfetchConfig` has `pub user_agent: String` (`:54`), a `Default`
impl setting `DEFAULT_USER_AGENT` (`:68`), and a builder `pub fn user_agent` (`:84-87`); `DEFAULT_USER_AGENT` is
`"eggup-eggfetch"` (`:18`). `grep -n "user_agent\|user-agent" crates/eggup-curl/src/lib.rs` returns **nothing**: the
curl adapter has no user-agent control, so its wire request carries curl's own default `curl/<version>`.

The seam contract does **not** promise a user-agent — `AcquisitionRequest` carries only a URL
(`crates/eggup-acquisition/src/lib.rs:23-26`) and `AcquisitionTransport` (`:578`) exposes no header surface. The
asymmetry is therefore in the **adapter configuration surface**, not the seam contract; no seam invariant is violated
today. The consequence for a caller is real: switching transports silently loses control of the `User-Agent` and changes
the value on the wire.

Adding that control to curl is not free: `CurlConfig` (`crates/eggup-curl/src/lib.rs:49-67`) has **public fields and is
not `#[non_exhaustive]`**, so a new `pub user_agent` field is source-breaking for every external struct-literal
construction of `CurlConfig`, on a crate whose published version is `0.1.2`.

### 3.4 Defect D — eight truncation copies across four crates

| Location | Function | Byte cap |
|---|---|---|
| `crates/eggup-acquisition/src/lib.rs:330` | `truncate_utf8_bytes(s, max)` | parameter |
| `crates/eggup-acquisition/src/lib.rs:372` | inline `is_char_boundary` walk in `redact_url` | 256, plus `…` |
| `crates/eggup-acquisition/src/lib.rs:432` | inline `is_char_boundary` walk in `__scrub_upstream_text` | 512 |
| `crates/eggup-curl/src/lib.rs:408` | `bound(s)` | 512 |
| `crates/eggup-eggfetch/src/lib.rs:460` | `bound(s)` | 512 |
| `crates/eggup-service/src/lib.rs:1773` | `truncate_utf8_bytes(text, max_bytes)` | parameter |
| `crates/eggup-service/src/disposition.rs:291` | `bound_detail(detail)` | 512 |
| `crates/eggup-service/src/lifecycle_update.rs:684` | `bounded_detail(detail)` | 512 |

The two `eggup-acquisition` inline walks do **not** call that crate's own helper at `:330`, even though the helper takes
the cap as a parameter and can express both exactly. `architecture/acquisition.md` §14 already records the adapter-level
half of this as known doc/code drift ("Detail bounding is duplicated per adapter"), but that row covers neither the two
inlined seam copies nor the service-side copies. The pattern was introduced deliberately by service M007
(`plans/implementation/service-lifecycle/007-utf8-safe-bounded-diagnostics-corrective.md`), which eliminated
panic-capable fixed-byte `String::truncate` from production diagnostics. The behavior is correct everywhere; the defect
is that the fix was applied by copy, so the caps and the exact truncation semantics can drift per copy.

### 3.5 The dependency boundary that determines what is possible

`eggup-acquisition` has **zero dependencies** — `crates/eggup-acquisition/Cargo.toml` has an empty `[dependencies]`
section, not even `sha2`. `eggup-core` depends only on `sha2` and cannot depend on `eggup-acquisition`. `eggup-curl`
depends **only** on `eggup-acquisition`; `eggup-eggfetch` depends only on `eggup-acquisition` plus `eggfetch-core`,
`tokio`, and `futures-util`. `eggup-service` depends on `eggup-core`. Therefore **no single location is reachable by all
four crates**: a shared helper can live in `eggup-acquisition` (reachable by acquisition, curl, eggfetch — but not core
or service) or in `eggup-core` (reachable by core and service — but not acquisition, curl, or eggfetch).

### 3.6 Defect E — the curl adapter's own bounded subprocess runner

`crates/eggup-curl/src/lib.rs:594-599` spawns the external process with `stdin(Stdio::null())`,
`stdout(Stdio::piped())`, `stderr(Stdio::piped())` and `env_clear()`; `cmd.spawn()` is at `:603-605` and kill/reap
happens on four failure paths at `:650` (overflow), `:658` (cancellation), `:669` (total deadline), and `:679`
(`try_wait` error). This is the **third** bounded subprocess implementation in the workspace; the others are
`eggup-core`'s `run_bounded` (`crates/eggup-core/src/candidate.rs:125`) and `eggup-service`'s `SystemExecutor::run`
(`crates/eggup-service/src/lib.rs:700`).

The curl copy is **justified**: `eggup-curl` may depend only on `eggup-acquisition` and must not depend on `eggup-core`,
so neither `run_bounded` nor `SystemExecutor::run` is reachable from it. The judgment is correct and must be preserved;
what is missing is the **written** justification, so a future reader does not "fix" the duplication by adding an illegal
dependency edge.

Separately, `eggup-core`'s `run_bounded` has its own deadline defect — the deadline is enforced only while the direct
child is alive (`crates/eggup-core/src/candidate.rs:186-190`; the loop breaks only once `status.is_some()`, so a
descendant holding the pipes outlives the deadline). That defect is tracked by
`plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`, and **M013 must not depend on
it, must not re-fix it, and must not duplicate its evidence**.

## 4. Invariants that must not regress

- **URL/credential material stays redacted in every error path.** `Transport` and `Unavailable` already carry redacted URL material (`crates/eggup-acquisition/src/lib.rs:255-257`, `:272-274`). Any new or retyped error surface must preserve this: a typed I/O error must never carry a raw URL, and `Display` must stay bounded.
- **`FetchLimits::effective` stays in the seam.** `min(request, adapter ceiling)` is derived once, at `crates/eggup-acquisition/src/lib.rs:143-151`, and both adapters delegate to it. This is the **correct** shared-primitive pattern and must not be re-duplicated per adapter.
- **The seam stays zero-dependency**; `crates/eggup-acquisition/Cargo.toml` keeps an empty `[dependencies]`.
- **The adapters stay mutually ignorant**; neither may know the other exists.
- **Transport composition stays in the seam** — `ComposedTransport` and `CompositionPolicy` live in `eggup-acquisition` (`:931-955`).
- **The `Unavailable` fallback contract does not change.** `should_fallback` (`crates/eggup-acquisition/src/lib.rs:987-994`): `Unavailable` always falls back; `Transport` and `Timeout` fall back **only** under `CompositionPolicy::UnavailableOrTransport`; every other error and `NotFound` are terminal; `Io` is terminal under every policy (`architecture/acquisition.md`:287-289).
- **`eggup-acquisition` learns no release policy** — no "try the next version", no mirror selection, no fallback source, no retry.
- **The workspace denies `unsafe_code`;** all library crates keep `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`.
- **Rust 1.89 remains the MSRV;** no new dependency is added anywhere.
- **Destination safety is unchanged** — exclusive owner-private temp, no-clobber promotion, no owned residue on any failure path.
- **A real timeout, cancellation, byte overflow, or I/O failure is never converted into a success**, and a genuine transport regression is never relabelled as test flakiness.

## 5. Scope

### In scope

- The nine vacuous `eggup-curl` test sites, with a per-site treatment.
- A regression guard detecting a reintroduced early-exit pattern in the acquisition-transport crates' test code, and a witness that the adapter's core behaviors are asserted.
- The typed I/O failure surface on `AcquisitionError`.
- The adapter-capability-parity decision for `User-Agent`, plus the recorded interim difference.
- Consolidation of the truncation primitive inside the reachable dependency groups, and removal of the two inlined copies inside `eggup-acquisition`.
- The recorded justification for the curl adapter's own bounded subprocess runner.

### Explicitly out of scope

- **No service-side helper consolidation.** The `bound_detail` / `bounded_detail` pair is owned by `plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`. M013 must not duplicate that work.
- **No core bounded-execution fix.** `eggup-core`'s `run_bounded` deadline defect belongs to `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`. M013 must not depend on it.
- **No republication and no tag movement.** M013 publishes nothing; a republication of any changed crate is a separate milestone.
- **No new dependency in `eggup-acquisition`** — it stays at zero dependencies.
- **No change to the `FetchLimits` ceiling model**, to `FetchLimits::effective`, or to any adapter ceiling.
- **No change to the `Unavailable` / fallback contract** and no change to `CompositionPolicy`.
- **No embedded HTTP/TLS stack in `eggup-curl`** — it keeps shelling out to an external `curl` and gains no HTTP client.
- **No change to M012's scope.** M012 owns the deadline assertions in `sub_second_total_timeout_kills_child_promptly`; M013 owns only the vacuity line at `:1394`.
- **No change to release/source selection, retries, mirrors, or authenticity.**
- **No change to the semantics of the `Unavailable` construction sites** in `eggup-curl` (`:384`, `:403`); they may be routed through the seam's bounding helper, but their classification and content must not change.
- **No CI configuration, runner sizing, or lane scheduling change.**

## 6. Required production changes

### 6.1 Defect A — per-site fixture treatment

The general rule: **a test that cannot construct its fixture must never report a pass.** Every one of the nine sites
gets exactly one of two treatments, and the choice is recorded per site.

**Treatment (a) — assert the fixture is constructible and fail loudly.** The test's first act obtains its fixture
through a helper that **panics with an actionable message** naming the missing prerequisite, instead of returning `None`
and exiting. The message must state what is missing and what to do about it (for example: no `curl` executable was
discoverable on `PATH`; install `curl` or add it to `PATH`).

**Treatment (b) — environment-conditional with a recorded reason.** Used only where the precondition is genuinely a
property of the host the repository cannot guarantee on every lane. Such a test must be **explicitly marked** with a
reason, must not silently exit, and the reason must be recorded in the source, in this plan, and in the closure record.
It is reported as *ignored*, not as *passed*.

| Site | Treatment | Reason |
|---|---|---|
| `:1036`, `:1060`, `:1122`, `:1149`, `:1454` | (a) fail loudly | need a system `curl`; the loopback fixture is Unix-only and these tests are already `#[cfg(not(windows))]` |
| `:1394` | (a) fail loudly | M013 owns only this vacuity line; M012 owns the deadline assertions in the same test |
| `:1432` | (a) fail loudly | needs a system `curl`; **not** `#[cfg]`-gated, so this site also runs on Windows and needs `curl.exe` |
| `:1485` | (a) fail loudly | needs a **real** `curl` that performs HTTP redirects; the `sh` fake fixture cannot |
| `:1183` | **split** — below | the connect branch needs a blackholed route; the total branch does not |

The `:1183` split is required, not optional:

- The **total-phase** branch stays an end-to-end assertion: the loopback server stalls the body and the test asserts `Timeout { phase: "total" }` exactly (`:1224`). Deterministic; needs no special environment.
- The **connect-phase** branch (the unroutable `http://10.255.255.1:81/unreachable` probe) is host-dependent: a host that fails the connect fast (no route, `ENETUNREACH`) reports a transport failure rather than a timeout, so the branch cannot be asserted everywhere. It therefore takes treatment **(b)** with a recorded reason stating that it requires a host that blackholes the route.
- To keep the connect/total **label** proven deterministically, M013 must add a **no-network** unit proof at the `classify_curl_result` layer (`crates/eggup-curl/src/lib.rs:719-749`): the classification is a pure function of `exit_code`, elapsed time, and the two ceilings, and the elapsed time is derived from the injected `start: Instant` (`:724`, `:729`).
- Injecting `start` proves `exit 28` is labelled `connect` versus `total` with **no subprocess and no wall clock**. This mirrors M012's private deadline-seam discipline (M012 §6.4) and must not introduce a sub-second wall-clock assertion.
- The weak assertion `matches!(err, AcquisitionError::Timeout { .. })` at `:1198-1201` must not be carried forward as the connect proof; it accepts either phase and therefore proves nothing about the connect label.

The `#[cfg(not(windows))]` gate itself is **legitimate and must be preserved**: the loopback HTTP fixture (`serve_once`,
`:890`) is Unix-only, and the crate documents this at `:945-953`. M013 must additionally record, in the test module, why
the gate exists, so the Windows exclusion reads as a known, deliberate platform difference rather than a silent coverage
gap.

**The regression guard.** M013 must add a guard that makes vacuity un-reintroducible:

1. **Pattern guard.** A source-scanning test in each of `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch` asserts that the crate's own `src/` contains **no** `else { return }` / `else { return; }` early exit inside test code. The guard reads the source at compile time (`include_str!`), so it needs no filesystem discovery and no network.
2. **Reason guard.** The same scan asserts every `#[ignore]` in those crates' `src/` carries an explicit reason (`#[ignore = "..."]`), so an environment-conditional test can never be skipped silently either.
3. **Behavior witness.** A witness test asserts, for each of the nine required behaviors, that a **named** `#[test]` exists in the crate source, that its body contains **no** early `return`, and that it contains at least one assertion. This makes "the adapter's core behaviors ARE asserted" checkable rather than aspirational, so a future refactor cannot silently reintroduce vacuity.

The guard must be deterministic and must not depend on wall-clock timing, the network, or the host's `PATH`.

### 6.2 Defect B — typed I/O failure surface

- The I/O variant of `AcquisitionError` must carry the **operation** that failed and the **underlying `std::io::Error`**, so a consumer can match on `std::io::ErrorKind` and walk the `source()` chain.
- `impl std::error::Error for AcquisitionError` must override `source()` and return the underlying `io::Error` for that variant, mirroring `crates/eggup-core/src/error.rs:192-205`.
- The seam must expose an **adapter-facing** constructor for the typed variant, because `AcquisitionError::io` is `pub(crate)` and the eleven adapter construction sites (§3.2) cannot construct the typed variant otherwise.
- It must follow the existing `#[doc(hidden)]` adapter-facing convention already used by `__adapter_unavailable` (`crates/eggup-acquisition/src/lib.rs:295-298`), `__adapter_metadata` (`:917`), `__adapter_artifact` (`:919`), `__acquire_exclusive_temp` (`:456`), and `__scrub_upstream_text` (`:388`).
- `Display` for the typed variant must keep the current bounded form: the rendered detail must not exceed 512 bytes and must end on a UTF-8 character boundary, preserving the seam's documented bounding rule (`architecture/acquisition.md`:266-269).
- The redaction invariant must hold on the typed path: the typed I/O error must never carry URL or credential material, and a failure occurring while acquiring a credential-bearing URL must not surface that URL through `Display` or `source()`.
- The variant's shape must be decided **explicitly**, and the choice recorded: either retype the published `Io(String)` variant in place, or add a new `#[non_exhaustive]`-safe variant. M013's decision is stated in §9.1.

M013 must **not** introduce a second, competing error type for the same event (§9.1 explains why the core
`RecoveryError` mechanism does not transfer).

### 6.3 Defect C — adapter capability parity (decided)

**M013 chooses option (c): defer to a named follow-up milestone, and record the asymmetry explicitly in the interim.**

Required outcomes of the deferral:

- The architecture deep dive (`architecture/acquisition.md`, plus the eggfetch/curl adapter deep dives as their owners require) must state, in one place and unambiguously:
  - the seam contract (`AcquisitionRequest`, `AcquisitionTransport`) does **not** include a user-agent, so no seam invariant is violated by the asymmetry;
  - `eggup-eggfetch` currently exposes a caller-settable `User-Agent` (`EggfetchConfig::user_agent`, defaulting to `DEFAULT_USER_AGENT`) and `eggup-curl` exposes none;
  - the concrete consequence — a caller that switches transports loses control of the `User-Agent`, and the wire value changes from `eggup-eggfetch` to curl's own default;
  - the named follow-up milestone that will resolve the parity, which must own the 0.2/1.0 API-boundary consequence for `CurlConfig`'s public, non-`#[non_exhaustive]` fields (§3.3).
- The follow-up must be named in this plan and in the closure record, so a medium finding does not become ownerless.

Rationale for choosing (c) over (a) and (b):

- **(a) is not a corrective, it is an API-boundary change.** Making the curl adapter's `User-Agent` caller-settable requires a new field on `CurlConfig`, whose fields are public and not `#[non_exhaustive]` (`crates/eggup-curl/src/lib.rs:49-67`).
- That is source-breaking for every external struct-literal construction of `CurlConfig` on a crate whose published version is `0.1.2`, and it changes the wire request. It also requires republication. That belongs at the intentional 0.2/1.0 boundary, in the same decision class as the registry's open `CleanupDisposition` / `Error`-extensibility item, not inside a corrective.
- **(b) alone is not sufficient.** A documented difference with no owner leaves a medium finding open indefinitely. The deferral must be paired with a named follow-up, which is what makes (c) different from (b).
- **(c) keeps the seam honest.** The asymmetry is in the *adapter configuration surface*, not the seam contract, so the seam's published surface does not change for M013, and M013 does not create a silent behavioral difference between adapters that the documentation does not disclose.

M013 must not add a `User-Agent` field, builder, or `--user-agent` argument to the curl adapter. If the implementer
concludes otherwise, that is a stop condition (§14).

### 6.4 Defect D — truncation consolidation (decided)

**M013 chooses option (a): consolidate within each reachable dependency group, accepting two implementations — one per
group.**

Group 1 — `eggup-acquisition` + `eggup-curl` + `eggup-eggfetch`:

- One implementation lives in `eggup-acquisition`. The seam's own `truncate_utf8_bytes` (`:330`) is the primitive.
- The two adapters must call the seam's primitive instead of keeping their own `bound` copies (`crates/eggup-curl/src/lib.rs:408`, `crates/eggup-eggfetch/src/lib.rs:460`). Because the seam's `bound_detail` (`:326-328`) is `pub(crate)`, the adapters need an **adapter-facing** `#[doc(hidden)]` entry point, following the same `__`-prefix convention as §6.2. This is a **source-compatible public API addition** to a published crate and must be recorded in §9.3 and in the crate changelog.
- The two **inlined** copies inside `eggup-acquisition` — the walk at `:372` inside `redact_url` and the walk at `:432` inside `__scrub_upstream_text` — **must** be replaced by that crate's own `truncate_utf8_bytes` helper. This consolidation is unambiguously within the boundary and is not optional.
- The observable output must be **byte-identical**: `redact_url` keeps its 256-byte cap **and** its trailing `…` (`out.truncate(end); out.push('…')`), and `__scrub_upstream_text` keeps its 512-byte cap with no ellipsis. The existing redaction tests pinning the 256/512-byte edges (`architecture/acquisition.md` §12, Redaction row) must remain green unchanged.

Group 2 — `eggup-core` + `eggup-service`:

- `eggup-service` keeps its own implementation; its `bound_detail` (`crates/eggup-service/src/disposition.rs:291`) and `bounded_detail` (`crates/eggup-service/src/lifecycle_update.rs:684`) pair is owned by `plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`. M013 must not touch it.

Because no location is reachable by both groups (§3.5), two implementations survive. M013 must record that
unreachability proof — zero dependencies in the seam, `eggup-core` depending only on `sha2` — as the reason the
duplication is bounded rather than accidental, and must extend the existing `architecture/acquisition.md` §14 drift row
to cover the two inlined seam copies and the service-side copies. To prevent the residual copies from **drifting** in
behavior, M013 must pin each remaining copy with a test asserting its exact byte cap and its UTF-8 boundary property.

### 6.5 Defect E — recorded justification for the curl subprocess runner

- `architecture/curl-adapter.md` (and `architecture/acquisition.md` where the adapter boundary is described) must record that the curl adapter implements its own bounded subprocess runner **because it may depend only on `eggup-acquisition`**, that `eggup-core`'s `run_bounded` and `eggup-service`'s `SystemExecutor::run` are therefore unreachable from it, and that adding either dependency edge to remove the duplication would be a **boundary violation**, not a simplification.
- The record must also name the shape the runner shares with core's `run_bounded` — an unbounded post-exit reader join — and state that the deadline defect it shares is tracked by `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md` and is **out of scope for M013**. M013 must not change the curl runner's process lifecycle, deadline, kill/reap behavior, or environment clearing.

## 7. Ordered work packages

1. **WP1 — per-site vacuity triage.** Enumerate the nine sites, record the `transport()` / `real_curl()` dependency chain, and apply the §6.1 treatment per site. Split `:1183` into a deterministic total-phase test and the environment-conditional connect-phase test, and add the no-network `classify_curl_result` connect/total label proof. Verifiable: `cargo test -p eggup-curl --locked` shows no vacuous site, and each site either asserts or is explicitly marked with a reason.
2. **WP2 — the vacuity regression guard.** Add the pattern guard, the `#[ignore]`-reason guard, and the behavior witness to the three acquisition crates. Verifiable: the guard fails when an `else { return }` is injected into any of the three crates' `src/`, and passes on the M013 tree.
3. **WP3 — typed I/O failure surface.** Retype or extend the I/O variant per §6.2, add the `source()` override, add the adapter-facing constructor, migrate the eight seam sites and eleven adapter sites, and keep the 512-byte bound and the redaction invariant. Verifiable: `cargo test -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --locked`.
4. **WP4 — the two inlined truncation copies.** Replace the walks at `crates/eggup-acquisition/src/lib.rs:372` and `:432` with that crate's own `truncate_utf8_bytes`, byte-for-byte identical output. Verifiable: the redaction tests pinning the 256/512-byte edges are unchanged and green.
5. **WP5 — adapter parity decision implementation.** Apply §6.3: the architecture record of the asymmetry, the named follow-up, and the explicit absence of any curl `User-Agent` surface. Verifiable: `grep -n "user_agent\|user-agent" crates/eggup-curl/src/lib.rs` still returns nothing, and the architecture record names the follow-up.
6. **WP6 — truncation consolidation within the reachable groups.** Add the adapter-facing seam entry point, migrate `eggup-curl`'s `bound` and `eggup-eggfetch`'s `bound` to it, and pin each residual copy's cap with a test. Verifiable: `cargo tree -p eggup-curl -p eggup-eggfetch --locked` still shows no new edge, and the cap pins are green.
7. **WP7 — curl bounded-exec justification note.** Apply §6.5 to `architecture/curl-adapter.md`. Verifiable: the note names the illegal-dependency prohibition and points at core M014.
8. **WP8 — tests, changelogs, and the full gate.** Per-crate `Unreleased` changelog entries, rustdoc on the new error surface, the full local gate, the baseline-failure proof, and a fresh hosted matrix.

## 8. Failure, cancellation, restart, and contention semantics

- **An I/O failure must report** the operation that failed and the underlying `io::Error`, and `source()` must return that `io::Error`. A consumer must be able to match on `ErrorKind` (`NotFound` vs `PermissionDenied` vs `AlreadyExists`) **without parsing `Display` text**.
- **A fixture that cannot be constructed must report a failure**, never a silent exit and never a reported pass. The only exception is a case that is **explicitly environment-conditional with a recorded reason**; such a case is reported as *ignored*, with the reason recorded in the source, this plan, and the closure record — and its coverage must not be claimed as proven.
- **The `Unavailable` vs `Transport` distinction must survive the typing change.** Typing an I/O failure must not reclassify it as `Unavailable` (which would silently enable transport fallback) or as `Transport` (which would fall back under `UnavailableOrTransport`). `Io` remains terminal under every policy (`crates/eggup-acquisition/src/lib.rs:987-994`).
- **A vacuous test must never be reported as a pass.** This is the acceptance criterion for Defect A; the §6.1 guard and witness enforce it mechanically.
- **Cancellation, timeout, byte overflow, and process failure semantics are unchanged.** The curl child is still killed and reaped on every failure path (`:650`, `:658`, `:669`, `:679`), the owned temp is still removed, and no owned residue survives a failed fetch.
- **The seam's staging sequence is unchanged** — exclusive owner-private temp, no-clobber promotion, no destination creation on failure.
- **No new failure mode is introduced by the consolidation.** The shared truncation helper must not panic, must not widen a cap, and must not change any observable byte.
- **Concurrency semantics are unchanged.** `AcquisitionError` remains `Send` and `Sync`; `ComposedTransport`'s fallback budget accounting is untouched.

## 9. Compatibility and migration

**Published versions in scope:** `eggup-acquisition 0.1.3` (published; `crates/eggup-acquisition/Cargo.toml` inherits
workspace `0.1.3`), `eggup-eggfetch 0.1.3` (published), `eggup-curl 0.1.2` (published; the tree carries `0.1.3` from the
workspace).

### 9.1 The I/O-typing change — the compatibility consequence

`AcquisitionError` is `#[non_exhaustive]` (`crates/eggup-acquisition/src/lib.rs:251`), so **adding** a variant is
source-compatible for consumers. **Changing the shape of the published `Io(String)` variant is not**:
`AcquisitionError::Io(_)` stops compiling against a struct-shaped variant, and the in-tree matches at
`crates/eggup-acquisition/src/lib.rs:1421`, `:1521`, `:1569` and `crates/eggup-eggfetch/src/lib.rs:1146` are all of that
form.

Embedding a `std::io::Error` also forces a decision on the enum's derives: `AcquisitionError` currently derives `Debug,
Clone, PartialEq, Eq` (`:250`), and `std::io::Error` implements neither `Clone` nor `PartialEq`/`Eq` (verified with
`rustc 1.89.0`). M013 must make this choice **explicitly** and record it: either **(i)** drop `Clone`, `PartialEq`, and
`Eq` from `AcquisitionError`, matching `eggup-core`'s published `Error`, which derives only `Debug`
(`crates/eggup-core/src/error.rs:8`) precisely because it carries an `io::Error`; or **(ii)** preserve them through a
manual implementation over the operation plus the error's kind and rendered message, with the equality semantics
documented as textual rather than identity-based. Either way the consequence is **source-breaking** and must be stated
in the crate changelog and in the closure record.

**Does M013 follow the core M012 `RecoveryError` precedent? No — and the reason is structural, not a preference.** Core
M012 (`plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`) introduced
`RecoveryError` because the new error state was confined to **unpublished** recovery entry points while the published
`Error` enum had to stay source-compatible, and because core's `Error` was **not** `#[non_exhaustive]` (M012 `:31`,
`:65`). In M013 the typed error must ride on the **published** seam API's error type — `fetch_metadata` and
`fetch_artifact` return `AcquisitionError` — so a parallel error type cannot be returned without changing published
signatures, which core M012 explicitly forbade ("Existing public 0.1.2 methods keep signatures and result types",
M012 `:74`). M013 therefore follows core M012's **principle** — "Do not create two competing meanings for the same
event" (M012 §4.4) — by retyping the existing variant rather than adding a competing `AcquisitionIoError`, and accepts
the source break at the next version boundary. The in-repo **shape** precedent it does follow is core's `Error::Io {
operation, source }` (`crates/eggup-core/src/error.rs:15-20`) with its `source()` override (`:192-205`).

**Version consequence.** The next publication of `eggup-acquisition` is a **source-breaking** release (`0.2.0`, or the
0.2/1.0 boundary release), not a `0.1.4`. That publication is a **separate milestone**. M013 must not publish, must not
bump a published version, and must not move any tag.

### 9.2 The adapter-parity change

None in M013: no `User-Agent` surface is added to `eggup-curl`, so the published `eggup-curl 0.1.2` API and wire
behavior are unchanged. The only consequence is documentation (§6.3) plus a named follow-up milestone.

### 9.3 The truncation consolidation

- The seam gains a **`#[doc(hidden)]` adapter-facing** entry point for the shared truncation helper: a **source-compatible public API addition** to a published crate (`eggup-acquisition 0.1.3`), not a break. It must be documented as adapter-facing, not consumer contract, and must carry rustdoc (`#![deny(missing_docs)]`).
- `eggup-curl` and `eggup-eggfetch` lose their private `bound` copies. Their public API and observable behavior are unchanged.
- `eggup-acquisition`'s own public behavior is unchanged: redaction and scrub outputs stay byte-identical.

### 9.4 The `eggup-eggpack` pin

`crates/eggup-eggpack/Cargo.toml:22` pins `eggup-acquisition = "=0.1.3"` (likewise `eggup-core = "=0.1.3"` and
`eggup-archive = "=0.1.3"`). Therefore **the acquisition-seam change in §6.2 cannot reach `eggup-eggpack`** — neither
through the tree-local path nor through any publication — until `eggup-acquisition` is republished at a version the pin
accepts. A separate adapter republication milestone is therefore **required** before eggpack can carry the typed I/O
surface. M013 states this and does not perform it.

### 9.5 What M013 does not authorize

M013 authorizes no publication, no tag movement, no version bump of a published crate, no dependency addition to
`eggup-acquisition`, no change to the seam contract, and no change to the fallback policy.

## 10. Required tests

- A fixture per previously-vacuous site proving the behavior is **asserted**: for each of the nine sites, the named test must fail when the behavior it claims to cover is broken. The §6.1 witness is the mechanical form of this.
- A **pattern guard** test per acquisition crate detecting a reintroduced `else { return }` early exit in that crate's `src/`.
- A **reason guard** test per acquisition crate detecting an `#[ignore]` without a recorded reason.
- A **behavior witness** test per acquisition crate asserting each required behavior has a named, non-vacuous, asserting test.
- A fixture proving `source()` is **preserved** on an I/O failure: the returned error's `source()` is the underlying `io::Error`, and its `ErrorKind` is matchable without parsing `Display`.
- A fixture proving the **redaction of URL material still holds on the new typed path**: an acquisition failure occurring while fetching a credential-bearing URL must not surface that URL through `Display` or `source()`.
- A fixture proving the typed I/O surface is **terminal** under composition: an `Io` failure does not trigger fallback on the secondary transport under either `CompositionPolicy` (`crates/eggup-acquisition/src/lib.rs:987-994`).
- A fixture proving the **512-byte bound** and UTF-8-boundary property of the typed variant's rendered detail.
- A fixture proving the **byte-identical** output of `redact_url` (256-byte cap plus `…`) and `__scrub_upstream_text` (512-byte cap) after the WP4 consolidation — the existing edge tests must remain unchanged and green.
- A fixture proving the **cap pins** for every residual truncation copy (§6.4).
- A deterministic **no-network** fixture proving the connect/total label at the `classify_curl_result` layer by injecting `start` (§6.1), with no subprocess and no sub-second wall-clock budget.
- The full `eggup-curl`, `eggup-acquisition`, and `eggup-eggfetch` suites green, and the full workspace all-target/all-feature suite green.

**The new fixtures must FAIL against the pre-M013 implementation.** The exact command is in §11. The expected failures
are: the pattern guard reports the nine sites; the behavior witness reports the missing assertions; and the typed-I/O
fixture **fails to compile** against the baseline — that compile failure is itself the discriminating evidence and must
be recorded verbatim.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-acquisition --locked
cargo test -p eggup-curl --locked
cargo test -p eggup-eggfetch --locked
cargo clippy -p eggup-acquisition --all-targets --locked -- -D warnings
cargo clippy -p eggup-curl --all-targets --locked -- -D warnings
cargo clippy -p eggup-eggfetch --all-targets --locked -- -D warnings
cargo doc -p eggup-acquisition --no-deps --locked
cargo doc -p eggup-curl --no-deps --locked
cargo doc -p eggup-eggfetch --no-deps --locked
cargo test --workspace --all-targets --all-features --locked
cargo tree --workspace --locked
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh
git diff --check
```

The dependency-boundary review step is a **read**, not a pass/fail gate: `cargo tree -p eggup-acquisition --locked` must
show **zero** dependencies, and `cargo tree -p eggup-curl --locked` must show only `eggup-acquisition` as an Eggup
dependency.

**Proving the new fixtures fail against the pre-M013 baseline** (verbatim):

```text
git worktree add /tmp/eggup-m013-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
cp -R crates/eggup-curl/tests/. /tmp/eggup-m013-baseline/crates/eggup-curl/tests/
cp -R crates/eggup-acquisition/tests/. /tmp/eggup-m013-baseline/crates/eggup-acquisition/tests/
cp -R crates/eggup-eggfetch/tests/. /tmp/eggup-m013-baseline/crates/eggup-eggfetch/tests/
cargo test --manifest-path /tmp/eggup-m013-baseline/Cargo.toml -p eggup-curl --locked --tests
cargo test --manifest-path /tmp/eggup-m013-baseline/Cargo.toml -p eggup-acquisition --locked --tests
cargo test --manifest-path /tmp/eggup-m013-baseline/Cargo.toml -p eggup-eggfetch --locked --tests
git worktree remove /tmp/eggup-m013-baseline
```

Every one of the three baseline runs must **fail**. The pattern guard must report the nine sites; the witness must
report the missing assertions; the typed-I/O fixture must fail to compile. A baseline run that passes means the new
fixtures are not discriminating, and M013 must stop.

## 12. Documentation updates

- **rustdoc** on the new/retyped I/O error surface: the variant's operation and source fields, the `source()` behavior, the 512-byte bounded `Display`, and the statement that no URL or credential material is carried (`#![deny(missing_docs)]`).
- **rustdoc** on the new `#[doc(hidden)]` adapter-facing constructor and on the adapter-facing truncation entry point, stating that they are adapter-facing and not part of the consumer contract.
- **Per-crate `CHANGELOG.md` `Unreleased` entries** for `eggup-acquisition`, `eggup-curl`, and `eggup-eggfetch`, each carrying the repository's **no-publication / no-migration** disclaimer, and each stating the compatibility consequence of its own change: the acquisition entry states the source break and the `0.2.0` consequence; the curl and eggfetch entries state that their public API is unchanged and that the change is test-strengthening plus internal consolidation.
- **`architecture/acquisition.md`** — the adapter-parity decision (§6.3), the recorded justification for the curl bounded-exec runner (§6.5), and the extension of the §14 drift row to cover the two inlined seam copies and the service-side copies.
- **`architecture/curl-adapter.md`** — the §6.5 justification, including the explicit statement that adding an `eggup-core` or `eggup-service` dependency to remove the duplication would be a boundary violation.
- **`architecture/eggfetch-adapter.md`** — the `User-Agent` capability note, if that deep dive owns the adapter capability surface.
- **`docs/` is non-normative.** It may point at these decisions, but it must never be the place a rule is introduced, and it must not restate them.
- **Registry and roadmap** are updated at closure time (§16), not by this plan.

## 13. Acceptance criteria

M013 closes only when:

- none of the nine sites at `:1036`, `:1060`, `:1122`, `:1149`, `:1183`, `:1394`, `:1432`, `:1454`, `:1485` can report a pass without asserting, and every one has its §6.1 treatment recorded per site;
- `:1183` is split, the total-phase branch asserts `Timeout { phase: "total" }` end-to-end, the connect/total **label** is proven at the `classify_curl_result` layer with no subprocess and no wall clock, and the end-to-end connect branch is explicitly marked environment-conditional with a recorded reason;
- the pattern guard, the reason guard, and the behavior witness exist in all three acquisition crates, and each fails when its condition is violated;
- the `#[cfg(not(windows))]` curl tests remain excluded on Windows and the closure records that the Windows lane runs a smaller test set;
- the I/O failure carries the operation and the underlying `io::Error`, `source()` returns it, the redaction invariant holds on the typed path (proven by a fixture), and the `Io` variant remains terminal under both composition policies;
- the adapter-parity decision is recorded in the architecture deep dive with a **named** follow-up milestone, and no curl `User-Agent` surface was added (`grep -n "user_agent\|user-agent" crates/eggup-curl/src/lib.rs` returns nothing);
- the two inlined truncation copies inside `eggup-acquisition` are gone, and `redact_url` / `__scrub_upstream_text` output is byte-identical;
- `grep -n "else { return }" crates/eggup-curl/src/lib.rs` returns **nothing**, and the same pattern returns nothing in `eggup-acquisition` and `eggup-eggfetch`;
- `cargo tree -p eggup-acquisition --locked` shows zero dependencies, and `cargo tree -p eggup-curl --locked` shows only `eggup-acquisition` as an Eggup dependency;
- the §11 baseline proof shows all three baseline runs **failing**;
- a fresh hosted matrix is green on every lane that was run;
- no publication was performed and no tag was moved;
- the closure record states the `eggup-eggpack` `=0.1.3` pin consequence and the separate-republication requirement.

## 14. Stop conditions

Stop and escalate to the parent, rather than proceeding, if:

- **The typed I/O surface cannot be introduced without a breaking public API change that must instead be sequenced at the 0.2/1.0 boundary.** M013's decision (§9.1) already accepts a source break at the next version boundary; a stop is required only if the implementer concludes the break is larger than the variant retype — for example if it would require changing a published method signature or the `AcquisitionTransport` trait.
- **Any required consolidation would add a dependency to `eggup-acquisition`.** The seam stays at zero dependencies; if the shared helper cannot be reached without an edge, the group-1 consolidation is abandoned and §6.4's rationale is recorded with the duplication retained.
- **A fixture cannot be made deterministic.** If a required fixture cannot be made deterministic — in particular if the connect-timeout branch cannot be proven without a host-dependent wall clock — it must become explicitly environment-conditional with a recorded reason, not a silent exit, and not a wall-clock assertion of the kind M012 exists to remove.
- **The `Io` retyping would change the `Unavailable` / `Transport` classification** of any existing failure, or would make `Io` fall back.
- **The pattern guard cannot be made deterministic** without filesystem discovery or network access.
- **Adding a `User-Agent` surface to the curl adapter appears necessary** to satisfy §6.3 — that is the §6.3 deferral's stop condition; do not widen.
- **The service-side helper pair is found to be in scope** — it belongs to `service-lifecycle/012`.
- **The core `run_bounded` deadline defect is found to affect the curl adapter's own runner** — that belongs to `verified-update-core/014`; M013 must not fix it and must not depend on its closure.

Do not classify a genuine runtime failure as test flakiness, and do not weaken an assertion to keep a lane green.

## 15. Closure evidence required

- **Requirement → evidence matrix**: one row per §6 requirement, each naming the exact file:line or the exact command that proves it.
- **Implementation SHAs** and the exact diff for every changed file.
- **Production implementation evidence**: the per-site treatment table with the resulting line numbers; the retyped I/O variant and its `source()` override; the adapter-facing constructors; the consolidated truncation entry point; and the architecture records.
- **Exact commands with results** for every command in §11, and **the baseline-failure proof**: the three baseline runs from §11, their exact output, and the verbatim compile error for the typed-I/O fixture.
- **Separate Linux, macOS, and Windows results.** A lane that was **not run** is recorded as **not run** — never inferred from another lane.
- In particular, the `#[cfg(not(windows))]` curl tests are **excluded** on Windows, so the Windows lane runs a smaller test set; the excluded coverage must **not** be inferred from the Linux or macOS lanes, and the closure must say so plainly. The Windows lane's result for `cancellation_before_spawn_cleans_temp` (not `#[cfg]`-gated) must be recorded separately.
- **The per-lane record of whether a system `curl` was discoverable**, since the new fail-loudly treatment makes that an explicit precondition.
- **Invariant review** against §4, item by item; **failure / rollback / recovery review** against §8.
- **Compatibility review**: the retype's source-break consequence, the derive decision, the `0.2.0` consequence, the `#[doc(hidden)]` public API addition, the `eggup-eggpack` `=0.1.3` pin consequence, and the separate-republication requirement.
- **Security review**: redaction on the typed path; confirmation that no URL, credential, or proxy material enters the typed I/O error, its `source()`, or its `Display`; confirmation that the seam still holds no transport or service-manager dependency.
- **Documentation evidence**: rustdoc, the three per-crate `Unreleased` changelog entries, and the architecture records for the parity decision and the curl bounded-exec justification.
- **Unresolved findings with severity**, each stating the severity and the reason. The adapter-parity finding must be recorded as **deferred to a named follow-up milestone**, with that milestone's name, and must not be recorded as closed.
- **Roadmap disposition and registry updates**, including adding the new `plans/registry.md` § "Post-0.1.3 open findings" entry as closed and leaving the deferred parity item visible with its owner.

## 16. Handoff notes

**Sequencing.** M013 is **independent of M012**: M012 owns the deadline assertions in
`sub_second_total_timeout_kills_child_promptly`; M013 owns only the vacuity line at `:1394`. If M012 lands first, M013
rebases onto it and changes only the fixture-acquisition line. If M013 lands first, M012 must preserve M013's
fail-loudly fixture acquisition. **Neither may revert the other.** M013 is also **independent of**
`service-lifecycle/012` (service-side helper pair) and of `verified-update-core/014` (core bounded-execution deadline);
it must not depend on either closing and must not duplicate either's work.

**What a separate republication milestone would require.** It would need to republish `eggup-acquisition` at a
source-breaking version (`0.2.0`, or the 0.2/1.0 boundary release), then `eggup-eggfetch` and `eggup-curl` against it if
their dependency requirements move, and then `eggup-eggpack`, whose `=0.1.3` pin (`crates/eggup-eggpack/Cargo.toml:22`)
must be widened in a separate, explicitly-tracked milestone before the typed I/O surface can reach it. Each publication
is its own milestone with its own closure record, per `.opencode/skills/release-workflow`. **M013 authorizes none of
it.**

**The named follow-up for Defect C** is "Acquisition M014 — adapter capability parity (`User-Agent` and the `CurlConfig`
public-field API boundary)". It is named here so the medium finding has an owner; the plan itself is authored later and
is not part of M013.

**What this plan does NOT authorize.** No publication, no version bump of a published crate, no tag movement. No
dependency in `eggup-acquisition`, which stays at zero dependencies. No change to the seam contract, the `FetchLimits`
ceiling model, the `Unavailable` / fallback contract, or any release/fallback policy. No `User-Agent` surface in
`eggup-curl`. No embedded HTTP/TLS stack in `eggup-curl`. No change to the curl runner's process lifecycle, and no fix
to core's `run_bounded` deadline. No service-side helper consolidation. No edit to `plans/registry.md` or to any roadmap
by this plan; those are updated at closure time.

**The proof to aim for.** A reviewer must be able to run the §11 baseline command and see the new fixtures fail against
`c480541`, then run the same fixtures on the M013 tree and see them pass — with the behavior they assert being real
adapter behavior, not a fixture that silently declined to run.
