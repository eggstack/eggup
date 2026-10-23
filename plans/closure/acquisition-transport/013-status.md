# Closure record — acquisition-transport M013

## Status

**Conditionally closed.**

Every behavioural defect the plan targets is fixed and proven on the native
macOS lane, and every new fixture is proven to **fail** against the
pre-M013 baseline. The condition is the same one the sibling service M013
closure carries: the **Linux** and **Windows** lanes and the hosted CI matrix
were **not run** at this stage, and the parent session owns the whole-tree
gate. Everything not run is recorded as **not run**, never inferred from the
macOS lane.

Two deviations are recorded as unresolved findings below with explicit
severity:

- **UF-1 (medium)** — the adapter-capability-parity finding (`User-Agent`) is
  **deferred, not closed**, to the named follow-up milestone *"Acquisition M014
  — adapter capability parity (`User-Agent` and the `CurlConfig` public-field
  API boundary)"*. It is an ownerless-medium risk that M013 is explicitly
  forbidden from closing (§6.3, §9.2).
- **UF-2 (low)** — `cargo test --workspace --all-targets --all-features
  --locked` and `./scripts/check-local.sh` were **not run**; the parent session
  scoped this session to the three acquisition crates. The three crates' own
  suites, clippy, fmt, doc, and the dependency-boundary review **were** run.

No publication was performed, no tag was moved, and no published version was
bumped.

## Source plan and roadmap

- Plan: `plans/implementation/acquisition-transport/013-adapter-assertion-and-failure-typing-corrective.md`
- Roadmap: `plans/subsystems/acquisition-transport-roadmap.md`
- Finding of record: `plans/registry.md` § "Post-0.1.3 open findings" — entry
  **8** (nine `eggup-curl` tests that pass while asserting nothing) and the
  sibling entries for the flattened `AcquisitionError::Io(String)` and the
  duplicated truncation helper.

The plan file's `Status:` header was updated from `ready for handoff` to
`conditionally closed` and now points at this record.

## Reviewed repository baseline

- `HEAD` = `c4805411f74b85158e0ae5d0a219a32c4209d98e`
  (`docs(service): reconcile wg-basic M004 closure`) — the same baseline the
  plan names. The tree carried **uncommitted work from earlier closed
  milestones** (core M014/M015, archive M005, service M012/M013, eggpack
  M006); those changes were treated as correct and were **not** reverted or
  modified.
- The pre-M013 baseline was materialised with `git worktree add` at a
  **unique** path (`/tmp/eggup-m013-baseline-17442`), never by mutating the
  main working tree's git state. The worktree was removed with
  `git worktree remove --force` afterwards.

## Implementation commits / PRs

**None.** No commit was created, no branch was created or switched, and no tag
was moved. The work exists as uncommitted changes in the working tree, in the
files listed under *Files touched* below, on top of the baseline above.

Diffstat for the changed files (baseline → M013 tree):

```text
 architecture/acquisition.md           | 376 ++++++++++++++++++++++++-----------
 architecture/curl-adapter.md          | 265 +++++++++++++++---------
 architecture/eggfetch-adapter.md      | 164 +++++++++------
 crates/eggup-acquisition/CHANGELOG.md |  52 +++++
 crates/eggup-acquisition/src/lib.rs   |  91 ++++++--
 crates/eggup-curl/CHANGELOG.md        |  44 +++-
 crates/eggup-curl/src/lib.rs          | 270 ++++++++++++++++++------
 crates/eggup-eggfetch/CHANGELOG.md    |  32 +++
 crates/eggup-eggfetch/src/lib.rs      |  24 +--
 9 files changed, 931 insertions(+), 387 deletions(-)
```

Plus three new untracked integration-test directories
(`crates/eggup-{acquisition,curl,eggfetch}/tests/`), this closure record, and
the plan's `Status:` header edit.

## Executive finding

The five characterized defects are closed as follows.

- **Defect A (nine vacuous `eggup-curl` tests).** All nine sites now obtain
  their fixture through a helper that **panics with an actionable message**
  naming the missing prerequisite and the remedy, instead of returning `None`
  and exiting early. The `:1183` site is **split**: the total-phase branch
  stays an end-to-end assertion of `Timeout { phase: "total" }`, and the
  connect-phase branch is **explicitly marked as ignored with a recorded
  reason** and is reported as *ignored*, never as *passed*. The connect/total
  **label** is proven deterministically at the `classify_curl_result` layer by
  injecting `start` — no subprocess, no network, no sub-second wall-clock
  budget. A pattern guard, a reason guard, and a behavior witness in **all
  three** acquisition crates make the defect class un-reintroducible.
- **Defect B (flattened `AcquisitionError::Io`).** The published `Io(String)`
  variant is **retyped in place** as `Io { operation: &'static str, source:
  std::io::Error }`, `impl std::error::Error` overrides `source()` to return
  that `io::Error`, and the seam gained an adapter-facing `__adapter_io`
  constructor. The typed variant remains **terminal** under both composition
  policies, keeps the 512-byte UTF-8-safe bounded `Display`, and carries **no**
  URL or credential material.
- **Defect C (adapter capability parity).** The plan's option (c) is applied:
  the asymmetry is **recorded explicitly** in
  `architecture/acquisition.md` §11.1, `architecture/curl-adapter.md` §13, and
  `architecture/eggfetch-adapter.md` §12.1, with the named follow-up
  milestone. **No `User-Agent` surface was added to the curl adapter** —
  `grep -n "user_agent\|user-agent" crates/eggup-curl/src/lib.rs` returns
  nothing.
- **Defect D (eight truncation copies).** The two **inlined** copies inside
  `eggup-acquisition` are gone (both now call that crate's own
  `truncate_utf8_bytes`), and both adapters delegate their private `bound`
  helpers to the seam's new adapter-facing `__adapter_bound_detail`. Output is
  **byte-identical**. The service-side pair is untouched and owned by
  `service-lifecycle/012`.
- **Defect E (curl's own bounded subprocess runner).** The written
  justification is recorded in `architecture/curl-adapter.md` §7.5 and
  `architecture/acquisition.md` §11.2, including the explicit statement that
  adding an `eggup-core` or `eggup-service` dependency edge would be a
  **boundary violation**, the shared "unbounded post-exit reader join" shape
  with core's `run_bounded`, and the pointer to
  `verified-update-core/014` as **out of scope**.

The one finding M013 is forbidden from closing is the adapter-capability
parity item; it is carried to the named follow-up (UF-1).

## Requirement → evidence matrix

| Req | Requirement | Evidence | Command / result |
|---|---|---|---|
| §6.1 (a) | Nine vacuous sites fail loudly with an actionable message | `crates/eggup-curl/src/lib.rs:915-934` (`require_curl_executable`, `require_transport`); sites at `1080`, `1104`, `1166`, `1193`, `1231`, `1276`, `1531`, `1569`, `1591`, `1622` | `cargo test -p eggup-curl --locked --offline` → 25 passed, 1 ignored, 0 failed |
| §6.1 (b) | Connect branch environment-conditional, marked, reason recorded, reported ignored | `crates/eggup-curl/src/lib.rs:1268-1290` (`#[ignore = "environment-conditional: …"]`) | `cargo test -p eggup-curl --locked --offline` → `tests::connect_phase_timeout_is_typed_end_to_end ... ignored, environment-conditional: …` |
| §6.1 | Total-phase branch asserts `Timeout { phase: "total" }` end to end | `crates/eggup-curl/src/lib.rs:1231-1266` | `cargo test -p eggup-curl --locked --offline` → `total_phase_timeout_is_typed_end_to_end ... ok` |
| §6.1 | Connect/total label proven at the `classify_curl_result` layer, no subprocess, no wall clock | `crates/eggup-curl/src/lib.rs:1307-1363` (injects `start`; whole-second elapsed values) | `cargo test -p eggup-curl --locked --offline` → `classify_curl_result_labels_connect_and_total_without_a_subprocess ... ok` |
| §6.1 | The weak `Timeout { .. }` assertion is not carried forward | `crates/eggup-curl/src/lib.rs:1259-1262` and `1288-1291` now assert the exact phase | grep of the file for `Timeout { .. }` in those two tests → none |
| §6.1 | The `#[cfg(not(windows))]` gate is preserved and its reason recorded in the test module | `crates/eggup-curl/src/lib.rs:817-824` (module-level note) | `cargo test -p eggup-curl --locked --offline` → 8 gated tests still present |
| §6.1 | Pattern guard in all three crates | `crates/eggup-acquisition/tests/assertion_guard.rs:89`, `crates/eggup-curl/tests/assertion_guard.rs:89`, `crates/eggup-eggfetch/tests/assertion_guard.rs:89` | each `crate_sources_have_no_vacuous_early_exit ... ok` |
| §6.1 | Reason guard in all three crates | same files, `every_ignored_test_records_a_reason` | each `... ok`; negative proof below |
| §6.1 | Behavior witness in all three crates | same files, `required_behaviors_have_named_asserting_tests` | each `... ok`; negative proof below |
| §6.2 | The I/O variant carries the operation and the underlying `io::Error` | `crates/eggup-acquisition/src/lib.rs:276-289` | `cargo test -p eggup-acquisition --locked --offline` → `typed_io_preserves_the_underlying_error ... ok` |
| §6.2 | `source()` is overridden and returns the `io::Error` | `crates/eggup-acquisition/src/lib.rs:356-366` | same test asserts `Error::source(&err)` downcasts to `io::Error` |
| §6.2 | Adapter-facing constructor for the typed variant | `crates/eggup-acquisition/src/lib.rs:320-327` (`__adapter_io`) | compiles in all three crates; baseline compile failure below |
| §6.2 | `Display` keeps the bounded form (≤512 bytes, UTF-8 boundary) | `crates/eggup-acquisition/src/lib.rs:347-351` | `typed_io_detail_is_bounded_and_utf8_safe ... ok` |
| §6.2 | The redaction invariant holds on the typed path | `crates/eggup-acquisition/src/lib.rs` + `tests/typed_io_contract.rs` | `typed_io_failure_does_not_surface_url_material ... ok` (acquisition and eggfetch) |
| §6.2 | The variant's shape decided explicitly and recorded | `architecture/acquisition.md` §6.1 "Derive decision"; `crates/eggup-acquisition/CHANGELOG.md` `Unreleased` | see *Compatibility review* below |
| §6.2 | No second competing error type introduced | the enum still has exactly one I/O variant (`crates/eggup-acquisition/src/lib.rs:258-297`) | `cargo doc -p eggup-acquisition --no-deps --locked --offline` → exit 0 |
| §6.3 | Parity decision recorded in one place, unambiguously | `architecture/acquisition.md` §11.1; `architecture/curl-adapter.md` §13; `architecture/eggfetch-adapter.md` §12.1 | docs read back; no code change |
| §6.3 | The named follow-up milestone is named | "Acquisition M014 — adapter capability parity (`User-Agent` and the `CurlConfig` public-field API boundary)" — in the plan §16, in `architecture/acquisition.md` §11.1, and in the eggfetch changelog | — |
| §6.3 | **No** curl `User-Agent` surface added | `crates/eggup-curl/src/lib.rs` | `grep -n "user_agent\|user-agent" crates/eggup-curl/src/lib.rs` → **no matches** |
| §6.4 | The two inlined seam copies are replaced by the crate's own primitive | `crates/eggup-acquisition/src/lib.rs:426-430` (`redact_url`) and `483-484` (`__scrub_upstream_text`) now call `truncate_utf8_bytes` (`384-392`) | `redaction_caps_are_byte_identical_after_consolidation ... ok` |
| §6.4 | The adapters call the seam's primitive, not a local copy | `crates/eggup-curl/src/lib.rs:407-415`, `crates/eggup-eggfetch/src/lib.rs:466-475` (both one-line delegations to `__adapter_bound_detail`) | `crates/eggup-acquisition/src/lib.rs:371-378` defines the single implementation |
| §6.4 | Observable output byte-identical | `redact_url` keeps the 256-byte cap **and** the trailing `…`; `__scrub_upstream_text` keeps the 512-byte cap with no ellipsis | `redaction_caps_are_byte_identical_after_consolidation ... ok`; the pre-existing `bounded_diagnostics_preserve_utf8_at_256_and_512_byte_edges` test is **unchanged** and green |
| §6.4 | Cap pins for every residual copy | `redact_url` (256 + `…`), `__scrub_upstream_text` (512), `__adapter_bound_detail` (512) — all pinned in `crates/eggup-acquisition/tests/typed_io_contract.rs` | same test |
| §6.4 | Unreachability proof recorded; §14 drift row extended | `architecture/acquisition.md` §11.2 and §14 | docs read back |
| §6.4 | Service-side pair untouched | `crates/eggup-service/src/disposition.rs`, `crates/eggup-service/src/lifecycle_update.rs` are **not** in the diff | `git status --short` shows no change to `crates/eggup-service/**` from this milestone |
| §6.5 | Curl's bounded-exec justification recorded | `architecture/curl-adapter.md` §7.5; `architecture/acquisition.md` §11.2 | docs read back |
| §6.5 | The illegal-dependency prohibition is named | both records state that adding `eggup-core` / `eggup-service` edges would be a **boundary violation** | docs read back |
| §6.5 | The shared "unbounded post-exit reader join" shape and the core M014 pointer | `architecture/curl-adapter.md` §7.5 | docs read back |
| §8 | `Io` remains terminal under both composition policies | `crates/eggup-acquisition/src/lib.rs:1034-1044` (`should_fallback`, `Io` falls into the terminal arm) | `typed_io_is_terminal_under_composition ... ok` |
| §8 | `Unavailable` / `Transport` classification unchanged by the typing change | `crates/eggup-acquisition/src/lib.rs:1034-1044`; curl's `Unavailable` sites now route through `__adapter_unavailable` with identical content | `cargo test -p eggup-curl --locked --offline` → `missing_executable_and_spawn_failure_are_unavailable ... ok`, `executable_selection_and_discovery_are_explicit ... ok` |
| §8 | `AcquisitionError` remains `Send` and `Sync` | `io::Error` is `Send + Sync`; the enum holds only `&'static str` and `io::Error` | `cargo check -p eggup-acquisition --locked --offline` → exit 0 |
| §10 | Fixture proving `source()` is preserved and `ErrorKind` is matchable | `crates/eggup-acquisition/tests/typed_io_contract.rs` (`typed_io_preserves_the_underlying_error`) | `ok` |
| §10 | Fixture proving the typed path surfaces no URL material | acquisition + eggfetch `typed_io_failure_does_not_surface_url_material` | both `ok` |
| §10 | Fixture proving terminality under composition | `typed_io_is_terminal_under_composition` | `ok` |
| §10 | Fixture proving the 512-byte bound and UTF-8 boundary | `typed_io_detail_is_bounded_and_utf8_safe` | `ok` |
| §10 | Fixture proving byte-identical redaction/scrub output | `redaction_caps_are_byte_identical_after_consolidation` | `ok` |
| §10 | Fixture proving the cap pins for every residual copy | same test | `ok` |
| §10 | Deterministic no-network fixture for the connect/total label | `crates/eggup-curl/src/lib.rs:1307-1363` | `ok` |
| §12 | Rustdoc on the retyped surface, `source()`, the bounded `Display`, and the no-URL statement | `crates/eggup-acquisition/src/lib.rs:249-297`, `320-327`, `356-366` | `cargo doc -p eggup-acquisition --no-deps --locked --offline` → exit 0, no `missing_docs` warning |
| §12 | Rustdoc on the two new adapter-facing entry points | `crates/eggup-acquisition/src/lib.rs:312-327`, `371-378` | same |
| §12 | Per-crate `Unreleased` changelog entries with the no-publication / no-migration disclaimer | `crates/eggup-acquisition/CHANGELOG.md`, `crates/eggup-curl/CHANGELOG.md`, `crates/eggup-eggfetch/CHANGELOG.md` | read back |
| §12 | The acquisition changelog states the source break and the `0.2.0` consequence; the curl and eggfetch entries state their public API is unchanged | acquisition entry: "source-breaking", "`0.2.0`, or the 0.2/1.0 boundary release", **not** a `0.1.4`; curl/eggfetch entries: "public API is unchanged" | read back |
| §12 | Architecture records for the parity decision and the bounded-exec justification | `architecture/acquisition.md` §11.1 / §11.2; `architecture/curl-adapter.md` §7.5 / §13; `architecture/eggfetch-adapter.md` §12.1 | read back |
| §12 | `docs/` is non-normative and untouched | `git status --short -- docs/` → empty | — |
| §13 | `cargo tree -p eggup-acquisition` shows zero dependencies | see *Dependency-boundary review* below | `cargo tree -p eggup-acquisition --locked --offline --depth 1` → one line, no children |
| §13 | `cargo tree -p eggup-curl` shows only `eggup-acquisition` | same | → only `eggup-acquisition` |
| §13 | The §11 baseline proof shows all three baseline runs failing | see *Baseline failure proofs* below | all three fail |
| §13 | No publication performed and no tag moved | no `cargo publish`, no `git tag` | `git tag` list unchanged; `git status` shows no new tag |
| §13 | The closure states the `eggup-eggpack` `=0.1.3` pin consequence | see *Compatibility review* below | — |

## Production implementation evidence

### Defect A — per-site treatment table

| Plan site | Plan line (pre-M013) | Treatment | Post-M013 location | Fixture acquisition |
|---|---|---|---|---|
| metadata success / not-found / server error | `:1036` | (a) fail loudly | `crates/eggup-curl/src/lib.rs:1080` | `require_transport()` |
| artifact success / not-found / server error | `:1060` | (a) fail loudly | `:1104` | `require_transport()` |
| truncated transfer is a hard failure | `:1122` | (a) fail loudly | `:1166` | `require_transport()` |
| metadata and artifact size overflow | `:1149` | (a) fail loudly | `:1193` | `require_transport()` |
| connect / total timeouts | `:1183` | **split** | total `:1231`; connect `:1276` | `require_transport()` in both |
| sub-second total timeout | `:1394` | (a) fail loudly | `:1531` | `require_transport()` |
| cancellation before spawn | `:1432` | (a) fail loudly | `:1569` | `require_transport()` |
| cancellation during body stream | `:1454` | (a) fail loudly | `:1591` | `require_transport()` |
| redirect follow and reject | `:1485` | (a) fail loudly | `:1622` | `require_curl_executable()` |

The fail-loudly helper (`crates/eggup-curl/src/lib.rs:915-934`):

```rust
fn require_curl_executable() -> PathBuf {
    match real_curl() {
        Some(exe) => exe,
        None => panic!(
            "no `curl` executable was discoverable on PATH. This test drives the real curl \
             subprocess and has nothing to assert without one. Install curl (for example \
             `brew install curl`) or put a curl executable on PATH, then re-run \
             `cargo test -p eggup-curl`."
        ),
    }
}
```

The environment-conditional connect branch (`crates/eggup-curl/src/lib.rs:1268`):

```rust
#[ignore = "environment-conditional: needs a host that blackholes the route to 10.255.255.1:81 so the connect ceiling expires instead of failing fast with ENETUNREACH/EHOSTUNREACH; the connect/total label itself is proven without a network by classify_curl_result_labels_connect_and_total_without_a_subprocess"]
```

### Defect B — the retyped I/O variant

`crates/eggup-acquisition/src/lib.rs:276-289`:

```rust
    /// A transaction-owned filesystem operation failed.
    ///
    /// The variant is **typed**: `operation` names the filesystem operation
    /// that failed and `source` preserves the underlying [`std::io::Error`].
    /// A consumer can therefore match on [`std::io::ErrorKind`] and walk the
    /// [`std::error::Error::source`] chain instead of parsing a flattened
    /// string. `operation` is a stable label and never carries URL material;
    /// the rendered detail is bounded exactly like every other variant.
    Io {
        /// The filesystem operation that failed.
        operation: &'static str,
        /// The underlying I/O error, preserved verbatim.
        source: std::io::Error,
    },
```

`crates/eggup-acquisition/src/lib.rs:356-366`:

```rust
impl std::error::Error for AcquisitionError {
    /// Returns the underlying I/O error for the `Io` variant.
    ///
    /// Every other variant is a seam-level category with no underlying cause,
    /// so it reports `None`.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
```

The thirteen adapter construction sites migrated to
`AcquisitionError::__adapter_io(operation, source)`:
`eggup-curl` at `254`, `288`, `339`, `590`, `682`, `690`, `696`, `707`, `711`,
`713`; `eggup-eggfetch` at `352`, `433`, `437`. The eight seam-internal sites
use the `pub(crate)` `AcquisitionError::io` constructor, unchanged in shape.

`__adapter_io` (`crates/eggup-acquisition/src/lib.rs:320-327`) and
`__adapter_bound_detail` (`crates/eggup-acquisition/src/lib.rs:371-378`) are
the two new adapter-facing entry points, both `#[doc(hidden)]` and both
rustdoc'd as adapter-facing, not consumer contract.

### Defect D — the consolidated truncation entry point

`crates/eggup-acquisition/src/lib.rs:371-378`:

```rust
/// Bounds an adapter-supplied diagnostic detail to 512 bytes, truncating only
/// at a UTF-8 character boundary.
///
/// `#[doc(hidden)]`: adapter-facing. This is the **single** implementation of
/// the seam's bounded-detail rule; adapters must call it instead of keeping a
/// local copy of the truncation logic.
#[doc(hidden)]
pub fn __adapter_bound_detail(detail: impl Into<String>) -> String {
    bound_detail(detail.into())
}
```

`crates/eggup-curl/src/lib.rs:407-415` and
`crates/eggup-eggfetch/src/lib.rs:466-475` are one-line delegations to it. The
two inlined seam walks are gone: `redact_url` now computes
`let truncated = out.len() > 256; out = truncate_utf8_bytes(out, 256); if
truncated { out.push('…'); }` (`:426-430`) and `__scrub_upstream_text` returns
`truncate_utf8_bytes(out, 512)` (`:483-484`).

### Defect C / E — the architecture records

- `architecture/acquisition.md` §11.1 — the adapter-capability-parity decision
  and the named follow-up.
- `architecture/acquisition.md` §11.2 — why the curl adapter owns its bounded
  subprocess runner.
- `architecture/acquisition.md` §14 — the extended drift row (the two inlined
  seam copies, the service-side pair, the unreachability proof, the cap pins).
- `architecture/curl-adapter.md` §7.5 — the same bounded-exec justification
  from the adapter's own deep dive.
- `architecture/curl-adapter.md` §13 — the `User-Agent` asymmetry from the
  curl side.
- `architecture/eggfetch-adapter.md` §12.1 — the `User-Agent` capability note
  from the adapter that owns the capability surface.
- `architecture/eggfetch-adapter.md` §9 — the typed `Io` row and the
  no-URL-material statement on the typed path.

### Regression guards

Three identical guard suites, one per crate, under `crates/<crate>/tests/`:

| Guard | What it fails on | Proof |
|---|---|---|
| `crate_sources_have_no_vacuous_early_exit` | an `else`-block early exit reintroduced in the crate's sources | baseline run 1 below |
| `every_ignored_test_records_a_reason` | an ignored test without `= "reason"` | negative proof below |
| `required_behaviors_have_named_asserting_tests` | a required behavior whose named test is missing, can exit early, or contains no assertion | baseline run 1 below |

The needles are assembled at runtime (`["else", "{", "return"].join(" ")` and
`["#[", "ignore"].join("")`) so a guard's own source can never contain the
literal pattern it forbids. The guards read the crate sources with
`include_str!` at compile time, so they need no filesystem discovery, no
network, and no wall clock.

## Exact commands run and results

All cargo invocations used `--offline` (the environment forbids network
access). The workspace was **not** mutated by any git command other than
`git status` / `git diff` / `git worktree add` / `git worktree remove`.

```text
$ cargo fmt --all -- --check
(no output)                                        → exit 0, formatting clean

$ cargo clippy -p eggup-acquisition -p eggup-curl -p eggup-eggfetch \
      --all-targets --all-features --locked --offline -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.38s
                                                 → exit 0, no warnings

$ cargo test -p eggup-acquisition --locked --offline
running 38 tests   → 38 passed; 0 failed; 0 ignored        (src/lib.rs)
running 3 tests    →  3 passed; 0 failed; 0 ignored        (tests/assertion_guard.rs)
running 5 tests    →  5 passed; 0 failed; 0 ignored        (tests/typed_io_contract.rs)
running 0 tests    →  0 passed                            (doc-tests)
                                                 → exit 0

$ cargo test -p eggup-curl --locked --offline
running 26 tests   → 25 passed; 0 failed; 1 ignored        (src/lib.rs)
running 3 tests    →  3 passed; 0 failed; 0 ignored        (tests/assertion_guard.rs)
running 0 tests    →  0 passed                            (doc-tests)
                                                 → exit 0

$ cargo test -p eggup-eggfetch --locked --offline
running 25 tests   → 25 passed; 0 failed; 0 ignored        (src/lib.rs)
running 3 tests    →  3 passed; 0 failed; 0 ignored        (tests/assertion_guard.rs)
running 2 tests    →  2 passed; 0 failed; 0 ignored        (tests/typed_io_contract.rs)
running 0 tests    →  0 passed                            (doc-tests)
                                                 → exit 0

$ cargo doc -p eggup-acquisition -p eggup-curl -p eggup-eggfetch --no-deps --locked --offline
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.97s
    Generated .../target/doc/eggup_acquisition/index.html and 2 other files
                                                 → exit 0, no rustdoc warnings

$ git diff --check
(no output)                                        → exit 0, no whitespace errors
```

### Dependency-boundary review (a **read**, not a gate)

```text
$ cargo tree -p eggup-acquisition --locked --offline --depth 1
eggup-acquisition v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-acquisition)
                            → ZERO dependencies (not even sha2)

$ cargo tree -p eggup-curl --locked --offline --depth 1
eggup-curl v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-curl)
└── eggup-acquisition v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-acquisition)
                            → only eggup-acquisition; eggup-eggfetch is unknown to it

$ cargo tree -p eggup-eggfetch --locked --offline --depth 1
eggup-eggfetch v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-eggfetch)
├── eggfetch-core v0.2.0
├── eggup-acquisition v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-acquisition)
├── futures-util v0.3.34
└── tokio v1.53.1
[dev-dependencies]
├── tokio v1.53.1 (*)
└── url v2.5.8
                            → eggup-acquisition + eggfetch-core + tokio + futures-util;
                              eggup-curl is unknown to it
```

### Manifest-integrity check

```text
$ git diff --name-only -- Cargo.toml Cargo.lock
(empty)                          → the workspace manifest and lock file are unchanged

$ git diff --name-only -- 'crates/*/Cargo.toml'
crates/eggup-eggpack/Cargo.toml
```

The single manifest entry is **pre-existing uncommitted work from the closed
eggpack M006 milestone** (it removes `sha2 = "0.10.9"` from `eggup-eggpack`),
present in `git status --short` at session start and **not** touched by this
milestone. No manifest was edited by M013; in particular
`crates/eggup-acquisition/Cargo.toml` still has an **empty** `[dependencies]`
section.

### Commands deliberately NOT run at this stage

```text
$ cargo test --workspace --all-targets --all-features --locked
$ cargo tree --workspace --locked
$ cargo +1.89.0 check --workspace --all-targets --locked
$ ./scripts/check-local.sh
```

These are §11 commands whose scope is the whole tree. The parent session
scoped this session to the three acquisition crates and owns the whole-tree
gate; the commands are therefore recorded as **not run** (see UF-2), not as
passing. A compile-level sanity check of the whole tree was not performed
either, for the same scoping reason — no other crate in the workspace names
`AcquisitionError` (verified by `grep -n "AcquisitionError" crates/` outside
the three acquisition crates → no matches), so the retype cannot break a
sibling crate's compilation.

### Platform evidence

| Lane | Status | Notes |
|---|---|---|
| **macOS** (darwin/arm64, this host) | **run — green** | `cargo fmt --all -- --check` → exit 0; clippy (3 crates, `-D warnings`) → exit 0; `cargo test` for the three crates → 38/3/5, 25(+1 ignored)/3, 25/3/2 all green; `cargo doc` (3 crates) → exit 0. A **system `curl` was discoverable** at `/usr/bin/curl` (`curl 8.7.1`), so the eight treatment-(a) tests that need a real `curl` **executed and asserted** on this lane rather than being skipped. |
| **Linux** | **not run** | No Linux host or container was available to this session. Nothing about the Linux lane is inferred from the macOS lane. |
| **Windows** | **not run** | No Windows host was available to this session. The Windows lane runs a **smaller** test set: the eight `#[cfg(not(windows))]` loopback-HTTP tests are excluded there. That excluded coverage is **not** inferred from the Linux or macOS lanes. `cancellation_before_spawn_cleans_temp` is **not** `#[cfg]`-gated, so it would run on Windows — and under the new fail-loudly treatment it needs a discoverable `curl.exe` there as well; its Windows result is **not run** and is **not** claimed. |
| **Hosted CI matrix** | **not run** | The hosted `stable` / `msrv` / `service-systemd` / `macos` / `windows-check` lanes were not triggered by this session. |

### Baseline failure proofs (plan §11)

**Mechanism.** A pristine baseline worktree was created from the plan's
baseline SHA with `git worktree add /tmp/eggup-m013-baseline-17442
c4805411f74b85158e0ae5d0a219a32c4209d98e`, and the three new `tests/`
directories were copied into it. The main working tree's git state was **not**
mutated: no stash, no checkout, no restore, no commit, no branch, no reset.
The worktree was removed with `git worktree remove --force` afterwards.

**Deviation from the plan's verbatim command.** Two, both recorded here:

1. The worktree path was made **unique** (`/tmp/eggup-m013-baseline-17442`)
   rather than the plan's fixed `/tmp/eggup-m013-baseline`, to avoid
   colliding with a concurrent run.
2. `mkdir -p` was added before each `cp -R`, because the baseline worktree has
   no `crates/*/tests/` directory to copy into and `cp -R src/. dest/`
   requires `dest` to exist.
3. `--offline` was added to the `cargo test` invocations, because this
   environment forbids network access.

**Baseline run 1 — `eggup-curl`: FAILED.**

```text
$ cargo test --manifest-path /tmp/eggup-m013-baseline-17442/Cargo.toml \
      -p eggup-curl --locked --offline --tests
...
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out   (baseline src/lib.rs)

     Running tests/assertion_guard.rs

running 3 tests
test crate_sources_have_no_vacuous_early_exit ... FAILED
test required_behaviors_have_named_asserting_tests ... FAILED
test every_ignored_test_records_a_reason ... ok

---- crate_sources_have_no_vacuous_early_exit stdout ----
thread 'crate_sources_have_no_vacuous_early_exit' panicked at crates/eggup-curl/tests/assertion_guard.rs:91:9:
src/lib.rs contains a vacuous early exit: a test returns from an `else` block without asserting anything, so it reports a pass while proving nothing

---- required_behaviors_have_named_asserting_tests stdout ----
thread 'required_behaviors_have_named_asserting_tests' panicked at crates/eggup-curl/tests/assertion_guard.rs:173:13:
src/lib.rs: `metadata_success_not_found_and_server_error` can exit early, so it may report a pass while asserting nothing

failures:
    crate_sources_have_no_vacuous_early_exit
    required_behaviors_have_named_asserting_tests

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out
error: test failed, to rerun pass `-p eggup-curl --test assertion_guard`
```

Note on honesty: the baseline's own 24 unit tests **passed** on this lane
because a system `curl` *was* discoverable here, so the fixtures were
constructible and the vacuity was not observable at runtime. The guards
detect it **structurally**, which is exactly the defect class the plan asks
to be made un-reintroducible. The pattern guard reports the presence of the
vacuous early exit; it does not count the sites, so the failure message names
the condition rather than the nine sites.

**Baseline run 2 — `eggup-acquisition`: FAILED to compile.** Verbatim:

```text
$ cargo test --manifest-path /tmp/eggup-m013-baseline-17442/Cargo.toml \
      -p eggup-acquisition --locked --offline --tests
   Compiling eggup-acquisition v0.1.3 (/tmp/eggup-m013-baseline-17442/crates/eggup-acquisition)
error[E0425]: cannot find function `__adapter_bound_detail` in crate `eggup_acquisition`
   --> crates/eggup-acquisition/tests/typed_io_contract.rs:235:38
    |
235 |     let bounded = eggup_acquisition::__adapter_bound_detail("c".repeat(600));
    |                                      ^^^^^^^^^^^^^^^^^^^^^^ not found in `eggup_acquisition`

error[E0425]: cannot find function `__adapter_bound_detail` in crate `eggup_acquisition`
   --> crates/eggup-acquisition/tests/typed_io_contract.rs:237:38
    |
237 |     let bounded = eggup_acquisition::__adapter_bound_detail(format!("{}étail", "c".repeat(511)));
    |                                      ^^^^^^^^^^^^^^^^^^^^^^ not found in `eggup_acquisition`

error[E0769]: tuple variant `AcquisitionError::Io` written as struct variant
  --> crates/eggup-acquisition/tests/typed_io_contract.rs:53:9
   |
53 |         AcquisitionError::Io { operation, source } => (operation, source),
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: use the tuple variant pattern syntax instead
   |
53 -         AcquisitionError::Io { operation, source } => (operation, source),
53 +         AcquisitionError::Io(_) => (operation, source),
   |

error[E0599]: no variant or associated item named `__adapter_io` found for enum `AcquisitionError` in the current scope
  --> crates/eggup-acquisition/tests/typed_io_contract.rs:63:27
   |
63 |         AcquisitionError::__adapter_io("reading artifact parent", io::Error::from_raw_os_error(2));
   |                           ^^^^^^^^^^^^ variant or associated item not found in `AcquisitionError`
   |
note: if you're trying to build a new `AcquisitionError`, consider using `AcquisitionError::__adapter_unavailable` which returns `AcquisitionError`
   --> /private/tmp/eggup-m013-baseline-17442/crates/eggup-acquisition/src/lib.rs:296:5
    |
296 |     pub fn __adapter_unavailable(detail: impl Into<String>) -> Self {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

error[E0599]: no variant or associated item named `__adapter_io` found for enum `AcquisitionError` in the current scope
  --> crates/eggup-acquisition/tests/typed_io_contract.rs:85:33
   |
85 |     let err = AcquisitionError::__adapter_io("writing part file", io::Error::other(long_message));
   |                                 ^^^^^^^^^^^^ variant or associated item not found in `AcquisitionError`
   |
... (3 further identical E0599 sites) ...
error: could not compile `eggup-acquisition` (test "typed_io_contract") due to 7 previous errors
```

**Baseline run 3 — `eggup-eggfetch`: FAILED to compile.** Verbatim:

```text
$ cargo test --manifest-path /tmp/eggup-m013-baseline-17442/Cargo.toml \
      -p eggup-eggfetch --locked --offline --tests
error[E0769]: tuple variant `AcquisitionError::Io` written as struct variant
  --> crates/eggup-eggfetch/tests/typed_io_contract.rs:62:9
   |
62 |         AcquisitionError::Io { operation, source } => (*operation, source),
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
help: use the tuple variant pattern syntax instead
   |
62 -         AcquisitionError::Io { operation, source } => (*operation, source),
62 +         AcquisitionError::Io(_) => (*operation, source),
   |

For more information about this error, try `rustc --explain E0769`.
error: could not compile `eggup-eggfetch` (test "typed_io_contract") due to 1 previous error
```

**All three baseline runs failed**, as plan §11 requires.

**Additional negative proof — the reason guard fires.** In the baseline
worktree only, an `#[ignore]` attribute with **no** recorded reason was
injected into `crates/eggup-curl/src/lib.rs`, and the guard was run:

```text
$ cargo test --manifest-path /tmp/eggup-m013-baseline-17442/Cargo.toml \
      -p eggup-curl --locked --offline --test assertion_guard every_ignored_test_records_a_reason
running 1 test
test every_ignored_test_records_a_reason ... FAILED

thread 'every_ignored_test_records_a_reason' panicked at crates/eggup-curl/tests/assertion_guard.rs:108:13:
src/lib.rs contains an ignored test without a recorded reason; every ignored test must carry an explicit reason and be reported as ignored, never as passed

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out
```

The baseline worktree was then removed in full (`git worktree remove --force`),
including the injected line.

## Invariant review

| §4 invariant | Status | Evidence |
|---|---|---|
| SHA-256 is integrity evidence only; no authenticity or signature claim | **held** | M013 touches no digest, verification, or signature path. The seam still computes no digest (`architecture/acquisition.md` §6). |
| Ownership `Absent \| Owned \| Foreign \| Unknown` is caller-proven; `Owned` is never inferred | **held** | M013 touches no ownership code. |
| `commit` revalidates ownership and staged digests under lock; failures fail closed with a receipt | **held** | M013 touches no core code (`crates/eggup-core/**` is not in the diff). |
| `validate` requires every member to carry `IntegrityStatus::Verified` | **held** | Not touched. |
| Core never creates live destination parents and never decides a lock is stale | **held** | Not touched. |
| A program can replace the executable it is running through the same transaction model | **held** | Not touched. |
| `eggup-eggpack` archives stop at extraction-required evidence | **held** | Not touched. |
| The seam stays at **zero dependencies** | **held** | `cargo tree -p eggup-acquisition --locked --offline --depth 1` → no children. |
| Adapters do not know each other; composition lives in the seam | **held** | `cargo tree` output above; no new edge. |
| Transport-neutrality (ADR-0003) | **held** | The curl adapter still spawns an external process and embeds no HTTP/TLS stack; the eggfetch adapter still embeds one. No seam change. |

## Failure / rollback / recovery review

| §8 concern | Status | Evidence |
|---|---|---|
| An I/O failure reports the operation and the underlying `io::Error`; `source()` returns it; `ErrorKind` is matchable without parsing `Display` | **satisfied** | `crates/eggup-acquisition/src/lib.rs:276-289`, `:356-366`; `typed_io_preserves_the_underlying_error`. |
| A fixture that cannot be constructed reports a failure, never a silent exit | **satisfied** | All nine sites fail loudly; the one environment-conditional case is explicitly ignored with a recorded reason and its coverage is **not** claimed. |
| Typing an I/O failure does not reclassify it as `Unavailable` or `Transport`; `Io` stays terminal under every policy | **satisfied** | `should_fallback` (`crates/eggup-acquisition/src/lib.rs:1034-1044`) is untouched; `Io` falls into the terminal arm. `typed_io_is_terminal_under_composition` proves it for both policies. |
| Cancellation, timeout, byte overflow, and process-failure semantics are unchanged | **held** | The curl child is still killed and reaped on every failure path; the owned temp is still removed. Only the *error construction* changed, not the control flow. |
| The seam's staging sequence is unchanged | **held** | Exclusive owner-private temp, no-clobber promotion, no destination creation on failure — the fixture path is untouched apart from the typed error. |
| The shared truncation helper must not panic, must not widen a cap, and must not change any observable byte | **held** | `truncate_utf8_bytes` is the pre-existing primitive, unchanged; the two inlined walks were replaced by calls to it. Byte-identical output is pinned by `redaction_caps_are_byte_identical_after_consolidation` and by the unchanged pre-existing edge test. |
| Concurrency semantics unchanged; `AcquisitionError` remains `Send` and `Sync` | **held** | The enum holds `&'static str` and `io::Error`, both `Send + Sync`; `ComposedTransport`'s budget accounting is untouched. |

**Rollback.** The change is a retyping plus a bounded-consolidation; reverting
is a source revert of the three `src/lib.rs` files and the removal of the three
`tests/` directories. The architecture and changelog edits are documentation
only. No state is persisted, no migration is required, and no data format
changes.

**Recovery.** There is no runtime recovery path to change: the seam has no
crash journal, no retry-on-I/O-error path, and no partial-commit state.

## Compatibility and migration review

**The retype's source-break consequence.** `AcquisitionError::Io(_)` stops
compiling against the struct-shaped variant. A consumer must write
`AcquisitionError::Io { .. }`. This is a **source break**, recorded in
`crates/eggup-acquisition/CHANGELOG.md` `Unreleased`.

**The derive decision (explicit, as §9.1 requires).** M013 chooses **option
(i)**: `Clone`, `PartialEq`, and `Eq` are **dropped** from `AcquisitionError`,
which now derives **only** `Debug`. `std::io::Error` implements none of the
three, and this matches `eggup-core`'s published `Error`, which derives only
`Debug` for exactly the same reason. Option (ii) — preserving them through a
manual implementation over the operation plus the error's kind and rendered
message — was **rejected** because it would have to fabricate a *different*
`io::Error` on `clone`, so a clone's `source()` would no longer be the error the
caller was given, which is the property this variant exists to provide. The
decision is recorded in `architecture/acquisition.md` §6.1 and in the crate
changelog.

**The `0.2.0` consequence.** The next publication of `eggup-acquisition` is a
**source-breaking** release (`0.2.0`, or the 0.2/1.0 boundary release), **not**
a `0.1.4`. M013 performs **no** publication, **no** version bump, and **no**
tag movement.

**The `#[doc(hidden)]` public API addition.** `__adapter_io` and
`__adapter_bound_detail` are **source-compatible public API additions** to the
published `eggup-acquisition 0.1.3`, documented as adapter-facing and not
consumer contract. Both carry rustdoc.

**The adapter crates' public APIs are unchanged.** `eggup-curl` and
`eggup-eggfetch` gained no public item and lost none; their private `bound`
helpers now delegate to the seam. Their observable behavior, error
classification, wire behavior, and `User-Agent` surface are unchanged.

**The `eggup-eggpack` `=0.1.3` pin consequence.**
`crates/eggup-eggpack/Cargo.toml:22` pins `eggup-acquisition = "=0.1.3"`
(likewise `eggup-core` and `eggup-archive`). Therefore the acquisition-seam
change in §6.2 **cannot reach `eggup-eggpack`** — neither through the
tree-local path nor through any publication — until `eggup-acquisition` is
republished at a version the pin accepts. **A separate adapter
republication milestone is required** before eggpack can carry the typed I/O
surface; that milestone would have to widen the `=0.1.3` pin in its own,
explicitly-tracked scope. M013 states this and does not perform it. Note that
`eggup-eggpack` does not name `AcquisitionError` at all, so the retype does
not break its compilation.

**Migration.** Consumers of `eggup-acquisition` must, at the next version
boundary, (a) match `AcquisitionError::Io { .. }` instead of
`AcquisitionError::Io(_)`, and (b) stop relying on `Clone` / `PartialEq` /
`Eq` on `AcquisitionError`. Consumers of the two adapters need **no**
migration.

## Security review

| Concern | Status | Evidence |
|---|---|---|
| Redaction on the typed path | **held** | The typed `Io` variant carries a static `operation` label and the raw `io::Error`; neither can contain request material. Proven by `typed_io_failure_does_not_surface_url_material` in both the acquisition and eggfetch suites: a failure raised while fetching a credential-bearing URL surfaces neither the userinfo nor the query through `Display` or `source()`. |
| No URL, credential, or proxy material enters the typed I/O error, its `source()`, or its `Display` | **held** | Same fixture, plus the static nature of `operation`. |
| The seam still holds no transport or service-manager dependency | **held** | `cargo tree -p eggup-acquisition --locked --offline --depth 1` → zero dependencies. |
| The curl adapter still embeds no HTTP/TLS stack and still clears the child environment | **held** | Unchanged; `env_clear()` untouched. |
| The exact URL in the child's argv (pre-existing exposure) | **unchanged, disclosed** | `architecture/curl-adapter.md` §7.4 already records it. M013 changed no process lifecycle. |
| SHA-256 remains integrity evidence only | **held** | Not touched. |

## Documentation / operations evidence

| Item | Status |
|---|---|
| Rustdoc on the retyped I/O surface (operation field, source field, `source()` behaviour, the bounded `Display`, the no-URL statement) | **done** — `crates/eggup-acquisition/src/lib.rs:249-297`, `:339-354`, `:356-366`; `cargo doc` exit 0 |
| Rustdoc on `__adapter_io` and `__adapter_bound_detail`, stating they are adapter-facing and not consumer contract | **done** — `:312-327`, `:371-378` |
| `crates/eggup-acquisition/CHANGELOG.md` `Unreleased` — typed I/O surface, source break, `0.2.0` consequence, the `#[doc(hidden)]` addition, the truncation consolidation, and the no-publication disclaimer | **done** |
| `crates/eggup-curl/CHANGELOG.md` `Unreleased` — the nine fail-loudly sites, the split, the guard suite, the truncation delegation, "public API is unchanged", no-publication/no-migration disclaimer | **done** |
| `crates/eggup-eggfetch/CHANGELOG.md` `Unreleased` — the typed I/O surface, the truncation delegation, the guard suite, "public API is unchanged", the parity deferral, no-publication/no-migration disclaimer | **done** |
| `architecture/acquisition.md` — parity decision (§11.1), curl bounded-exec justification (§11.2), §14 drift row extended, the typed I/O subsection (§6.1), and the line anchors refreshed | **done** |
| `architecture/curl-adapter.md` — §7.5 bounded-exec justification, §13 parity record, §10 typed `Io` row, §12 rewritten for the new fixture discipline and the guard suite, the §14 drift row for `Unavailable` marked resolved | **done** |
| `architecture/eggfetch-adapter.md` — §12.1 `User-Agent` capability note, §9 typed `Io` row and the typed-path redaction statement, §11 integration-suite rows, line anchors refreshed | **done** |
| `docs/` (non-normative) | **untouched** — `git status --short -- docs/` is empty; no rule is introduced there |
| `crates/eggup-{acquisition,curl,eggfetch}/README.md` | **unchanged** — the READMEs describe the transport contract, the policy surface, and the redaction posture; none of them documented the `Io` variant's shape or the `Clone`/`PartialEq`/`Eq` derives, so no README claim became false. The three `Unreleased` changelog entries carry the compatibility consequence instead. |

## Unresolved findings

### UF-1 — The adapter-capability-parity finding is deferred, not closed — **medium**

`eggup-eggfetch` exposes a caller-settable `User-Agent`
(`EggfetchConfig::user_agent`, default `DEFAULT_USER_AGENT`); `eggup-curl`
exposes none. A caller that switches transports therefore loses control of the
`User-Agent`, and the wire value changes from the eggfetch default to curl's
own default.

**Severity: medium.** The asymmetry is in the *adapter configuration surface*,
not in the seam contract, so no seam invariant is violated and no consumer
currently breaks. It is nevertheless a real behavioural difference between two
interchangeable transports.

**Disposition: deferred to a named follow-up milestone — "Acquisition M014 —
adapter capability parity (`User-Agent` and the `CurlConfig` public-field API
boundary)"** — and **not** closed by M013. The plan forbids adding the
`User-Agent` surface here: `CurlConfig`'s fields are public and not
`#[non_exhaustive]`, so adding one is source-breaking for every external
struct-literal construction and belongs at the intentional 0.2/1.0 boundary.
The asymmetry is recorded in `architecture/acquisition.md` §11.1,
`architecture/curl-adapter.md` §13, `architecture/eggfetch-adapter.md` §12.1,
and in `crates/eggup-eggfetch/CHANGELOG.md`. The follow-up milestone owns the
API-boundary consequence. **This item must remain visible with its owner in
`plans/registry.md`.**

### UF-2 — The whole-tree gate and the non-macOS lanes were not run — **low**

`cargo test --workspace --all-targets --all-features --locked`,
`cargo tree --workspace --locked`, `cargo +1.89.0 check --workspace
--all-targets --locked`, and `./scripts/check-local.sh` were **not run**, and
the Linux, Windows, and hosted-CI lanes were **not run**.

**Severity: low** for the milestone's own scope (the three acquisition crates
are fully green on the native lane with a discriminating baseline proof), but
the acceptance criteria that depend on those commands are **not** met by this
record and must be closed by the parent session's whole-tree gate. Nothing
about the unrun lanes is inferred from the macOS lane.

### UF-3 — The service-side truncation pair is not pinned by an M013 test — **low**

The group-2 copies — `eggup-service`'s `bound_detail`
(`crates/eggup-service/src/disposition.rs:291`) and `bounded_detail`
(`crates/eggup-service/src/lifecycle_update.rs:684`) — remain, by design:
they are owned by
`plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`
and the plan forbids M013 from touching them. The plan's §6.4 requirement to
"pin each remaining copy with a test asserting its exact byte cap and its UTF-8
boundary property" is therefore met **for the group-1 copies** (the seam's
`redact_url` at 256 bytes + `…`, `__scrub_upstream_text` at 512 bytes, and
`__adapter_bound_detail` at 512 bytes, all pinned in
`crates/eggup-acquisition/tests/typed_io_contract.rs`) but **not** for the two
service-side copies, which are pinned by their own owner. This is recorded
rather than silently skipped.

### Adjacent findings (recorded, deliberately not fixed here)

- **The `eggup-curl` `Unbound`/`Unavailable` construction sites were changed
  from direct construction to `__adapter_unavailable`.** The plan permits this
  ("they may be routed through the seam's bounding helper, but their
  classification and content must not change"). Classification and rendered
  content are byte-identical, because `bound_detail` and the adapter's former
  `bound` implemented the same rule. The curl deep dive's drift row that
  recorded the old direct construction is now marked resolved.
- **Line anchors in the three architecture deep dives were refreshed** because
  the seam and adapter sources moved. The anchors were remapped against the
  baseline with a diff-based mapping and then verified against the current
  sources; the bare-number tables (the API surface tables, the staging-sequence
  table, the test tables) were recomputed from the current sources. The
  eggfetch deep dive's own pre-existing drift row about stale offsets was
  updated (1229 lines).

## Roadmap disposition

`plans/subsystems/acquisition-transport-roadmap.md` and
`plans/registry.md` are **owned by the parent session** and were **not** edited
by this milestone (the plan §16 explicitly reserves them for closure time).
The parent session should:

1. mark the acquisition-transport M013 row **closed (conditionally)**, with
   the pointer to this record;
2. mark the "Post-0.1.3 open findings" entry **8** (the nine vacuous
   `eggup-curl` tests) as **closed**, with a pointer to this record;
3. leave the **adapter-capability-parity** finding **open and visible**,
   re-owned by "Acquisition M014 — adapter capability parity (`User-Agent` and
   the `CurlConfig` public-field API boundary)" (UF-1);
4. record the separate-republication requirement: `eggup-acquisition` must be
   republished at a source-breaking version before `eggup-eggpack`'s
   `=0.1.3` pin can be widened and the typed I/O surface can reach eggpack.

## Registry updates

**None performed by this milestone** — `plans/registry.md` is owned by the
parent session (see *Roadmap disposition*). This section records what the
parent session must apply.

## Files touched

**Source (3 files):**

- `crates/eggup-acquisition/src/lib.rs`
- `crates/eggup-curl/src/lib.rs`
- `crates/eggup-eggfetch/src/lib.rs`

**New integration tests (3 directories, 6 files):**

- `crates/eggup-acquisition/tests/assertion_guard.rs`
- `crates/eggup-acquisition/tests/typed_io_contract.rs`
- `crates/eggup-curl/tests/assertion_guard.rs`
- `crates/eggup-eggfetch/tests/assertion_guard.rs`
- `crates/eggup-eggfetch/tests/typed_io_contract.rs`

**Changelogs (3 files):**

- `crates/eggup-acquisition/CHANGELOG.md`
- `crates/eggup-curl/CHANGELOG.md`
- `crates/eggup-eggfetch/CHANGELOG.md`

**Architecture (3 files):**

- `architecture/acquisition.md`
- `architecture/curl-adapter.md`
- `architecture/eggfetch-adapter.md`

**Plan + closure (2 files):**

- `plans/implementation/acquisition-transport/013-adapter-assertion-and-failure-typing-corrective.md`
  (`Status:` header only)
- `plans/closure/acquisition-transport/013-status.md` (this file)

**Deliberately NOT touched:** `Cargo.toml`, `Cargo.lock`, every
`crates/*/Cargo.toml`, `crates/eggup-core/**`, `crates/eggup-archive/**`,
`crates/eggup-service/**`, `crates/eggup-eggpack/**`,
`crates/eggup-transport-footprint/**`, `docs/**`, `plans/registry.md`,
`plans/subsystems/**`, the root `CHANGELOG.md`, and every other crate.
