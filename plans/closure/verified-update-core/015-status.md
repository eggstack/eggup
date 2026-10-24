# Verified Update Core — M015 Public-API Contract Truthfulness Corrective — Closure

Status: **conditionally closed** — every acceptance criterion in plan §13 that is
in scope for this stage is met and checked; one §6.2 sub-requirement (type-level
rustdoc for four types that live in `crates/eggup-core/src/error.rs`) is **not met**
because that file was outside this session's ownership grant. The gap is
documentation-only, has no behavioral, contract, or compatibility effect, and is
recorded as unresolved finding **U1 (low)** below. Everything else is closed.

Source plan: `plans/implementation/verified-update-core/015-public-api-contract-truthfulness-corrective.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md` (row `M015`)

Reviewed repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`
(branch `plans/service-m010-failed-unit-recovery`)

Implementation commits / PRs: **working tree, uncommitted — no commit was made.**
No git state was mutated: no `commit`, `push`, `tag`, `stash`, `checkout <path>`,
`restore`, or `clean` was run. The only git state written was
`git worktree add` / `git worktree remove` for the baseline proof, both of which
were removed again (`git worktree list` shows only the primary worktree).

## Executive finding

The published failure taxonomy of `eggup-core` contained a category that no
producer could ever construct. `FailureCategory::RetainedEvidence` was declared
at `crates/eggup-core/src/transaction.rs:116` **before this change** and `as_str()` published
the name `"retained-evidence"` at `:131`, but the only producer — the error-to-category
mapping `report_for_error` (`transaction.rs:255`) — has no arm that yields it,
and the variant was never constructed anywhere in the workspace. Separately, the
crate carried **zero** `# Errors` rustdoc sections across all of `src/`, so the
entire public fallible surface documented no failure modes at all.

Both defects are now closed. The unreachable variant and its `as_str` arm are
removed; the remaining seven categories each have exactly one producer (or are
documented as test-only), and their `as_str()` strings are byte-identical to what
`eggup-core 0.1.3` published. All 32 public fallible items now carry an
`# Errors` section that names the `Error` variants the item can return, states
the receipt-versus-`Err` distinction, and points at the repository-resident
contracts rather than restating them. A crate-local `clippy::missing_errors_doc`
attribute makes that documentation a hard gate error for this crate.

**No failure behavior changed.** The taxonomy changed shape; the failure space did
not. `Error` still has exactly the seven published variants, `FailureCategory` is
still `#[non_exhaustive]`, the transaction state machine, the lock, the receipts,
and the ownership contract are untouched, and no dependency was added.

## Requirement-to-evidence matrix

| # | Requirement (plan section) | Result | Evidence |
|---|---|---|---|
| 1 | Remove `FailureCategory::RetainedEvidence` and its `as_str` arm (§6.1) | pass | `crates/eggup-core/src/transaction.rs:135-168` — seven variants; `grep -rn "RetainedEvidence" crates/ --include='*.rs'` → zero hits (exit 1) |
| 2 | Remaining seven categories, each with a producer; `Injected` documented as test-only (§6.1, §13.2) | pass | `crates/eggup-core/src/transaction.rs:135-168` (per-variant producer docs); `architecture/core-transaction.md` §5 "The published failure taxonomy" (producer table) |
| 3 | `as_str()` byte-identical for all seven remaining categories, pinned by T2 (§6.1, §13.3) | pass | `crates/eggup-core/src/failure_taxonomy_tests.rs` — `category_strings_are_byte_identical` |
| 4 | `FailureCategory` remains `#[non_exhaustive]` (§6.1, §9) | pass | `crates/eggup-core/src/transaction.rs:134` (`#[non_exhaustive]`), unchanged |
| 5 | All 32 public fallible items carry an `# Errors` section (§6.2, §13.4) | pass | `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` → clean with `clippy::missing_errors_doc` enabled crate-locally (`crates/eggup-core/src/lib.rs:8`) |
| 6 | Each section names the real `Error` variants and the receipt-vs-`Err` distinction (§6.2 items 1–2, §13.5) | pass | `crates/eggup-core/src/candidate.rs:181, 342, 532, 565, 582, 617, 656, 687` |
| 7 | Fail-closed direction stated where it applies (§6.2 item 3) | pass | `crates/eggup-core/src/candidate.rs:592-597`; `crates/eggup-core/src/domain.rs:155-186` (`OwnershipVerifier`) |
| 8 | `IntegrityRequirement::None` rule stated on validation/commit items (§6.2 item 4) | pass | `crates/eggup-core/src/candidate.rs:532, 582`; `crates/eggup-core/src/integrity.rs:194` |
| 9 | No authenticity or signature claim anywhere (§6.2 item 5, §13.7) | pass | Every added `# Errors` section says "SHA-256 digest evidence" / "integrity evidence only"; `grep -n "signature\|authenticity" crates/eggup-core/src/*.rs` → only the pre-existing negative statements in `integrity.rs` |
| 10 | Rustdoc points at the contracts, does not restate them (§6.3, §13.6) | pass | Every section points at `crates/eggup-core/docs/{domain,verification,transaction}.md` and/or `architecture/core-transaction.md` §8 |
| 11 | Type-level rustdoc for the §6.2 "also required" types | **partial** | `FailureCategory`, `FailureReport`, `FailurePhase`, `TransactionReceipt`, `TransactionDisposition`, `CleanupDisposition`, `OwnershipVerifier` done. `Error`, `RecoveryError`, `RecoveryResult`, `Result` are in `crates/eggup-core/src/error.rs`, **outside this session's ownership grant** → unresolved finding **U1** |
| 12 | T1 fixture fails against the pre-M015 baseline with the recorded diagnostic and passes on the post-M015 tree (§13.8) | pass | Baseline worktree run → `EXIT=101`, `2 failed; 1 passed`, diagnostic `declared category RetainedEvidence has no producer`; post-M015 tree → `3 passed` |
| 13 | The crate-local doc-accuracy check is enabled and reports zero missing sections (§6.4, §13.4) | pass | `crates/eggup-core/src/lib.rs:8`; probe below shows the lint fires when a section is removed |
| 14 | `eggup-core` dependency surface unchanged; no other crate changed (§13.10) | pass | `cargo tree -p eggup-core --locked --offline` → `sha2` only; `--target x86_64-pc-windows-msvc` → `sha2` + `self-replace`; no `Cargo.toml`/`Cargo.lock` edit |
| 15 | Whole `eggup-core` suite green, no weakened assertion (§13.9) | pass (crate scope) | `cargo test -p eggup-core --locked --offline` → 93 tests, 0 failed |
| 16 | Whole workspace suite green (§13.9) | pass | Run by the parent session: `cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast` → 483 passed, 0 failed, 44 test binaries. See § "Parent-session verification addendum (2026-10-23, hosted qualification)". |
| 17 | `./scripts/check-local.sh` green (§13.11) | pass | Run by the parent session on the same tree → exit 0. See § "Parent-session verification addendum (2026-10-23, hosted qualification)". |
| 18 | Linux / macOS / Windows results separated; unrun lanes marked "not run" (§13.12) | pass | Platform section below; every lane has since run, so no lane remains unrun. |
| 19 | Changelog entry stating the removal, the unchanged strings, the no-migration statement, and the unpublished status (§12) | pass | `crates/eggup-core/CHANGELOG.md` — `## Unreleased`, M015 entry |
| 20 | Architecture deep dive reflects the post-M015 taxonomy and carries no stale description of the removed variant (§12) | pass | `architecture/core-transaction.md:141` (public-surface table) and the new §5 subsection "The published failure taxonomy" |

## Production implementation evidence

| Change | Location | Note |
|---|---|---|
| `FailureCategory::RetainedEvidence` removed | `crates/eggup-core/src/transaction.rs:135-168` | Seven variants remain |
| `Self::RetainedEvidence => "retained-evidence"` arm removed | `crates/eggup-core/src/transaction.rs:177-187` | The `as_str()` match now has seven arms |
| `FailureCategory` type-level rustdoc | `crates/eggup-core/src/transaction.rs:110-134` | States that every member describes a failure that can occur, that `Injected` is test-only, that `as_str()` is the only stable identifier, and that retained evidence is a disposition, not a category |
| Per-variant producer docs | `crates/eggup-core/src/transaction.rs:136-167` | Each variant names the `Error` variant(s) that produce it |
| `as_str()` rustdoc | `crates/eggup-core/src/transaction.rs:171-176` | Stable-identifier contract |
| `FailureReport` type-level rustdoc | `crates/eggup-core/src/transaction.rs:190-200` | Evidence, not disposition; never carries a retained path |
| `FailurePhase` type-level rustdoc | `crates/eggup-core/src/transaction.rs:68-74` | Where vs. what |
| `TransactionReceipt` type-level rustdoc | `crates/eggup-core/src/transaction.rs:283-295` | A receipt — not an `Err` — reports a post-lock outcome |
| `TransactionDisposition` type-level rustdoc | `crates/eggup-core/src/transaction.rs:14-22` | Disposition says coherence, not cause |
| `CleanupDisposition` type-level rustdoc | `crates/eggup-core/src/transaction.rs:32-42` | Retained evidence lives here, not in the taxonomy |
| `OwnershipVerifier` type-level rustdoc | `crates/eggup-core/src/domain.rs:155-186` | The only source of `Owned`; infallible by design |
| `# Errors` — `run_bounded` | `crates/eggup-core/src/candidate.rs:181` | Consistent with M014: a timeout, an output bound, and a reader error are **not** `Err` |
| `# Errors` — `CandidateValidator::validate` | `crates/eggup-core/src/candidate.rs:342` | |
| `# Errors` — `VerifiedTransaction::validate` | `crates/eggup-core/src/candidate.rs:532` | `None` is always rejected |
| `# Errors` — `ValidatedTransaction::staged_path` | `crates/eggup-core/src/candidate.rs:565` | |
| `# Errors` — `ValidatedTransaction::commit` | `crates/eggup-core/src/candidate.rs:582` | Receipt-vs-`Err`; fail-closed ownership |
| `# Errors` — `commit_with_post_commit` | `crates/eggup-core/src/candidate.rs:617` | A failing check is a receipt, not an `Err` |
| `# Errors` — `commit_with_stale_lock_recovery` | `crates/eggup-core/src/candidate.rs:656` | `RecoveryError::Core` vs. `RecoveryError::RecoveryRequired` |
| `# Errors` — `commit_with_post_commit_and_stale_lock_recovery` | `crates/eggup-core/src/candidate.rs:687` | Union of the two |
| `# Errors` — `CurrentExecutable::resolve` / `bind` | `crates/eggup-core/src/current_exe.rs:38, 58` | |
| `# Errors` — `ProductId::new`, `ReleaseId::new`, `MemberId::new` | `crates/eggup-core/src/domain.rs:17, 44, 71` | |
| `# Errors` — `ArtifactMember::new`, `ArtifactSet::new`, `ArtifactSet::single` | `crates/eggup-core/src/domain.rs:320, 395, 419` | |
| `# Errors` — `InstallPlan::new`, `for_current_executable`, `destination`, `prepare`, `prepare_with_bound_sources` | `crates/eggup-core/src/domain.rs:522, 597, 668, 683, 707` | |
| `# Errors` — `hash_file`, `parse_sha256_sidecar`, `verify_file` | `crates/eggup-core/src/integrity.rs:66, 91, 132` | |
| `# Errors` — `PreparedTransaction::verify_integrity`, `VerifiedTransaction::staged_path` | `crates/eggup-core/src/integrity.rs:194, 268` | |
| `# Errors` — `MutationLock::acquire`, `acquire_with_recovery`, `observe`, `inspect` | `crates/eggup-core/src/lock.rs:206, 243, 331, 358` | |
| `# Errors` — `PreparedTransaction::destination`, `staged_path` | `crates/eggup-core/src/stage.rs:171, 181` | |
| Crate-local doc-accuracy check | `crates/eggup-core/src/lib.rs:3-8` | `#![warn(clippy::missing_errors_doc)]` |
| New test module declaration | `crates/eggup-core/src/lib.rs:58-59` | `#[cfg(test)] mod failure_taxonomy_tests;` |
| New taxonomy fixture | `crates/eggup-core/src/failure_taxonomy_tests.rs` (new, 378 lines) | T1 + T2 |
| Changelog entry | `crates/eggup-core/CHANGELOG.md` — `## Unreleased` | Added alongside M014's and M012's entries; M014's entry was not touched |
| Architecture: public-surface table row | `architecture/core-transaction.md:141` | Seven categories, "each with a producer" |
| Architecture: new §5 subsection | `architecture/core-transaction.md` — "The published failure taxonomy" | Producer table, `as_str()` contract, retained-evidence-is-a-disposition statement, doc-accuracy note |
| Architecture: §8 cross-reference | `architecture/core-transaction.md` — "Failure semantics" | Points at the taxonomy subsection |
| Architecture: status line | `architecture/core-transaction.md:3-5` | States that the taxonomy material reflects M015 and that `0.1.3` remains the published baseline |

## Exact commands run and results

All cargo invocations used `--offline`; no network was available or used.

### 1. Crate test suite

```text
$ cargo test -p eggup-core --locked --offline
   Compiling eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.63s
     Running unittests src/lib.rs (target/debug/deps/eggup_core-592fb1cc0a42851a)
running 56 tests
test result: ok. 56 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.80s
     Running tests/candidate_deadline.rs
running 12 tests
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.01s
     Running tests/current_executable.rs
running 11 tests
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
     Running tests/stale_lock_recovery.rs
running 14 tests
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
   Doc-tests eggup_core
running 0 tests
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

93 tests, 0 failed. The 56 unit tests include the 3 new
`failure_taxonomy_tests::*` fixtures. No pre-existing assertion was weakened,
removed, or reworded.

### 2. Clippy gate (this is where the new crate-local lint proves itself)

```text
$ cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings
    Checking eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
clippy-exit=0
```

### 3. Formatting (per-crate form; `--all` is out of scope — `crates/eggup-service/` is owned by a concurrent agent)

```text
$ cargo fmt -p eggup-core -- --check
fmt-exit=0
```

`cargo fmt -p eggup-core` was run once to format the new fixture file. It
reported diffs **only** in `crates/eggup-core/src/failure_taxonomy_tests.rs`; no
other file in the crate, and in particular none of M014's `candidate.rs` code, was
reformatted.

### 4. Documentation build and rendered-section count

```text
$ cargo doc -p eggup-core --no-deps --locked --offline
 Documenting eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s
   Generated /Users/davidbowman/projects/eggup/target/doc/eggup_core/index.html
doc-exit=0

$ grep -rho 'id="errors"' target/doc/eggup_core/ | wc -l
      16
$ grep -rho 'id="errors[^"]*"' target/doc/eggup_core/ | wc -l
      32
```

**Read this pair carefully.** The plan's literal command reports **16**, not 32,
because rustdoc suffixes a repeated `id="errors"` anchor on a page that carries
several methods (`errors-1`, `errors-2`, …). The complete anchor breakdown is:

```text
$ grep -rho 'id="[^"]*errors[^"]*"' target/doc/eggup_core/ | sort | uniq -c | sort -rn
     16 id="errors"
      7 id="errors-1"
      4 id="errors-2"
      3 id="errors-3"
      2 id="errors-4"
```

16 + 7 + 4 + 3 + 2 = **32** rendered `# Errors` sections, one per public fallible
item, matching the 32 items in plan §3.4 exactly. The plan's command remains a
valid zero-vs-non-zero sanity figure (it reports `0` on the pre-M015 tree, verified
below); the lint is the canonical check.

### 5. Pre-M015 baseline failure proof — **mechanism: `git worktree`**

The plan's preferred mechanism was used; no file in the primary working tree was
swapped or restored.

```text
$ git worktree add /tmp/eggup-baseline-core015 c4805411f74b85158e0ae5d0a219a32c4209d98e
Preparing worktree (detached HEAD c480541)
HEAD is now at c480541 docs(service): reconcile wg-basic M004 closure
$ cp crates/eggup-core/src/failure_taxonomy_tests.rs \
    /tmp/eggup-baseline-core015/crates/eggup-core/src/failure_taxonomy_tests.rs
$ printf '\n#[cfg(test)]\nmod failure_taxonomy_tests;\n' \
    >> /tmp/eggup-baseline-core015/crates/eggup-core/src/lib.rs
$ cd /tmp/eggup-baseline-core015 && cargo test -p eggup-core --locked --offline failure_taxonomy
   Compiling eggup-core v0.1.3 (/private/tmp/eggup-baseline-core015/crates/eggup-core)
warning: this lint expectation is unfulfilled
  --> crates/eggup-core/src/failure_taxonomy_tests.rs:74:5
   |
74 |     unreachable_patterns,
   |     ^^^^^^^^^^^^^^^^^^^^
   = note: the taxonomy has exactly the seven members pinned by this milestone
   = note: `#[warn(unfulfilled_lint_expectations)]` on by default
warning: `eggup-core` (lib test) generated 1 warning
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.05s
     Running unittests src/lib.rs (target/debug/deps/eggup_core-592fb1cc0a42851a)

running 3 tests
test failure_taxonomy_tests::category_strings_are_byte_identical ... ok
test failure_taxonomy_tests::declared_taxonomy_has_exactly_the_published_members ... FAILED
test failure_taxonomy_tests::every_declared_category_has_a_producer ... FAILED

---- failure_taxonomy_tests::declared_taxonomy_has_exactly_the_published_members stdout ----
thread '...' panicked at crates/eggup-core/src/failure_taxonomy_tests.rs:301:5:
declared category RetainedEvidence is not part of the published taxonomy; declared members: ["LockContention", "OwnershipConflict", "Verification", "Filesystem", "InvalidInput", "Injected", "RetainedEvidence", "PostCommitCheck"]

---- failure_taxonomy_tests::every_declared_category_has_a_producer stdout ----
thread '...' panicked at crates/eggup-core/src/failure_taxonomy_tests.rs:322:5:
declared category RetainedEvidence has no producer; declared members: ["LockContention", "OwnershipConflict", "Verification", "Filesystem", "InvalidInput", "Injected", "RetainedEvidence", "PostCommitCheck"]

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 53 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p eggup-core --lib`
EXIT=101
```

**The fixture compiles against the pre-M015 tree and fails at runtime** — the
required evidence form. The recorded diagnostic is the plan's own wording:
`declared category RetainedEvidence has no producer`. The baseline also emits an
`unfulfilled_lint_expectations` compiler diagnostic, because the baseline's
binding match arm is reachable there — a second, independent signal that the
baseline declares eight members.

The baseline tree's own 53 unit tests were untouched and still pass there; the
two failures above are the new fixtures only.

### 6. Baseline documentation count

```text
$ cd /tmp/eggup-baseline-core015 && cargo doc -p eggup-core --no-deps --locked --offline
$ grep -rho 'id="errors"' target/doc/eggup_core/ | wc -l
       0
$ grep -rho 'id="errors[^"]*"' target/doc/eggup_core/ | wc -l
       0
```

The pre-M015 tree renders **zero** `# Errors` sections — the defect the milestone
set out to fix.

### 7. Worktree teardown

```text
$ git worktree remove --force /tmp/eggup-baseline-core015
$ git worktree list
/Users/davidbowman/projects/eggup c480541 [plans/service-m010-failed-unit-recovery]
```

Only the primary worktree remains. The primary working tree was never mutated for
the proof.

### 8. Lint-fires probe (proves the crate-local attribute is load-bearing)

The `# Errors` heading of `integrity.rs::hash_file` was temporarily renamed to
`# Error`, clippy was re-run, and the heading was restored:

```text
$ cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings
error: docs for function returning `Result` missing `# Errors` section
  --> crates/eggup-core/src/integrity.rs:72:1
   |
72 | pub fn hash_file(path: &Path) -> Result<[u8; 32]> {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   = help: for further information visit https://rust-lang.github.io/rust-clippy/master/index.html#missing_errors_doc
error: could not compile `eggup-core` (lib) due to 1 previous error
```

The section was restored immediately afterwards and the clippy gate re-run clean
(§2 above). The file is byte-identical to its pre-probe state.

### 9. Acceptance-criteria checks

```text
$ grep -rn "RetainedEvidence" crates/ --include='*.rs'
(zero hits; grep exit status 1)

$ cargo tree -p eggup-core --locked --offline
eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
└── sha2 v0.10.9
    ...

$ cargo tree -p eggup-core --locked --offline --target x86_64-pc-windows-msvc
eggup-core v0.1.3 (/Users/davidbowman/projects/eggup/crates/eggup-core)
├── self-replace v1.5.0
    ...
    └── sha2 v0.10.9
```

The dependency surface is unchanged: `sha2` plus the Windows-only `self-replace`.

### 10. Ownership-boundary check

```text
$ git diff -U0 crates/eggup-core/src/candidate.rs | grep -E '^\+' | grep -vE '^\+\+\+' \
    | grep -vE '^\+\s*(///|//!)' | grep -vE '^\+\s*$'
```

Every non-doc addition in `candidate.rs` is inside M014's `run_bounded` region
(the `poll_reader` helper and its call sites), i.e. pre-existing working-tree
work that this milestone did not author and did not revert. `current_exe.rs`,
`domain.rs`, `integrity.rs`, `lock.rs`, and `stage.rs` show **zero** non-doc
changes. `lib.rs` shows exactly the two permitted additions (the crate-local lint
attribute and the `#[cfg(test)] mod failure_taxonomy_tests;` declaration).

## Platform evidence

| Lane | Platform | Result |
|---|---|---|
| `cargo test -p eggup-core --locked --offline` | macOS (darwin arm64), native | **run — 93 passed, 0 failed** |
| `cargo clippy -p eggup-core --all-targets --locked --offline -- -D warnings` | macOS (darwin arm64), native | **run — clean** |
| `cargo fmt -p eggup-core -- --check` | platform-independent | **run — clean** |
| `cargo doc -p eggup-core --no-deps --locked --offline` | platform-independent | **run — clean, 32 rendered `# Errors` sections** |
| Pre-M015 baseline failure proof (`git worktree`) | macOS (darwin arm64), native | **run — EXIT=101, runtime failure with the recorded diagnostic** |
| `cargo tree` dependency review | macOS + `x86_64-pc-windows-msvc` target | **run — `sha2` (+ Windows-only `self-replace`)** |
| Linux | — | **PASS** — hosted run `40008039843`: the `Stable checks` lane (ubuntu-latest), the `MSRV check` lane (Rust 1.89.0), and the `Linux systemd service qualification` lane. See the addendum at the end of this record. |
| Windows | — | **PASS** — hosted run `40008039843`: the `Windows archive, acquisition, and service tests and check` lane (windows-latest) — 598 passed, 0 failed, 0 ignored, 63 test binaries. See the addendum at the end of this record. |
| Hosted CI `stable` | — | **PASS** — hosted run `40008039843`. See the addendum at the end of this record. |
| Hosted CI `msrv` | — | **PASS** — hosted run `40008039843`. See the addendum at the end of this record. |
| Hosted CI `macos` | — | **PASS** — hosted run `40008039843`. See the addendum at the end of this record. |
| Hosted CI `windows-check` | — | **PASS** — hosted run `40008039843`. See the addendum at the end of this record. |
| Hosted CI `service-systemd` | — | **PASS** — hosted run `40008039843`. See the addendum at the end of this record. |
| `cargo test --workspace` | — | **PASS** — run by the parent session: 483 passed, 0 failed, 44 test binaries. See the addendum at the end of this record. |
| `./scripts/check-local.sh` | — | **PASS** — run by the parent session on the same tree → exit 0. See the addendum at the end of this record. |

No result on one platform was inferred from another. The change is
documentation-and-taxonomy-shape only and contains no platform-conditional code;
the Linux, macOS, and Windows lanes have now all run and all passed.

## Invariant review

| Invariant (AGENTS.md / `architecture/overview.md`) | Result | Evidence |
|---|---|---|
| SHA-256 is integrity evidence only — no authenticity or signature claim | pass | No added wording implies publisher trust or signature verification; `integrity.rs` additions say "integrity evidence only" |
| Ownership `Owned` is never inferred; ambiguity is `Unknown`, which fails closed | pass | No ownership code touched; `domain.rs:141-160` now states the rule on `OwnershipVerifier` |
| `commit` revalidates ownership and staged digests under lock; `Foreign`/`Unknown`/flap produce a receipt, not `Err` | pass | No transaction code touched. Stated in the new `# Errors` sections: `candidate.rs:581-598` |
| `validate` requires every member to carry `IntegrityStatus::Verified`; a `None` member always fails, with no escape hatch | pass | No validation code touched. Stated in `candidate.rs:531-538` and `integrity.rs:170-182` |
| Core never creates live destination parents and never decides a lock is stale | pass | No code touched; the new fixture itself creates the destination parent (`failure_taxonomy_tests.rs`), and the lock rustdoc says the same |
| `MutationLock::acquire` never recovers; recovery is opt-in via a caller-supplied `StaleLockVerifier` | pass | No lock code touched; `lock.rs:205-216, 234-252` state it |
| Retained evidence is reported as a disposition plus a real path, never as a failure category | pass | The removed variant is exactly the residue this rule forbade; `CleanupDisposition::RetainedForRecovery` and `TransactionReceipt::recovery_path` are unchanged and now documented as the carriers |
| The `Error` variant set is unchanged | pass | `crates/eggup-core/src/error.rs` was **not modified**; `Error` still has exactly the seven published variants plus the `#[cfg(test)]`-only `Error::Injected` |
| `FailureCategory` stays `#[non_exhaustive]` | pass | `transaction.rs:134` |
| The transaction state machine, the lock, the receipts, and the failure behavior are unchanged | pass | `transaction.rs`'s only behavioral edit is the removal of one never-constructed variant and its `as_str` arm; every other change in `transaction.rs` is rustdoc |
| No dependency added, including dev-dependencies | pass | No `Cargo.toml`/`Cargo.lock` edit; `cargo tree` unchanged |
| Crate dependency boundaries (core stays policy-neutral; no transport/service/Eggpack deps) | pass | `cargo tree` unchanged |

## Failure, rollback, and recovery review

No failure path, rollback path, or recovery path was modified. This milestone
removes a category that no failure could carry and adds documentation.

- **Failure space unchanged.** Every `FailureReport` a caller can observe before
  and after this change carries one of the same six producible categories, plus
  `PostCommitCheck` on the post-commit path. `FailureCategory::Injected` remains
  reachable only through the `#[cfg(test)]`-only `Error::Injected`, so a shipped
  build still cannot produce the `injected` category.
- **Rollback unchanged.** `TransactionDisposition::RolledBack`,
  `CleanupDisposition`, `rollback_performed`, `rollback_verified`, and
  `recovery_path` are byte-for-byte unchanged. The only textual change is that
  the rustdoc now names `CleanupDisposition::RetainedForRecovery` and
  `TransactionReceipt::recovery_path` as the carriers of retained evidence, which
  is where they already were.
- **Recovery unchanged.** `RecoveryError` keeps its two variants
  (`Core`, `RecoveryRequired`) and its `#[non_exhaustive]` attribute;
  `MutationLock::acquire_with_recovery` is untouched and still fail-closed
  unless the caller's `StaleLockVerifier` returns `ProvenStale` for the exact
  observed record.
- **The fixture exercises the failure path, not just the taxonomy.** The
  post-commit category in the T1 fixture is obtained from a real transaction:
  a real plan, a real private stage, a real SHA-256 verification, a real
  validation, and a real `commit_with_post_commit` whose check fails, with the
  category read from the receipt's real `post_commit_failure` report. The
  fixture creates the live destination parent itself, because core never does.
- **No crash-journaling or timeout claim was introduced.** The `run_bounded`
  `# Errors` section restates M014's wording and adds no new guarantee.

## Compatibility and migration review

- **`Error` still has exactly the seven published variants.** `Error` is a
  **closed** enum (it carries no `#[non_exhaustive]`) with `InvalidInput`,
  `UnknownMember`, `Io`, `UpdateInProgress`, `DestinationConflict`,
  `VerificationFailed`, `CandidateExecution`, plus the `#[cfg(test)]`-only
  `Error::Injected`. A downstream exhaustive `match` on `Error` compiles
  unchanged. `crates/eggup-core/src/error.rs` was not modified at all.
- **`FailureCategory` is `#[non_exhaustive]`, so downstream matchers keep
  compiling.** Every downstream matcher must already carry a wildcard arm;
  removing one member leaves that arm covering one fewer variant and changes no
  downstream compile outcome.
- **`RecoveryError` is unchanged** — two variants, `#[non_exhaustive]`.
- **The one source-visible change:** `eggup-core 0.1.3` on crates.io **does**
  publish `FailureCategory::RetainedEvidence` and the string
  `"retained-evidence"` (verified by reading the published tree at the 0.1.3
  publication SHA `bd43683`). A downstream crate that *named* the variant
  explicitly — a match arm, an equality assertion, or a constant `as_str()`
  call — will fail to compile once this fix is published. That is a
  **compile-time** break with no silent behavior change, and it is the only
  downstream break this correction introduces. `eggup-core 0.1.2` never
  published the variant (verified against the vendored 0.1.2 package fixture at
  `plans/closure/verified-update-core/008-package-fixture/vendor/eggup-core-0.1.2/`),
  so the break window is `0.1.3` and later only.
- **No runtime behavior change.** No produced `FailureReport`, no
  `TransactionReceipt`, no `Error`, and no `RecoveryError` differs before and
  after.
- **No migration path is required.** No data format changed, no wire format
  changed, no configuration changed. A consumer that persists `as_str()` strings
  keeps working for all seven remaining categories; only the never-observable
  `"retained-evidence"` string disappears.
- **Publication is a separate milestone.** `eggup-core 0.1.3` on crates.io
  still carries both defects. The fix reaches consumers only when a new version
  is published, and no publication milestone authorizes that here. Because
  `eggup-eggpack` pins `eggup-core = "=0.1.3"`, an adapter republication is
  required before the fix can reach that crate. The `v0.1.3` tag must not move.

## Security review

- **No authenticity or signature claim was added.** Every new `# Errors`
  section that touches integrity says "SHA-256 digest evidence" or "integrity
  evidence only". The crate contains no authenticity verifier and still claims
  none.
- **No secret, path-disclosure, or permission change.** The new fixture reads
  only this crate's own `src/transaction.rs` and writes only into the test
  harness's temporary directories. It prints no path that is not already printed
  by the existing test suite.
- **The taxonomy change reduces, not expands, the published attack surface.** A
  category that no producer can construct cannot be produced, so removing it
  cannot change a decision any consumer makes at runtime.
- **The doc-accuracy check does not weaken any existing lint.** The crate-local
  `#![warn(clippy::missing_errors_doc)]` adds a lint; it removes none. The
  workspace lint table is unchanged and no other crate is affected.
- **No `unsafe` was introduced.** The crate still carries
  `#![forbid(unsafe_code)]`; the new fixture uses only safe Rust.
- **The fixture does not weaken a security assertion.** It reads its own source
  file read-only and asserts; it never mutates the tree it inspects.

## Documentation and operations evidence

- `crates/eggup-core/CHANGELOG.md` — an `## Unreleased` entry added **alongside**
  M014's and M012's entries (neither was replaced). It states: the unreachable
  variant and its `as_str` arm are removed; the remaining seven `as_str()`
  strings are byte-identical to what `0.1.3` published; the failure space is
  unchanged and `Error` keeps exactly the seven published variants; the public
  fallible surface now documents its failure modes; the crate-local
  doc-accuracy check was added; `FailureCategory::Injected` is documented as
  test-only; the compatibility caveat and the "no consumer migration required"
  statement; and that nothing here has been published, that `0.1.3` is the
  published baseline, that `eggup-eggpack`'s `=0.1.3` pin blocks propagation, and
  that republication is a separate milestone with the `v0.1.3` tag unmoved. The
  historical M012 bullet that called the now-removed variant "genuinely additive"
  was annotated with a one-line cross-reference to the M015 entry rather than
  being rewritten.
- `architecture/core-transaction.md` — the §3 public-surface table row for
  `FailureCategory` now lists the seven categories and states that each has a
  producer; a new §5 subsection "The published failure taxonomy" gives the
  per-category producer table, the `as_str()` stability contract, the statement
  that retained evidence is a disposition and not a category, the origin of the
  removed variant, the test-only status of `Injected`, and the
  `clippy::missing_errors_doc` note; §8 now cross-references the subsection; the
  status line states that the taxonomy material reflects M015 and that `0.1.3`
  remains the published baseline.
- `crates/eggup-core/docs/{domain,verification,transaction}.md` — **not
  modified.** They are owned by other milestones, and the sweep confirmed that
  none of them mentions `FailureCategory` at all, so there was no contract
  statement to reconcile.
- **The `Known doc/code drift` list in `architecture/core-transaction.md` was not
  edited.** The sweep found **no new** code-versus-contract disagreement: the
  contract docs never described the taxonomy, so removing the unreachable
  variant contradicts no contract. The four pre-existing drift entries (about
  report retention, `rollback_verified` precision, the unconditional permission
  sentence, and the previous revision's public surface) are untouched.
- **Operational impact: none at runtime.** No configuration, no flag, no
  migration, no operator procedure changes. The only operator-visible change is
  in the published rustdoc, which is strictly more informative than before.

## Unresolved findings

### U1 — Four §6.2 type-level rustdocs were not written, because their file was outside the ownership grant

**Severity: low.**

**Finding.** Plan §6.2's "Also required" clause lists eleven types whose
type-level rustdoc must state what the type means. Seven of them are done
(`FailureCategory`, `FailureReport`, `FailurePhase`, `TransactionReceipt`,
`TransactionDisposition`, `CleanupDisposition`, `OwnershipVerifier`). The
remaining four — `Error`, `RecoveryError`, `RecoveryResult`, and the `Result`
alias — all live in `crates/eggup-core/src/error.rs`, which was **not** in this
session's ownership grant; the grant enumerated
`transaction.rs`, `candidate.rs`, `current_exe.rs`, `domain.rs`, `integrity.rs`,
`lock.rs`, `stage.rs`, `failure_taxonomy_tests.rs`, `lib.rs`, the crate
`CHANGELOG.md`, and `architecture/core-transaction.md`. The file was therefore
left untouched rather than edited outside the grant.

**Why this severity.** The gap is **documentation-only** and has no behavioral,
contract, or compatibility effect:

1. `clippy::missing_errors_doc` does not require an `# Errors` section on an
   error *type* — the lint targets fallible *functions*. Acceptance criterion 4
   is therefore met without these edits, and the gate is clean.
2. `RecoveryError` already carries type-level rustdoc that states what it means
   (its `error.rs:46-52` doc explains the split from `Error` and the
   exhaustiveness rationale) and `RecoveryResult` already says "The result type
   returned by the recovery-aware entry points." Both already satisfy §6.2's
   requirement as written.
3. `Error`'s existing one-line doc — "Errors produced while validating a local
   update plan or preparing private stage state" — is narrower than the type's
   real role (it also carries commit-time `UpdateInProgress`,
   `DestinationConflict`, `VerificationFailed`, and `CandidateExecution`). That
   is a pre-existing imprecision, not a defect this milestone introduced.
4. The severity is **not** raised to medium because the plan's §13 acceptance
   criteria do not list these type docs, the defect the milestone exists to fix
   (an unproducible published category, and a public `Result` surface with no
   documented failure modes) is fully closed, and no consumer-visible contract
   depends on the wording. It is not lowered below low because a reader of the
   published `Error` docs can still form a wrong model of which failures reach
   `Err` versus a receipt.

**Ready-to-apply follow-up** (one file, `crates/eggup-core/src/error.rs`, rustdoc
only, no code change). The parent session owns that file's grant, or can extend
this milestone's:

```rust
/// The error type of every `eggup-core` fallible entry point.
///
/// `Error` is a **closed** enum with exactly the seven variants below, so a
/// downstream exhaustive `match` compiles unchanged. It covers both the
/// pre-transaction failures (plan validation, private stage creation and copy)
/// and the commit-time failures that are returned as `Err` rather than as a
/// receipt: lock contention (`UpdateInProgress`), a preflight ownership
/// conflict (`DestinationConflict`), failed integrity or candidate evidence
/// (`VerificationFailed`, `CandidateExecution`), and live filesystem failures
/// (`Io`).
///
/// Everything reached **after** the mutation lock is held is reported through a
/// [`crate::TransactionReceipt`], not through this type: an ownership conflict
/// or flap produces a `RolledBack` receipt, and a failure that cannot be
/// resolved produces a `RecoveryRequired` receipt with a real
/// `recovery_path`. See `architecture/core-transaction.md` §8 "Failure
/// semantics".
```

and, on the `Result` alias:

```rust
/// The result type used by eggup-core.
///
/// `Err` means setup failed and nothing live changed. A post-lock outcome is
/// reported through a [`crate::TransactionReceipt`], not through this type.
```

**Everything else is closed.** No other requirement in the plan was left
unmet, weakened, or approximated.

## Roadmap disposition

`M015` moves from **`ready for handoff`** to **`closed`** in
`plans/subsystems/verified-update-core-roadmap.md`.

- The milestone closes registry open finding **10** (**medium**).
- The milestone is **unpublished**: the defect still exists in `eggup-core
  0.1.3` on crates.io. The roadmap row must keep saying so.
- `eggup-eggpack`'s `eggup-core = "=0.1.3"` pin blocks propagation to that
  adapter; an adapter republication is required before the fix reaches it.
- The next verified-update-core milestone should be the publication of the
  corrected `eggup-core`, with the `=0.1.3` pin cascade handled by the
  release workflow. No publication is authorized by this closure.
- Because the change is taxonomy-shape plus documentation only, no roadmap
  status of any other milestone changes, and no subsystem other than
  `verified-update-core` is affected.

## Registry updates

The parent session applies these edits; this closure only describes them.

### `plans/registry.md` — post-0.1.3 open finding 10

Finding 10 moves from **open** to **closed**, with this transition recorded in
the table's status column:

> **Closed by Verified Update Core M015** —
> `plans/implementation/verified-update-core/015-public-api-contract-truthfulness-corrective.md`,
> `plans/closure/verified-update-core/015-status.md`. The unreachable
> `FailureCategory::RetainedEvidence` variant and its `as_str` arm are removed;
> the remaining seven categories each have exactly one producer, or are
> documented as test-only (`Injected`), and their `as_str()` strings are
> byte-identical to what `0.1.3` published. All 32 public fallible items now
> carry an `# Errors` section, and `clippy::missing_errors_doc` is enabled
> crate-locally so the existing gate keeps that documentation honest. **Not
> yet published**: `eggup-core 0.1.3` on crates.io still carries the defect, and
> `eggup-eggpack`'s `=0.1.3` pin blocks propagation. One residual documentation
> item remains open at low severity — the type-level rustdoc for `Error`,
> `RecoveryError`, `RecoveryResult`, and `Result` in
> `crates/eggup-core/src/error.rs` (closure finding U1) — with no behavioral or
> compatibility effect.

### `plans/subsystems/verified-update-core-roadmap.md` — status table row

Replace the `M015` row with:

```markdown
| M015 | closed | `plans/implementation/verified-update-core/015-public-api-contract-truthfulness-corrective.md` | `plans/closure/verified-update-core/015-status.md` | Closes registry open finding 10 (**medium**). `FailureCategory::RetainedEvidence` is removed as unproducible; the remaining seven categories each have exactly one producer (or are documented test-only) and their `as_str()` strings are byte-identical to the published ones. All 32 public fallible items carry an `# Errors` section, enforced by a crate-local `clippy::missing_errors_doc`. **Unpublished**: published `eggup-core 0.1.3` keeps the defect; `eggup-eggpack`'s `=0.1.3` pin blocks propagation; one low-severity residual rustdoc item remains open in `crates/eggup-core/src/error.rs` |
```

### Root `CHANGELOG.md` — `Unreleased` aggregate

```markdown
- **`eggup-core`** — corrected the published failure taxonomy and documented the
  public failure surface. The unreachable `FailureCategory::RetainedEvidence`
  variant and its `retained-evidence` string are removed; the remaining seven
  categories each have exactly one producer (or are documented as test-only),
  and their strings are byte-identical to the published ones. Every public
  fallible item now carries an `# Errors` section, enforced by a crate-local
  `clippy::missing_errors_doc`. `Error` still has exactly the seven published
  variants and `FailureCategory` is still `#[non_exhaustive]`, so downstream
  matchers keep compiling; the only downstream break is an explicit reference to
  the removed variant name. Unpublished — `eggup-core 0.1.3` on crates.io still
  carries the defect, and `eggup-eggpack`'s `=0.1.3` pin blocks propagation.
```

---

## Parent-session verification addendum (2026-10-23)

This record listed the whole-workspace suite and `./scripts/check-local.sh` as
**not run**, because the parent session owns them. The parent session has since
run both. This addendum records the result.

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 449 passed; 0 failed; 36 test binaries
```

The crate-local `clippy::missing_errors_doc` attribute is therefore exercised by
the workspace gate's `-D warnings` clippy run, and the crate still builds clean.

**Superseded:** the "still not run" statement this section made about Linux,
Windows, and every hosted CI lane was true when it was written and is **no longer
true** — all of them have since run. The whole-tree figures above (449 passed /
36 test binaries) are also superseded by the later gate on the same tree, which
reports 483 passed / 44 test binaries after further milestones landed. See
§ "Parent-session verification addendum (2026-10-23, hosted qualification)" at the
end of this record.

---

## Parent-session verification addendum (2026-10-23, hosted qualification)

**What this supersedes.** This record previously made three claims that a hosted
runner has now overtaken. Each is superseded by the qualification recorded below,
not merely contradicted by it:

1. § "Requirement-to-evidence matrix" rows 16 and 17 — the whole-workspace suite
   and `./scripts/check-local.sh` were both recorded as **not run**, "Deliberately
   out of this stage's scope".
2. § "Platform evidence" — the `Linux` and `Windows` rows, the five hosted-lane
   rows, and the two whole-tree rows, each **NOT RUN** with the reason "no hosted
   runner is reachable from this machine"; and the sentence beneath the table
   stating that "the Windows and Linux lanes remain genuinely unverified until CI
   runs them".
3. § "Parent-session verification addendum (2026-10-23)" — its "still not run"
   statement for Linux, Windows, and the hosted lanes, and its whole-tree figure
   of *449 passed / 36 test binaries*, which was the state of the tree when that
   gate ran. The later gate on the same tree reports *483 passed / 44 test
   binaries*, because further milestones have landed since. Both gates exited 0.

**No numbered open finding in this record was conditional on the unrun
qualification.** The record's single finding, **U1 (low)**, is a documentation-only
gap — the type-level rustdoc for `Error`, `RecoveryError`, `RecoveryResult`, and
the `Result` alias in `crates/eggup-core/src/error.rs`, a file outside this
milestone's ownership grant. **U1 remains open**: no platform result can settle
it, and the hosted run neither closes nor worsens it. The milestone's own closure
status is therefore unchanged on U1's account — "conditionally closed" on that
finding alone, with the hosted qualification no longer among the open conditions.
The finding text and its severity are retained above unchanged.

### Hosted evidence — CI run `40008039843`

Workflow `CI` (`.github/workflows/ci.yml`), branch `plans/corrective-batch-2026-10-23`,
commit `0cb2c53`, push-triggered. **All five hosted lanes are green.**

| Lane | Conclusion | Evidence |
|---|---|---|
| `Stable checks` (ubuntu-latest) | **success** | `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo test --workspace --all-targets --all-features --locked` → **483 passed, 0 failed, 1 ignored, 44 test binaries**; `cargo doc --workspace --no-deps --locked`; `cargo package -p eggup-service --locked` |
| `MSRV check` (ubuntu-latest, Rust 1.89.0) | **success** | `cargo check --workspace --all-targets --locked` on 1.89.0, plus the packaged-service verification |
| `macOS tests` | **success** | **483 passed, 0 failed, 1 ignored, 44 test binaries** |
| `Windows archive, acquisition, and service tests and check` (windows-latest) | **success** | **598 passed, 0 failed, 0 ignored, 63 test binaries** |
| `Linux systemd service qualification` (ubuntu-latest) | **success** | **9 passed, 0 failed, 0 ignored, 7 test binaries** |

The single ignored test in the Stable and macOS lanes is the pre-existing,
environment-conditional curl connect-timeout test; its skip reason is recorded in
the test source. It is **not** a new ignore introduced by this batch.

The `Stable checks` lane is the one that exercises this milestone's load-bearing
lint: it runs the workspace clippy with `-D warnings`, so the crate-local
`clippy::missing_errors_doc` attribute in `crates/eggup-core/src/lib.rs:8` is
enforced there, and the lane is green.

**Why the Windows aggregate is larger.** The Windows lane runs nine separate
`cargo test` steps — acquisition/archive/curl, eggpack, three named `eggup-service`
diagnostics, the SCM unit tests, core's `current_executable` and
`stale_lock_recovery` fixtures, and then the whole workspace with `--no-fail-fast`
— whereas the Linux and macOS lanes run the whole workspace in a single step, so
the Windows aggregate counts the targeted steps' tests again. The documented
`#[cfg(not(windows))]` exclusion applies in the opposite direction: the curl
loopback-server tests are excluded on Windows and run on Linux and macOS. **Neither
difference is a coverage gap** — it is lane shape plus a documented platform
exclusion, and every workspace test target runs on all three platforms.

### Per-platform results

Reported separately. No platform result is inferred from another.

| Platform | Result | Evidence |
|---|---|---|
| **Linux** | **PASS** | Hosted run `40008039843`: the `Stable checks` lane (ubuntu-latest) — fmt, clippy with `-D warnings`, `cargo test --workspace --all-targets --all-features --locked` (**483 passed, 0 failed, 1 ignored, 44 test binaries**), `cargo doc --workspace --no-deps --locked`, `cargo package -p eggup-service --locked`; the `MSRV check` lane on Rust 1.89.0 — `cargo check --workspace --all-targets --locked` plus the packaged-service verification; and the `Linux systemd service qualification` lane — **9 passed, 0 failed, 0 ignored, 7 test binaries**. |
| **macOS** | **PASS** | Hosted run `40008039843`: the `macOS tests` lane — **483 passed, 0 failed, 1 ignored, 44 test binaries**. The native macOS evidence recorded in this record — the 93-test crate suite, the 32 rendered `# Errors` sections, the lint-fires probe, the pre-M015 baseline-failure proof, and the baseline documentation count — is unchanged and remains valid. |
| **Windows** | **PASS** | Hosted run `40008039843`: the `Windows archive, acquisition, and service tests and check` lane (windows-latest) — **598 passed, 0 failed, 0 ignored, 63 test binaries**. The lane's workspace step (`cargo test --workspace --all-targets --locked --no-fail-fast`) includes `crates/eggup-core/src/failure_taxonomy_tests.rs` through the crate's unit-test target, so the taxonomy fixtures executed natively on Windows. |

After this addendum the milestone is qualified across **all three** platforms:
Linux, macOS, and Windows.

### Local whole-tree gate

Run by the parent session on the same tree:

```sh
./scripts/check-local.sh
# → exit 0
cargo test --workspace --all-targets --all-features --locked --offline --no-fail-fast
# → 483 passed; 0 failed; 44 test binaries
```

`./scripts/check-local.sh` runs `cargo fmt --all -- --check`, the workspace clippy
with `-D warnings`, the workspace test suite, `cargo doc --workspace --no-deps
--locked`, `cargo tree --workspace --locked`, and `./scripts/check-docs-truth.sh`.
All of them are clean; the gate exits 0. The clippy run in that gate is the one
that enforces `clippy::missing_errors_doc` on `eggup-core`.

### Compatibility consequence

This milestone published nothing and moved no tag. The published `eggup-core
0.1.3` on crates.io still carries **every defect this milestone fixed** — the
unproducible `FailureCategory::RetainedEvidence` variant and the public `Result`
surface that documented no failure modes. `eggup-eggpack`'s
`eggup-core = "=0.1.3"` pin blocks propagation to that adapter, so reaching it
needs a separate core publication and a separate adapter republication with the
exact-pin cascade. Neither is authorized by this milestone, and this addendum
changes nothing about that: the hosted qualification qualifies the **working
tree**, not the published crate.

## Addendum (2026-10-24, Verified Update Core M018) — U1 closed, and the status reconciled

This addendum was written by **Verified Update Core M018**
(`plans/closure/verified-update-core/018-bounded-stdin-outcome-priority-corrective.md`,
`plans/closure/verified-update-core/018-status.md`), which owns this file. It
corrects two things in this record and does not rewrite any other evidence in
it.

### U1 is closed on the evidence of commit `ab90688`

The finding **U1** above states that the four missing type-level rustdocs —
`Error`, `RecoveryError`, `RecoveryResult`, and the `Result` alias — were not
written because `crates/eggup-core/src/error.rs` "was **left untouched**" by
that session's ownership grant.

**That premise is false against the baseline.** Commit `ab90688` — the very
commit that ships this closure record — added all four rustdocs to
`crates/eggup-core/src/error.rs`:

```text
$ git show ab90688 -- crates/eggup-core/src/error.rs
-/// Errors produced while validating a local update plan or preparing private stage state.
+/// The error type of every `eggup-core` fallible entry point.
+///
+/// `Error` is a **closed** enum with exactly the seven variants below, so a
+/// downstream exhaustive `match` compiles unchanged. …
```

The four rustdocs are present in the baseline tree at
`crates/eggup-core/src/error.rs`: `Result` at `:4-8`, `Error` at `:10-27`,
`RecoveryError` at `:65-71`, and `RecoveryResult` at `:89-95`. The requirement
of plan §6.2 is therefore met, and **U1 is closed**. The finding's severity was
**low** because it was documentation-only; it is now closed, not carried.

### The status wording is reconciled with the registry and the roadmap

This record's header reads "conditionally closed", while
`plans/registry.md` ("M001-M017 closed") and
`plans/subsystems/verified-update-core-roadmap.md` (the `M015` row, "closed")
read closed. `AGENTS.md` requires the closure record, the registry, and the
roadmap to agree, and with U1 closed the remaining basis for "conditionally
closed" is gone.

The parent session owns `plans/registry.md` and `plans/subsystems/**`; the
reconciliation of those two files is to be applied by the parent session, and
the exact wording to use is drafted in
`plans/closure/verified-update-core/018-status.md` § "Appendix — ready-to-paste
rows for the parent session", under "### `plans/closure/verified-update-core/015-status.md` — status reconciliation".

### What this addendum does not change

- The **implementation evidence** of M015 is untouched: the taxonomy change, the
  `as_str()` strings, the `# Errors` coverage, and the crate-local lint are
  exactly as recorded above.
- The **hosted qualification** recorded in this record's own addendum is
  untouched.
- Nothing here publishes anything, moves a tag, or changes the published
  `eggup-core 0.1.3`.
