# Eggpack Manifest Interoperability M002a — Hosted Qualification and Clippy Corrective

Status: implemented; closed by `plans/closure/eggpack-manifest-interoperability/002a-status.md` (hosted run `36477024102` green on all lanes)

Repository baseline: `f708e45ab82a35df740704f1af36a7d2275794cf`

Original plan:

- `plans/implementation/eggpack-manifest-interoperability/002-archive-extraction-handoff.md`

Original closure:

- `plans/closure/eggpack-manifest-interoperability/002-status.md`

Observed failed qualification:

- hosted run `36463041223`
- Stable Linux failed at `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- failing location: `crates/eggup-eggpack/src/lib.rs:559`
- lint: `clippy::useless_conversion`
- macOS: passed
- Rust 1.89 MSRV: passed
- Windows existing archive/acquisition/service lane: passed

Primary class: polish/corrective

## 1. Objective

Restore truthful hosted qualification for Eggpack Interop M002 by fixing the single clippy regression in the new archive-handoff adapter and adding direct Windows execution of the new `eggup-eggpack` archive-handoff test target.

M002's architecture and public API remain unchanged.

## 2. Readiness and dependencies

M002a is dependency-ready.

The original M002 implementation is present and its local/macOS/MSRV evidence is otherwise strong. The hosted failure is narrow and mechanically identified. No upstream package or producer change is required.

## 3. Current evidence

Stable Linux run `36463041223` failed on:

~~~text
crates/eggup-eggpack/src/lib.rs:559
members.iter().zip(bound_members.into_iter())
clippy::useless_conversion
~~~

Clippy recommends:

~~~rust
members.iter().zip(bound_members)
~~~

Because Stable checks stop at clippy, the same hosted run did not execute the full Stable workspace test/doc steps.

The M002 closure also explicitly records a LOW evidence gap: the new `tests/archive_handoff.rs` target was not executed on hosted Windows. The existing Windows lane validates `eggup-archive` but not the adapter's new filesystem/mapping composition itself.

## 4. Invariants that must not regress

- no public API change is required;
- the five M002 adapter helpers retain their semantics;
- declaration-order binding remains fail-closed;
- direct/bundle adapter APIs remain unchanged;
- no producer authority enters Eggup;
- `eggup-core` remains archive/Eggpack independent;
- no platform-specific behavior is added merely to satisfy CI;
- Rust 1.89 remains supported;
- first-party unsafe remains forbidden.

## 5. Scope and non-scope

### In scope

- remove the redundant `.into_iter()` triggering current Stable clippy;
- run the focused adapter test suite after the edit;
- extend the hosted Windows lane to execute `eggup-eggpack` archive-handoff tests;
- rerun full Stable/MSRV/macOS/Windows qualification;
- update the M002 closure record from local/partial evidence to hosted evidence;
- reconcile registry/roadmap current-head status.

### Out of scope

- redesigning M002 helper APIs;
- changing archive safety semantics;
- changing Eggpack schema or producer code;
- changing package versions;
- publishing crates;
- Egress migration.

## 6. Required production/workflow changes

### 6.1 Clippy correction

Apply the minimal source correction:

~~~rust
for (declared, bound) in members.iter().zip(bound_members) {
    // existing body unchanged
}
~~~

Do not add a lint allow.

### 6.2 Windows adapter execution

Update CI so the Windows qualification lane executes at least:

~~~text
cargo test -p eggup-eggpack --test archive_handoff --locked
~~~

If all `eggup-eggpack` tests are portable and runtime cost remains acceptable, prefer:

~~~text
cargo test -p eggup-eggpack --all-targets --all-features --locked
~~~

The closure must state which command actually ran.

## 7. Ordered work packages

1. Apply the one-line clippy correction.
2. Run focused adapter tests locally.
3. Add direct hosted Windows adapter execution.
4. Run full local fmt/clippy/workspace tests/docs/MSRV.
5. Push and obtain a fresh hosted matrix.
6. Amend M002 closure evidence with the new run and exact Windows result.
7. Reconcile registry/roadmap and unblock M008a evidence reconciliation.

## 8. Failure, restart, cancellation, and contention semantics

No runtime failure semantics change.

If Windows adapter execution exposes a real platform defect, stop and expand M002a rather than weakening or skipping the test.

## 9. Compatibility and migration

No compatibility or consumer migration effect.

## 10. Required tests

- `eggup-eggpack` unit tests;
- `tests/interoperability.rs`;
- `tests/archive_handoff.rs`;
- full workspace all-target/all-feature tests;
- Rust 1.89 all-target check;
- hosted Windows direct adapter runtime execution;
- hosted macOS full workspace;
- hosted Stable fmt/clippy/tests/docs.

## 11. Verification commands

~~~bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-eggpack --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo +1.89.0 check --workspace --all-targets --locked
cargo doc --workspace --no-deps --locked
git diff --check
~~~

Hosted Windows must execute `eggup-eggpack` tests, not only compile them.

## 12. Documentation updates

- `plans/closure/eggpack-manifest-interoperability/002-status.md`;
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`;
- `plans/registry.md`.

No changelog entry is required for the lint-only source correction unless implementation policy prefers one.

## 13. Acceptance criteria

M002a closes only when:

- current Stable clippy is green;
- full Stable tests/docs run after clippy;
- direct hosted Windows adapter tests are green;
- macOS and MSRV remain green;
- M002 closure no longer carries the LOW "Windows adapter tests not run" finding;
- no runtime/public API behavior changed.

## 14. Stop conditions

Stop and broaden the corrective if:

- Windows exposes a real adapter/archive handoff defect;
- removing `.into_iter()` changes type inference or ownership behavior unexpectedly;
- any M002 negative test regresses;
- current-head CI shows an unrelated failure.

## 15. Closure evidence required

Record:

- implementation SHA;
- exact one-line source diff;
- workflow diff;
- original failed run `36463041223`;
- new hosted run ID;
- per-platform job conclusions;
- exact Windows adapter test command/result;
- M002 closure addendum;
- unresolved findings by severity.

## 16. Handoff notes

Do not use `#[allow(clippy::useless_conversion)]`. The current source has a straightforward idiomatic correction and closure requires proving the final adapter on Windows, not merely compiling it.
