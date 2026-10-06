# Verified Update Core M012 — Closure and Verification Record

Status: **closed**; compatibility-corrected candidate hosted-green

Source plan: `plans/implementation/verified-update-core/012-0.1x-public-error-compatibility-corrective.md`

Source roadmap: `plans/subsystems/verified-update-core-roadmap.md`

Implementation commit: `cb9d045` (merged to `main` as `c519a83`)

Publication status: M012 itself publishes nothing. The corrected candidate was
published as `eggup-core 0.1.3` by M013 (`plans/closure/verified-update-core/013-status.md`).

## Executive finding

The published `0.1.2` `Error` enum has exactly seven variants. The M010/M011 tree
had added two, and because `Error` is not `#[non_exhaustive]` either addition
broke a downstream exhaustive `match` even though Cargo treats `0.1.2 -> 0.1.3` as
compatible. The candidate now keeps the seven-variant surface exactly, while every
M010/M011 safety semantic is retained.

Retained evidence moved to a new `#[non_exhaustive] RecoveryError`, used only by
the three still-unpublished M011 entry points. Injected fault failures remain
**structurally** classified through a `#[cfg(test)]`-only marker; the old
substring match on `"injected"` does not come back.

## Requirement → evidence matrix

| # | Requirement | Evidence | Result |
|---|---|---|---|
| 1 | Capture the exact `0.1.2` public `Error` definition | downloaded `eggup-core-0.1.2.crate`, sha256 `0f44129c…8e6`; read `src/error.rs` from the published artifact | pass |
| 2 | `Error` has the same variant set as `0.1.2` | candidate `Error` = the 7 published variants; `Injected` is `#[cfg(test)]`, `RecoveryRequired` moved out | pass |
| 3 | Fixture A fails against pre-M012 main | `E0004: non-exhaustive patterns: RecoveryRequired and Injected not covered` | pass (detection gap proven) |
| 4 | Fixture A compiles unchanged against `0.1.2` and the candidate | byte-identical `main.rs`; only the dependency source was retargeted; both compile and run | pass |
| 5 | No old public method signature changed | `acquire`, `prepare*`, `verify_integrity`, `validate`, `commit`, `commit_with_post_commit` all keep `Result<T, Error>` | pass |
| 6 | Only unpublished M011 APIs changed signature | exactly `acquire_with_recovery`, `commit_with_stale_lock_recovery`, `commit_with_post_commit_and_stale_lock_recovery` | pass |
| 7 | Default `acquire` remains fail-closed | `stale_lock_recovery.rs::ordinary_acquire_never_recovers_an_existing_record` asserts the published `Error` result type directly | pass |
| 8 | Injected classification structural, not public | `#[cfg(test)]` variant + `#[cfg(test)]` fault branches; packaged build has no such branch | pass |
| 9 | Recovery-required stays typed, retains a real path | `RecoveryError::RecoveryRequired { evidence, detail }`; `evidence()` accessor; Fixture C asserts a real path | pass |
| 10 | Receipt-level recovery semantics preserved | `TransactionDisposition::RecoveryRequired`, `CleanupDisposition::RetainedForRecovery`, `FailureCategory::RetainedEvidence` untouched | pass |
| 11 | `eggup-service 0.1.2` compiles/runs against the candidate | Fixture B: published service resolves and runs | pass |
| 12 | M010/M011 behavioral suites green | full gate, **381 tests** | pass |
| 13 | Full hosted matrix green | run `37527706900` on `bd43683` (which contains this change): Stable ✅ MSRV ✅ macOS ✅ Windows ✅ | pass |
| 14 | No message-substring classification | no `"injected"` substring match remains in `src/` | pass |

## Compatibility proof (external fixtures, outside the workspace)

**Fixture A — published exhaustive match.** Matches all seven published `Error`
variants with no wildcard arm. Compiles and runs against published `=0.1.2`, then
against the candidate with only the dependency source retargeted. `main.rs` was
verified byte-identical between the two runs. It **fails** against pre-M012 main
with exactly the two added variants — so the fixture is discriminating, not
decorative.

**Fixture B — service transitive.** Depends only on published `eggup-service
0.1.2`; exercises M006 disposition and lifecycle types. Resolves and runs against
the candidate core.

**Fixture C — M011 recovery API.** `ProvenStale` success; `Active`/`Unknown`
typed contention with the record preserved byte-for-byte; recovery-required
failure matchable without parsing `Display`, with `evidence()` naming a real path;
`Core` chaining via `source()`; `From<Error>` ergonomics.

## Finding the plan did not anticipate

The plan's objective is a compatibility-preserving `0.1.3`, and it names two
`Error` variants. It did **not** account for the other two closed enums the
M010/M011 tree had extended. Auditing the published `0.1.2` artifact directly:

| Enum | `#[non_exhaustive]` in `0.1.2`? | Added variant | Breaking? |
|---|---|---|---|
| `Error` | no | `Injected`, `RecoveryRequired` | **yes** — named by M012, fixed |
| `FailureCategory` | **yes** | `RetainedEvidence` | no — genuinely additive |
| `CleanupDisposition` | **no** | `DeferredToProcessExit` | **yes** — not named by M012 |

The crate changelog had asserted all three enums were `#[non_exhaustive]`. Only
`FailureCategory` was; that false claim is what produced the gap, and it is
corrected in the changelog rather than left standing.

`CleanupDisposition::DeferredToProcessExit` was **kept**, because removing it would
weaken M010 truthfulness. A kept-installed Windows self-update cannot unlink the
previous generation while the running process still maps it, and neither
published value is honest: `Cleaned` would claim the old image is gone, and
`RetainedForRecovery` would page an operator for a state that needs no operator
action. Marking the enum `#[non_exhaustive]` was rejected because it breaks the
same matches a new variant breaks, and M012 deliberately reserves that attribute
for an intentional 0.2/1.0 boundary.

Impact is bounded: the affected surface is a single accessor on
`TransactionReceipt`, and a repository search across `egstack/{eggsact,
stegoeggo, egress, eggsearch, codegg}` found **zero** references to
`CleanupDisposition`.

## Unresolved findings

| Finding | Severity | Disposition |
|---|---|---|
| `CleanupDisposition::DeferredToProcessExit` is a source-visible addition to a closed enum | **low** | Deliberate and documented in the crate changelog, the root changelog, and the `0.1.3` release notes. Required by M010; zero current consumers affected. Revisit at the 0.2/1.0 boundary. |
| `eggup-curl` sub-second-deadline tests are load-sensitive on a busy host | **low** | Pre-existing, reproduced at pristine HEAD under load, `eggup-curl` source is outside M012 scope. Hosted CI passes both. Worth a corrective. |

No `medium-or-higher` defect remains open. M013 is unblocked.