# Acquisition Transport Milestone 014 — adapter capability parity corrective

Status: implemented

Repository baseline: 20ac770dacfdcd3d82e734c22613bfccd36f9c09

Source roadmap:
- plans/subsystems/acquisition-transport-roadmap.md

Original closed work affected:
- plans/implementation/acquisition-transport/013-adapter-assertion-and-failure-typing-corrective.md
- plans/closure/acquisition-transport/013-status.md

Finding of record:
- plans/registry.md — dependency-ready implementation work, row "Acquisition transport | M014 adapter capability parity corrective": closes the adapter-capability-parity finding that M013 deferred as **UF-1 (medium)**.
- plans/closure/acquisition-transport/013-status.md — § "Unresolved findings", UF-1: "`eggup-eggfetch` exposes a caller-settable `User-Agent` (`EggfetchConfig::user_agent`, published at `0.1.3`); `eggup-curl` does not expose the same capability. The adapters are therefore **not capability-parity**. Not resolved in M013 and not claimed to be. Named follow-on: **M014**."

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md

Primary class: capability

## 1. Objective

Give the `eggup-curl` adapter the same caller capability the `eggup-eggfetch`
adapter already has: a caller-settable outbound `User-Agent` header value.

M013 recorded that the two adapters are **not capability-parity** and explicitly
declined to close the gap. This milestone closes it, in the curl adapter only,
without changing the seam's API, without changing the eggfetch adapter's API or
behavior, and without adding a dependency.

Deliverables:

1. A public, fallible way for a caller of `eggup-curl` to set the outbound
   `User-Agent` value used for **every** request the adapter makes.
2. Validation of that value **before it can reach the wire**, rejecting with
   `AcquisitionError::InvalidInput` (never `Unavailable`).
3. The validated value reaching `curl` as a literal `--user-agent <value>` argv
   element, positioned **before** the `--` separator and the URL.
4. An explicit, documented decision that the published adapter's **default wire
   behavior does not change**.
5. A documented definition of the parity that is actually achieved
   (**capability parity**, not placement parity), recorded in
   `architecture/acquisition.md`, `architecture/curl-adapter.md`, and
   `architecture/eggfetch-adapter.md`.
6. An honest statement of how parity is enforced given that a mechanical
   cross-adapter parity test is impossible.

## 2. Why this milestone is ready

- M013 is closed and its closure record names M014 as the follow-on for exactly
  this finding (`plans/closure/acquisition-transport/013-status.md`, UF-1). The
  registry already carries the M014 row with its scope, so no new authority is
  needed to start.
- The seam contract is published and frozen for this milestone
  (`eggup-acquisition 0.1.3`, pinned by `eggup-eggpack` as `=0.1.3`), so the
  only available implementation surface is the adapter itself.
- The `eggup-eggfetch` side of the comparison is already implemented, published,
  and documented (`EggfetchConfig::user_agent`), so the target behavior is
  concrete rather than hypothetical.
- The curl adapter's argv construction is already pure and directly testable
  (`build_curl_args`), and the crate already has a recording fake-`curl`
  harness, so the new behavior can be proven without a network.

## 3. Current implementation evidence

### 3.1 `eggup-curl` (the adapter being changed)

- `CurlConfig` (`crates/eggup-curl/src/lib.rs:49-67`) has **public** fields
  (`connect_timeout`, `total_timeout`, `max_redirects`, `follow_redirects`,
  `allowed_protocols`, `proxy`) and is **not** `#[non_exhaustive]`.
- `CurlConfig::validate` (`crates/eggup-curl/src/lib.rs:127-156`) is the
  existing validation choke point for config values and already rejects
  out-of-range values as `AcquisitionError::InvalidInput`.
- `CurlTransport` (`crates/eggup-curl/src/lib.rs:165-168`) is `#[derive(Debug)]`
  with **private** fields `executable: PathBuf` and `config: CurlConfig`. Its
  only construction path is `CurlTransport::with_executable`
  (`crates/eggup-curl/src/lib.rs:177-189`); `executable()` (`:192-194`) and
  `config()` (`:197-199`) are the only accessors.
- `build_curl_args` (`crates/eggup-curl/src/lib.rs:540-585`) is a pure function
  that builds the child's argv. Its tail is `--output -`,
  `--write-out %{stderr}%{http_code}`, then `--`, then the URL
  (`crates/eggup-curl/src/lib.rs:578-584`).
- `run_curl_to_file` (`crates/eggup-curl/src/lib.rs:601-762`) spawns the child
  with exactly that argv via `std::process::Command::args` — **no shell** is
  involved anywhere in this adapter, so an argv element is transmitted
  literally.
- The adapter currently emits **no** `--user-agent` element, so `curl` applies
  its own default (`User-Agent: curl/<version>`). This is the observable wire
  behavior that existing consumers of the published `eggup-curl 0.1.2` rely on.

### 3.2 `eggup-eggfetch` (reference only — not modified by this milestone)

- `pub const DEFAULT_USER_AGENT: &str = "eggup-eggfetch";`
  (`crates/eggup-eggfetch/src/lib.rs:18`).
- `EggfetchConfig` (`crates/eggup-eggfetch/src/lib.rs:52-63`) carries
  `pub user_agent: String` (`:54`), defaulted to `DEFAULT_USER_AGENT` (`:68`),
  with the builder `pub fn user_agent(mut self, agent: impl Into<String>) -> Self`
  (`:84-87`); the value is applied to the client at
  `crates/eggup-eggfetch/src/lib.rs:168`.
- `eggup-eggfetch` is published at `0.1.3`. `EggfetchConfig`'s `user_agent` is a
  **public field of a public, non-`#[non_exhaustive]` struct**: moving it, or
  removing it, is a source break for consumers of the published crate.

### 3.3 The seam (frozen for this milestone)

- `AcquisitionError::InvalidInput(String)` is the terminal caller-input variant;
  `Unavailable { .. }` is the **only** variant `ComposedTransport` falls back
  on (`crates/eggup-acquisition/src/lib.rs`, `should_fallback`).
- Therefore a rejected caller-supplied value must be `InvalidInput`: an
  `Unavailable` verdict on a malformed *input* would silently hand the request
  to a different transport.

### 3.4 Empirical evidence gathered for this plan

Two behaviors were confirmed against the local `curl 8.7.1` (macOS) with a
loopback listener, because they determine the required validation:

- `curl --user-agent <value>` takes the **next argv element as the value
  unconditionally** — a value beginning with `-` is not mistaken for an option,
  and interior spaces and parentheses are transmitted unchanged.
- `curl` does **not** sanitize the value: a value containing `CR LF` was
  emitted into the request head verbatim, producing a second, injected header
  (`X-Injected: 1`). Curl's own `--user-agent` is therefore **not** a
  header-injection defense, and the adapter's validation is what makes this
  control safe.
- An interior NUL cannot be transmitted at all: `std::process::Command::spawn`
  fails with `io::ErrorKind::InvalidInput` ("nul byte found in provided data"),
  which this adapter maps to `Unavailable` — the one variant that **falls
  back**. Rejecting NUL at validation time is what keeps a malformed value from
  silently becoming a transport fallback.

## 4. Invariants that must not regress

1. **Transport neutrality.** `eggup-curl` must not learn that `eggup-eggfetch`
   exists, and `eggup-acquisition` must not depend on any adapter. The
   dependency set of `eggup-curl` stays exactly `eggup-acquisition` (+ std).
2. **The seam's API is unchanged.** No change to `crates/eggup-acquisition/**`.
3. **No shell.** The child is spawned with `Command::args`; the new value is a
   literal argv element, never interpolated into a command string.
4. **URL isolation.** The URL keeps its `--` separator and remains the last
   argv element; the new option is emitted before it.
5. **Fallback semantics unchanged.** No change to `ComposedTransport`, to
   `CompositionPolicy`, or to which failures fall back.
6. **No authenticity claim.** A `User-Agent` is an identity string; nothing
   here implies publisher trust, signature validity, or provenance.
7. **No invented timeouts and no new deadline semantics.** The new option does
   not interact with the `min(request, adapter)` deadline clamp.
8. **Existing tests are not weakened.** No existing assertion is removed or
   relaxed; the `tests/assertion_guard.rs` pattern/reason/behavior guards must
   still pass.
9. **Existing published behavior is preserved by default.** A consumer of the
   published `eggup-curl 0.1.2` must observe no change in the bytes it sends.

## 5. Scope

### In scope

- The `eggup-curl` adapter's `User-Agent` control: public API, validation,
  argv emission, and the accessor that makes the accepted value observable.
- The crate's own tests for the above.
- `crates/eggup-curl/README.md` and `crates/eggup-curl/CHANGELOG.md`
  (`## Unreleased` only).
- `architecture/acquisition.md`, `architecture/curl-adapter.md`,
  `architecture/eggfetch-adapter.md`: the parity record and the re-alignment of
  `file:line` anchors that the code change shifts.
- This plan and its closure record.

### Explicitly out of scope

- Any change to `crates/eggup-acquisition/**` (the seam's published API at
  `0.1.3` is frozen; `eggup-eggpack` pins it as `=0.1.3`).
- Any change to `crates/eggup-eggfetch/**` (its published API and behavior at
  `0.1.3` are frozen; it is **described**, not modified).
- Any change to `crates/eggup-core/**`, `crates/eggup-service/**`,
  `crates/eggup-archive/**`, `crates/eggup-eggpack/**`.
- Any new dependency, in `eggup-curl` or anywhere else.
- Any change to the composition/fallback contract or to the acquisition error
  taxonomy.
- Any change to the published `eggup-curl 0.1.2` version, the `v0.1.2` tag, or
  any publication act. **No republication is performed by this milestone.**
- `plans/registry.md`, `plans/subsystems/**`, the root `CHANGELOG.md`,
  `architecture/overview.md`, `architecture/tooling-governance.md`,
  `scripts/**`, `.github/**`, `AGENTS.md`, `docs/**` — all owned by the parent
  session or another milestone. The intended prose for them is drafted in the
  closure record, not applied.

## 6. Required production changes

### 6.1 Placement decision: `CurlTransport`, not `CurlConfig`

**Decision: the control lives on `CurlTransport` as a private field, surfaced
through a fallible constructor and a public accessor.**

Justification, against the exact compatibility facts:

- `CurlConfig` has **public** fields and is **not** `#[non_exhaustive]`
  (`crates/eggup-curl/src/lib.rs:49-67`). Adding a field to it is therefore a
  **source-breaking** change for any consumer that constructs it with a struct
  literal, and it cannot be made `#[non_exhaustive]` without itself being
  source-breaking.
- `CurlTransport`'s fields are **private** and its only construction path is
  `with_executable` (`crates/eggup-curl/src/lib.rs:177-189`). Adding a private
  field is therefore **not** source-breaking: no external code can construct or
  destructure it.
- Consequence: the change is **purely additive at the API level**, so a future
  `eggup-curl` republication would be a non-breaking `0.1.3` (minor/patch),
  not a `0.2.0` boundary. This is the republication implication of the
  choice: **no source break is introduced, so no major-version bump is
  implied; the published `0.1.2` crate keeps its current shape and behavior
  until an explicit republication, which this milestone does not perform.**

The cost of this choice is that the `User-Agent` is *not* part of
`CurlConfig`'s "all policy lives here" surface, and `CurlTransport::config()`
does not report it. The separate accessor (`user_agent()`) is what makes the
accepted value observable, so a caller can still confirm what the adapter will
send.

### 6.2 Public API added (all additive)

```rust
pub const MAX_USER_AGENT_BYTES: usize = 512;

impl CurlTransport {
    pub fn with_executable_and_user_agent(
        executable: impl Into<PathBuf>,
        config: CurlConfig,
        user_agent: impl Into<String>,
    ) -> Result<Self, AcquisitionError>;

    pub fn user_agent(&self) -> Option<&str>;
}
```

- `with_executable_and_user_agent` is the only way to install a `User-Agent`.
  It validates the value and returns `AcquisitionError::InvalidInput` for a
  rejected one; the transport is not constructed in that case.
- `user_agent()` returns `Some(&str)` for the value the adapter will send, or
  `None` when no value is configured (the default), which means "send no
  `--user-agent` element and let `curl` apply its own default".
- `CurlTransport::with_executable` keeps its exact signature and behavior; it
  constructs a transport with **no** `User-Agent`.

### 6.3 Validation grammar and bound

The accepted grammar, stated exactly (also reproduced in the rustdoc):

```text
user-agent = 1*( VCHAR / SP / obs-text )
VCHAR      = %x21-7E          ; printable ASCII, excluding SP
SP         = %x20            ; permitted only between two non-SP characters
obs-text   = %xA0-10FFFF     ; any non-ASCII scalar value (C1 excluded);
                             ; in UTF-8 every such scalar encodes to bytes
                             ; in %x80-FF, i.e. HTTP obs-text
```

A value is **rejected** (`AcquisitionError::InvalidInput`) when any of the
following holds:

| # | Rejection rule | Rationale |
|---|---|---|
| R1 | the value is empty | an empty `User-Agent` is not a usable identity string |
| R2 | the value's UTF-8 length exceeds `MAX_USER_AGENT_BYTES` (512 bytes) | keeps the header bounded; 512 is the same magnitude the adapter already uses for bounded diagnostics |
| R3 | the value contains any C0 control character — including `CR` (`U+000D`), `LF` (`U+000A`), `NUL` (`U+0000`) and `HTAB` (`U+0009`) | `curl` does not sanitize `--user-agent`: a `CR LF` in the value is written into the request head verbatim and injects a header (see §3.4) |
| R4 | the value contains `DEL` (`U+007F`) | not a legal field-value character |
| R5 | the value contains any C1 control character (`U+0080`–`U+009F`) | not a legal field-value character |
| R6 | the value begins or ends with `SP` (`U+0020`) | an HTTP field value is delimited by optional whitespace that a recipient may strip, so such a value would not be transmitted faithfully |

Every rejection is `AcquisitionError::InvalidInput`. `Unavailable` is **never**
used for a rejected value: `Unavailable` is the one variant `ComposedTransport`
falls back on, so a malformed caller input must not be able to turn into a
silent fallback to another transport.

### 6.4 argv emission

`build_curl_args` gains a `user_agent: Option<&str>` parameter and, when it is
`Some`, appends **two** argv elements — `--user-agent` and the exact value —
after the `--noproxy` block and **before** `--output -`, `--write-out …`, `--`,
and the URL. The `--` separator and the URL keep their current positions and
meaning.

The value is a literal argv element: the child is spawned with
`Command::args`, so no shell ever sees it, and it cannot be re-parsed as an
option (curl takes the element following `--user-agent` as the value
unconditionally, verified in §3.4).

### 6.5 Default wire behavior

**Decision: the adapter sends no `User-Agent` by default** — `with_executable`
produces a transport whose argv contains no `--user-agent` element, and
`curl` applies its own default (`User-Agent: curl/<version>`).

Compatibility consequence for a consumer of the published `eggup-curl 0.1.2`:
**none**. The default path's argv is byte-for-byte what it was before this
milestone, so the bytes such a consumer sends are unchanged. The alternative
(defaulting to `eggup-curl`) would have changed the observable `User-Agent`
header of every existing consumer of the published crate, which is not
justified by any concrete need.

The **default value** therefore remains different between the two adapters
(`curl/<version>` vs `eggup-eggfetch`). That difference is deliberate and is
recorded as such; the parity claimed by this milestone is a parity of
*capability*, not of *default value*.

### 6.6 Parity definition

**Capability parity** (what this milestone achieves): a caller of either
adapter can set the outbound `User-Agent` header value for every request the
adapter makes. Both adapters expose the control, validate it before the wire,
and apply it to every request.

**Not claimed — placement parity:** the two controls do not live in the same
place. `eggup-eggfetch`'s control lives in `EggfetchConfig`
(`pub user_agent: String` plus a builder), and `eggup-eggfetch` is published at
`0.1.3`; moving that field would be a source break for published consumers,
so it is not moved. `eggup-curl`'s control lives on `CurlTransport` because
`CurlConfig`'s public fields forbid an additive field there (§6.1). The two
shapes are therefore **deliberately different**, and neither adapter may be
changed to match the other.

**Not claimed — value parity:** the accepted *value sets* differ in strictness
(the curl adapter applies the §6.3 grammar; the eggfetch adapter applies the
HTTP header library's rules). The curl adapter's grammar is a strict subset of
what a well-formed HTTP field value permits, so any value both adapters accept
behaves the same way.

### 6.7 Boundary consequence: parity cannot be tested mechanically

A mechanical cross-adapter parity test is **impossible** in this workspace and
none is claimed:

- `eggup-curl` must not know `eggup-eggfetch` exists, and `eggup-eggfetch`
  must not know `eggup-curl` exists (ADR-0003; transport neutrality).
- `eggup-acquisition` is transport-neutral with zero dependencies and must not
  depend on its adapters.

A single test that asserted "both adapters expose the same capability" would
have to import both adapters into one crate, which would violate the boundary
in one direction or the other, and a test in `eggup-acquisition` would have to
depend on both adapters.

Parity is therefore enforced by:

1. **Per-adapter tests.** Each adapter has its own tests for its own
   `User-Agent` control (this milestone adds them to `eggup-curl`; the
   eggfetch adapter's control is already tested by its own suite).
2. **A documented contract.** The capability that is promised is written down
   in `architecture/acquisition.md`, `architecture/curl-adapter.md`, and
   `architecture/eggfetch-adapter.md` — the same place the M013 closure recorded
   the asymmetry — so a reviewer compares two documented contracts rather than
   a shared assertion.
3. **A registry/roadmap record.** The parity claim and its limits are recorded
   in the milestone's closure record so the *scope* of the claim is auditable.

## 7. Ordered work packages

- **WP1 — Plan.** This document, with the placement decision (§6.1), the
  grammar and bound (§6.3), the default decision (§6.5), the parity definition
  (§6.6), and the boundary consequence (§6.7) fixed before any code is written.
- **WP2 — Production change.** `crates/eggup-curl/src/lib.rs`:
  1. `MAX_USER_AGENT_BYTES` public constant with rustdoc.
  2. `CurlTransport` gains the private `user_agent: Option<String>` field.
  3. `with_executable_and_user_agent` (fallible) and `user_agent()`
     (accessor), both with rustdoc.
  4. `validate_user_agent` (private) implementing §6.3, returning
     `AcquisitionError::InvalidInput`.
  5. `build_curl_args` gains `user_agent: Option<&str>`, validates it first
     (so a rejected value produces **no** argv), and emits
     `--user-agent <value>` before `--output -`.
  6. `run_curl_to_file` gains `user_agent: Option<&str>` and forwards it.
  7. `fetch_metadata` / `fetch_artifact` forward `self.user_agent.as_deref()`.
- **WP3 — Tests.** Add to the crate's existing test module in
  `crates/eggup-curl/src/lib.rs` (§10).
- **WP4 — Documentation.** `crates/eggup-curl/README.md`,
  `crates/eggup-curl/CHANGELOG.md` (`## Unreleased` only), and the three
  architecture documents (§12).
- **WP5 — Verification.** Run the §11 commands; record exact output.
- **WP6 — Closure.** Write `plans/closure/acquisition-transport/014-status.md`.

## 8. Failure, cancellation, restart, and contention semantics

- **Rejected value at construction.** `with_executable_and_user_agent` returns
  `Err(AcquisitionError::InvalidInput(...))`; no transport is constructed, no
  process is spawned, no file is created.
- **Rejected value at argv build.** `build_curl_args` returns
  `Err(AcquisitionError::InvalidInput(...))` before any element is pushed, so
  no argv exists and no child can be spawned with an invalid value. This is the
  choke point every request path goes through.
- **No fallback.** Because the rejection is `InvalidInput` and not
  `Unavailable`, a `ComposedTransport` will **not** silently retry the request
  on another transport.
- **Cancellation.** Unchanged. The `CancelFlag` is polled exactly where it was
  polled before; the new option adds no wait and no new blocking point.
- **Restart/retry.** Unchanged. The adapter performs one child process per
  request; the `User-Agent` is a static property of the transport, so a retry
  sends the same value.
- **Contention.** Unchanged. The new option adds no lock, no shared state, and
  no file. `CurlTransport` remains `Send`/`Sync` by virtue of its field types
  (`PathBuf`, `CurlConfig`, `Option<String>`).
- **Failure of the child.** Unchanged: exit-code and status classification
  (§8 of `architecture/curl-adapter.md`) is untouched.

## 9. Compatibility and migration

### 9.1 What is source-breaking for a consumer of the published `eggup-curl 0.1.2`

**Nothing.** The change is additive:

- `CurlConfig` is **unchanged** — no field added, no attribute added, so struct
  literals in consumer code keep compiling.
- `CurlTransport` keeps its existing public items and gains a **private**
  field, which no external code can name or construct; `with_executable`
  keeps its exact signature and behavior.
- The new items (`MAX_USER_AGENT_BYTES`, `with_executable_and_user_agent`,
  `user_agent`) are additions. A name collision with a consumer's own
  `MAX_USER_AGENT_BYTES` glob import is the only conceivable friction, and it
  is the ordinary friction of any additive API.
- `CurlTransport` gains no trait implementations and loses none.

### 9.2 Behavioral compatibility

- Default (no `User-Agent` configured): argv and wire behavior are unchanged.
- Opt-in (`User-Agent` configured): a new `--user-agent` element appears in the
  child's argv before `--` and the URL, and the peer observes the caller's
  `User-Agent` header.

### 9.3 Publication status

**No republication is performed by this milestone.** `eggup-curl` remains
published at `0.1.2` with its current shape; the `v0.1.2` tag is not moved;
no version is bumped. The published crate keeps whatever this milestone changes
only after an explicit, separately-governed republication (release workflow:
manual, dependency-ordered, with the exact-pin cascade). Because the change is
additive, such a republication would be a non-breaking `0.1.3`.

No other crate in the workspace pins `eggup-curl`
(`crates/eggup-eggpack/Cargo.toml` pins `eggup-core`, `eggup-archive`, and
`eggup-acquisition` only), so there is no pin cascade to plan.

### 9.4 Migration

None required. Adopting the control is optional and additive.

## 10. Required tests

All new tests live in the crate's existing test module in
`crates/eggup-curl/src/lib.rs`, following that module's conventions. The
crate's `tests/assertion_guard.rs` pattern guard, reason guard, and behavior
witness must still pass.

| # | Test | Asserts |
|---|---|---|
| T1 | `user_agent_is_emitted_as_a_literal_argv_element_before_the_url` | A configured value appears in the built argv as the element immediately after `--user-agent`, with the exact bytes; the `--user-agent` element index is **before** the `--` separator index, which is before the URL index |
| T2 | `user_agent_rejects_control_characters_without_producing_argv` | `\r`, `\n`, `\0`, `\t`, `\u{7F}`, and a C1 char (`\u{85}`) are each rejected with `AcquisitionError::InvalidInput`, and `build_curl_args` returns `Err` (no argv) for each |
| T3 | `user_agent_rejects_empty_and_over_bound_values` | The empty string and a value of `MAX_USER_AGENT_BYTES + 1` bytes are rejected with `AcquisitionError::InvalidInput`; a value of exactly `MAX_USER_AGENT_BYTES` bytes is accepted |
| T4 | `default_transport_emits_no_user_agent_and_keeps_the_previous_argv` | `with_executable` produces a transport with `user_agent() == None`; its argv contains no `--user-agent` element; the argv is element-for-element the previously observed one |
| T5 | `rejected_user_agent_is_a_terminal_invalid_input_not_unavailable` | The rejection is `InvalidInput` and specifically **not** `Unavailable`, so `ComposedTransport` will not fall back on it |
| T6 | `user_agent_reaches_the_wire_as_the_configured_header` | Against the crate's loopback fixture, a real `curl` run with a configured value is observed by the server as exactly that `User-Agent` header value |

T1–T5 are pure (no subprocess, no network). T6 reuses the crate's existing
loopback harness and inherits its platform gate.

The pre-existing crate test suite must still pass unchanged: no existing
assertion is removed or weakened.

## 11. Required verification commands

```sh
cd /Users/davidbowman/projects/eggup
cargo fmt -p eggup-curl -- --check
cargo clippy -p eggup-curl --all-targets --locked --offline -- -D warnings
cargo test -p eggup-curl --locked --offline
cargo doc -p eggup-curl --no-deps --locked --offline
cargo tree -p eggup-curl --locked --offline --depth 1
```

`cargo tree` is a **review step, not a gate** (it always exits 0 on a
resolvable graph); its output is recorded in the closure record as evidence
that `eggup-curl` depends only on `eggup-acquisition`.

The workspace-wide gate (`./scripts/check-local.sh`) is run by the parent
session, not by this milestone.

## 12. Documentation updates

| Document | Change |
|---|---|
| `crates/eggup-curl/README.md` | Document the `User-Agent` control: how to set it, the accepted grammar and the 512-byte bound, the default (none sent; `curl`'s own default applies), and the `InvalidInput` rejection behavior. |
| `crates/eggup-curl/CHANGELOG.md` | Add an entry under `## Unreleased` only (never a published version section): the added `User-Agent` control, the validation rules, the unchanged default wire behavior, and the no-republication statement. |
| `architecture/acquisition.md` | Update the M013 parity note **in place** (§11.1): the asymmetry is now resolved as capability parity, with the placement and default-value asymmetries explicitly retained. |
| `architecture/curl-adapter.md` | Record the new control in §3 (public surface), §4 (where the value does **not** live and why), §7.1 (argv construction), §12 (testing), and add the parity record to §13 (trade-offs). Re-align the `file:line` anchors the code change shifts. |
| `architecture/eggfetch-adapter.md` | Update the M013 parity note **in place** (§12.1) to record the resolution; the eggfetch adapter's own API and behavior are unchanged, so only the note changes. Re-align any `file:line` anchors the change shifts. |

`architecture/overview.md`, `architecture/tooling-governance.md`, `docs/**`,
the root `CHANGELOG.md`, `plans/registry.md`, and
`plans/subsystems/acquisition-transport-roadmap.md` are **not** edited here;
the intended prose for them is drafted in the closure record.

## 13. Acceptance criteria

1. A caller of `eggup-curl` can set the outbound `User-Agent` for every request
   the adapter makes, through a public API that returns
   `Result<_, AcquisitionError>`.
2. The accepted value is observable through a public accessor.
3. A value containing `CR`, `LF`, `NUL`, any other C0 control, `DEL`, or any C1
   control is rejected with `AcquisitionError::InvalidInput`, and **no** argv is
   produced for it.
4. An empty value and a value over the stated bound are rejected with
   `AcquisitionError::InvalidInput`.
5. The value is emitted as `--user-agent <value>` before the `--` separator and
   the URL, as a literal argv element with no shell involvement.
6. The default transport's argv is unchanged, and the default sends no
   `User-Agent`.
7. The exact grammar and bound are stated in the plan, in the rustdoc, and in
   the README.
8. The parity claim is defined as capability parity, explicitly not as
   placement parity, and is recorded in all three architecture documents in
   the place the M013 note already occupies.
9. The boundary consequence (no mechanical cross-adapter parity test) is
   stated, together with how parity is enforced instead.
10. `eggup-curl` still depends only on `eggup-acquisition`; the seam and the
    eggfetch adapter are unmodified.
11. All §11 commands pass; the crate's existing test suite still passes; the
    `tests/assertion_guard.rs` guards still pass.
12. The closure record exists at `plans/closure/acquisition-transport/014-status.md`
    and this plan's `Status:` header reads `implemented`.

## 14. Stop conditions

Stop and report rather than improvise if any of these occur:

- The implementation appears to require a change to
  `crates/eggup-acquisition/**` or `crates/eggup-eggfetch/**`.
- The implementation appears to require a new dependency for `eggup-curl`.
- The `User-Agent` control cannot be expressed as an additive change to
  `eggup-curl` (i.e. the only workable placement turns out to be a public
  field on `CurlConfig`).
- A test appears to require weakening or deleting an existing assertion, or
  the `tests/assertion_guard.rs` guards cannot be satisfied by the new tests.
- The default wire behavior of the published adapter would have to change for
  the control to be expressible.
- Any acceptance criterion in §13 cannot be met within this scope.

## 15. Closure evidence required

The closure record `plans/closure/acquisition-transport/014-status.md` must
carry:

1. A requirement → evidence matrix, each requirement tied to a `file:line`
   anchor and the **exact** command and its result.
2. Per-platform results reported **separately** (Linux / macOS / Windows), with
   a platform that was not run reported as *not run* — never inferred.
3. The exact verification commands from §11 and their outputs, including
   `cargo tree -p eggup-curl --locked --offline --depth 1` as the dependency
   boundary evidence.
4. An explicit statement that the **parent session** owns
   `plans/registry.md`, `plans/subsystems/acquisition-transport-roadmap.md`,
   and the root `CHANGELOG.md`, with the **intended** prose for each drafted
   (not applied) under "Registry updates" and "Roadmap disposition".
5. Unresolved findings with explicit severities, and — because
   **medium-or-higher is the blocking-severity threshold** — any finding still
   open at that severity must be named as a follow-on corrective in §16.
6. The compatibility review: exactly what is (and is not) source-breaking for
   a consumer of the published `eggup-curl 0.1.2`, and the statement that **no
   republication was performed**.
7. A documentation/operations evidence table: every document changed and what
   changed in it.
8. **No** "Parent-session verification addendum" section — that is added by the
   parent after the hosted CI qualification.

## 16. Handoff notes

- The parity asymmetry deferred by M013 is **closed by this milestone** for the
  capability of setting a `User-Agent`. It is **not** closed as placement
  parity and **not** closed as default-value parity; both remain documented,
  deliberate differences (§6.6).
- The remaining known, accepted differences between the two adapters'
  `User-Agent` controls are:
  1. **Placement** — `EggfetchConfig` (published `0.1.3`, immovable) versus
     `CurlTransport` (private field, additive). Deliberate; closing it would
     require a source break in a published crate.
  2. **Default value** — `eggup-eggfetch` versus "no `User-Agent` sent, so
     `curl`'s own default applies". Deliberate; changing it would alter the
     observable wire behavior of published `eggup-curl 0.1.2` consumers with no
     concrete benefit.
  3. **Accepted value set** — the curl adapter applies the strict grammar of
     §6.3; the eggfetch adapter applies the HTTP header library's rules. The
     curl grammar is a strict subset of a well-formed HTTP field value, so no
     value accepted by both behaves differently.
  These are **low** severity by the repository's scale (each is a documented,
  deliberate difference, not a defect) and are recorded here so they are not
  re-raised as a new finding.
- **No follow-on corrective is required by this milestone.** If the parent
  session judges any of the three differences above to warrant one, the natural
  candidate is **M016** (a republication-and-alignment milestone), whose scope
  would be: publish `eggup-curl 0.1.3` with this additive control, and — only
  if a published `eggup-eggfetch` consumer impact is acceptable — consider
  aligning the eggfetch control's placement. Neither is in scope here.
- The parent session owns all git mutations (commit, tag, publication). This
  milestone produces working-tree changes only.

## Addendum (2026-10-24) — the republication consequence is superseded by M016

**Acquisition Transport M016** — plan
`plans/implementation/acquisition-transport/016-adapter-publication-coupling-corrective.md`,
closure `plans/closure/acquisition-transport/016-status.md` — supersedes three
statements in this plan. The plan's own text is left in place, as a closed
plan's text must be; this addendum is the record of the correction.

### Superseded — §6.1 (lines 211-216): the republication is not a non-breaking `0.1.3`

This plan stated:

> Consequence: the change is **purely additive at the API level**, so a future
> `eggup-curl` republication would be a non-breaking `0.1.3` (minor/patch),
> not a `0.2.0` boundary.

**That is false once the seam's next release is published.** The seam's next
release is `eggup-acquisition 0.2.0`, and it is **source-breaking** (M013
retyped `AcquisitionError::Io` and dropped `Clone`/`PartialEq`/`Eq`). `0.2.0`
does **not** satisfy the `^0.1.0` requirement this adapter declares today, so
the adapter's `eggup-acquisition` requirement **must widen to `0.2`** at
publication time. The workspace is **version-lockstep**, so the seam's `0.2.0`
release moves the workspace version to `0.2.0` and the adapter's next
publication is therefore **`eggup-curl 0.2.0`, not `0.1.3`**. A consumer of the
seam — directly, or through this adapter once it is republished with the
widened requirement — **inherits the seam's source break** and must update its
`AcquisitionError::Io` matchers.

What §6.1 correctly established is unchanged and still true: the adapter's
**own** API change is purely additive, so it introduces no source break *of its
own*. The corrected statement is narrower, and it is about the **dependency**:
the republication is not a non-breaking `0.1.3`, because the adapter's
`eggup-acquisition` requirement must cross the seam's major boundary.

### Superseded — §9.3 (line 439): the same consequence, restated

§9.3 closed with "Because the change is additive, such a republication would
be a non-breaking `0.1.3`." The same correction applies: the republication
must widen the seam requirement to `0.2` and is published as `0.2.0`, and a
consumer inherits the seam's source break. Everything else in §9.3 — that no
republication is performed by this milestone, that the `v0.1.2` tag is not
moved, and that no version is bumped — **remains true**.

### Superseded — §6.3 (line 256): the `SP` line of the accepted grammar

§6.3's grammar block stated:

```text
SP         = %x20            ; permitted only between two non-SP characters
```

The implementation has always permitted a **run** of interior `SP` — only a
**leading** and a **trailing** `SP` are refused — so the documented rule was
stricter than the code. RFC 7230 §3.2 permits the run anyway
(`field-content = field-vchar [ 1*( SP / HTAB ) field-vchar ]`, whose interior
run is `1*`), and repeated interior `SP` cannot inject a header, become an
option, or escape the field value. **M016 corrects the documentation, not the
behavior**: the accepted value set is unchanged. The same correction applies
to the grammar block reproduced in this milestone's closure record.

The residual copies of the corrected grammar in
`crates/eggup-curl/README.md` and `architecture/curl-adapter.md` are owned by
another writer and are reported in the M016 closure record.
