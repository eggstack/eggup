# Acquisition Transport Milestone 006 — M005 Windows Portability and Cross-Closure Qualification Corrective

Status: ready for handoff

Repository baseline: `eb989659feabd44e3c1441bb8eb522614ce96a31`

Source roadmap:

- `plans/subsystems/acquisition-transport-roadmap.md`

Original work corrected by this pass:

- `plans/implementation/acquisition-transport/005-curl-adapter-and-transport-composition.md`
- `plans/closure/acquisition-transport/005-status.md`
- `plans/implementation/service-lifecycle/006-daemon-update-disposition-and-reference-qualification.md`
- `plans/closure/service-lifecycle/006-status.md`

Failure evidence:

- GitHub Actions run `36169295410` on `eb989659feabd44e3c1441bb8eb522614ce96a31`
- Windows job `108184625301`
- failing command: `cargo check --workspace --all-targets --locked`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`
- `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md`
- `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md`

Primary class: invariant/corrective

## 1. Objective

Correct the post-M005/M006 hosted-qualification defect and reconcile closure evidence so Eggup does not claim cross-platform qualification while the final Windows workspace lane is red.

The implementation defect is narrow: `eggup-curl`'s test module unconditionally imports Unix-only `std::os::unix::fs::PermissionsExt` and calls `Permissions::set_mode`, so Windows cannot compile the test target under `--all-targets`. The same Windows run also exposes an `eggup-core::stage` unused-import warning for `PermissionsIntent`.

This pass must make the test/support code portable without weakening Unix permission assertions, remove the Windows-only warning, rerun the full hosted matrix, and reconcile M005/M006 closure evidence plus stale roadmap/registry status.

Gregg remains read-only. Do not author or implement Gregg consumer M004 in this pass.

## 2. Why this corrective is ready

The defect is reproduced by hosted CI on the current main head:

```text
error[E0433]: cannot find `unix` in `os`
 --> crates/eggup-curl/src/lib.rs:662:18

error[E0599]: no method named `set_mode` found for struct `Permissions`
 --> crates/eggup-curl/src/lib.rs:787:15
```

The same run reports:

```text
warning: unused import: `PermissionsIntent`
 --> crates/eggup-core/src/stage.rs:5:50
```

Other hosted jobs on the same head passed: Stable checks, Rust 1.89 MSRV, and macOS tests. Windows check failed.

No architecture decision is required.

## 3. Current implementation evidence

Acquisition M005 already implemented `eggup-curl`, `AcquisitionError::Unavailable`, `ComposedTransport`, and footprint fixtures.

Service M006 already implemented the five daemon-update dispositions, Unix/Windows planners, `DirectRuntimeControl`, and `commit_with_disposition`.

Both closure records were written while hosted CI was still pending. The subsequent push proved the Windows workspace qualification was incomplete.

Current evidence points to a test/support portability defect, not a production transport or service semantic defect.

## 4. Invariants that must not regress

- M005 transport semantics remain unchanged.
- Exact `NotFound` remains terminal.
- Default composed transport fallback remains unavailability-only.
- Curl-only consumers still avoid Eggfetch/TLS linkage.
- Unix permission-hardening assertions remain meaningful.
- Windows must not compile Unix-only APIs.
- M006 disposition/revalidation semantics remain unchanged.
- Gregg remains untouched and unlinked.
- Closure evidence must record pending, failed, and corrected hosted results truthfully.
- A red required hosted lane blocks corrective closure.

## 5. Scope

### In scope

- target-gating or splitting Unix-only curl test helpers/imports;
- a Windows-valid fake-child/test strategy only where needed;
- preserving Unix 0700/0600 permission assertions under `#[cfg(unix)]`;
- target-appropriate `PermissionsIntent` import/use cleanup in `eggup-core::stage`;
- Windows package/workspace all-target checks;
- full hosted Stable/MSRV/macOS/Windows rerun;
- focused core/acquisition/curl/service regression tests;
- M005 and M006 closure addenda/reconciliation;
- acquisition/service/consumer roadmap reconciliation;
- registry reconciliation.

### Explicitly out of scope

- modifying Gregg;
- authoring Gregg M004;
- changing transport fallback policy;
- changing daemon disposition semantics;
- release/version selection;
- Cargo/source fallback;
- publishing a new release;
- archive extraction;
- Eggpack producer work;
- unrelated refactors.

## 6. Required production changes

### 6.1 Curl test portability

Do not import `std::os::unix::fs::PermissionsExt` on Windows.

Prefer the smallest truthful fix:

- gate Unix-only imports and mode-setting with `#[cfg(unix)]`;
- keep Unix mode assertions on Unix;
- preserve platform-neutral transport/composition tests on all platforms;
- if fake curl execution is required on Windows, use a deterministic Windows-valid fixture executable/command strategy without changing production `CurlTransport`'s direct-executable/no-shell invariant.

Do not simply disable the entire curl test module on Windows.

### 6.2 Core warning cleanup

Make `PermissionsIntent` import/use target-specific so Windows no longer reports it unused.

Do not change permission semantics.

### 6.3 Hosted qualification

Required final evidence:

- Stable checks green;
- Rust 1.89/MSRV green;
- macOS tests green;
- Windows `cargo check --workspace --all-targets --locked` green.

If a hosted lane fails for a repository defect, keep the corrective open.

### 6.4 Closure evidence reconciliation

Append clearly marked post-closure corrective evidence to:

- `plans/closure/acquisition-transport/005-status.md`;
- `plans/closure/service-lifecycle/006-status.md`.

Do not erase the historical fact that the original records were written with hosted CI pending.

Record the original failed workflow/job IDs, corrective implementation SHA, succeeding hosted workflow/job IDs, and final disposition.

### 6.5 Planning/status reconciliation

At minimum correct:

- service roadmap dependency graph showing M006 as `[ready]`;
- consumer roadmap status/graph/blocker still saying M005/M006 must close;
- registry recently-closed service range stopping at M005;
- registry “Gregg remains footprint-gated” wording;
- registry implementation baseline/current qualification text.

Until this corrective closes, Gregg M004 remains deferred and unwritten.

## 7. Ordered work packages

1. Add or preserve a Windows compile regression reproducing the current curl test-target failure.
2. Correct Unix-only fake-curl permission setup with target-specific test support.
3. Preserve Unix permission assertions and platform-neutral tests.
4. Clean the core Windows unused import.
5. Run focused core/acquisition/curl/service checks.
6. Run full workspace fmt/clippy/test/doc/check and Rust 1.89 checks.
7. Push and require all hosted lanes to pass.
8. Amend M005/M006 closure evidence with failed and succeeding hosted runs.
9. Reconcile acquisition/service/consumer roadmaps and registry.
10. Write the M006 corrective closure record.

## 8. Failure, cancellation, restart, and contention semantics

This corrective changes no runtime transport cancellation, lifecycle restart, rollback, or mutation semantics.

Any production-path semantic change is scope expansion and requires a new corrective/ADR.

## 9. Compatibility and migration

No public API change is expected.

No downstream migration is required.

No crate publication occurs.

Gregg remains untouched and M004 remains unwritten.

## 10. Required tests

At minimum:

- Windows `cargo check -p eggup-curl --all-targets --locked`;
- Windows `cargo check --workspace --all-targets --locked`;
- Unix curl permission-mode test still asserts intended modes;
- curl status/timeout/cancellation/fake-child tests remain green where supported;
- acquisition composition tests remain green;
- core staging/permission tests remain green;
- service M006 regression suite remains green;
- no Gregg dependency appears in `cargo tree`.

## 11. Required verification commands

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo check --workspace --all-targets --locked
cargo check -p eggup-curl --all-targets --locked
cargo check -p eggup-core --all-targets --locked
cargo test -p eggup-acquisition --locked
cargo test -p eggup-curl --locked
cargo test -p eggup-service --locked
cargo tree --workspace --locked
```

Also run the repository's Rust 1.89/MSRV and hosted Stable/macOS/Windows jobs.

## 12. Documentation updates

Update only qualification/status documents:

- M005 closure addendum;
- M006 closure addendum;
- acquisition roadmap;
- service roadmap;
- consumer-adoption roadmap;
- registry;
- M006 corrective closure record.

## 13. Acceptance criteria

M006 closes only when:

- Windows compiles `eggup-curl` all targets without Unix-only API errors;
- Unix permission-hardening coverage remains present;
- the Windows-only `PermissionsIntent` warning is gone;
- Stable, MSRV, macOS, and Windows hosted jobs all pass on the corrective head;
- M005/M006 closure records truthfully record original pending/failure and corrected evidence;
- stale service/consumer/registry status text is reconciled;
- Gregg M004 remains intentionally unwritten and Gregg is unmodified;
- no public API/semantic regression is introduced;
- no medium-or-higher finding remains open.

## 14. Stop conditions

Stop and write a new corrective/ADR if:

- the Windows failure exposes production `CurlTransport` behavior;
- a Windows fake-child strategy requires weakening the production no-shell invariant;
- M006 service code itself fails Windows compilation or semantic tests;
- a public API change is required;
- hosted Windows continues to fail after the identified Unix-only test code is corrected;
- any Gregg modification becomes necessary.

## 15. Closure evidence required

The closure record must include:

- corrective implementation commit(s);
- original failed run `36169295410` and Windows job `108184625301`;
- exact original compiler diagnostics;
- focused Windows compile evidence;
- succeeding hosted workflow and per-job conclusions;
- Unix permission-test evidence;
- core warning cleanup evidence;
- M005/M006 closure addendum links;
- roadmap/registry reconciliation evidence;
- Gregg non-modification/non-dependency confirmation;
- unresolved findings/severity;
- final disposition.

## 16. Handoff notes

This is a qualification corrective, not a feature milestone. The expected production diff should be small; most work is platform-correct test support plus truthful evidence/bookkeeping.

Do not redesign curl transport composition or daemon lifecycle behavior. Do not author Gregg M004.
