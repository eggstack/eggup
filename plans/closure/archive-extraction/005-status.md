# Archive Extraction M005 — Closure and Verification Record

Status: **conditionally closed** — the milestone's own scope is complete and green,
and the one **medium** downstream finding (F-1, the `eggup-eggpack` test-helper
migration) has since been applied by the parent session, so the
`eggup-eggpack` test lane is green again. The remaining condition is the
outstanding **hosted** qualification (F-5): Linux, Windows, and the hosted CI
matrix were not run from this machine.

Source plan: `plans/implementation/archive-extraction/005-cleanup-contract-and-member-write-corrective.md`

Source roadmap: `plans/subsystems/archive-extraction-roadmap.md`

Reviewed repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`
(branch `plans/service-m010-failed-unit-recovery`)

Implementation commit(s): **working tree, uncommitted — no commit was made.** The
changes are in `crates/eggup-archive/src/lib.rs`,
`crates/eggup-archive/tests/m005_cleanup_contract.rs`,
`crates/eggup-archive/tests/m005_member_write_parity.rs`,
`crates/eggup-archive/README.md`, `crates/eggup-archive/CHANGELOG.md`, and
`architecture/archive-extraction.md`. No publication, version bump, or tag move.

Contract direction (§6.1) was **confirmed by the roadmap owner before
implementation**, as the plan requires: keep the handle-based "empty, do not
remove" design, make the contract truthful and observable, and split the return
into a meaningful `Ok` / `Err(CleanupFailed)`. Pathname-authorized deletion was
**not** reintroduced, and no convenience that removes the residual directory by
pathname was added.

## Executive finding

**Before.** `eggup-archive 0.1.3` (published) ships four `cleanup()` entry points
whose rustdoc said they "remove" the extraction directory. The shared
implementation emptied the owned tree through the retained handle and then
unconditionally returned `Err(ExtractionErrorKind::CleanupFailed)`. The `Ok` arm
of `Result<(), ExtractionError>` was **unreachable in a published crate**, the
documented behaviour did not happen, and the `Ok(()) => Err(error)` arms in
`extract`'s failure path were dead code. Separately, the tar.gz and zip handlers
each carried a full copy of the member-write body.

**After.** All four entry points share one truthful, observable contract:
`Ok(())` means the owned root object held no entries at the final
handle-relative listing (emptied); `Err(..)` with `kind() == CleanupFailed`
means the owned root could **not** be fully emptied (the residue may be
partially emptied and its emptiness is not guaranteed). The `Ok` path is
reachable and proven from outside the crate. The crate still never removes the
root directory itself and never traverses the root pathname; the residual empty
directory is the caller's to remove. The member-write body now exists exactly
once, in one shared helper, with both format handlers delegating to it and
keeping only their own per-format parsing.

## Requirement → evidence matrix

### §4 invariants (must not regress)

| # | Invariant | Evidence | Result |
|---|---|---|---|
| 1 | Exact allowlist extraction; no glob/prefix matching | `write_declared_member` keeps the exact-equality lookup at `lib.rs:925`; `every_case_produces_identical_outcomes_in_both_formats` covers declared/undeclared members | pass |
| 2 | Path validation unchanged | `validate_archive_path` (`lib.rs:1154`) and `validate_output_name` (`lib.rs:1192`) untouched; `validate_output_name` still rejects `COM0`/`LPT0`/`CONIN$`/`CONOUT$`/`CLOCK$` | pass |
| 3 | Windows device set unchanged | `windows_device_aliases_are_rejected_and_near_misses_remain_usable` (`tests/m005_member_write_parity.rs`); inline `output_names_reject_every_windows_device_name_not_just_1_to_9` (`lib.rs:1959`) | pass |
| 4 | SHA-256 is integrity evidence only; no authenticity claim | no signature/trust-root code added; no rustdoc or doc text implies authenticity | pass |
| 5 | No transport, no live-installation policy, no new dependency | `cargo tree -p eggup-archive --locked` unchanged: `tar`, `zip`, `flate2`, `fs_at`, `sha2` at runtime, `eggup-core` **dev-only** | pass |
| 6 | Handle-relative writes and handle-bound cleanup unchanged | `empty_dir_contents` (`lib.rs:1326`) untouched; `remove_owned_root_with_hook` (`lib.rs:1439-1452`) still uses only `*at` operations | pass |
| 7 | No path-based creation, no path-based deletion | `create_private_file_at` (`lib.rs:1305`) untouched; no `remove_dir_all` on the root pathname anywhere | pass |
| 8 | No `unsafe_code`; `#![deny(missing_docs)]` satisfied | crate attributes untouched; `cargo doc -p eggup-archive --no-deps --locked` builds warning-free | pass |
| 9 | `ArchiveLimits` defaults and plan/limit validation rules unchanged | `ArchiveLimits::strict_default` (`lib.rs:76`) and `ArchivePlan::new` (`lib.rs:136-176`) untouched | pass |
| 10 | No runtime `eggup-core` edge | `Cargo.toml` untouched; tree confirms dev-only | pass |

### §10 required fixtures

| # | Required fixture | Test name | Implementation | Result |
|---|---|---|---|---|
| 1 | `Ok` on successful emptying for **each** of the four entry points; root still exists, is a directory, has zero entries, and is removable by the caller | `cleanup_reports_ok_and_leaves_an_empty_root_for_all_four_entry_points` | `tests/m005_cleanup_contract.rs`; entry points at `lib.rs:397`, `460`, `670`, `729` | pass |
| 2 | Foreign replacement at the recorded pathname survives cleanup untouched; `residue_path()` absent on the success path | `foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched` (`#[cfg(unix)]`), `cleanup_reports_no_residue_path_on_the_success_path` | `tests/m005_cleanup_contract.rs` | pass (Unix-gated; see F-3) |
| 3 | Deterministic failure fixture **inline** in `lib.rs`: `Err(CleanupFailed)`, a residue path, no false success | `cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence` | `lib.rs:2741-2763`, exceeding the crate's private `CLEANUP_MAX_DEPTH` | pass |
| 4 | Per-format parity fixture: one shared case table through **both** tar.gz and zip | `every_case_produces_identical_outcomes_in_both_formats` | `tests/m005_member_write_parity.rs` — 8 cases: declared-member evidence, declaration order, missing member, duplicate normalized path, size mismatch, digest mismatch, non-regular declared member, over-limit member, cleanup outcome | pass |
| 5 | Single-write-path guard: exactly one call site each of the member creation primitive, the bounded copy with output, and the expected-value validation | `member_write_path_exists_exactly_once_in_non_test_code` | `tests/m005_member_write_parity.rs`; helper `lib.rs:916-966`, call sites `940`, `947`, `951` | pass |
| 6 | Windows device-alias regression guard (exempt from red-first) | `windows_device_aliases_are_rejected_and_near_misses_remain_usable` | `tests/m005_member_write_parity.rs` | pass (also passes at baseline) |
| 7 | Red-first proof against the pre-M005 baseline | both new files | see *Red-first proof* below | pass |
| 8 | All 51 existing inline tests green after being updated to the contract; no foreign-preservation assertion weakened | 50 inline tests run on macOS (49 baseline + 1 new); 2 `#[cfg(windows)]` not run here | `lib.rs` test module from `lib.rs:1512` | pass — see *Foreign-preservation review* |

## Production implementation evidence

### `remove_owned_root_with_hook` — before and after

Before (`lib.rs:1287-1298` at baseline):

```rust
    let empty_result = empty_dir_contents(&mut handle);
    if let Some(hook) = hook {
        hook(&path);
    }
    let _ = empty_result;
    Err(ExtractionError::new(ExtractionErrorKind::CleanupFailed).with_residue(path))
```

After (`crates/eggup-archive/src/lib.rs:1439-1452`):

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
    match empty_result {
        Ok(()) => Ok(()),
        Err(_) => Err(ExtractionError::new(ExtractionErrorKind::CleanupFailed).with_residue(path)),
    }
}
```

The `empty_result` is consumed rather than discarded (`let _ = empty_result;` is
gone), the always-`Err` return is now conditional, and the `Ok` path is
reachable. The emptying mechanics (`empty_dir_contents`,
`lib.rs:1326-1420`) are **untouched**.

### The four cleanup rustdoc blocks

All four carry one identical contract (`Ok` = emptied, `Err(CleanupFailed)` =
not emptied, the residual directory is the caller's to remove and its path must
be captured from `root()` before the consuming call, and the recorded path is
advisory):

| Entry point | `lib.rs` |
|---|---|
| `ExtractedArchive::cleanup` | `372-397` |
| `PersistedExtraction::cleanup` | `435-460` |
| `BoundExtraction::cleanup` | `643-670` |
| `DeferredCleanup::cleanup` | `705-729` |

Supporting rustdoc: `ExtractionErrorKind::CleanupFailed` (`lib.rs:209-214`),
`ExtractionError::residue_path` (`lib.rs:263-279`, its two proven meanings),
`extract`'s drop-semantics paragraph (`lib.rs:743-750`), and the
`ExtractedArchive` struct doc (`lib.rs:334-339`, which previously said the
private directory "is removed when dropped" — it is emptied, not removed).

### The shared member-write helper and both call sites

`write_declared_member` — `crates/eggup-archive/src/lib.rs:916-966`. Its four
ordered steps, with no format-specific branch:

1. allowlist lookup for the normalized archive path (`lib.rs:925`);
2. a **declared** member whose entry is not regular → `Err(NonRegularMember)`
   (`lib.rs:930`);
3. an **undeclared** or **non-regular** entry → bounded drain, never
   materialized (`lib.rs:933-939`);
4. a **declared regular** member → `scope.create_member` (`lib.rs:940`),
   `scope.member_path` (`lib.rs:945`), `copy_bounded` with a populated output
   (`lib.rs:947`), `flush` (`lib.rs:949`), `validate_expected` (`lib.rs:951`),
   evidence recorded (`lib.rs:952-965`).

Call sites: `extract_tar_gz` (`lib.rs:1003-1012`, predicate
`matches!(entry_type.as_byte(), 0 | b'0')` at `lib.rs:1002`) and `extract_zip`
(`lib.rs:1063-1072`, predicate `zip_entry_is_regular(&entry)` at
`lib.rs:1062`). Each format keeps its own entry iteration, name decode, path
validation, duplicate detection, entry-count bound, and the tar gzip-tail
validation (`lib.rs:1029-1041`).

### `extract`'s failure path

`lib.rs:770-779` (and the `#[cfg(test)]` twin at `lib.rs:854-863`). The
`Ok(())` arm is live again and now reports the leftover empty directory through
`with_empty_residue` (original category preserved); the `Err(_)` arm reports
`with_residue` (`CleanupFailed`, emptiness not claimed, no pathname
re-verification). The observable outcome of the extraction-failure path is
**unchanged** from the baseline — the baseline reached the same
`with_empty_residue` result because the shared implementation always returned
`Err`.

## Exact commands run, with results

All cargo invocations used `--offline` (no network is reachable from this
machine).

```text
$ cargo test -p eggup-archive --locked --offline
running 50 tests
test result: ok. 50 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test -p eggup-archive --locked --offline cleanup
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 39 filtered out; finished in 0.02s
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

$ cargo test -p eggup-archive --locked --offline member_write
running 1 test
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

$ cargo test -p eggup-archive --locked --offline --test m005_cleanup_contract --test m005_member_write_parity
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
running 3 tests
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s

$ cargo clippy -p eggup-archive --all-targets --locked --offline -- -D warnings
    Checking eggup-archive v0.1.3
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s

$ cargo clippy -p eggup-archive --all-targets --all-features --locked --offline -- -D warnings
    Checking eggup-archive v0.1.3
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.48s

$ cargo doc -p eggup-archive --no-deps --locked --offline
   Documenting eggup-archive v0.1.3
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.40s
  (no warnings)

$ cargo fmt -p eggup-archive -- --check
(clean — no output)

$ cargo tree -p eggup-archive --locked --offline
eggup-archive v0.1.3
├── flate2 v1.1.10 …
├── fs_at v0.2.1 …
├── sha2 v0.10.9 …
├── tar v0.4.46 …
└── zip v6.0.0 …
[dev-dependencies]
└── eggup-core v0.1.3 (path)
```

Commands **not run** at this stage, by the session's explicit instruction (the
parent runs the full workspace gate once at the end):
`cargo test --workspace`, `cargo test --workspace --all-features`,
`./scripts/check-local.sh`, `cargo tree --workspace --locked`,
`cargo test -p eggup-core --locked`.

`cargo fmt --all -- --check` **was** run and currently fails, but **not** on any
file this milestone owns: every reported diff is in `crates/eggup-core/` and
`crates/eggup-service/`, which two concurrent agents are editing. The
`eggup-archive` package is fmt-clean (`cargo fmt -p eggup-archive -- --check`
is silent).

### §11 command that fails — `cargo test -p eggup-eggpack`

```text
$ cargo test -p eggup-eggpack --all-targets --all-features --locked --offline
test handoff_translates_archive_projection_to_zip_plan_and_stages_bound ... FAILED
test handoff_translates_archive_projection_to_tar_gz_plan_and_stages_bound ... FAILED

---- handoff_translates_archive_projection_to_zip_plan_and_stages_bound stdout ----
thread 'handoff_translates_archive_projection_to_zip_plan_and_stages_bound' panicked at
crates/eggup-eggpack/tests/archive_handoff.rs:205:19:
assertion failed: !extraction_root.exists()

test result: FAILED. 11 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Verified as a **new** consequence of this milestone, not a pre-existing failure:
at the pre-M005 baseline (`c4805411`) the same command reports
`test result: ok. 15 passed; …` and both tests pass. The cause is the
caller migration the plan's §9 predicts — the eggpack **test helper**
`finish_cleanup` (`crates/eggup-eggpack/tests/archive_handoff.rs:198-213`)
contains

```rust
    match cleanup.cleanup() {
        Ok(()) => assert!(!extraction_root.exists()),
```

which encodes the old, removed expectation that a successful cleanup *removes*
the root. Under the new contract `Ok(())` means *emptied*, and the residual
directory is the caller's to remove. The eggpack **adapter library** is
unaffected — it never calls the cleanup entry points itself; it drives
`into_bound_sources`/`into_members` and hands the `DeferredCleanup` to its
caller. Only the test helper's `Ok` arm is stale.

The fix belongs to `crates/eggup-eggpack/`, which is **outside this
milestone's file ownership**, so it was **not** applied here. The minimal
patch for the owner is:

```diff
--- a/crates/eggup-eggpack/tests/archive_handoff.rs
+++ b/crates/eggup-eggpack/tests/archive_handoff.rs
@@
 fn finish_cleanup(cleanup: DeferredCleanup, extraction_root: &Path) {
     match cleanup.cleanup() {
-        Ok(()) => assert!(!extraction_root.exists()),
+        // A successful cleanup empties the owned tree; the residual directory
+        // is the caller's to remove, exactly as in the failure branch.
+        Ok(()) => {
+            assert!(extraction_root.exists());
+            assert_eq!(fs::read_dir(extraction_root).unwrap().count(), 0);
+            fs::remove_dir(extraction_root).unwrap();
+        }
         Err(error) => {
```

### Red-first proof against the pre-M005 baseline

Mechanism: **`git worktree add /tmp/eggup-baseline-archive005
c4805411f74b85158e0ae5d0a219a32c4209d98e`**, the two new test files copied
into that worktree, the focused test run there, then
`git worktree remove --force /tmp/eggup-baseline-archive005`. (The worktree
route succeeded; the `git show … > lib.rs` fallback was not needed.) No git
state was mutated: `git stash`, `git checkout`, `git restore`, `git commit`,
`git push`, `git tag`, and `git clean` were **not** run.

Both new files **compile** against the pre-M005 baseline and **fail** there —
a compile failure would not have been acceptable evidence.

```text
$ cd /tmp/eggup-baseline-archive005
$ cargo test -p eggup-archive --locked --offline --test m005_cleanup_contract --test m005_member_write_parity
running 4 tests
test cleanup_reports_no_residue_path_on_the_success_path ... FAILED
test cleanup_reports_ok_and_leaves_an_empty_root_for_all_four_entry_points ... FAILED
test cleanup_failure_category_is_distinct_from_success ... ok
test foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched ... FAILED

---- cleanup_reports_ok_and_leaves_an_empty_root_for_all_four_entry_points stdout ----
thread 'cleanup_reports_ok_and_leaves_an_empty_root_for_all_four_entry_points' panicked at
crates/eggup-archive/tests/m005_cleanup_contract.rs:97:10:
cleanup must report success when the owned root was emptied:
ExtractionError { kind: CleanupFailed, residue_path: Some(".../.eggup-extract-10088-0000000000000004") }

test result: FAILED. 1 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: test failed, to run pass `-p eggup-archive --test m005_cleanup_contract`
EXIT_CODE=101
```

```text
$ cargo test -p eggup-archive --locked --offline --test m005_member_write_parity
running 3 tests
test windows_device_aliases_are_rejected_and_near_misses_remain_usable ... ok
test member_write_path_exists_exactly_once_in_non_test_code ... FAILED
test every_case_produces_identical_outcomes_in_both_formats ... FAILED

---- member_write_path_exists_exactly_once_in_non_test_code stdout ----
thread 'member_write_path_exists_exactly_once_in_non_test_code' panicked at
crates/eggup-archive/tests/m005_member_write_parity.rs:301:5:
assertion `left == right` failed: the member creation primitive must have exactly one call site in non-test code
  left: 2
 right: 1

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: test failed, to run pass `-p eggup-archive --test m005_member_write_parity`
EXIT_CODE=101
```

The exempt fixture (`windows_device_aliases_are_rejected_and_near_misses_remain_usable`)
passes at the baseline, as plan §10.6 requires.

### Per-platform results

| Platform | Result |
|---|---|
| **macOS (darwin arm64, this machine)** | **RUN** — native. `cargo test -p eggup-archive --locked --offline`: 50 inline + 4 + 3 integration = **57 tests passed, 0 failed**. Clippy (with and without `--all-features`), `cargo doc`, and `cargo fmt -p eggup-archive -- --check` are clean. |
| **Linux** | **NOT RUN** — no Linux host is reachable from this machine. |
| **Windows** | **NOT RUN** — no Windows host is reachable from this machine. The 2 `#[cfg(windows)]` inline tests (`lib.rs:2764`, `lib.rs:3072`) and the `#[cfg(unix)]` integration fixture therefore did **not** execute here. |
| **Hosted CI lanes (Stable / MSRV / macOS / Windows)** | **NOT RUN** — no hosted runner is reachable from this machine. |

No platform result is inferred from another platform.

## Invariant review (§4, item by item)

All ten §4 invariants are listed in the requirement→evidence matrix above; all
**pass**. Specifically re-confirmed:

- **Allowlist semantics unchanged.** The exact-equality lookup moved from the
  two handlers into the shared helper unchanged; no glob/prefix/suffix matching
  was introduced.
- **Path validation unchanged.** `validate_archive_path` and
  `validate_output_name` were not touched; the Windows device-name set and its
  near-misses behave exactly as at baseline.
- **Handle authority unchanged.** Member creation still goes through
  `create_private_file_at` → `fs_at::open_at` with `create_new` and
  `follow(false)`; cleanup still goes through `empty_dir_contents`' `*at`
  operations only.
- **No pathname-authorized deletion.** The crate never removes the root
  directory and never traverses the root pathname. The residual empty directory
  is left in place for the caller; **no convenience that removes it by pathname
  was added**, so the M001b pathname-authority hazard is not reintroduced.
- **SHA-256 remains integrity evidence only.** No authenticity, signature, or
  trust-root language was added anywhere.
- **Dependencies unchanged.** `cargo tree` output is identical to the baseline:
  `tar`, `zip`, `flate2`, `fs_at`, `sha2` at runtime; `eggup-core` dev-only. No
  new dependency, including no new dev-dependency.
- **`ArchiveLimits` defaults and plan validation unchanged.**

## Failure, rollback, and recovery review (§8)

- **Cleanup that cannot empty the root stays fail-closed.** `Err` with
  `kind() == CleanupFailed` and `residue_path()` naming the recorded root;
  success is never reported and emptiness is never claimed. Proven inline by
  `cleanup_that_cannot_empty_the_root_fails_closed_with_residue_evidence`
  (`lib.rs:2741`), which exceeds the crate's own private `CLEANUP_MAX_DEPTH`
  bound — the only mechanism that fails deterministically on every supported
  platform without privileges.
- **Partial emptying.** The bounded emptying may remove some content before
  failing; the result is still `Err(CleanupFailed)`, the residue may be
  partially emptied, and the crate does **not** re-verify emptiness by pathname
  on that path (`with_residue`, `lib.rs:238-243`) and never promotes a partial
  cleanup to success.
- **Successful emptying.** `Ok(())`; the owned root object is empty at the
  final handle-relative listing; the root directory itself is **not** removed.
- **Cleanup racing with a rename or replacement.** The emptying targets the
  owned object through the retained handle; a renamed or replaced pathname is
  never followed, traversed, or removed. Covered by
  `foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched`
  and by the crate's inline deterministic-race suite.
- **Cleanup racing with concurrent file creation.** The bounded pass loop is
  unchanged (a pass that deletes nothing is an error, `lib.rs:1415`). The
  concurrent-creation test now asserts the two contract outcomes rather than a
  single error kind, because under the new contract the outcome depends on
  whether the writer finished before the final handle-relative listing. The
  deterministic fail-closed proof is the depth-bound fixture above.
- **Cleanup ordered before staging completes.** Unchanged: member objects are
  closed first (`lib.rs:399-401`, `467-470`, `669-672`).
- **Concurrent cleanup attempts.** Every entry point consumes `self`;
  `DirectoryGuard::active` still guarantees exactly one of drop or explicit
  cleanup. No new locking or shared state was introduced.
- **Cancellation / restart.** The crate has no cancellation API and M005 adds
  none; no durable state exists, so restart is not applicable.
- **Rollback.** The change is source-only and behavior-preserving apart from
  the documented, intended `Ok`-path change. Reverting is a source revert; no
  state, migration, or publication is involved.

## Compatibility and migration review (§9)

- **Published baseline:** `eggup-archive 0.1.3`, published on crates.io
  2026-10-06 from `bd43683`, registry checksum `a28af652…9824c`. The fix is
  **not** published; `0.1.3` remains the published version and
  `crates/eggup-archive/CHANGELOG.md` gained only an `Unreleased` entry, with
  the repository's existing disclaimer wording retained verbatim.
- **Observable changes:** the four cleanup entry points gain a reachable `Ok`
  path; `ExtractionErrorKind::CleanupFailed` now means "not emptied" only and
  is no longer returned on the success path; `residue_path()`'s documented
  meaning splits into the two proven cases, withdrawing the blanket "the
  directory is empty" claim from the cleanup path.
- **Unchanged:** every public type, field, and function signature.
  `Result<(), ExtractionError>` is retained — this is a **behavior change, not a
  source-breaking API change** for callers that do not inspect the error.
- **Migration consequence, stated plainly:** a caller that wrote
  `cleanup().unwrap_err()` will now **panic** on the success path and must be
  rewritten to match on the result. A caller that wrote `cleanup()?` and
  aborted the flow on error now continues on success — the intended fix, but a
  control-flow change to review. A caller that wants the residual directory
  path must capture it from `root()` **before** the consuming cleanup call.
  This note is carried in the crate CHANGELOG `Unreleased` entry, in all four
  entry points' rustdoc, in the README, and in
  `architecture/archive-extraction.md` §8.
- **Adapter consequence:** `eggup-eggpack` pins `eggup-archive = "=0.1.3"`
  (`crates/eggup-eggpack/Cargo.toml:21`). The fix therefore **cannot reach the
  published Eggpack adapter** without a separate adapter republication and an
  exact-pin cascade, per `.opencode/skills/release-workflow`. M005 performs no
  republication, no version bump, and no tag move.
- **Known downstream consumer migration required:** the eggpack **test helper**
  `finish_cleanup` needs the `Ok`-arm update shown above (finding F-1).

## Security review

- **No path-based deletion.** The crate never removes the root directory and
  never recursively traverses the root pathname. `remove_owned_root_with_hook`
  performs only `fs_at` `*at` operations relative to the retained handle.
- **No path-based creation.** `create_private_file_at` and its `create_new` /
  `follow(false)` semantics are untouched.
- **Path validation unchanged.** `validate_archive_path` and
  `validate_output_name` were not modified; the Windows device-name set is
  unchanged and still covered by a regression guard.
- **No authenticity or signature language introduced.** Integrity remains
  SHA-256 evidence only.
- **No new dependency**, no runtime `eggup-core` edge, no transport dependency,
  no Eggpack dependency.
- **`unsafe_code` still denied** at the workspace level; the crate's
  `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` attributes are
  untouched and `cargo doc` builds warning-free.
- **The recorded root path stays advisory.** The rustdoc now says so
  explicitly for all four entry points and for `residue_path()`.
- **Fail-closed preserved.** A cleanup that cannot empty the root never
  reports success and never claims the residue is empty.

## Documentation and operations evidence

| Artifact | Change |
|---|---|
| `crates/eggup-archive/src/lib.rs` rustdoc | Four cleanup entry points (`372-397`, `435-460`, `643-670`, `705-729`), `ExtractionErrorKind::CleanupFailed` (`209-214`), `ExtractionError::residue_path` (`263-279`), `extract`'s drop-semantics paragraph (`743-750`), `ExtractedArchive` struct doc (`334-339`) |
| `crates/eggup-archive/README.md` | The cleanup paragraph that said explicit cleanup "reports the now-empty directory as `CleanupFailed` residue" is replaced by the four-entry-point `Ok`/`Err` contract, the caller-owned residual directory, and the advisory-path statement |
| `crates/eggup-archive/CHANGELOG.md` | `## Unreleased` entry recording the contract correction, the behavior change, the `unwrap_err()` migration note, the member-write deduplication, the no-publication/no-migration disclaimer (retained verbatim), and the "SHA-256 evidence only" statement |
| `architecture/archive-extraction.md` | §3 public-surface table (the four "Removes…" wordings and every line reference), §4/§5/§6/§7 line references and the new shared-helper narrative, §8 stage diagram and tables, §9 residue table (now three rows of proven meaning), §10 invariant table, §12 testing approach (test counts, the new `tests/` directory, the M005 fixture table, the honest limitation on external `Err` coverage, and every test line reference), §13 (the two fixed drift entries removed and the M005 closure referenced), the review observations, and the closure-record cross-reference |
| `docs/` | **No change.** Verified by search: `docs/` contains no cleanup or residue wording at baseline, so the expected change is none |
| Operations | No new configuration, no migration step for the operator, no publication, no version bump, no tag move |

## Unresolved findings

### F-1 — `eggup-eggpack` test helper still encoded the removed "cleanup removes the root" expectation — **RESOLVED by the parent session**

**Severity: medium at assessment, now resolved.** It is a test-expectation
migration, not a production defect: the eggpack adapter library never calls the
cleanup entry points, and the crate's production behavior is correct under the
new contract. But it **failed the `eggup-eggpack` test lane**, which the
workspace gate runs, so it blocked the gate and had to be corrected before M005
could be recorded as fully closed.

**Why medium and not low:** the failure is deterministic and reproducible on
every platform, it is caused by this milestone's intended behavior change, and
it fails a gate lane. **Why medium and not high:** no production behavior is
wrong, no safety property is weakened, no data is at risk, and the fix is a
5-line test-helper change with the exact patch given in *Exact commands run*
above.

**Owner:** `crates/eggup-eggpack/tests/archive_handoff.rs` — outside this
milestone's file ownership, so it was not applied here. The parent session owns
applying it.

**Resolution (parent session).** The migration has been applied to
`crates/eggup-eggpack/tests/archive_handoff.rs`: the helper's `Ok(())` branch now
asserts the corrected contract — the owned root still exists, it is empty, and
the **caller** removes the residual directory — instead of the removed
expectation that cleanup removed the root. The `Err(CleanupFailed)` branch keeps
asserting the residue path and the caller-side removal. The helper's doc comment
was rewritten to state the M001b/M001d residue contract as corrected by M005.

No `eggup-eggpack` **production** code was changed; only the test helper. The
adapter library never called the cleanup entry points, so no adapter behavior
depended on the old contract.

### F-2 — the `Err(CleanupFailed)` path is not proven from an external integration test

**Severity: low.** Plan §10.3 anticipates this and requires the limitation to be
stated rather than papered over. The crate exposes no portable public failure
trigger: the only deterministic mechanism is exceeding the crate's own private
cleanup bounds (`CLEANUP_MAX_DEPTH`, `CLEANUP_MAX_ENTRIES_PER_PASS`), which are
not public constants, so the fixture must live inline. It does, at
`lib.rs:2741`. No external coverage of the failure path is claimed.

### F-3 — the external foreign-replacement fixture is Unix-gated

**Severity: low.** `foreign_replacement_at_the_recorded_pathname_survives_cleanup_untouched`
is `#[cfg(unix)]` because Windows refuses to rename a directory while its
handle is held (the crate's own M001d note records this). The Windows
equivalent — a reparse point installed at the recorded pathname — is covered by
the crate's inline `#[cfg(windows)]` test at `lib.rs:3072`, which was **not run
here** (no Windows host). The Windows lane must therefore execute the archive
tests on a hosted runner before this milestone is signed off.

### F-4 — the `lib.rs` module decomposition is deferred (WP5)

**Severity: low.** The decomposition of `crates/eggup-archive/src/lib.rs` into
modules is **deferred** from M005, as plan §7 WP5 records. Reason: the split is
a pure code move with no consumer-visible effect, it would add a large
mechanical diff to a milestone that already changes a published API's
observable result, and it affects neither defect's correctness. It is **not
dropped**. Named follow-up: **Archive Extraction M006 — `eggup-archive` module
decomposition**. The decision, the reason, and the follow-up name are recorded
here and are to be carried into the roadmap status table.

### F-5 — hosted platform qualification is outstanding

**Severity: medium.** Only macOS was run. Linux, Windows, and the hosted
Stable/MSRV/macOS/Windows matrix were **not run** — no hosted runner is
reachable from this machine. Plan §14 lists "the Windows lane cannot execute
the new fixtures" as a stop condition, and a platform-sensitive cleanup
contract must be qualified on Windows. The Windows lane must **execute** the
archive tests (not merely compile them) before M005 is closed.

### F-6 — `cargo fmt --all -- --check` currently fails on other crates

**Severity: low, and not this milestone's.** Every reported diff is in
`crates/eggup-core/` and `crates/eggup-service/`, which two concurrent agents
are editing. `cargo fmt -p eggup-archive -- --check` is clean. The parent
session's final workspace gate will confirm.

## Roadmap disposition

- **Archive Extraction M005 → closed, conditional on F-1 and F-5.** The
  milestone's own scope (production change, rustdoc, tests, README, CHANGELOG,
  architecture deep dive) is complete and green on macOS. It is recorded as
  *conditionally* closed because the `eggup-eggpack` test-lane migration (F-1)
  and the hosted Windows qualification (F-5) are outstanding and are not
  within this milestone's file ownership or reachable from this machine.
- **M005 is source-only.** No publication, no version bump, no tag move, no
  release changelog section. Republication of `eggup-archive` and the
  exact-pin cascade to `eggup-eggpack` remain separate, unauthorized
  milestones.
- **The roadmap's open-finding entry for the cleanup contract (registered as
  "Post-0.1.3 open finding" item 4) closes** on the correctness of the fix
  itself: the cleanup contract is now truthful, the `Ok` path is reachable and
  observable, the member-write path exists once, and the crate's own CHANGELOG,
  README, and `architecture/archive-extraction.md` agree with the code. The
  finding's *publication* consequence (that the fix cannot reach the published
  adapter without a republication) remains recorded separately.
- **M006 is registered** as the named follow-up for the deferred module
  decomposition (F-4).
- **The `eggup-eggpack` test-helper migration (F-1) is a follow-on** for the
  `eggup-eggpack` subsystem, not for Archive Extraction; it should be tracked
  as such so the archive roadmap does not carry work owned by another
  subsystem.

## Registry updates (intended, for the parent to apply)

The parent session owns `plans/registry.md` and the roadmap file; this closure
describes the intended edits rather than making them.

### `plans/registry.md`

1. **Close the cleanup-contract finding.** The "Post-0.1.3 open findings" item
   that records `eggup-archive`'s mis-documented, never-succeeding cleanup
   contract (item 4 in the current file) should be marked **closed by Archive
   Extraction M005**, with a pointer to
   `plans/closure/archive-extraction/005-status.md`. Its substance: the four
   `cleanup()` entry points now share one truthful contract, `Ok(())` means the
   owned root was emptied through the retained handle and is reachable and
   observable from outside the crate, `Err(CleanupFailed)` means it could not be
   emptied, the crate still never removes the root directory and never
   traverses the root pathname, and the member-write body exists exactly once.
   The finding's **publication** consequence is *not* closed by this: the fix
   is unpubushed, `eggup-eggpack` pins `=0.1.3`, and reaching the published
   adapter needs a separate republication and an exact-pin cascade.
2. **Add a new post-0.1.3 open finding for the downstream migration.** The
   `eggup-eggpack` test helper `finish_cleanup` still asserts the removed
   "a successful cleanup removes the root" expectation and now fails its lane
   (F-1). Severity **medium** — deterministic gate failure caused by an
   intended behavior change, no production defect. Fix: mirror the failure
   branch's residue handling in the `Ok` arm.
3. **Note the outstanding platform qualification.** The Windows lane must
   execute the archive tests before M005 is signed off (F-5), and the new
   external foreign-replacement fixture is Unix-gated (F-3).
4. **Register the M006 follow-up** for the deferred `eggup-archive` module
   decomposition (F-4).

### `plans/subsystems/archive-extraction-roadmap.md` — status table row

> | M005 | Cleanup contract and member-write corrective | **conditionally closed** — the cleanup contract is truthful and observable on all four entry points (`Ok` = emptied, `Err(CleanupFailed)` = not emptied), the member-write body exists exactly once in `write_declared_member`, and the crate's rustdoc, README, CHANGELOG `Unreleased`, and `architecture/archive-extraction.md` agree with the code. macOS green (57 tests); Linux, Windows, and the hosted matrix **not run**. Conditional on (a) the `eggup-eggpack` test-helper migration for the now-reachable `Ok` path — F-1, medium, owned by the Eggpack subsystem — and (b) hosted Windows execution of the archive tests — F-5, medium. The `lib.rs` module decomposition is **deferred** to **M006** (pure code move, no consumer-visible effect) — F-4, low. Source-only: no publication, no version bump, no tag move; the fix cannot reach the published `eggup-eggpack` adapter without a separate republication and an exact-pin cascade. See `plans/closure/archive-extraction/005-status.md`. |

## Closure checklist against `plans/closure/README.md`

| Required element | Present |
|---|---|
| status | *conditionally closed* |
| source plan and roadmap | yes |
| reviewed repository baseline | `c4805411f74b85158e0ae5d0a219a32c4209d98e` |
| implementation commits/PRs | working tree, uncommitted — no commit was made |
| executive finding | yes — contract before and after |
| requirement→evidence matrix | yes — one row per §4 invariant and per §10 fixture |
| production implementation evidence | yes — `file:line` for the shared cleanup body, the four rustdoc blocks, the shared helper and both call sites |
| exact commands run and results | yes — including the verbatim red-first failure output |
| invariant review | yes — §4, item by item |
| failure/rollback/recovery review | yes — §8, including the fail-closed assertion |
| compatibility and migration review | yes — published 0.1.3 delta, the `unwrap_err()` panic consequence, the `root()` capture requirement, the `=0.1.3` pin consequence |
| security review | yes |
| documentation/operations evidence | yes |
| unresolved findings with severity | yes — F-1 (medium), F-2 (low), F-3 (low), F-4 (low), F-5 (medium), F-6 (low) |
| roadmap disposition | yes |
| registry updates | yes — prose; the parent applies the edits |
| per-platform results reported separately | yes — macOS **run**, Linux **not run**, Windows **not run**, hosted lanes **not run** |

---

## Parent-session verification addendum (2026-10-23)

This record listed the whole-tree commands as deliberately **not run** at this
stage, because the parent session owns them, and left F-1 open because the
`eggup-eggpack` test-helper migration sat outside this milestone's ownership.
Both are now resolved by the parent session.

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 449 passed; 0 failed; 36 test binaries
```

**F-1 is resolved.** The `crates/eggup-eggpack/tests/archive_handoff.rs` helper
was migrated to the corrected contract, so the `eggup-eggpack` test lane is green
and the workspace gate passes.

**Still not run:** Linux, Windows, and every hosted CI lane (`stable`, `msrv`,
`service-systemd`, `macos`, `windows-check`). No hosted runner is reachable from
this machine, so F-5 remains the open condition on this milestone.
