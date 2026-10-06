# Verified Update Core M011 — Closure and Verification Record

Status: closed; hosted-qualified on Stable/MSRV/macOS/Windows (`37376971555`)

Source plan: `plans/implementation/verified-update-core/011-proof-authorized-stale-lock-recovery.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md#M011--proof-authorized-stale-lock-recovery`

Implementation commit: `95f756786708521c3315439df882106a53211cce`

Publication status: **superseded by M012.** These APIs first appeared on the
registry in `eggup-core 0.1.3` (M013), not 0.1.2.

> **M012 addendum (2026-10-06).** This record originally flagged
> `Error::RecoveryRequired` as a breaking addition to `Error`. M012 removed it
> from the public enum before publication: retained evidence now lives in the new
> `#[non_exhaustive] RecoveryError`, and `Error` keeps exactly the seven
> variants `0.1.2` published. The two recovery entry points documented here
> (`acquire_with_recovery`, `commit_with_stale_lock_recovery`) therefore return
> `RecoveryResult<T>`. No behavior described below changed — the claim/recovery
> protocol, the `ProvenStale`-only authorization rule, and the fail-closed
> default are all unchanged. See `plans/closure/verified-update-core/012-status.md`.

## Executive finding

Stale-lock recovery now exists, and Core still does not decide staleness. The
split is explicit in the type system: **authorization** is the caller's
(`StaleLockDecision` returned by `StaleLockVerifier::classify` for one
`LockObservation`), **mechanism** is Core's
(`MutationLock::acquire_with_recovery`). Only `ProvenStale` authorizes a
mutation, and it authorizes exactly one observation, bound to its exact bytes.

Two things this milestone deliberately did **not** do:

- **It did not make PID liveness a Core policy.** The plan rejected that
  explicitly, and Core supplies no liveness, age, executable, or service fact to
  the verifier. `LockObservation` carries the path, the exact bounded record
  bytes, and whatever the known format parses. The example makes the reasoning
  concrete: a supervised pid that the consumer can no longer see is *suggestive*,
  and only the deployment's own out-of-band statement makes it conclusive. A pid
  the consumer never supervised stays `Unknown`.
- **It did not change the default.** `MutationLock::acquire` still returns
  `UpdateInProgress` for any existing record, twice in a row, forever.
  `MutationLock::inspect` is untouched. Existing commit methods keep their exact
  signatures. Recovery is opt-in per commit via
  `ValidatedTransaction::commit_with_stale_lock_recovery` or
  `commit_with_post_commit_and_stale_lock_recovery`, so an existing consumer
  that ignores recovery cannot start deleting locks by upgrading.

The claim sequence is race-safe without unsafe code: re-read the record's exact
bytes, rename the pathname into a unique Eggup-owned claim path **in the same
directory**, then re-read the *claimed object* and require it to still equal the
authorized observation before anything is created. A record that changed in
between is never deleted. A writer that creates the lock after the claim wins,
and its record is never removed.

## Requirement-to-evidence matrix

| Requirement (source plan §) | Evidence | Result |
|---|---|---|
| §4 ordinary `MutationLock::acquire` remains fail-closed | `ordinary_acquire_never_recovers_an_existing_record`: two consecutive acquires both return `UpdateInProgress`, the record survives, and no claim is ever created | passed |
| §4 no PID-liveness policy introduced into Core | `LockObservation` exposes only `path`, `record`, and `Option` fields from the known format; the trait returns a verdict and cannot mutate; the example separates `supervised` from `alive` and requires `declared_idle` | passed |
| §6.3 explicit, visibly separate recovery API | `acquire_with_recovery` plus two opt-in commit methods; the default acquire and every existing commit signature are unchanged | passed |
| §6.3 bounded typed observation | `LockObservation` with accessors; `MAX_LOCK_BYTES` = 4096 enforced in `inspect`, `read_record`, `create_new`, and record parsing | passed |
| §6.3 verifier sees the exact record, not a pathname | `caller_supplied_proof_sees_the_exact_record`: exactly one observation, `record()` equal to the planted bytes, all four fields parsed, `format_known()` true | passed |
| §6.4 create-new first | `acquire_with_recovery` returns immediately on a successful create-new, so the common path never reaches the claim machinery | passed |
| §6.4 claim-and-verify before deletion | `claim` (`lock.rs:404`) re-reads, renames, then re-reads the claimed object; the record is deleted by `remove_claim` only after `create_new` has returned `Some` | passed |
| §6.4 same-filesystem claim path | `claim_path` (`lock.rs:481`) joins the same installation root as the lock | passed |
| §6.4 object changed → no destructive delete | `record_replaced_after_observation_is_never_deleted`: the verifier swaps the record after observation; result is `UpdateInProgress`, the replacement survives byte-identical, and **no claim file is left behind** | passed |
| §6.4 second writer after claim wins | `competing_writer_after_the_claim_is_preserved` (in-crate, using the `cfg(test)` post-claim hook so the window is hit deterministically): the competing writer's record survives byte-identical and our own claim is cleaned up | passed |
| §6.4 restore when safe | `restore_claim` (`lock.rs:535`) re-materializes the record with `create_new`, which cannot replace an existing path; when the path was re-taken it retains the evidence and reports it. `restore_never_clobbers_a_lock_taken_in_the_meantime` | passed |
| §6.4 claim cleanup failure reports the real retained path | `remove_claim` (`lock.rs:566`) returns `Error::RecoveryRequired { evidence: <real path> }`; `retained_evidence_error_names_the_real_path` asserts the path is real and named in the message | passed (contract), see residual risk |
| §6.4 bounded loop, then contention | one observation, one create-new retry after an observation that found nothing; no unbounded retry | passed |
| §6.4 return `Err` only for genuine setup failures | contention, unknown, malformed, and changed-record all map to typed `Err` variants carrying real paths, never a synthetic success | passed |
| §4.2 malformed / oversized / symlink / non-regular / unreadable never displaced | `unsafe_records_never_reach_destructive_recovery`: non-UTF-8, 8 KiB record, a directory in the lock's place, and a symlink (Unix) each retain the record and create no claim | passed |
| §4.4 old-format parsing is conservative | `unknown_record_format_is_observed_without_inventing_fields`: an unrecognised record is still observable with exact bytes, but `format_known()` is false and every parsed field is `None` | passed |
| §4.5 crash after a claim is not deletion permission | there is no sweeper, no age-based collector, and no code path anywhere that deletes a `.eggup-stale-claim-*` path it did not just create | passed |
| §4.3 bounded non-ASCII diagnostics panic-free | `bounded_non_ascii_record_diagnostics_stay_panic_free`: 300 two-byte characters, rendered diagnostics stay valid UTF-8 under 512 bytes, the exact bytes are preserved in the observation | passed |
| §4.3 current owner never deletes a replacement lock | `owner_drop_never_deletes_a_replacement_record` (pre-existing `Drop` semantics, now covered) | passed |
| §6.4 no process-enumeration dependency enters Core | `self-replace` is the only added dependency and is Windows-only for M010; M011 adds none. `cargo tree` on macOS: `eggup-core → sha2` | passed |
| §8 deterministic fixtures | no sleeps and no threads; races are injected deterministically from inside the verifier callback, which is the only hook that runs in the required order | passed |
| §8 record replaced before claim | `record_replaced_after_observation_is_never_deleted` | passed |
| §8 claim cleanup failure | contract test — see residual risk 1 | partial, disclosed |
| §9 example verifier demonstrating external evidence without Core PID policy | `examples/stale_lock_recovery.rs` runs and prints all four cases: default acquire retains, conclusive proof recovers, no out-of-band proof retains, foreign deployment retains | passed |
| §10 docs updated | `crates/eggup-core/docs/transaction.md`, `architecture/core-transaction.md` (two new sections), `architecture/overview.md`, `AGENTS.md`, crate `README.md`, both changelogs | passed |

## Design decisions worth recording

**Authorization binds to bytes, not to a path.** `StaleLockDecision::ProvenStale`
is a verdict about one `LockObservation`, and `claim` re-reads before displacing.
That is what makes "record replaced after observation is never deleted" true
rather than aspirational.

**The verifier cannot mutate.** `StaleLockVerifier::classify` takes
`&LockObservation` and returns a `Copy` enum. It has no path it can write to and
no lock handle, so even a hostile or buggy implementation cannot delete
anything. Core performs every mutation.

**A partial recovery fails loudly.** If the claim cannot be cleaned after
ownership is established, the lock is dropped and
`Error::RecoveryRequired` names the real retained path, rather than returning
the lock and implying recovery completed. In practice `remove_file` on a file
this code just renamed in a directory this code must be able to write to
essentially always succeeds, so this is a correctness statement rather than an
expected path.

**The restore does not use `rename`.** POSIX `rename` atomically *replaces* its
destination, so restoring the displaced record by renaming the claim back over
the lock path would unlink a competing writer's record underneath it. A freeness
check before that rename is a TOCTOU. `restore_claim` therefore re-materializes
the record with `create_new`, which by definition cannot replace anything.

**The post-claim window needs a test seam, not a race.** The branch between the
claim rename and the create-new retry is microseconds wide; exercising it with a
real multi-process race would prove nothing about ordering. A `cfg(test)` hook
fires there so the competing write is injected deterministically. It compiles
away from production builds, and `hook_ran` is asserted so the test cannot
silently degrade into asserting the *pre-claim* rejection path — which is
exactly the bug an earlier draft of that test had.

**`Unknown` is the default a careful caller should return.** The example's
cautious verifier deliberately returns `Unknown` for a supervised-but-invisible
pid with no out-of-band statement, and the fixture
`unknown_record_format_is_observed_without_inventing_fields` shows the same
discipline for an unrecognised record. Both retain the record.

**Backwards compatibility is structural, not documented.** The opt-in lives on
`ValidatedTransaction`, and `commit_inner` gained an
`Option<&dyn StaleLockVerifier>` parameter that every pre-existing caller passes
as `None`. A consumer that never names the new methods cannot change behaviour
by upgrading.

## Test inventory

`crates/eggup-core/tests/stale_lock_recovery.rs` — 14 tests, all passing:

`ordinary_acquire_never_recovers_an_existing_record`,
`only_proven_stale_displaces_the_record`,
`proven_stale_record_is_claimed_and_acquisition_succeeds`,
`caller_supplied_proof_sees_the_exact_record`,
`unknown_record_format_is_observed_without_inventing_fields`,
`unsafe_records_never_reach_destructive_recovery`,
`record_replaced_after_observation_is_never_deleted`,
`record_replaced_during_verification_is_never_claimed`,
`retained_evidence_error_names_the_real_path`,
`owner_drop_never_deletes_a_replacement_record`,
`inspect_remains_read_only_and_backward_compatible`,
`observation_of_an_absent_record_is_none`,
`bounded_non_ascii_record_diagnostics_stay_panic_free`,
`commit_recovers_only_with_caller_proof`.

Two further tests live in `src/lib.rs` because they need `pub(crate)` access:

- `competing_writer_after_the_claim_is_preserved` — the genuinely post-claim
  window, via the `cfg(test)` hook. Asserts the hook actually ran, so the test
  cannot pass vacuously by falling back to the pre-claim rejection path.
  Verified by mutation: removing the `remove_claim` call makes it fail.
- `restore_never_clobbers_a_lock_taken_in_the_meantime` — the restore cannot
  clobber a lock a competing writer took.

`commit_recovers_only_with_caller_proof` is the end-to-end proof of the opt-in:
the same fixture, with an identical planted record, fails with
`UpdateInProgress` through `commit` and succeeds with `Committed` through
`commit_with_stale_lock_recovery`.

Races are injected from inside `StaleLockVerifier::classify` rather than by
timing. That works because the required ordering is fixed — observation happens,
then the verdict, then the re-read, then the claim — so writing the competing
record during the verdict lands exactly in the window the claim has to survive.
The test therefore proves the *logic* of the race handling, not the occurrence
of a real race. See residual risk 2.

## Verification results

Local gate, `./scripts/check-local.sh` — exit 0 (fmt, clippy
`--all-targets --all-features --locked -- -D warnings`, `cargo test
--workspace --all-targets --all-features --locked`, `cargo doc --workspace
--no-deps --locked`).

`cargo +1.89.0 check --workspace --all-targets --locked` — green.

```text
stale_lock_recovery (integration)   14 passed; 0 failed
```

`cargo run -p eggup-core --example stale_lock_recovery` output:

```text
default acquisition never recovers:
  installation update already in progress: .../.eggup-mutation.lock
recovered after caller proof:
  lock record: .../.eggup-mutation.lock
  our own record is removed when it is dropped
without out-of-band proof:
  installation update already in progress: .../.eggup-mutation.lock
  record still present: true
for another deployment's record:
  installation update already in progress: .../.eggup-mutation.lock
  record still present: true
```

## Hosted evidence

Run `37376971555` on `c53b55c`, all four lanes green. The `windows-check` lane
executes `cargo test -p eggup-core --test stale_lock_recovery --locked` natively
— `rename` and file-deletion semantics differ per platform, and a macOS pass is
not evidence for Windows. The Windows lane reports **489 passing tests and 0
failing targets** across the full workspace run.

## Residual risk

1. **`remove_claim` failure still has no deterministic trigger.** The unsafe
   *branch* of `restore_claim` is now covered
   (`restore_never_clobbers_a_lock_taken_in_the_meantime`), but a filesystem
   fault mid-`write_all` is not: that path is exercised only by reading the code.
   A truly exhaustive proof would need an injected filesystem fault, which is new
   machinery this milestone does not have.
3. **`MutationLock::drop` has a pre-existing check-then-remove window.** The
   contents comparison and `remove_file` are separate syscalls, so a recovering
   writer that swaps in a fresh record between them can have it deleted. This is
   **unchanged by these milestones** and not a regression, but it is the same
   class of defect as the `rename` issue found above and is recorded here rather
   than silently left unmentioned. `owner_drop_never_deletes_a_replacement_record`
   cannot catch it, because there is no interleaving hook. Fixing it properly
   needs atomic compare-and-delete, which std does not offer portably.
2. **The race tests are logically ordered, not genuinely concurrent.** They
   prove the implementation handles a changed record and a competing writer
   *when those events occur at the specified point*. They do not demonstrate
   robustness under real multi-process contention. The `windows-check` job runs
   the same fixtures, which improves platform coverage but does not change this.
4. **No crash-recovery sweep exists, by design.** A process killed between the
   claim rename and `create_new` leaves a `.eggup-stale-claim-*` file behind and
   the lock path free — so the *next* acquisition succeeds normally, and the
   orphaned claim is inert garbage the operator must remove by hand. The plan
   treats that as the correct trade-off rather than a follow-up sweeper, and no
   automatic collector was added.