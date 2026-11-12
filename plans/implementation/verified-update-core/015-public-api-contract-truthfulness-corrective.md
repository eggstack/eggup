# Verified Update Core Milestone 015 — public API contract truthfulness corrective

Status: closed 2026-10-23; implemented and green on the native macOS lane and hosted-qualified on Stable/MSRV/macOS/Windows/systemd, run `40008039843`. Closure record: `plans/closure/verified-update-core/015-status.md` — one low-severity documentation residual (U1) remains open, and the published `eggup-core 0.1.3` still carries the defect this milestone fixed.

Repository baseline: `c4805411f74b85158e0ae5d0a219a32c4209d98e`

Source roadmap:
- `plans/subsystems/verified-update-core-roadmap.md`

Original closed work affected:
- `plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`
- `plans/closure/verified-update-core/012-status.md`

Finding of record:
- `plans/registry.md` § "Post-0.1.3 open findings" — entry **10**: "`eggup-core`'s published
  failure taxonomy contains a category that can never be produced, and the crate's public `Result` surface documents no failure
  modes." Registered at baseline `c480541` alongside this plan. The finding is assessed **medium** severity and is closed when
  this milestone closes.

Long-term requirements:
- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`

Applicable ADRs:
- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`

Primary class: polish

## 1. Objective

Two observable outcomes in `eggup-core` only.

1. **The published failure taxonomy describes the failure space.** `FailureCategory` is a published, machine-readable enum whose
   `pub fn as_str()` promises a stable category name. It contains a variant no failure can ever carry, so a consumer that switches
   on it — or aggregates by the `as_str()` string — holds a taxonomy with one unreachable branch. After M015 every category is
   either producible, or documented as deliberately unreachable with the reason stated.
2. **The public `Result` surface states its failure modes.** This crate's failure semantics *are* the product: `Foreign`/`Unknown`
   ownership fails closed with a **receipt**, not `Err`; `validate` requires every member to carry `IntegrityStatus::Verified`; a
   member declared `IntegrityRequirement::None` always fails. None of that is discoverable from the API documentation today. After
   M015 every public fallible item carries an `# Errors` section stating its real failure modes, including the
   receipt-versus-`Err` distinction.

Nothing else changes: no runtime behavior, no produced report, no published `Error` variant, no dependency, no publication.

## 2. Why this milestone is ready

- **Both findings are fully characterized by reading the baseline tree.** §3 is a read of
  `crates/eggup-core/src/{transaction,error}.rs`, an exhaustive grep, and a clippy measurement. It depends on no unverified
  runtime assumption; §10 requires fixtures that prove the claims.
- **Scope is bounded to one crate, and the subsystem's one sibling milestone does not overlap.**
  `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md` owns the `run_bounded` deadline defect
  and its fixtures; M015 touches `crates/eggup-core/src/candidate.rs` **rustdoc only**, never behavior (§5). M015's own footprint
  is one published enum in `crates/eggup-core/src/transaction.rs`, rustdoc across `crates/eggup-core/src/`, one new `#[cfg(test)]`
  module file, one crate-local lint attribute, two changelogs, and `architecture/core-transaction.md` — no other crate's
  production code, no dependency, no publication.
- **Severity: Finding A is medium; Finding B is medium.** Finding A: a published machine-readable taxonomy contains a category
  that can never be produced, so a consumer building a retry or diagnostic policy over it holds an unreachable branch and the
  taxonomy does not faithfully describe the failure space. Finding B: the crate's primary published API surface documents no
  failure modes at all. Per this repository's gate (`.opencode/skills/planning-workflow/SKILL.md` § "Severity gate"),
  "medium-or-higher" forces a corrective before a milestone closes; both findings are at that level.
- **Honest statement of what this is not.** There is **no failing production report**, and none is claimed. The current behavior
  is **correct**: the unreachable variant is simply unreachable, and every other member of the taxonomy accurately describes its
  producer. This is a **contract-truthfulness correction**, not a bug fix; it is classified `polish` for that reason while still
  being registered as a medium finding, because the defect is in the *published contract* — the artifact downstream code consumes.
- **Precedent shape.** M012 (`plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`) is the
  direct precedent for published-error-surface handling in this crate: it moved retained-evidence reporting out of the published
  `Error` surface into a separate `RecoveryError` type, and that move is the *cause* of Finding A (§3.1). M015 must respect M012's
  compatibility reasoning and must not reopen the `Error` variant set.
- **The doc-accuracy check runs inside the existing gate.** Verified by probe: `#![warn(clippy::missing_errors_doc)]` in the crate
  root plus the gate's `-D warnings` turns a missing `# Errors` section into a hard error, with no change to the workspace lint
  table and no effect on any other crate (§6.4).

## 3. Current implementation evidence

### 3.1 The published taxonomy and the unreachable variant

`crates/eggup-core/src/transaction.rs:99-119` declares `FailureCategory` — `LockContention`, `OwnershipConflict`, `Verification`,
`Filesystem`, `InvalidInput`, **`RetainedEvidence` (`:116`)**, `PostCommitCheck` (`:118`) — with `#[non_exhaustive]` at `:101` and
`pub fn as_str()` at `:121-135`, where `Self::RetainedEvidence => "retained-evidence"` is at `:131`.

The variant is **never constructed**. Exhaustive grep over the crate returns exactly two hits — the declaration and its `as_str`
arm — and it appears in no other crate, no test, no doc, and no example:

```text
$ grep -rn "RetainedEvidence" crates/ --include='*.rs'
crates/eggup-core/src/transaction.rs:116:    RetainedEvidence,
crates/eggup-core/src/transaction.rs:131:            Self::RetainedEvidence => "retained-evidence",
```

**Why it is unreachable.** The only producer of a `FailureCategory` from an `Error` is `report_for_error`
(`transaction.rs:193-219`), whose exhaustive mapping reaches exactly six categories — `LockContention` (`:199`),
`OwnershipConflict` (`:203`), `Verification` (`:207-208`), `InvalidInput` (`:211-212`), `Filesystem` (`:213`), and, in test builds
only, `Injected` (`:209-210`). `Error` is a **closed** enum (`crates/eggup-core/src/error.rs:9-44`) with the seven variants
published in `0.1.2` plus `#[cfg(test)] Injected` (`error.rs:42-43`). No arm maps to `RetainedEvidence`, so no `FailureReport` can
carry it.

**Origin.** M012 moved retained-evidence reporting out of the published `Error` surface into `RecoveryError` (`error.rs:52-68`),
keeping the *disposition* surface (`TransactionDisposition::RecoveryRequired`, `CleanupDisposition::RetainedForRecovery`) as the
carrier of that meaning (M012 plan §4.2, §4.4). The `FailureCategory` variant was left behind as residue and never received a
producer.

### 3.2 The other two producers

`PostCommitCheck` **is** produced — directly, not through `report_for_error` — at `transaction.rs:485-490`. `Injected` is produced
at `transaction.rs:535-539` (the `CommitFault::Finalize` path) and through the `#[cfg(test)]` mapping arm at
`transaction.rs:209-210`.

### 3.3 A second unreachability that M015 documents but does **not** remove

`FailureCategory::Injected` is reachable **only in this crate's own test builds**, because its only producer path is
`Error::Injected`, which is `#[cfg(test)]` (`error.rs:42-43`); a shipped `eggup-core` binary therefore never produces the
`injected` category. This is *not* residue — M012 §4.1 deliberately kept the structural fault-harness classification and removed
only the public `Error::Injected` variant — but it is the same class of unreachable branch as Finding A, so it is in scope for
M015's documentation half: the rustdoc for `FailureCategory::Injected` must state that it is produced only by this crate's own
test fault harness and never by a shipped build. M015 does **not** remove it (§6.1).

### 3.4 The public `Result` surface documents no failure modes

Measured over the whole crate: `grep -rc "# Errors" crates/eggup-core/src/*.rs` returns **zero** for every file; there are zero `#
Panics` sections and zero doc code fences anywhere in `crates/eggup-core/src/`. The crate is `#![forbid(unsafe_code)]` and
`#![deny(missing_docs)]` (`crates/eggup-core/src/lib.rs:1-2`) and re-exports a large public `Result`-returning surface
(`lib.rs:26-47`), yet none of it states a failure mode.

The exact set of public fallible items carrying no `# Errors` section was measured with the lint M015 will enable (§6.4): `cargo
clippy -p eggup-core --all-targets --locked -W clippy::missing_errors_doc` reports **32** items, one warning per public item
returning `Result` or `RecoveryResult`:

| Module | Items (`file:line`) |
|---|---|
| `candidate.rs` | `run_bounded` `:125`; `CandidateValidator::validate` `:230`; `ValidatedTransaction::validate` `:412`; `staged_path` `:437`; `commit` `:449`; `commit_with_post_commit` `:466`; `commit_with_stale_lock_recovery` `:494`; `commit_with_post_commit_and_stale_lock_recovery` `:507` |
| `current_exe.rs` | `CurrentExecutable::resolve` `:37`; `bind` `:48` |
| `domain.rs` | `ProductId::new` `:16`; `ReleaseId::new` `:38`; `MemberId::new` `:60`; `ArtifactMember::new` `:296`; `ArtifactSet::new` `:365`; `ArtifactSet::single` `:384`; `InstallPlan::new` `:481`; `for_current_executable` `:547`; `destination` `:610`; `prepare` `:620`; `prepare_with_bound_sources` `:637` |
| `integrity.rs` | `hash_file` `:65`; `parse_sha256_sidecar` `:83`; `verify_file` `:117`; `PreparedTransaction::verify_integrity` `:171`; `VerifiedTransaction::staged_path` `:232` |
| `lock.rs` | `MutationLock::acquire` `:205`; `acquire_with_recovery` `:234`; `observe` `:306`; `inspect` `:325` |
| `stage.rs` | `PreparedTransaction::destination` `:170`; `PreparedTransaction::staged_path` `:175` |

**One item is deliberately excluded.** `OwnershipVerifier::verify` (`crates/eggup-core/src/domain.rs:151-154`) returns
`Ownership`, **not** a `Result`; it is infallible, takes no `# Errors` section, and the lint does not fire on it. What it needs is
the contract statement it already carries (`domain.rs:141-150`: a verifier must treat errors or ambiguity as `Ownership::Unknown`
or a hard failure, **never** `Ownership::Owned`). M015 must not invent a fallibility that does not exist; instead the `commit`
family sections point at that rule (§6.2).

### 3.5 Why the gate does not catch this today

`clippy::missing_errors_doc` lives in `clippy::pedantic`, not `clippy::all`. The workspace lint table sets `clippy = { all =
"warn" }` and the gate and CI run clippy with `-D warnings`, so a never-enabled pedantic lint is never warned about and `-D
warnings` does not catch this class of defect. §6.4 makes the check runnable inside the existing gate without touching the
workspace lint table.

## 4. Invariants that must not regress

- **The fail-closed ownership direction is unchanged.** `Foreign` and `Unknown` never permit mutation; `Owned` is never inferred;
  only lock contention and setup failures return `Err`, while `Foreign`/`Unknown`/ownership-flap produce a **receipt**
  (`RolledBack`/`RecoveryRequired`), never `Err` (`architecture/core-transaction.md` §8). M015 documents this rule; it never
  changes it.
- **The published `Error` variant set is unchanged** — the seven variants published in `0.1.2` and preserved by core M012
  (`error.rs:9-44`). **Adding or removing an `Error` variant is out of scope and a stop condition** (§14). `RecoveryError` remains
  `#[non_exhaustive]` (`error.rs:52-53`).
- **`IntegrityRequirement::None` still always fails.** Only `Sha256` members can be verified and committed; `None` is observable
  during preparation but rejected by validation and by the final staged-digest revalidation (`crates/eggup-core/docs/domain.md`).
- **SHA-256 remains integrity evidence only.** No authenticity or signature claim is introduced anywhere, including in the new
  rustdoc; there is no authenticity verifier in this crate (`crates/eggup-core/src/lib.rs:11-12`, `docs/verification.md`).
- **Core never creates a live destination parent.** A missing parent fails closed and nothing is created.
- **The transaction stage order is unchanged:** `InstallPlan → Prepared → Verified → Validated → Receipt`.
- **The `as_str()` strings of the remaining categories are byte-identical**, and **no produced report changes** — removing an
  unreachable category changes no `FailureReport`, no `TransactionReceipt`, and no `Error` (§8).
- **No new dependency.** `eggup-core` stays at `sha2` plus the Windows-only `self-replace` (`crates/eggup-core/Cargo.toml:24-33`).
- **No `unsafe_code`** (workspace deny; crate `#![forbid(unsafe_code)]` at `lib.rs:1`). MSRV stays 1.89.

## 5. Scope

### In scope

- `crates/eggup-core/src/transaction.rs`: the `FailureCategory` enum (`:99-119`) and its `as_str` impl (`:121-135`) — variant
  removal only, plus rustdoc.
- Rustdoc for every public fallible item in §3.4, across
  `crates/eggup-core/src/{candidate,current_exe,domain,integrity,lock,stage}.rs`.
- Rustdoc for `Error`, `RecoveryError`, `RecoveryResult`, `Result`, `FailureCategory`, `FailureReport`, `FailurePhase`,
  `TransactionReceipt`, `TransactionDisposition`, `CleanupDisposition`, and the `OwnershipVerifier` trait.
- One new `#[cfg(test)]` module file, `crates/eggup-core/src/failure_taxonomy_tests.rs`, plus its one-line `#[cfg(test)] mod
  failure_taxonomy_tests;` declaration in `crates/eggup-core/src/lib.rs`. The precedent for a `#[cfg(test)]` module file inside
  `src/` already exists (`lib.rs:49-50`, `src/test_support.rs`).
- One crate-local lint attribute enabling the doc-accuracy check (§6.4).
- `crates/eggup-core/CHANGELOG.md` `Unreleased`, the root `CHANGELOG.md` `Unreleased`, and `architecture/core-transaction.md`.

### Explicitly out of scope

- **No behavior change to `run_bounded`** — milestone
  `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md` owns the deadline defect and its
  fixtures. M015 touches `crates/eggup-core/src/candidate.rs` **rustdoc only**, and where M014's wording and M015's `# Errors`
  section cover the same item, M015 must not pre-empt or contradict M014 (§16).
- **No packaging-manifest change** — milestone
  `plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md` owns whether
  `crates/eggup-core/docs/` ships (`crates/eggup-core/Cargo.toml:15-19` currently excludes `docs/`). M015 must **not** change the
  `exclude` list and must cross-reference that milestone rather than resolving the packaging question (§16).
- **No change to `eggup-service`** — the service-side ownership-enum and `CommandOutput` duplication belong to
  `plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`; and **no `eggup-service` doc sweep** —
  the crate-wide `# Errors` sweep for `eggup-service` belongs to
  `plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`. M015's sweep is
  scoped to `eggup-core` **only**.
- **No new dependency** — in particular no lint-config crate and no dev-dependency. The doc-accuracy check uses a built-in clippy
  lint only.
- **No change to the workspace lint table.** `Cargo.toml`'s `[lints]` is unchanged; the new lint is enabled **crate-locally** in
  `crates/eggup-core/src/lib.rs` so no other crate is affected.
- **No republication and no publication.** `eggup-core 0.1.3` is published (2026-10-06, source `bd43683`, hosted run
  `37527706900`); M015 publishes nothing and must not move the `v0.1.3` tag (§9).
- **No change to the `Error` variant set** and no new public error type.
- **No change to `Ownership` or `OwnershipVerifier`** (the verifier stays infallible; no `Result` enters its signature), to the
  ownership model, the transaction state machine, the lock, the receipts, the staging rules, or the failure *behavior*. The
  taxonomy changes shape (one variant removed); the failure space does not change.
- **No authenticity or signature work.** SHA-256 remains integrity evidence only.
- **No change to `eggup-eggpack`, `eggup-archive`, `eggup-acquisition`, `eggup-eggfetch`, `eggup-curl`, or
  `eggup-transport-footprint`.**
- **No CI configuration change**, unless §11's Windows requirement cannot be met by the existing lane steps — and then only by
  adding a step, never by weakening one.
- **No consumer-facing how-to rewrite.** `docs/` is non-normative and must not become the place a rule is introduced (§12).

## 6. Required production changes

Required **outcomes**, not implementation. The implementer chooses the mechanism; the observables below are fixed.

### 6.1 Finding A — the disposition of `FailureCategory::RetainedEvidence`

**Decision: REMOVE the variant and its `as_str` arm.** After M015 the published taxonomy is exactly:

| Category | `as_str()` | Producer |
|---|---|---|
| `LockContention` | `"lock-contention"` | `report_for_error` (`transaction.rs:199`) |
| `OwnershipConflict` | `"ownership-conflict"` | `report_for_error` (`transaction.rs:203`) |
| `Verification` | `"verification"` | `report_for_error` (`transaction.rs:207-208`) |
| `Filesystem` | `"filesystem"` | `report_for_error` (`transaction.rs:213`) |
| `InvalidInput` | `"invalid-input"` | `report_for_error` (`transaction.rs:211-212`) |
| `Injected` | `"injected"` | test fault harness only — documented as such (§3.3) |
| `PostCommitCheck` | `"post-commit-check"` | direct construction (`transaction.rs:485-490`) |

**Justification.** The variant is residue of M012's relocation of retained-evidence reporting into `RecoveryError`; that meaning
is carried by `TransactionDisposition::RecoveryRequired` and `CleanupDisposition::RetainedForRecovery`, not by a failure category.
Removing it restores the property that makes the taxonomy useful: every member describes a failure that can actually occur.

**Alternatives considered and rejected.** *Give it a real producer* (map `RecoveryError::RecoveryRequired` onto it) would change
`report_for_error`'s input type from `&Error` to a recovery-aware type and would change **produced reports**, which a
truthfulness-only milestone must not do (§8); the retained-evidence condition is already reported as a *disposition* plus
`TransactionReceipt::recovery_path()`. *Keeping it as reserved* re-creates the defect it would document. *`#[doc(hidden)]`* leaves
it constructible and keeps `as_str()` returning `"retained-evidence"` — it hides the defect rather than removing it.

**What a consumer's exhaustive-but-wildcard match will now see.** Because `FailureCategory` is `#[non_exhaustive]`
(`transaction.rs:101`), every downstream matcher already requires a wildcard arm; that wildcard arm will now cover one fewer
variant. No downstream matcher can observe a `RetainedEvidence` category today, because no `FailureReport` can carry one, so no
consumer's runtime behavior changes (§9).

### 6.2 Finding B — the `# Errors` sweep

**Required items.** All 32 public fallible items in §3.4, and no others. `OwnershipVerifier::verify` is excluded because it is
infallible (§3.4).

**Required content of each `# Errors` section.** Each states, in the item's own terms:

1. **Which `Error` variants the item can return**, and under what condition — not a generic "may fail".
2. **The receipt-versus-`Err` distinction**, wherever the item can reach the commit path. A reader must **not** be able to
   conclude that an ownership conflict produces `Err`: `Foreign`, `Unknown`, and ownership-flap produce a **receipt**
   (`RolledBack`/`RecoveryRequired`), while only lock contention and setup failures produce `Err`. The authoritative statement is
   `architecture/core-transaction.md` §8 "Failure semantics"; the rustdoc must be consistent with it and point at it.
3. **The fail-closed direction** where it applies: `Owned` is never inferred; a verifier's ambiguity is `Unknown`, never `Owned`
   (`crates/eggup-core/src/domain.rs:141-150`).
4. **The `IntegrityRequirement::None` rule** on the items that reach validation or commit: a member without
   `IntegrityStatus::Verified` — including a `None` member — is rejected; there is no escape hatch.
5. **No authenticity claim.** Integrity wording stays "SHA-256 digest evidence"; nothing may imply publisher trust or signature
   verification.
6. **A pointer, not a restatement** (§6.3).

**Also required.** The rustdoc of `Error`, `RecoveryError`, `RecoveryResult`, `Result`, `FailureCategory`, `FailureReport`,
`FailurePhase`, `TransactionReceipt`, `TransactionDisposition`, `CleanupDisposition`, and `OwnershipVerifier` must each state what
the type means — in particular that `FailureCategory` is a stable machine-readable classification of *produced* failures, and that
`TransactionReceipt` (not an `Err`) is how a post-lock outcome is reported.

### 6.3 Rustdoc-to-contract cross-references

The rustdoc must **point at** the contracts, not restate them. The authoritative sources are
`crates/eggup-core/docs/{domain,verification,transaction}.md` and `architecture/core-transaction.md`; the repo's doc-hygiene rule
is that a document points at the contract and never introduces a rule of its own, and the same discipline applies to rustdoc.

**One honest constraint.** `crates/eggup-core/Cargo.toml:15-19` excludes `docs/` from the packaged crate, so a rustdoc reference
to `crates/eggup-core/docs/*.md` does **not** resolve for a crates.io consumer today, and M015 must not change that manifest (§5).
Each `# Errors` section must therefore be **self-sufficient about its own failure modes** (what it returns, when) and point at the
repository-resident contracts for the deeper rule. The packaging interaction is recorded for
`plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md` and is not
resolved by M015 (§16).

### 6.4 The doc-accuracy check runs inside the existing gate

`crates/eggup-core/src/lib.rs` gains a crate-local lint attribute enabling `clippy::missing_errors_doc`, so the existing gate's
`-D warnings` makes a missing `# Errors` section a hard error **for `eggup-core` only**. Verified by probe before this plan was
written: with the attribute present, `cargo clippy --all-targets -- -D warnings` fails with `error: docs for function returning
`Result` missing `# Errors` section`, and passes once the section is added. The workspace lint set is unchanged, so no other crate
is affected.

## 7. Ordered work packages

- **WP1 — Decide and apply the variant disposition.** Remove `FailureCategory::RetainedEvidence` (`transaction.rs:116`) and its
  `as_str` arm (`:131`); add rustdoc stating which categories are producible and that `Injected` is produced only by this crate's
  own test fault harness. *Verifiable:* `cargo test -p eggup-core --locked` green; the two-hit grep now returns zero hits.
- **WP2 — The `# Errors` sweep.** Add an `# Errors` section to each of the 32 items in §3.4, in the module order of §3.4.
  *Verifiable:* `cargo clippy -p eggup-core --all-targets --locked -- -D warnings` clean with the lint enabled.
- **WP3 — Rustdoc-to-contract cross-references.** Add the pointers required by §6.3 to the type-level rustdoc in §6.2 and to the
  `commit` family. *Verifiable:* every pointer resolves to an existing repository file; `cargo doc -p eggup-core --no-deps
  --locked` clean.
- **WP4 — The doc-accuracy check.** Enable `clippy::missing_errors_doc` crate-locally in `crates/eggup-core/src/lib.rs` and land
  the new `#[cfg(test)] mod failure_taxonomy_tests;` declaration with its fixture (§10). *Verifiable:* the workspace clippy gate
  is clean, and the fixture fails against the pre-M015 baseline (§11).
- **WP5 — Changelogs.** Update `crates/eggup-core/CHANGELOG.md` `Unreleased` and the root `CHANGELOG.md` `Unreleased` with the
  repo's standard disclaimer (not published, no consumer migration required, republication is a separate milestone). *Verifiable:*
  both entries state the disclaimer.

WP1 precedes WP2 because the `# Errors` text describes the taxonomy as it will exist after WP1. WP4 follows WP2 so the lint is
enabled only once the sweep is complete. WP3 may run in parallel with WP2 but must land before the closure.

## 8. Failure, cancellation, restart, and contention semantics

This milestone produces no runtime behavior, so its own failure semantics are about **what the correction is permitted to
change**.

- **Removing the variant must not change any produced report.** No `FailureReport`, `TransactionReceipt`, `Error`,
  `RecoveryError`, or `TransactionDisposition` may differ before and after. Because no producer existed this holds by
  construction; §10 requires a fixture that proves the producible set, and the existing core suites must remain green with no
  assertion change.
- **The `as_str()` strings of the remaining categories must be byte-identical.** They are a published contract; a consumer may
  persist or aggregate them. The only string that disappears is `"retained-evidence"`, and no consumer could have observed it,
  because no failure could carry that category.
- **No `# Errors` text may assert a behavior the code does not have.** If the sweep finds a place where the **code** and the
  **contract** disagree, the finding MUST be recorded with an explicit severity and reason — it must NOT be papered over by
  writing the doc to match the code, nor "fixed" by changing behavior here (§14). `architecture/core-transaction.md` § "Known
  doc/code drift" already lists four such disagreements (`docs/transaction.md:31-34` report retention;
  `TransactionReceipt::rollback_verified` rustdoc; `docs/domain.md:11` permission behavior; a stale public-surface description);
  the sweep must **not** restate a drifted claim as true nor re-litigate those four as new findings, and where they touch a swept
  item the rustdoc must not inherit the drift.
- **No new contention, cancellation, or restart surface.** The lock, the stale recovery opt-in, the receipt dispositions, and the
  rollback paths are untouched. `MutationLock::acquire` still never recovers a record.
- **No crash-journaling or atomicity claim** may appear in the new rustdoc. The implementation does not claim crash-safe
  journaling or literal filesystem-wide atomicity (`crates/eggup-core/docs/transaction.md`).

## 9. Compatibility and migration

- **`eggup-core 0.1.3` is published** (2026-10-06, source `bd43683`, hosted run `37527706900`; registry checksum
  `161b244b…789ad`). M015 lands on the source line only.
- **Removing `FailureCategory::RetainedEvidence` is source-visible.** The reasoning: `FailureCategory` is `#[non_exhaustive]`
  (`transaction.rs:101`), so a downstream matcher **must** already carry a wildcard arm; removing a variant therefore does not
  break that matcher's compilation. Under Cargo's rules this is a compatible change and may ship in a `0.1.x` patch. **The one
  caveat**: a downstream crate that *named* the variant explicitly — a match arm or an equality assertion against it — **will fail
  to compile** after removal. That is a compile-time break, not a silent behavior change, and it is the only downstream break M015
  introduces.
- **The `as_str()` stability guarantee** is preserved for every remaining category. The `"retained-evidence"` string disappears,
  and no consumer could have observed it (§8).
- **`eggup-eggpack` pins `eggup-core` as `=0.1.3`** (`crates/eggup-eggpack/Cargo.toml:20`). A republished core therefore **cannot
  reach eggpack** until eggpack is separately republished against the new pin. M015 must state this and must not touch eggpack.
- `eggup-service` depends on `eggup-core` by caret (`"0.1.0"`, `crates/eggup-service/Cargo.toml:24`) and would pick up a
  republished patch automatically; `eggup-archive` uses a path dependency with no version (`crates/eggup-archive/Cargo.toml:24`).
  Neither is affected by M015, which publishes nothing. **Republication is a separate milestone** (§16).
- **Republication is a separate milestone.** It would need to bump the version past `0.1.3`, re-run the published-API
  compatibility fixtures, re-tag or add a tag, and decide the version boundary in light of the already-registered
  `CleanupDisposition` / `Error`-extensibility finding (`plans/registry.md` § "Post-0.1.3 open findings" item 2). **M015 does not
  authorize any of that, and the `v0.1.3` tag must not move.**
- **Consumer migration is not required by M015**, except for the one caveat above: a downstream crate naming
  `FailureCategory::RetainedEvidence` by name must delete that reference when the change is eventually published.

## 10. Required tests

- **T1 — The producible-category fixture.** A new in-crate unit test in `crates/eggup-core/src/failure_taxonomy_tests.rs`,
  declared by `#[cfg(test)] mod failure_taxonomy_tests;` in `crates/eggup-core/src/lib.rs`. It MUST: (a) derive the **declared**
  category set from an **exhaustive** match over `FailureCategory` — the only in-crate way to enumerate the variants, because
  `#[non_exhaustive]` forbids a downstream crate from doing so; (b) derive the **producible** set **empirically** — call
  `report_for_error` (`transaction.rs:193`) once per `Error` variant (the seven published variants plus the `#[cfg(test)]
  Injected`) and add the category produced by a **real failing post-commit check**, read from
  `TransactionReceipt::post_commit_failure()`; and (c) assert the two sets are equal in **both** directions. The post-commit
  fixture must create the live destination parent itself (core never creates one), must declare `IntegrityRequirement::Sha256`
  with the member's real digest so `validate` does not reject the member, and must use `PostCommitFailurePolicy::KeepInstalled` so
  the receipt carries `post_commit_failure`.

**This fixture MUST FAIL against the pre-M015 implementation.** Both failure forms were verified by probe against the baseline
worktree (§11): it fails **at runtime** with `declared category RetainedEvidence has no producer` when it lists the baseline's
eight variants, and it fails **to compile** with `error[E0004]: non-exhaustive patterns: `FailureCategory::RetainedEvidence` not
covered` when the post-M015 seven-variant form is dropped into the baseline tree. Post-M015 it compiles and passes.
- **T2 — The `as_str()` byte-identity fixture.** Assert that every remaining category's `as_str()` returns the exact published
  string listed in §6.1. **This fixture PASSES both before and after M015** — it is a *regression guard* pinning strings that must
  not change, not a failure proof. Stated honestly so the closure does not claim it as evidence of the correction.
- **T3 — The doc-accuracy check.** `clippy::missing_errors_doc` enabled crate-locally (§6.4) makes the existing gate fail if any
  of the 32 items loses its `# Errors` section. It runs inside `cargo clippy --workspace --all-targets --all-features --locked --
  -D warnings`. The closure records the exact rendered-section count from `cargo doc` as a secondary sanity figure and states that
  the lint is the canonical check.
- **T4 — Existing core suites unchanged.** The whole `eggup-core` suite must remain green **with no assertion weakened and no
  expected value changed** by this milestone. Any test that had to change is a stop condition (§14).
- **T5 — No timing sensitivity.** The fixtures are filesystem- and process-local and must not carry sub-second wall-clock budgets
  (see the load-sensitivity precedent in
  `plans/implementation/acquisition-transport/012-sub-second-deadline-test-determinism-corrective.md`).

## 11. Required verification commands

Exact strings, run from the repository root:

```text
cargo test -p eggup-core --locked
cargo test -p eggup-core --locked failure_taxonomy
cargo clippy -p eggup-core --all-targets --locked -- -D warnings
cargo doc -p eggup-core --no-deps --locked
grep -rho 'id="errors"' target/doc/eggup_core/ | wc -l
cargo test --workspace --all-targets --all-features --locked
cargo fmt --all -- --check
./scripts/check-local.sh
```

**Note on the doc-assertion command.** `cargo doc` writes its progress output to stdout and the rendered HTML to `target/doc/`;
the stdout does **not** contain the rendered sections. A command of the form `cargo doc -p eggup-core --no-deps --locked 2>&1 |
grep -c "# Errors"` therefore returns `0` **whether or not** the sections exist and is **not** a valid assertion. The valid form
is the `grep -rho 'id="errors"' target/doc/eggup_core/ | wc -l` line above, verified to report `0` on the pre-M015 tree. The
**canonical** check remains the lint (T3).

**Proving the new fixtures fail against the pre-M015 baseline.** Run verbatim from the repository root:

```text
git worktree add /tmp/eggup-m015-baseline c4805411f74b85158e0ae5d0a219a32c4209d98e
cp crates/eggup-core/src/failure_taxonomy_tests.rs /tmp/eggup-m015-baseline/crates/eggup-core/src/failure_taxonomy_tests.rs
printf '\n#[cfg(test)]\nmod failure_taxonomy_tests;\n' >> /tmp/eggup-m015-baseline/crates/eggup-core/src/lib.rs
cd /tmp/eggup-m015-baseline && cargo test -p eggup-core --locked failure_taxonomy
```

The expected outcome is a **failure**, and the closure must record the exact diagnostic. Both failure forms were reproduced by
probe at this baseline: the post-M015 seven-variant fixture gives `error[E0004]: non-exhaustive patterns:
`FailureCategory::RetainedEvidence` not covered`, pointing at `crates/eggup-core/src/transaction.rs:102`; a fixture listing the
baseline's eight variants gives `test result: FAILED` with `declared category RetainedEvidence has no producer`.

The worktree is removed afterwards with `git worktree remove --force /tmp/eggup-m015-baseline`.

**CI lanes.** The closure must record results for the `stable`, `msrv`, `service-systemd`, `macos`, and `windows-check` lanes. A
lane not run is **not run**, never inferred (§15).

## 12. Documentation updates

- **`crates/eggup-core/CHANGELOG.md`** — an `Unreleased` entry stating: the unreachable `FailureCategory::RetainedEvidence`
  variant was removed; the remaining `as_str()` strings are unchanged; the crate's public fallible API now documents its failure
  modes; **nothing in this section has been published**; `0.1.3` is the published baseline; **no consumer migration is required**
  except for a downstream reference to the removed variant by name; and republication is a separate milestone.
- **Root `CHANGELOG.md`** — an `Unreleased` entry aggregating the above. The root changelog aggregates; it does not replace the
  per-crate one.
- **`crates/eggup-core/docs/`** — edited **only** if a contract statement changes. If it does, that is a **contract change** and
  must be recorded as such in the closure (requirement→evidence matrix and unresolved findings), **not** edited silently. M015's
  expected outcome is that no contract statement changes: removing an unreachable category and documenting failure modes are
  truthfulness corrections, not contract changes. `docs/` is **non-normative** and must not be edited to carry these facts.
- **`architecture/core-transaction.md`** — the public-surface and failure-semantics material must reflect the post-M015 taxonomy
  (seven categories, each with its producer) and must not carry a stale description of the removed variant. The "Known doc/code
  drift" list is **not** edited by M015 except to add an entry if the sweep confirms a **new** code-versus-contract disagreement
  (§8, §14).
- **`docs/` (consumer-facing)** — non-normative. It points at the contracts; it does not restate them and does not introduce a
  rule there. M015 does not rewrite a consumer how-to.

## 13. Acceptance criteria

M015 closes only when all of the following are checkable:

1. `grep -rn "RetainedEvidence" crates/ --include='*.rs'` returns **zero** hits in `eggup-core`, and the variant and its `as_str`
   arm are gone.
2. The published `FailureCategory` has exactly the seven categories in §6.1, each with a producer, or documented as test-only
   (`Injected`).
3. `as_str()` returns byte-identical strings for all seven remaining categories, pinned by T2.
4. Every one of the 32 public fallible items in §3.4 carries an `# Errors` section, and `cargo clippy -p eggup-core --all-targets
   --locked -- -D warnings` is clean with `clippy::missing_errors_doc` enabled crate-locally.
5. Each `# Errors` section states the real failure modes, and the commit-family sections state the receipt-versus-`Err`
   distinction so that a reader cannot conclude an ownership conflict produces `Err`.
6. The rustdoc points at `crates/eggup-core/docs/{domain,verification,transaction}.md` and `architecture/core-transaction.md` §8
   rather than restating them, and each section is self-sufficient about its own failure modes given that `docs/` is not packaged.
7. No new authenticity or signature claim appears anywhere.
8. The T1 fixture fails against the pre-M015 baseline with the recorded diagnostic and passes on the post-M015 tree.
9. The whole `eggup-core` suite and the whole workspace suite are green with **no weakened assertion**.
10. `cargo tree --workspace --locked` shows `eggup-core` still at `sha2` plus the Windows-only `self-replace`, and no other crate
    changed.
11. `./scripts/check-local.sh` is green.
12. The closure records separate Linux, macOS, and Windows results for every lane it ran, and marks a lane it did not run as **not
    run**.

## 14. Stop conditions

Stop and open a new corrective, rather than proceeding, if any of the following holds:

- **The variant cannot be removed without a breaking change** that cannot be sequenced later — for example if removal turns out to
  require adding a replacement variant, changing `FailureCategory`'s `#[non_exhaustive]` attribute, or altering a published method
  signature. Such a change must be sequenced at the intentional **0.2/1.0** boundary together with the already registered
  `CleanupDisposition` / `Error`-extensibility finding (`plans/registry.md` § "Post-0.1.3 open findings" item 2), not forced here.
- **The `# Errors` sweep reveals a code-versus-contract disagreement** that requires a behavior change to resolve. That
  disagreement must become a **separate corrective** with its own severity, not be documented around and not be "fixed" inside
  M015.
- **The doc-accuracy check cannot be made to run inside the existing gate.** If enabling `clippy::missing_errors_doc`
  crate-locally proves impossible without touching the workspace lint table or another crate, M015 stops and the enforcement
  mechanism is re-scoped rather than the check being dropped.
- **A proposed `# Errors` section would require claiming behavior the crate does not have** — in particular any authenticity
  claim, any crash-journaling or atomicity claim, or any statement that an ownership conflict returns `Err`.
- **Any existing core test would have to change its expected value.** A taxonomy-shape change must not disturb a behavioral
  assertion; if it does, the change is a behavior change and is out of scope.
- **Any crate other than `eggup-core` would have to change** for M015 to compile, other than a mechanical consequence of a removed
  variant — in which case that crate's milestone must land first, or M015 must be re-scoped.

## 15. Closure evidence required

The closure record is `plans/closure/verified-update-core/015-status.md` and must follow `plans/closure/README.md`:

- **Status** (closed / conditionally closed / corrective pass required / blocked); **source plan and roadmap**; **reviewed
  baseline** SHA; **implementation commits**.
- **Executive finding** — one paragraph: what was wrong, what was changed, what was proven.
- **Requirement → evidence matrix** — one row per acceptance criterion in §13, each with the command and the `file:line` that
  satisfies it.
- **Production implementation evidence** — the final `FailureCategory` enum with its `file:line`; the final `as_str` impl; the `#
  Errors` sections with their `file:line`; the crate-local lint attribute with its `file:line`.
- **Exact commands and results** — every command in §11 with its actual output, plus the pre-M015 baseline failure proof with the
  **exact diagnostic** and the worktree removal.
- **Platform results, reported separately** — Linux, macOS, and Windows. A lane that was not run is recorded as **not run**, never
  inferred from another lane.
- **Invariant review** — each invariant in §4 and how it was checked.
- **Failure/rollback/recovery review** — confirmation that no produced report changed, that the `as_str()` strings are
  byte-identical, and that no new failure surface was introduced.
- **Compatibility and migration review** — §9's items, including the explicit statement that the removal is compatible for
  `#[non_exhaustive]` matchers and the single caveat about a downstream name reference.
- **Security review** — confirmation that no authenticity claim was introduced, that the fail-closed ownership direction is
  unchanged, and that no `unsafe` and no new dependency entered the crate.
- **Documentation/operations evidence** — the changelog entries, the `architecture/core-transaction.md` update, and the
  confirmation that `crates/eggup-core/docs/` was **not** used to carry these facts.
- **Unresolved findings with severity** — each with an explicit severity and reason, including any code-versus-contract
  disagreement the sweep surfaced and the residual "documentation lives outside the published package" interaction with
  `plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md`.
- **Roadmap disposition** and **registry updates** — including the numbered entry **10** in `plans/registry.md` § "Post-0.1.3
  open findings".

## 16. Handoff notes

- **Sibling `plans/implementation/verified-update-core/014-bounded-execution-deadline-corrective.md`.** M014 owns the
  `run_bounded` deadline defect and its fixtures; M015 touches `candidate.rs` **rustdoc only**. If M014 lands first, M015's
  `run_bounded` `# Errors` section must be written against M014's post-fix wording, not the baseline's. If M015 lands first, M014
  must reconcile the `run_bounded` rustdoc it rewrites with M015's section rather than dropping it. Neither milestone may change
  the other's behavior.
- **`plans/implementation/service-lifecycle/012-commit-engine-deduplication-corrective.md`** — owns the service-side
  ownership-enum and `CommandOutput` duplication. M015 touches no `eggup-service` code; if service M012 lands first, the
  `Ownership`-related rustdoc M015 writes in `eggup-core` is unaffected, since M012 does not move core's own `Ownership` type.
- **`plans/implementation/service-lifecycle/013-manager-failure-taxonomy-and-deadline-contract-corrective.md`** — owns the
  crate-wide `# Errors` sweep for `eggup-service` and the manager failure taxonomy. M015's sweep is scoped to `eggup-core`
  **only**; the two crates must not both claim the service-side surface.
- **`plans/implementation/planning-closure-hygiene-corrective/013-post-audit-documentation-truth-reconciliation.md`** — owns the
  `eggup-core` **packaging** question (whether `crates/eggup-core/docs/` ships; `Cargo.toml:15-19` currently excludes it). M015
  must **not** change the packaging manifest. M015's rustdoc therefore states each item's failure modes self-sufficiently and
  points at the repository-resident contracts; if that milestone later ships the contract docs, M015's cross-references become
  resolvable for crates.io consumers with no further rustdoc change.
- **What a separate republication milestone would require.** Bump `eggup-core` past `0.1.3`; re-run the published-API
  compatibility fixtures against the new version; decide the version boundary (`0.1.4` vs `0.2.0`) in light of the
  `CleanupDisposition` / `Error`-extensibility finding already recorded at `plans/registry.md` § "Post-0.1.3 open findings" item
  2; and, if eggpack is to receive the change, separately republish `eggup-eggpack` off its `=0.1.3` pin
  (`crates/eggup-eggpack/Cargo.toml:20`). **M015 does not authorize any of that, and the `v0.1.3` tag must not move.**
- **What this plan does NOT authorize.** Everything listed in §5 "Explicitly out of scope", and in particular: publication of any
  crate; moving the `v0.1.3` tag; a new dependency of any kind, including dev-dependencies; a change to the workspace lint table;
  a change to the `Error` variant set or to `RecoveryError`; any authenticity or signature work; any behavior change to
  `run_bounded`; any change to `crates/eggup-core/Cargo.toml`'s `exclude` list; any CI configuration change beyond adding a step;
  and any edit to `plans/registry.md`, a roadmap, or `plans/000`–`003`.
- **Reuse warning for the next author.** `FailureCategory` is `#[non_exhaustive]`, which is what makes the removal compatible for
  downstream matchers — but it also means a downstream crate **cannot** enumerate the categories, so an external consumer sees the
  taxonomy only through a wildcard arm and `as_str()`. Any future addition therefore reaches consumers without a compile break,
  and the `as_str()` string is the only stable identifier a consumer can safely persist. Do not rename an existing `as_str()`
  string.
