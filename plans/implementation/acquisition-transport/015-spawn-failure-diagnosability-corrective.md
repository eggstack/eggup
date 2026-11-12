# Acquisition Transport Milestone 015 — spawn-failure diagnosability corrective

Status: closed 2026-10-23; the spawn cause is now attributable — the `Unavailable` detail names the `ErrorKind` and the raw OS code — with the new diagnostic assertion proven to fail against the pre-M015 baseline, and hosted-qualified on Linux, macOS, and Windows by CI run `40008039843` (commit `0cb2c53`, all five lanes green; local whole-tree gate exit 0). Closure record: `plans/closure/acquisition-transport/015-status.md`. Registry finding 1 stays **open and narrowed** by M012 and M015 rather than closed, and the published `eggup-curl 0.1.2` still carries the cause-free detail.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Branch: `plans/corrective-batch-2026-10-23`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Original closed work affected:

- `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`
- `plans/implementation/acquisition-transport/009-eggup-curl-0.1.2-publication.md`
- `plans/closure/acquisition-transport/009-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — item **1** (the `eggup-curl`
  sub-second-deadline tests are load-sensitive on a busy host). M015 **narrows**
  item 1 rather than closing it; see §16.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`

Primary class: invariant

## 1. Objective

Make an `eggup-curl` **spawn failure** diagnosable: when `Command::spawn()`
fails, the adapter's `Unavailable` diagnostic must carry the **OS cause** of the
failure — the `std::io::ErrorKind` and, when the error carries one, the raw OS
code — instead of discarding the `io::Error` and reporting a cause-free string.

The defect is a single discarded value at `crates/eggup-curl/src/lib.rs:601-603`:

```rust
let mut child = cmd.spawn().map_err(|_| {
    AcquisitionError::__adapter_unavailable(format!("curl spawn failed for {redacted}"))
})?;
```

The `io::Error` returned by `spawn()` carries `ErrorKind` and `raw_os_error()`.
The closure discards it with `|_|`, so the rendered diagnostic is identical for
every possible cause. A missing executable (`ENOENT`), a non-executable file
(`EACCES`), and a transient resource exhaustion (`EAGAIN`/`ENOMEM`) are
therefore **indistinguishable** to anyone reading the failure.

Required end state: the classification is **unchanged** — a spawn failure is
still `AcquisitionError::Unavailable` and still falls back under
`CompositionPolicy::UnavailableOnly` — while the detail names the cause, stays
**bounded**, and carries **no URL material and no credential material**.

## 2. Why this milestone is ready

- The defect is **fully characterized by reading the code**, and the exact site
  is cited in §3.2. No further diagnosis is needed to fix it.
- There is now **hosted evidence** that the cause-free diagnostic is
  operationally useless: a hosted CI run failed with a message that could not be
  attributed (§3.1). This is the first *observed* production-shaped failure in
  this defect area; the registry's finding 1 was previously reproduced only
  locally under load.
- The classification decision (spawn failure ⇒ `Unavailable`) is **deliberate
  and already documented** (`architecture/curl-adapter.md` §7.2). Only the
  **discarded cause** is the defect. M015 therefore does not reopen a settled
  design question.
- Scope is bounded: **one crate** (`eggup-curl`), one production statement, one
  new test, one architecture section, one changelog entry. No dependency change,
  no seam change, no public API change.
- Registry finding 1's *timeout-coupling* mode was closed by acquisition M012
  (`plans/closure/acquisition-transport/012-status.md`). What remains from the
  hosted failure is the **spawn-resource** mode, and its residual defect is
  **diagnosability**, not classification. M015 addresses exactly that residue.

### 2.1 The hosted evidence

A hosted CI run failed on the `Stable` lane (ubuntu-latest):

| Run | Lane | Result |
|---|---|---|
| `40004604594` | `Stable` (ubuntu-latest) | **passed** this lane on identical code |
| `40005168760` | `Stable` (ubuntu-latest) | **failed** this lane on identical code |

Failing test and exact panic text:

```text
test tests::build_curl_args_passes_sub_second_deadlines_to_fake_curl ... FAILED
thread 'tests::build_curl_args_passes_sub_second_deadlines_to_fake_curl' panicked at crates/eggup-curl/src/lib.rs:1609:14:
fake curl completes well inside the 30s budget: Unavailable("curl spawn failed for https://example.com/x")
```

Two properties of this evidence matter and must not be conflated:

1. **It is intermittent and environment-level.** The identical code passed in the
   immediately preceding hosted run. The failure is therefore *not* a
   deterministic defect in the adapter's logic.
2. **It could not be attributed.** The panic names the adapter's classification
   (`Unavailable`) and the redacted URL, and nothing else. `spawn()` failed, but
   the diagnostic does not say *why*: the executable was missing, was not
   executable, or the host was momentarily out of resources. Without the cause,
   the failure cannot be classified, triaged, or reproduced, and the finding's
   **substantive observation** — that a hosted run of this lane can fail for a
   reason the adapter cannot explain — stands unaddressed.

The registry's finding 1 records that the same test family is host-sensitive.
The hosted failure above is a **different mode** of the same test family: it is
not a `Timeout { phase: "total" }` overrun of a sub-second budget (the mode M012
closed) but a **spawn failure** whose cause the adapter discarded.

## 3. Current implementation evidence

### 3.1 The production site

`crates/eggup-curl/src/lib.rs:601-603` (inside `run_curl_to_file`):

```rust
let mut child = cmd.spawn().map_err(|_| {
    AcquisitionError::__adapter_unavailable(format!("curl spawn failed for {redacted}"))
})?;
```

The `io::Error` is dropped by the `|_|` pattern. `std::io::Error` carries two
pieces of diagnostic evidence that this site throws away:

- `io::Error::kind()` — the portable classification (`NotFound`,
  `PermissionDenied`, `WouldBlock`, …);
- `io::Error::raw_os_error()` — the platform's numeric OS code (`Some(2)` for
  `ENOENT` on Unix, `Some(2)` for `ERROR_FILE_NOT_FOUND` on Windows,
  `Some(11)` for `EAGAIN`, `Some(13)` for `EACCES`, …).

The same site existed in the reviewed baseline `c4805411f74b85158e0ae5d0a219a32c4209d98e`
at `crates/eggup-curl/src/lib.rs:604`, in the form
`AcquisitionError::Unavailable(bound(format!("curl spawn failed for {redacted}")))`.
M013 retyped the constructor to the seam's adapter-facing
`__adapter_unavailable`; the **cause-discarding behavior is identical** in
both, and the defect survives M013 untouched.

### 3.2 The classification decision is deliberate

`architecture/curl-adapter.md` §7.2 states:

> Any `spawn()` error — `ENOENT`, `EACCES`, `ENOTDIR`, and equally a real
> resource failure — is mapped to
> `AcquisitionError::Unavailable("curl spawn failed for {redacted}")`
> (`lib.rs:601-603`). This is the single most important classification decision
> in the crate: `Unavailable` is the only hard failure that `ComposedTransport`
> will fall back on under the default `CompositionPolicy::UnavailableOnly`.

The **classification** is therefore intentional and reviewed: a missing
executable and a transient resource failure are both "this transport cannot
serve the request right now", and both should fall back to the secondary
transport. M015 does not revisit it. The **defect** is that the diagnostic
discards the cause, so the reviewable record of *which* of those conditions
occurred is lost.

### 3.3 The three causes that are currently indistinguishable

| Condition | Unix errno | `ErrorKind` | Raw OS code |
|---|---|---|---|
| Executable absent | `ENOENT` | `NotFound` | `2` |
| Executable present but not executable | `EACCES` | `PermissionDenied` | `13` |
| Host momentarily out of resources | `EAGAIN` | `WouldBlock` | `11` |
| Path component is not a directory | `ENOTDIR` | `NotADirectory` | `20` |

All four render today as the identical string
`curl spawn failed for {redacted}`.

### 3.4 The fallback contract that must not change

`crates/eggup-acquisition/src/lib.rs:1034-1044` (`should_fallback`):
`Unavailable` **always** falls back; `Transport` and `Timeout` fall back only
under `CompositionPolicy::UnavailableOrTransport`; every other error — including
`Io` — is **terminal** under every policy (`architecture/acquisition.md` §6).

The consequence for this milestone is a hard constraint: the cause must be
preserved **inside the `Unavailable` detail**. Re-typing the failure as
`AcquisitionError::Io` would preserve the cause *and* destroy the fallback
contract, because `Io` is terminal under `CompositionPolicy::UnavailableOnly`.
**`Io` must not be used for a spawn failure.**

### 3.5 The existing contract that must keep passing

`crates/eggup-curl/src/lib.rs:1052-1076`
(`missing_executable_and_spawn_failure_are_unavailable`) already asserts that a
missing executable yields `AcquisitionError::Unavailable` on both
`fetch_metadata` and `fetch_artifact`, that no destination is created, and that
no residue survives. That test is **unchanged** by M015 and must keep passing
**unchanged**.

## 4. Invariants that must not regress

- **The variant does not change.** A spawn failure is `AcquisitionError::Unavailable`,
  before and after. The classification is the contract
  (`architecture/curl-adapter.md` §7.2); only the detail gains evidence.
- **The fallback contract does not change.** `Unavailable` must keep falling
  back under `CompositionPolicy::UnavailableOnly`. `Io` is terminal and must not
  be used here. No `CompositionPolicy` change, no `should_fallback` change.
- **URL redaction is preserved.** The detail keeps the existing
  `request.redacted()` form: no userinfo, no query string, no fragment, and the
  seam's 256-byte truncation with its ellipsis. The **cause segment must add no
  URL material of any kind.**
- **Credential material stays out.** The detail must never contain a password,
  a token, a query value, or a fragment value.
- **The detail stays bounded.** `AcquisitionError::__adapter_unavailable`
  applies the seam's shared 512-byte UTF-8-safe bound
  (`crates/eggup-acquisition/src/lib.rs:314-316`, `:380-382`), so the detail is
  truncated at a UTF-8 character boundary. The added cause segment must not
  push the common case over that cap.
- **No dependency change.** `eggup-curl` keeps its single dependency,
  `eggup-acquisition`. `std::io::ErrorKind` and `std::io::Error::raw_os_error`
  are in `std`; nothing is added.
- **No public API change.** No new public item, no changed signature, no new
  variant. The helper that renders the detail is **private**.
- **No authenticity, signature, or publisher-trust claim** may appear in the
  detail, in the docs, or in the changelog.
- **Process lifecycle is unchanged.** The child is still killed and reaped on
  every failure path; a spawn failure still happens before any child exists.
- **Rust 1.89 remains the MSRV.** `io::ErrorKind`'s `Debug` rendering and
  `raw_os_error()` are stable well below 1.89.

## 5. Scope

### In scope

- The `cmd.spawn()` error mapping at `crates/eggup-curl/src/lib.rs:601-603`.
- A **private** helper that renders the spawn-failure detail.
- One new test proving: the classification is still `Unavailable`; the detail
  names the `ErrorKind` and the raw OS code; the detail carries no URL or
  credential material; the detail is bounded.
- `architecture/curl-adapter.md` §7.2 and the §10 / §11 tables.
- `crates/eggup-curl/CHANGELOG.md` `Unreleased`.

### Explicitly out of scope

- **The error variant does not change.** `Unavailable` stays `Unavailable`. No
  new variant, no retyping, no `Io` substitution.
- **The composition fallback contract does not change.** `Unavailable` must keep
  falling back under `CompositionPolicy::UnavailableOnly`; `Io` is terminal
  under every policy and **must not be used** for a spawn failure. No change to
  `should_fallback`, `CompositionPolicy`, or the seam's error type.
- **No dependency change** in any crate; `eggup-acquisition` stays at zero
  dependencies and `eggup-curl` keeps exactly one.
- **No republication and no version bump.** `eggup-curl` is published at
  **`0.1.2`** and is **not republished** by this milestone. The published
  `0.1.2` artifact keeps the cause-free detail until a separate publication
  milestone ships this change.
- **No change to the seam.** `crates/eggup-acquisition/**` is not touched; the
  existing `__adapter_unavailable` constructor and its 512-byte bound are used
  as they are.
- **No change to the `Timeout` / `Transport` / `TooLarge` / `Cancelled`
  classification**, to the deadline model, to `POLL_INTERVAL`, or to any
  fallback policy.
- **No change to M012's or M013's scope.** M012 owns the deadline
  determinism work; M013 owns the typed `Io` surface and the vacuity guards.
  M015 does not re-fix, re-test, or re-document either.
- **No CI configuration, runner sizing, or lane scheduling change.** The
  hosted failure is evidence, not a CI defect.
- **No change to release/source selection, retries, mirrors, or authenticity.**
- **No change to `docs/`** (non-normative).
- **No edit to `plans/registry.md`, `plans/subsystems/*.md`, or the root
  `CHANGELOG.md`** — the parent session owns those. This plan *states* what the
  registry needs; it does not apply it.

## 6. Required production changes

### 6.1 Preserve the OS cause in the detail

The `cmd.spawn()` error mapping must carry the cause into the `Unavailable`
detail. The **exact** detail format is normative:

```text
curl spawn failed for {redacted}: {kind}[ (os error {code})]
```

- `{redacted}` — the **existing** `request.redacted()` string, unchanged and
  passed through exactly as it is today.
- `{kind}` — the `std::io::ErrorKind` rendered with its `Debug` form, e.g.
  `NotFound`, `PermissionDenied`, `WouldBlock`, `NotADirectory`.
- ` (os error {code})` — present **only** when `io::Error::raw_os_error()` is
  `Some(code)`; `code` is the decimal raw OS code.

Worked examples:

| Cause | Rendered detail |
|---|---|
| Executable absent (`ENOENT`) | `curl spawn failed for https://example.com/x: NotFound (os error 2)` |
| Not executable (`EACCES`) | `curl spawn failed for https://example.com/x: PermissionDenied (os error 13)` |
| Resource exhaustion (`EAGAIN`) | `curl spawn failed for https://example.com/x: WouldBlock (os error 11)` |

The variant stays `AcquisitionError::Unavailable`, so every consumer that
matches on the variant — and the `should_fallback` policy — behaves exactly as
before.

### 6.2 The cause segment carries no free-form OS text

The cause segment is derived **only** from `io::Error::kind()` and
`io::Error::raw_os_error()`. The OS-provided message text is **not** rendered.

Rationale: an OS message is free-form, platform-specific, and in some
situations echoes a path. Rendering it would risk carrying the executable
path (and therefore any caller-supplied material) into a diagnostic that the
crate's redaction rule governs. The `ErrorKind` discriminant plus the numeric
code is sufficient to separate `ENOENT` from `EACCES` from `EAGAIN`, is stable
across platforms, and cannot contain URL or credential material.

### 6.3 The implementation shape

The mapping must become something equivalent to:

```rust
let mut child = cmd.spawn().map_err(|e| {
    AcquisitionError::__adapter_unavailable(spawn_failure_detail(&redacted, &e))
})?;
```

with a **private** helper in `crates/eggup-curl/src/lib.rs`:

```rust
fn spawn_failure_detail(redacted: &str, error: &std::io::Error) -> String {
    let kind = format!("{:?}", error.kind());
    match error.raw_os_error() {
        Some(code) => format!("curl spawn failed for {redacted}: {kind} (os error {code})"),
        None => format!("curl spawn failed for {redacted}: {kind}"),
    }
}
```

The helper must be private (no `pub`), must not take a `&str` URL, and must
receive the **already redacted** string. It must not be given the raw URL.

### 6.4 Bounding

The helper's output is passed through
`AcquisitionError::__adapter_unavailable`, which applies the seam's shared
512-byte UTF-8-safe bound. The longest realistic detail is
`"curl spawn failed for "` (23 bytes) + a redacted URL (≤ 257 bytes including
the ellipsis) + `": WouldBlock (os error 11)"` (28 bytes) ≈ 308 bytes, so the
common case is never truncated. Truncation, if it ever applied, would still be
UTF-8-safe and would drop the **tail** (the cause segment) rather than the
redaction — which is the correct order of loss, and must not be "fixed" by
changing the bound.

## 7. Ordered work packages

1. **WP1 — the production change.** Replace the cause-discarding `map_err` at
   `crates/eggup-curl/src/lib.rs:601-603` with the cause-preserving mapping
   and add the private `spawn_failure_detail` helper. Verifiable: the variant
   is still `Unavailable` and the rendered detail names the cause.
2. **WP2 — the diagnostic test.** Add the test required by §10. Verifiable: it
   passes on the M015 tree and **fails** on the pre-M015 tree (§11).
3. **WP3 — the architecture update.** Update `architecture/curl-adapter.md`
   §7.2 (the exact detail string and the reason the cause is now preserved),
   the §10 message table row, and the §11 failure-mode table row for
   "Executable missing, non-executable, or spawn failure".
4. **WP4 — the changelog entry.** Append to `crates/eggup-curl/CHANGELOG.md`
   `Unreleased`, keeping the M012 and M013 entries intact and keeping the
   existing no-publication / no-migration disclaimer.
5. **WP5 — the closure record.** Write
   `plans/closure/acquisition-transport/015-status.md` per
   `plans/closure/README.md`, including the baseline-failure proof, the
   per-platform split, and the parent-session verification addendum.

## 8. Failure, cancellation, restart, and contention semantics

- **A spawn failure still yields `Unavailable`.** The classification is
  unchanged, so `ComposedTransport` still falls back to the secondary transport
  under the default `CompositionPolicy::UnavailableOnly`. The added evidence
  changes **what is reported**, never **what is decided**.
- **A spawn failure still leaves no residue.** The failure happens before any
  child process exists; the owned temp file is removed by the existing `Guard`
  and the artifact destination is never created.
- **The cause is evidence, not a control-flow input.** Nothing in the adapter
  branches on the cause. The adapter does not retry, does not re-select an
  executable, and does not escalate to a different transport on its own.
- **Cancellation is unchanged.** A cancellation observed before the spawn still
  returns `Cancelled` and never reaches the spawn site.
- **Timeouts are unchanged.** The spawn happens after `start`/`deadline` are
  taken; a spawn failure still returns `Unavailable`, not `Timeout`.
- **Concurrency is unchanged.** The adapter holds no shared mutable state on
  this path; `AcquisitionError` remains `Send + Sync` (the detail is a
  `String`).
- **Restart/recovery semantics are unchanged.** A spawn failure leaves no
  lock, no staging directory, and no partially written artifact for a later
  run to reconcile.

## 9. Compatibility and migration

**Published version in scope:** `eggup-curl 0.1.2` (published on crates.io on
2026-10-04; `plans/closure/acquisition-transport/009-status.md`). The working
tree carries the workspace version `0.1.3`, which is **not published** for this
crate.

### 9.1 The compatibility consequence, stated precisely

- **The public API is unchanged.** No signature, field, variant, constructor, or
  feature changes. `AcquisitionError::Unavailable(String)` is unchanged.
- **The classification is unchanged.** Every consumer that matches on the
  variant keeps working; the fallback policy keeps working.
- **Only the detail string changes.** A consumer that matched on the exact
  literal `"curl spawn failed for "` prefix still matches; a consumer that
  asserted the *entire* detail equals `"curl spawn failed for {redacted}"`
  would see a longer string. This is a **diagnostic-text** change, not a
  behavior change, and the crate's published contract does not promise a
  byte-exact detail.
- **No consumer migration is required.**
- **No republication.** The published `0.1.2` artifact keeps the cause-free
  detail. This milestone publishes nothing, bumps no published version, and
  moves no tag. A republication of `eggup-curl` — and the pin cascade it
  implies for `eggup-eggpack` — is a **separate milestone**.

### 9.2 What M015 does not authorize

M015 authorizes no publication, no tag movement, no published-version bump, no
dependency addition, no change to the seam contract, and no change to the
fallback policy.

## 10. Required tests

Exactly one new test is required, in `crates/eggup-curl/src/lib.rs`:

**`spawn_failure_detail_names_the_os_cause_and_stays_unavailable`**

It must prove, in this order:

1. **The classification is unchanged.** A fetch through a transport whose
   executable does not exist still yields `AcquisitionError::Unavailable`, and
   `err.is_unavailable()` is true. The existing
   `missing_executable_and_spawn_failure_are_unavailable` contract keeps
   passing **unchanged**.
2. **The detail names the `ErrorKind`.** The detail contains the `Debug` form
   of the `ErrorKind` observed for that spawn.
3. **The detail names the raw OS code.** When the observed error carries
   `raw_os_error()`, the detail contains `(os error {code})` with that exact
   numeric code; when it does not, the detail contains no `os error` segment.
4. **The detail carries no URL or credential material.** The URL used for the
   failing fetch carries userinfo, query, and fragment sentinels; none of them
   may appear in the detail, and the redacted form must.
5. **The detail is bounded** (≤ 512 bytes).

The test must observe the platform's own OS error for the same spawn
**independently** (its own `Command::new(<missing path>).spawn()` call) rather
than hard-coding a number, so the assertion is not circular with the adapter's
rendering and does not encode a platform-specific errno.

The test must be **deterministic**: no network, no wall-clock budget, no
`PATH` dependency, and no reliance on a real `curl` being installed.

The existing `missing_executable_and_spawn_failure_are_unavailable` test is
**not modified**.

## 11. Required verification commands

Scoped to this milestone's change (the whole-tree gate is the parent session's):

```text
cargo fmt --all -- --check
cargo clippy -p eggup-curl --all-targets --all-features --locked --offline -- -D warnings
cargo test -p eggup-curl --locked --offline
cargo doc -p eggup-curl --no-deps --locked --offline
```

The **baseline-failure proof** must show the new test **failing** against the
pre-M015 behavior. The mechanism is a detached `git worktree` at a unique path
(`/tmp/eggup-acq-m015-base-$$`), into which the M015 test is copied while the
diagnostic change is reverted **inside the copy only**:

```text
git worktree add --detach /tmp/eggup-acq-m015-base-$$ HEAD
# copy the M015 crates/eggup-curl/src/lib.rs into the worktree
# revert ONLY the spawn-failure mapping inside the worktree copy
#     (restore `map_err(|_| ...)` and drop the helper)
cargo test -p eggup-curl --locked --offline spawn_failure_detail_names_the_os_cause_and_stays_unavailable
# → must FAIL
git worktree remove --force /tmp/eggup-acq-m015-base-$$
```

The main working tree's git state must not be mutated: no `stash`, no
`checkout <path>`, no `restore`, no `commit`, no `push`, no `tag`, no `clean`,
no `reset`, and no branch creation or switch.

## 12. Documentation updates

- `architecture/curl-adapter.md` §7.2 — replace the exact detail string
  `AcquisitionError::Unavailable("curl spawn failed for {redacted}")` with the
  new format, and record **why** the cause is now preserved: the classification
  was already deliberate, but the discarded `io::Error` made `ENOENT`,
  `EACCES`, and a real resource failure indistinguishable in the only record a
  consumer has of the failure. Record the "cause segment carries no free-form
  OS text" rule (§6.2) and the bounded-length guarantee.
- `architecture/curl-adapter.md` §10 — the message table row for the spawn
  failure must show the new format and name the `ErrorKind` + raw OS code as
  the message contents.
- `architecture/curl-adapter.md` §11 — the failure-mode table row "Executable
  missing / not executable / spawn failure" must not imply a cause-free
  detail; it must state that the outcome is still `Unavailable` and that the
  detail carries the OS cause.
- `crates/eggup-curl/CHANGELOG.md` `Unreleased` — **append** an entry for this
  milestone. The M012 and M013 entries stay exactly as they are. The entry
  must keep the section's existing no-publication / no-migration disclaimer
  and must state that the variant and the fallback contract are unchanged.
- `plans/registry.md`, `plans/subsystems/acquisition-transport-roadmap.md`, and
  the root `CHANGELOG.md` — **not edited by this milestone**; the parent
  session owns them. The closure record states the exact text the parent must
  apply.

## 13. Acceptance criteria

1. `cmd.spawn()`'s `io::Error` is no longer discarded; the `Unavailable`
   detail names the `ErrorKind` and, when present, the raw OS code, in the
   exact §6.1 format.
2. The variant is still `AcquisitionError::Unavailable`; the fallback contract
   is unchanged; `Io` is not used.
3. The URL redaction is unchanged and the cause segment adds no URL or
   credential material; the detail remains bounded by the seam's 512-byte rule.
4. `missing_executable_and_spawn_failure_are_unavailable` passes **unchanged**.
5. The new test passes on the M015 tree and **fails** against the pre-M015
   behavior, with the mechanism and the exact observed failure recorded.
6. `cargo fmt --all -- --check`, the scoped clippy, `cargo test -p eggup-curl`,
   and `cargo doc -p eggup-curl` are green.
7. `architecture/curl-adapter.md` §7.2, §10, and §11 and the crate changelog
   are updated.
8. The closure record reports macOS **run**, Linux **not run**, Windows **not
   run**, and carries the parent-session verification addendum.
9. No republication, no version bump, no tag movement, no dependency change,
   and no edit outside this milestone's owned files.

## 14. Stop conditions

Stop and report rather than proceeding if any of the following holds:

- Preserving the cause requires changing the **variant** (e.g. re-typing to
  `Io`, or adding a variant). The variant is fixed by §5.
- Preserving the cause requires changing `should_fallback`,
  `CompositionPolicy`, or the seam's error type.
- The cause segment cannot be rendered without emitting URL, credential, or
  path material.
- The change requires a new dependency, a manifest edit, or a `Cargo.lock`
  update.
- The scoped clippy or fmt gate cannot be made green without touching a file
  this milestone does not own — in particular `crates/eggup-service/**`,
  `crates/eggup-acquisition/**`, or any manifest.
- The baseline-failure proof cannot be produced without mutating the main
  working tree's git state.

## 15. Closure evidence required

`plans/closure/acquisition-transport/015-status.md` must carry:

- status, source plan and roadmap, reviewed repository baseline, and
  implementation commits/PRs (none);
- an executive finding;
- a requirement→evidence matrix, one row per §6 requirement and §10 assertion;
- production implementation evidence with `file:line` references;
- the **exact** commands run and their results;
- the **baseline-failure proof**: the mechanism, the exact command, and the
  verbatim observed failure;
- invariant review (§4, item by item);
- failure/rollback/recovery review (§8);
- compatibility and migration review (§9), stating plainly that the published
  `0.1.2` artifact keeps the cause-free detail and that **no republication**
  occurred;
- security review (redaction, credential material, bounded detail);
- documentation/operations evidence (§12);
- unresolved findings with severity;
- roadmap disposition;
- registry updates — **as prose**, stating the exact text the parent must
  apply, because `plans/registry.md` is the parent's to edit;
- **per-platform results reported separately**: macOS (darwin/arm64) **run**,
  Linux **not run**, Windows **not run** — a lane not run is recorded as *not
  run* and is never inferred from another lane;
- a **"Parent-session verification addendum"** section recording that the
  whole-tree gate (`./scripts/check-local.sh`,
  `cargo test --workspace --all-targets --all-features --locked`) and the
  hosted CI lanes (`stable`, `msrv`, `service-systemd`, `macos`,
  `windows-check`) are owned by the parent session and were **not run** by this
  milestone.

## 16. Handoff notes

- **This milestone narrows registry finding 1; it does not close it.** Finding
  1's *timeout-coupling* mode — the sub-second deadline asserting the host's
  scheduler — was closed by acquisition M012
  (`plans/closure/acquisition-transport/012-status.md`). What the hosted run
  `40005168760` adds is a **second mode**: a **spawn failure** whose cause the
  adapter discarded. M015 fixes that mode's **diagnosability** only. It does
  not make the adapter's behavior deterministic under host resource pressure,
  and it does not close finding 1. The parent session owns the registry edit
  and must record finding 1 as **partially addressed** — "the hosted
  spawn-failure mode is now diagnosable; the timeout-coupling mode was closed
  by M012" — rather than as closed.
- **The finding's substantive observation is re-observed hosted evidence.** The
  registry records finding 1 as reproduced only locally under load. M015
  re-observes it in a **hosted** run: `40004604594` passed the `Stable` lane and
  `40005168760` failed it on identical code, with the exact panic text in §2.1.
  The parent session's registry edit must carry this as hosted evidence,
  attributed to those two run IDs, in addition to the local reproductions.
- **The one-line nature of the fix is deliberate.** The defect is a discarded
  value, not a design flaw. M015 must not grow into a re-classification of
  spawn failures, a retry policy, an executable re-selection policy, or a
  typed-error redesign — each of those is a different milestone with a
  different contract consequence.
- **The change is unpublished.** `eggup-curl 0.1.2` on crates.io keeps the
  cause-free detail. A republication milestone — with its `eggup-eggpack`
  pin consequence — is separate and is **not** authorized here.
- **Scope discipline.** This milestone owns
  `plans/implementation/acquisition-transport/015-spawn-failure-diagnosability-corrective.md`,
  `plans/closure/acquisition-transport/015-status.md`,
  `crates/eggup-curl/src/lib.rs`, `crates/eggup-curl/CHANGELOG.md`, and
  `architecture/curl-adapter.md`. It owns nothing else.
