# Archive Extraction Milestone 005 — cleanup contract truthfulness and member-write deduplication corrective

Status: conditionally closed 2026-10-23; implemented and green on the native macOS lane. Closure record: `plans/closure/archive-extraction/005-status.md`. The remaining condition is the outstanding hosted qualification: Linux, Windows, and the hosted CI lanes were **not run**.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:

- `plans/subsystems/archive-extraction-roadmap.md`

Original closed work affected:

- `plans/implementation/archive-extraction/001-bounded-allowlisted-extraction-contract.md`
- `plans/implementation/archive-extraction/001b-handle-bound-cleanup-and-windows-portability-corrective.md`
- `plans/closure/archive-extraction/001-status.md`
- `plans/closure/archive-extraction/001b-status.md`

Finding of record:

- `plans/registry.md` § "Post-0.1.3 open findings" — entry **4**: "`eggup-archive` public
  cleanup contract is mis-documented and can never succeed." Registered at baseline `c480541`
  alongside this plan; the entry did not exist before it, since that baseline carried two
  *low*-severity findings only. The finding is assessed **high** severity and is closed when
  this milestone closes.

Long-term requirements:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:

- `plans/adrs/ADR-0005-local-archive-extraction-safety-contract.md`

Primary class: invariant

## 1. Objective

Two bounded corrections to `eggup-archive`, one crate, no new dependency.

1. **Truthful cleanup contract.** Four public cleanup entry points — `ExtractedArchive::cleanup`
   (`lib.rs:353`), `PersistedExtraction::cleanup` (`lib.rs:392`), `BoundExtraction::cleanup`
   (`lib.rs:580`), `DeferredCleanup::cleanup` (`lib.rs:617`) — document the operation three
   different ways, none matching the shared implementation, which empties the owned root through
   the retained directory handle and then **unconditionally returns `Err(CleanupFailed)`**,
   discarding the emptying result (`lib.rs:1287-1298`). M005 must make contract and
   implementation agree, and the `Result` must distinguish "emptied" from "failed to empty".
2. **One member-write path.** The tar.gz and zip handlers duplicate the security-critical
   member-write body (`lib.rs:767-866`, `lib.rs:868-935`). M005 must collapse it into one shared
   implementation parameterised by the per-format entry-kind predicate, so a hardening change
   lands once.

M005 does **not** change the extraction safety model, the handle-authority design, the
dependency graph, or any error category.

## 2. Why this milestone is ready

The defects are fully characterized by reading the code at baseline `c480541`. `eggup-archive`
has no open milestone; M004 (the 0.1.3 correctness publication) closed 2026-10-06. Scope is
bounded to one crate plus its own documentation.

| Defect | Severity | Reason |
|---|---|---|
| A — public cleanup contract is false and can never succeed | **high** | `eggup-archive 0.1.3` is **published**. A public API's `Result` success path is unreachable, and its rustdoc promises directory removal that never happens. `extracted.cleanup()?` yields `Err` on the success path and aborts a flow that succeeded; a caller that logs and continues believes the directory is gone when an empty `.eggup-extract-*` directory remains in its output parent. |
| B — duplicated security-critical member-write path | **medium** | The path that turns an archive entry into a file exists twice. A hardening change (byte cap, fsync, extra predicate) must be applied twice and can silently diverge. It has already diverged in naming (`is_regular`, `lib.rs:805`, vs `regular`, `lib.rs:898`) and comment wording. |
| C — single-file crate mixes error, plan, driver, authority, and test concerns | **medium / polish** | Maintainability and review cost, not a behavior defect. Deferred with a named follow-up (§7 WP5). |

**Provenance, stated honestly.** There is **no failing production report**; both defects were found
by static reading at baseline `c480541`. Defect A went undetected because the crate's own tests
encode the *implementation*, not the *documented contract*: `assert_cleanup_reports_residue`
(`lib.rs:3040-3046`) calls `cleanup.cleanup().unwrap_err()` and then asserts the residue directory
still exists and is empty — the test asserts the always-`Err` behavior, so the rustdoc divergence is
invisible to CI. The same pattern appears at `lib.rs:2182`, `2222`, `2262`, `2305`, `2379`, `2416`,
`2603`, `2661`. The divergence is already recorded internally as "Known doc/code drift" items 1–2 in
`architecture/archive-extraction.md` §13 (`604-630`), but the **published rustdoc** and the crate
**README** still state the false contract, so consumers inherit it.

**Dependency readiness.** No hard dependency of this milestone is open. The only cross-crate
constraint is the republication gate, which M005 does not cross (§9).

## 3. Current implementation evidence

All line references are `crates/eggup-archive/src/lib.rs` unless stated.

**3.1 The divergent doc strings.**

```text
lib.rs:351-352   /// Removes the operation-owned extraction directory and reports residue
                /// evidence if cleanup fails.                       (ExtractedArchive::cleanup, 353)
lib.rs:391      /// Removes the retained directory when the caller no longer needs it.
                                                                        (PersistedExtraction::cleanup, 392)
lib.rs:575-579  /// Removes the retained directory without staging, for abort paths.
                                                                        (BoundExtraction::cleanup, 580)
lib.rs:615-617  /// Empties the owned root via the retained handle and reports residue
                /// evidence if cleanup fails.                       (DeferredCleanup::cleanup, 617)
```

`lib.rs:615-617` is accurate. `351-352`, `391`, and `575-579` claim the directory is
**removed**; the crate never removes the root directory.

**3.2 The shared implementation** (`lib.rs:1287-1298`):

```rust
fn remove_owned_root_with_hook(
    path: PathBuf,
    mut handle: File,
    hook: Option<&dyn Fn(&Path)>,
) -> Result<(), ExtractionError> {
    let empty_result = empty_dir_contents(&mut handle);
    if let Some(hook) = hook {
        hook(&path);
    }
    let _ = empty_result;
    Err(ExtractionError::new(ExtractionErrorKind::CleanupFailed).with_residue(path))
}
```

Verified consequences: it only **empties** the root (`empty_dir_contents`, `lib.rs:1187`,
recursing through `empty_dir_contents_at_depth`, `lib.rs:1191`) and never removes the directory;
it **unconditionally returns `Err(CleanupFailed)`** with the residue path; the `empty_result` is
computed and **discarded** (`lib.rs:1296`), so the caller cannot distinguish "emptied
successfully" from "failed to empty"; because it can never return `Ok`, the `Ok(())` arms at
`lib.rs:649` and `lib.rs:727` are **unreachable dead code**; and
`ExtractionErrorKind::CleanupFailed` therefore currently means "emptied, directory not removed"
— the opposite of its own rustdoc (`lib.rs:209-210`: "Cleanup failed; the residue path is
available from the error").

**3.3 The two format handlers.** `extract_tar_gz` (`lib.rs:767-866`, 100 LOC) and `extract_zip`
(`lib.rs:868-935`, 68 LOC) share this shape:

```rust
let declared = plan.members.iter().find(|m| m.source_path == normalized);
if declared.is_some() && !is_regular { return Err(NonRegularMember); }
if !is_regular { drain_bounded(...); continue; }
if let Some(member) = declared {
    let mut output = scope.create_member(...)?;
    let path = scope.member_path(...);
    let (bytes_written, digest) = copy_bounded(&mut entry, Some(&mut output), plan.limits, total)?;
    output.flush()?;
    validate_expected(member, bytes_written, digest)?;
    extracted.insert(normalized.clone(), (...));
} else { drain_bounded(...); }
```

Roughly 45 of the 68 zip lines are a transliteration of the tar handler, differing only in the
entry-kind predicate (`lib.rs:805` vs `zip_entry_is_regular`, `lib.rs:898`) and comment wording.
Shared helpers: `copy_bounded` (`947`), `drain_bounded` (`987`), `validate_expected` (`995`).
Shared write primitives: `ExtractionScope::create_member` (`686`),
`ExtractionScope::member_path` (`697`).

**3.4 Crate layout (Defect C).** `crates/eggup-archive/src/lib.rs` is the crate's **only**
source file: 3,512 lines, 51 `#[test]` functions from `lib.rs:1358`, no `tests/` directory. It
mixes error types (`182`, `215`, `264`, `270`), plan/limit types (`29`, `38`, `89`, `129`),
evidence types (`274`, `318`, `363`, `472`, `536`, `596`), the extraction driver (`630`, `707`,
`734`), path validation (`1015`, `1053`), private-root authority machinery (`1096`, `1104`,
`1120`, `1166`, `1187`, `1191`, `1283`, `1287`, `1313`, `1322`, `1348`), and the test module
(`1358`).

**3.5 What CI cannot see.** `rustc`/`clippy` do not flag the unreachable `Ok(())` arms: `Ok(())`
is a valid `Result` pattern and `unreachable_patterns` does not fire on it, and an always-`Err`
function is a legal `Result` producer. Only a contract-level fixture can catch this.

## 4. Invariants that must not regress

- **Path validation, archive side.** `validate_archive_path` (`lib.rs:1015`) still rejects
  empty/overlong paths, absolute paths (`/`), `\`, `:`, control characters, `.`/`..`/empty
  components, and non-UTF-8 names.
- **Path validation, output side.** `validate_output_name` (`lib.rs:1053`) still rejects
  empty/overlong names, `.`/`..`, multi-component names (`/`), `\`, `:`, names ending in `.` or
  a space, non-ASCII names, and `< > " | ? *`; and still rejects the Windows reserved device set
  `CON`, `PRN`, `AUX`, `NUL`, `CLOCK$`, `CONIN$`, `CONOUT$`, plus any `COM`/`LPT` stem with a
  single ASCII digit suffix — i.e. `COM0`–`COM9` and `LPT0`–`LPT9` (`lib.rs:1070-1084`).
  Near-misses (`com10`, `console`, `auxiliary`) stay valid.
- **Bounds.** `max_entries` (`782`, `879`), `max_archive_bytes` (`639`), per-member (`969`) and
  aggregate (`972`) uncompressed limits are unchanged, and the counters still use `checked_add`
  with `MemberTooLarge` / `TotalTooLarge` on overflow (`963-968`).
- **Cleanup bounds.** Depth ≤ `CLEANUP_MAX_DEPTH` (32), ≤ `CLEANUP_MAX_ENTRIES_PER_PASS` (10
  000) entries per pass, ≤ `CLEANUP_MAX_PASSES` (8) passes (`lib.rs:22-24`, `1192`, `1207`,
  `1276-1280`).
- **Integrity, not authenticity.** SHA-256 is still streamed during the copy (`lib.rs:979`) and
  is **integrity evidence only**. No authenticity, signature, certificate, key, or trust-store
  language may be introduced.
- **Handle-relative authority.** `create_private_root` still creates the root with `mkdir_at`
  from the parent handle (`lib.rs:1137`); member files are still created with `open_at` +
  `create_new(true)` + `follow(false)` (`lib.rs:1178`); Unix modes remain `0700` / `0600`
  (`1135`, `1176`). No path-based creation may be reintroduced.
- **No live destination mutation.** `extract` still requires an existing real `output_parent`
  (`lib.rs:634-641`, `1121-1125`) and never creates parents of a live install root.
- **Deletion stays inside the owned root.** `empty_dir_contents` and its recursion use only
  `fs_at` `*at` operations relative to the retained handle (`lib.rs:1187-1281`).
  `fs::remove_dir_all` must not appear in production code, and the drop path (`1348-1356`) must
  not fall back to it.
- **Drop semantics.** `DirectoryGuard::drop` remains a best-effort emptying whose result is
  discarded, and `active` still guarantees exactly one of drop or explicit cleanup runs
  (`lib.rs:1335-1355`).
- **Public surface.** No existing public item is removed or renamed and no public function
  signature changes (`Result<(), ExtractionError>` is retained). Any new public item requires a
  stated reason in the closure record.
- **Workspace rules.** `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` stay
  (`lib.rs:1-2`); Rust 1.89 holds; workspace clippy with `-D warnings` stays clean.
- **Dependencies.** `eggup-archive` keeps exactly `tar`, `zip`, `flate2`, `fs_at`, `sha2` at
  runtime and `eggup-core` as a **dev-only** dependency
  (`crates/eggup-archive/Cargo.toml:16-24`).

## 5. Scope

### In scope

- The shared cleanup implementation (`lib.rs:1287-1298`) and the four public cleanup entry
  points.
- Rustdoc for the four cleanup entry points, `ExtractionError::residue_path` (`lib.rs:255-261`),
  `ExtractionErrorKind::CleanupFailed` (`209-210`), and `extract` (`622-629`).
- The member-write body shared by `extract_tar_gz` and `extract_zip`.
- The crate's cleanup-related tests, rewritten to assert the contract rather than the
  implementation.
- New tests under `crates/eggup-archive/tests/` proving the published contract from outside the
  crate.
- `crates/eggup-archive/README.md`, `crates/eggup-archive/CHANGELOG.md` (`Unreleased`), and
  `architecture/archive-extraction.md`.

### Explicitly out of scope

- **No transport, authenticity, or signature policy.**
- **No live-installation policy** — no destination selection, no install-root creation, no
  staging or commit behavior.
- **No runtime `eggup-core` dependency.** It stays dev-only. No Eggpack dependency, no service
  dependency.
- **No republication.** M005 does not publish, bump a version, move a tag, or turn `Unreleased`
  into a release section.
- **No change to the extraction safety model** beyond §6.4: the accepted/ rejected archive set,
  the error taxonomy, the private-root authority, and the allowlist model are unchanged.
- **No new dependencies**, including no new dev-dependency beyond what the workspace already
  carries.
- **No change to `ArchiveLimits` defaults** (`lib.rs:38`, `48`, `76`) or to the plan/limit
  validation rules.
- **No change to the `eggup-core` transaction model**, the `BoundSources` handoff, or the
  `eggup-eggpack` adapter.
- **No module split** of `crates/eggup-archive/src/lib.rs` here (§7 WP5 records the deferral and
  names the follow-up).
- **No edit to `plans/registry.md` or the roadmap** — those are the closure's to write.

## 6. Required production changes

Required observable outcomes, not code. The implementer may choose the internal shape; the
observable contract is fixed.

### 6.1 Contract decision

**Keep the handle-based "empty, do not remove" design and make the contract truthful and
observable.** That design is the deliberate handle-authority model established by archive
M001a/M001b/M001c/M001d and confirmed by `plans/closure/archive-extraction/001b-status.md`:
there is no portable object-bound root unlink, so the crate empties the owned tree through the
retained handle and never recursively touches the replacement pathname. M005 does **not**
reverse that decision and must not reintroduce pathname-authorized deletion. The plan author
must confirm this direction with the roadmap owner before implementation begins, because it
changes the observable result of a published API (§9).

### 6.2 The one cleanup contract, shared by all four entry points

Every one of `ExtractedArchive::cleanup`, `PersistedExtraction::cleanup`,
`BoundExtraction::cleanup`, `DeferredCleanup::cleanup` must:

1. **Empty** the operation-owned root through the retained root handle, and **never remove** the
   root directory itself and **never traverse** the root pathname.
2. Return **`Ok(())`** when the owned root was emptied — meaning exactly that the owned root
   object held no entries at the final handle-relative listing.
3. Return **`Err(ExtractionError)`** with `kind() == ExtractionErrorKind::CleanupFailed` when
   the owned root could **not** be fully emptied. Then `residue_path()` names the recorded root
   path, the residue **may be partially emptied**, and emptiness is **not** guaranteed. The
   crate never reports a partial cleanup as complete.
4. Document that the **residual empty directory is the caller's to remove** and that the crate
   will not remove it. Because the cleanup call consumes the value, the rustdoc must tell the
   caller to capture the residual path from `root()` **before** the cleanup call —
   `ExtractedArchive::root` (`lib.rs:328`), `PersistedExtraction::root` (`382`),
   `BoundExtraction::root` (`555`), `DeferredCleanup::root` (`611`) exist for exactly this.
5. Document that the recorded root path is **advisory**: if the root was renamed or replaced by
   a third party, the crate emptied the owned object and left whatever occupies the recorded
   pathname untouched. This mirrors the existing advisory-path contract on
   `ExtractedMember::path` (`293-303`).

**The distinguishing signal is the `Result`**: `Ok` = emptied, `Err(CleanupFailed)` = not
emptied. The discarded `empty_result` (`lib.rs:1296`) must be consumed, not dropped, and the
always-`Err` return at `lib.rs:1297` must become conditional.

### 6.3 What `CleanupFailed` and `residue_path()` mean after the change

`ExtractionErrorKind::CleanupFailed` (`lib.rs:209-210`) becomes strictly a failure category:
"the owned root could not be fully emptied; the residue path is available from the error and its
emptiness is not guaranteed."

`ExtractionError::residue_path()` (`lib.rs:255-261`) must document its **two** meanings,
matching `architecture/archive-extraction.md` §9 (`473-476`): on the `extract()` failure path
(`with_empty_residue`, `lib.rs:242-248`) the directory is verified empty by
`private_root_is_empty`; on the explicit-cleanup failure path (`with_residue`, `228-232`) the
directory may be partially emptied and the caller must inspect it. The current claim "When
present, the directory is empty" is false for the cleanup path and must not survive. The crate
does not remove the residual empty directory on any path, including the success path; the caller
owns removing it, and the crate must not add a convenience that removes the directory by
pathname — that would reintroduce the pathname-authority hazard M001b closed.

### 6.4 One member-write path

The member-write body must exist **exactly once** in non-test code, in one shared helper,
parameterised by the per-format entry-kind predicate. The helper must perform, in this order and
with no format-specific branch:

1. Look up the declared member for the normalized archive path (the allowlist match).
2. A **declared** member whose entry is not regular → `Err(NonRegularMember)`; the whole
   extraction fails closed.
3. An **undeclared** or **non-regular** entry → bounded drain (`drain_bounded`), never
   materialized.
4. A **declared regular** member → create the file through the retained root handle
   (`scope.create_member`), compute the advisory path (`scope.member_path`), bounded copy with
   streaming SHA-256 (`copy_bounded`), `flush`, verify the declared size/digest
   (`validate_expected`), record the evidence.

Both handlers delegate steps 1–4 to that helper. Each format keeps its own parsing: entry
iteration/open, name decode, `validate_archive_path`, duplicate detection (`seen`), the
entry-count bound, its own entry-kind predicate, and the tar gzip-tail validation
(`lib.rs:845-864`).

The shared helper must be the **only** place in non-test code that calls
`scope.create_member(...)`, `copy_bounded(...)` with a populated output, and
`validate_expected(...)`; a source-level guard test (§10) must assert this.

The refactor must be **behavior-preserving**: same bytes written, same order of operations, same
error kinds, same evidence, same handle-authority calls. The predicate is passed in (tar
`lib.rs:805`; zip `lib.rs:898`); the helper must not re-derive it from a format enum, which
would push the branch back into the shared path.

## 7. Ordered work packages

- **WP1 — Contract decision and rustdoc truth.** Record the §6.1 decision. Fix the rustdoc of
  the four cleanup entry points, `residue_path()`, `CleanupFailed`, and `extract`. Verify:
  `cargo doc -p eggup-archive --no-deps --locked` builds warning-free; a reviewer checks every
  sentence against the code. *No behavior change in WP1.*
- **WP2 — Residue/emptied distinction.** Implement §6.2–§6.3: consume `empty_result`; make `Ok`
  reachable and meaningful; make `CleanupFailed` strictly a failure; keep the residue path on
  the failure path only; the `Ok(())` arms at `lib.rs:649` and `727` become live again. Verify:
  the crate's cleanup tests, rewritten to the new contract, pass, with foreign-preservation
  assertions unchanged in strength.
- **WP3 — Member-write extraction.** Implement §6.4: introduce the shared helper; reduce both
  handlers to per-format parsing plus a delegation. Verify: every extraction test passes with
  identical evidence; the single-write-path guard shows one write path.
- **WP4 — Tests.** Implement §10: the new integration fixtures under
  `crates/eggup-archive/tests/`, the inline deterministic failure fixture, the per-format parity
  fixture, the single-write-path guard, the Windows device-alias regression fixture, and the
  red-first proof (§11).
- **WP5 — Module-split decision.** Record that the `crates/eggup-archive/src/lib.rs`
  decomposition (§3.4) is **deferred** from M005, with the reason and a named follow-up:
  *Archive Extraction M006 — `eggup-archive` module decomposition*. Reason: the split is a pure
  code-move with no consumer-visible effect, it would add a large mechanical diff to a milestone
  that already changes a published API's observable result, and it affects neither defect's
  correctness. It is **not** dropped: the decision, reason, and follow-up name go into the
  closure record and the roadmap status table.
- **WP6 — Documentation and changelog.** Implement §12. No release section is created; the
  `Unreleased` entry keeps the repository's no-publication / no-migration disclaimer wording.

## 8. Failure, cancellation, restart, and contention semantics

- **Cleanup that cannot empty the root** stays **fail-closed**: `Err` with `kind() ==
  CleanupFailed` and `residue_path()` naming the recorded root. It never reports success and
  never claims the residue is empty.
- **Partial emptying.** The bounded emptying may remove some content before failing. The result
  is still `Err(CleanupFailed)`; the residue may be partially emptied; the crate does not
  re-verify emptiness by pathname on this path and never promotes a partial cleanup to success.
- **Successful emptying.** `Ok(())`; the owned root object is empty at the final handle-relative
  listing; the root directory itself is not removed.
- **Concurrent cleanup attempts.** Every cleanup entry point consumes `self`, and the four types
  are single-owner handle holders. `DirectoryGuard::active` (`lib.rs:1335-1355`) guarantees
  exactly one of drop or explicit cleanup runs. M005 introduces no new locking or shared state.
- **Cleanup racing with a directory rename or replacement.** The emptying targets the owned
  object through the retained handle; a renamed or replaced pathname is never followed,
  traversed, or removed; the residue path stays advisory. The crate's foreign-preservation
  assertions (`lib.rs:2200`, `2231`, `2279`, `2358`, `2389`, `2435`, `2466`, `2491`) must remain
  true and must not be weakened.
- **Cleanup racing with concurrent file creation inside the root.** The bounded pass loop is
  unchanged: a pass that deletes nothing is an error (`lib.rs:1276-1278`), so the cleanup fails
  closed with `CleanupFailed` rather than looping or reporting success.
- **Cleanup ordered before staging completes.** Unchanged: member objects are closed first
  (`lib.rs:357`, `401`, `586`); where open handles pin directory entries the emptying may fail
  closed with `CleanupFailed`.
- **Cancellation.** The crate has no cancellation API (`architecture/archive-extraction.md` §13
  item 3). M005 adds none and does not change the per-chunk budget check.
- **Restart.** No durable state exists in this crate; not applicable.

## 9. Compatibility and migration

`eggup-archive 0.1.3` is **published** on crates.io (2026-10-06, published from `bd43683`,
registry checksum `a28af652…9824c`).

**Observable changes:** all four cleanup entry points gain a **reachable** `Ok` path;
`ExtractionErrorKind::CleanupFailed` now means "not emptied" only and is no longer returned on
the success path; `residue_path()`'s documented meaning splits into the two proven cases (§6.3),
withdrawing the blanket "the directory is empty" claim from the cleanup path.

**Unchanged:** every public type, field, and function signature. `Result<(), ExtractionError>`
is retained, so this is a **behavior change, not a source-breaking API change** for callers that
do not inspect the error.

**Migration consequence, stated plainly:** the success path of `cleanup()` becoming reachable is
a behavior change for existing callers. A caller that wrote `cleanup().unwrap_err()` will now
**panic** on the success path and must be rewritten to match on the result. A caller that wrote
`cleanup()?` inside a `Result<(), E>` function and aborted the flow on error now continues on
success — the intended fix, but a control-flow change the caller must review. A caller that
wants the residual directory path must capture it from `root()` before the consuming cleanup
call. The crate CHANGELOG `Unreleased` entry must carry this note.

**Adapter consequence:** `eggup-eggpack` pins `eggup-archive = "=0.1.3"`
(`crates/eggup-eggpack/Cargo.toml:21`). The fix therefore **cannot reach the published Eggpack
adapter** without a separate adapter republication and an exact-pin cascade, per
`.opencode/skills/release-workflow`. M005 does not perform it. (At baseline `eggup-eggpack` does
not call the cleanup entry points itself — it drives `into_bound_sources`/`into_members` and
hands the `DeferredCleanup` to its caller — so the adapter's own behavior is unchanged; the pin
consequence is stated for the general republication path.)

**Publication:** M005 **must not publish** — no version bump, no tag move, no `cargo publish`,
no release changelog section. Republication is a separate milestone that must sequence the
version, the exact-pin cascade to `eggup-eggpack`, the changelog release section, and hosted
qualification.

## 10. Required tests

New fixtures live in `crates/eggup-archive/tests/`, so the *published* contract is proven from
outside the crate. `crates/eggup-archive/Cargo.toml:14` already excludes `tests/` from the
packaged crate, so adding this directory does not change the published package contents.
Required file names, so §11's commands are exact:
`crates/eggup-archive/tests/m005_cleanup_contract.rs` and
`crates/eggup-archive/tests/m005_member_write_parity.rs`.

At minimum:

1. `Ok` on successful emptying — for **each** of the four entry points — and the "cleaned root still
   exists and is empty" observable, asserted from outside the crate: after `Ok`, the root path still
   exists, is a directory, has zero entries, and is removable by the caller with `fs::remove_dir`.
   The crate did not remove it.
2. A foreign replacement at the recorded pathname survives cleanup untouched (the crate never
   traverses the pathname), and `residue_path()` is absent on the success path while naming the
   recorded root on the failure path.
3. **Deterministic failure fixture (inline, in `lib.rs`).** A fixture that makes the emptying fail
   deterministically and portably, asserting `Err(CleanupFailed)`, a residue path, and that the
   crate does not report success. Preferred mechanism: exceeding the crate's **own** private cleanup
   bounds (depth > `CLEANUP_MAX_DEPTH`, or entries per pass > `CLEANUP_MAX_ENTRIES_PER_PASS`) — the
   only mechanism that fails on every supported platform without privileges or platform-specific
   behavior. It must live inline because the bounds are private constants. *Honest limitation to
   state in the closure:* the `Err` path is therefore **not** proven from an external integration
   test, because the crate exposes no portable public failure trigger. Do not claim external
   coverage that does not exist.
4. **Per-format parity fixture** (`m005_member_write_parity.rs`): one shared case table driven
   through **both** tar.gz and zip, asserting identical observable outcomes per case —
   declared-member evidence (size, digest, declaration order), missing member, duplicate member,
   size mismatch, digest mismatch, non-regular declared member, over-limit member, cleanup
   outcome. This is the fixture that would catch a future divergence between formats.
5. **Single-write-path guard**: a source-level assertion (for example via
   `include_str!("../src/lib.rs")`) that the member-write path exists in exactly one place in
   non-test code — one call site of the member creation primitive, one of the bounded copy with
   output, one of the expected-value validation. The assertion must fail if the path is duplicated.
6. **Windows device-alias regression fixture**: `COM0`, `LPT0`, `CONIN$`, `CONOUT$`, `CLOCK$` are
   rejected as `InvalidPath`; near-misses (`com10`, `console`, `auxiliary`) remain valid. This is a
   **regression guard**, not a defect fix: it must pass **before and after** M005 and is explicitly
   exempt from the red-first requirement.
7. **Red-first proof**: every new fixture asserting M005's *changed* behavior must be shown to
   **fail** against the pre-M005 implementation (§11). Fixture 6 is the only stated exemption.
8. All 51 existing inline tests remain green after being updated to the new contract. No
   foreign-preservation assertion may be weakened: the assertions that a foreign directory, empty
   directory, symlink, or Windows reparse point survives cleanup stay exactly as strong, including
   `assert_cleanup_reports_residue` (`lib.rs:3040-3046`), which must be rewritten to the new
   contract without dropping its "residue directory still exists and is empty" check. The full
   workspace suite stays green.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy -p eggup-archive --all-targets --locked -- -D warnings
cargo clippy -p eggup-archive --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-archive --locked
cargo test -p eggup-archive --locked cleanup
cargo test -p eggup-archive --locked member_write
cargo test -p eggup-archive --locked --test m005_cleanup_contract --test m005_member_write_parity
cargo doc -p eggup-archive --no-deps --locked
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo tree --workspace --locked
cargo tree -p eggup-archive --locked
cargo tree -p eggup-core --locked
./scripts/check-local.sh
git diff --check
```

**Red-first proof against the pre-M005 baseline** (required; the new fixtures must fail there).
Run verbatim from the repository root:

```text
git worktree add ../eggup-m005-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
mkdir -p ../eggup-m005-baseline/crates/eggup-archive/tests
cp crates/eggup-archive/tests/m005_cleanup_contract.rs ../eggup-m005-baseline/crates/eggup-archive/tests/
cp crates/eggup-archive/tests/m005_member_write_parity.rs ../eggup-m005-baseline/crates/eggup-archive/tests/
cd ../eggup-m005-baseline && cargo test -p eggup-archive --locked --test m005_cleanup_contract --test m005_member_write_parity
```

Expected: the run **fails** — the baseline `cleanup()` returns `Err` where the new contract
requires `Ok`, and the baseline has two write paths where the new guard requires one. Record the
verbatim failure output in the closure record, then:

```text
git worktree remove ../eggup-m005-baseline
```

Hosted closure requires the full matrix: Stable Linux, Rust 1.89 MSRV, macOS, and the Windows
lane, which must **execute** the archive tests rather than merely compile them.

## 12. Documentation updates

- **rustdoc**: the four cleanup entry points, `ExtractionError::residue_path`,
  `ExtractionErrorKind::CleanupFailed`, and `extract`'s drop-semantics paragraph — each
  rewritten to the §6.2 contract.
- **`crates/eggup-archive/CHANGELOG.md`**: an `Unreleased` entry recording the contract
  correction and the member-write deduplication, keeping the repository's existing disclaimer
  wording ("Nothing in this section has been published. `0.1.3` is the published baseline on
  crates.io, and no publication milestone authorizes these entries."). It must state the
  behavior change (the `Ok` path is now reachable), the migration note for callers that used
  `unwrap_err()`, and that integrity remains SHA-256 evidence only with no authenticity claim.
- **`crates/eggup-archive/README.md`**: the cleanup paragraph (currently `README.md:33-37`,
  which says explicit cleanup "reports the now-empty directory as `CleanupFailed` residue") must
  be corrected to the new contract.
- **`architecture/archive-extraction.md`**: the §3 public-surface table (the "Removes…" wording
  at lines 87, 90, 100, 102), the §8 stage diagram (361, 368, 375, 380), the §9 residue table
  (473-476), §12 testing approach (test counts and the new `tests/` directory), and §13 — drift
  item 2 ("`cleanup()` never returns `Ok`") is **fixed** by M005 and must be rewritten or
  removed, not left standing.
- **`plans/registry.md`** and **`plans/subsystems/archive-extraction-roadmap.md`**: updated by
  the **closure record**, not by this plan. The closure registers M005, adds the §"Post-0.1.3
  open findings" item 3, and adds the roadmap status row.
- **`docs/` is non-normative.** It points at the contracts; it never restates them. At baseline
  `c480541` `docs/` contains no cleanup or residue wording (verified by search), so the expected
  change is none. If any consumer-facing page describes cleanup, it must link to the contract
  rather than paraphrase it.

## 13. Acceptance criteria

M005 closes only when all of the following hold:

- All four cleanup entry points have one identical, truthful contract (§6.2), their rustdoc says
  what the crate does and what the caller owns, `cleanup()` returns `Ok` on a successful emptying
  and `Err(CleanupFailed)` with a residue path on a failure, and that distinction is observable
  from outside the crate.
- `ExtractionErrorKind::CleanupFailed` is returned only on failure, `residue_path()`'s rustdoc
  states its two proven meanings, and the residual empty directory is left in place by the crate
  with the rustdoc telling the caller to capture its path from `root()` and remove it.
- The member-write path exists exactly once in non-test code, is parameterised by the entry-kind
  predicate, and both handlers keep their own per-format parsing.
- Every existing extraction test is green with identical evidence, and no foreign-preservation
  assertion was weakened.
- The new fixtures fail against the pre-M005 baseline (§11) and pass after.
- `cargo tree -p eggup-archive --locked` still shows only `tar`, `zip`, `flate2`, `fs_at`, `sha2`
  (plus `eggup-core` as a dev-dependency), and `cargo tree -p eggup-core --locked` is unchanged.
- A fresh hosted Stable/MSRV/macOS/Windows matrix is green, with the Windows lane **executing**
  the archive tests.
- The crate CHANGELOG `Unreleased` entry, the README, and `architecture/archive-extraction.md`
  agree with the code, and `architecture/archive-extraction.md` §13 no longer lists the fixed
  drift.
- The WP5 deferral decision and the named M006 follow-up are recorded in the closure record and
  the roadmap status table.
- Nothing was published, no version was bumped, and no tag was moved.

## 14. Stop conditions

Stop and write another plan if:

- The truthful contract **cannot** be expressed without a breaking public API change — for
  example if the implementer concludes the `Ok` path must carry the residue path and that
  requires a new public result type or a changed signature. That decision belongs at the
  intentional 0.2/1.0 API boundary, where `CleanupDisposition`/`Error` extensibility is already
  scheduled; do not smuggle it into M005.
- The member-write refactor would change extraction semantics — different bytes written, a
  different order of operations, a different error kind, different evidence, or a different
  handle-authority call.
- A new dependency is required, including a runtime `eggup-core` edge, a transport dependency,
  or an Eggpack dependency.
- The contract appears to require removing the root directory by pathname. The handle-authority
  design must not regress to path-based creation or deletion.
- The red-first proof cannot be produced — that means the new fixtures do not actually
  discriminate the pre-M005 implementation.
- Making `Ok` reachable appears to require weakening fail-closed behavior (for example by
  reporting a partially emptied root as cleaned).
- The implementer concludes publication is required. It is not; that is a separate milestone.
- The Windows lane cannot execute the new fixtures. A lane that only compiles is not a run, and
  a platform-sensitive cleanup contract must be qualified on Windows.

## 15. Closure evidence required

Per `plans/closure/README.md`, the closure record must contain:

- **Status**, source plan, source roadmap, reviewed baseline, implementation commit(s), and an
  **executive finding** stating the contract before and after.
- **Requirement→evidence matrix**: one row per §4 invariant and per §10 fixture, each with the
  test name, the implementation file:line, and the pass/fail result.
- **Production implementation evidence**: the before/after of `remove_owned_root_with_hook`
  (`lib.rs:1287-1298`), the four cleanup rustdoc blocks, and the shared member-write helper with
  both handler call sites.
- **Exact commands run and their results** for every command in §11, including the verbatim
  red-first failure output from the baseline worktree.
- **Per-platform results, reported separately** for Linux, macOS, and Windows. A lane that was
  not run is **not run** — never inferred from another platform. The Windows lane must be
  recorded as executing the archive tests, with the test count.
- **Invariant review** against §4, item by item.
- **Failure/rollback/recovery review** against §8, including the fail-closed assertion for a
  cleanup that cannot empty the root.
- **Compatibility and migration review**: the published `0.1.3` delta, the `unwrap_err()` panic
  consequence, the `root()` capture requirement, and the `eggup-eggpack` `=0.1.3` pin
  consequence.
- **Security review**: no path-based deletion, no path-based creation, path validation
  unchanged, the Windows device set unchanged, no authenticity or signature language introduced,
  no new dependency, `unsafe_code` still denied, `#![deny(missing_docs)]` still satisfied.
- **Documentation/operations evidence**: rustdoc, README, CHANGELOG `Unreleased`, and
  `architecture/archive-extraction.md` (including §13).
- **Unresolved findings with severity**, explicitly including the deferred module decomposition
  as a named follow-up (M006) and the honest limitation that the `Err(CleanupFailed)` path is
  proven inline rather than from an external integration test.
- **Roadmap disposition and registry updates**: the M005 row, the new "Post-0.1.3 open findings"
  item 3 (removed when the finding closes), and the WP5 deferral record.

## 16. Handoff notes

**Sequencing.** M005 lands as a source-only corrective. It is followed, only if separately
authorized, by (a) a republication milestone for `eggup-archive` (new version, release changelog
section, hosted qualification, crates.io publication), and (b) an `eggup-eggpack` republication
to move the `=0.1.3` exact pin and reach the fix downstream. The module decomposition (M006) is
independent of both and may be sequenced at the 0.2/1.0 boundary.

**What a separate republication milestone would require** (not authorized here): a version bump
in `Cargo.toml`, an exact-pin cascade to `eggup-eggpack`, a release section in the crate
CHANGELOG restating the published baseline, hosted qualification on all lanes, and the manual
publication steps in `.opencode/skills/release-workflow`. CI never publishes.

**What this plan explicitly does NOT authorize**: publication or a version bump; a tag move; a
runtime `eggup-core` dependency; transport, authenticity, or Eggpack policy; live-installation
policy; a change to the extraction safety model beyond §6.4; any new dependency; a change to
`ArchiveLimits` defaults; the module split (WP5 defers it with a named follow-up); or edits to
`plans/registry.md` and the roadmap — those belong to the closure record.

**Keep the change honest.** The point of this milestone is not "the tests are green"; it is that
a published API's `Result` can finally mean what its documentation says, and that the
security-critical member-write path exists once. If the implementation drifts from the contract
again, the contract is what must change — subject to §14.
