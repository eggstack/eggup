# Service Lifecycle Milestone 007 — UTF-8-Safe Bounded Diagnostics Corrective

Status: implemented

Repository baseline: `ee1476ef2a9e0d5569d6dc2469e780a4435cc426`

Source roadmap:

- `plans/subsystems/service-lifecycle-roadmap.md`

Original work corrected by this pass:

- service M001-M006 bounded diagnostic/error contracts.

Long-term requirements:

- `plans/000-long-term-specification.md#13-service-lifecycle`
- `plans/000-long-term-specification.md#18-security-model`
- `plans/000-long-term-specification.md#21-observability`

Applicable ADRs:

- `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md`
- `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md`

Primary class: invariant/corrective

## 1. Objective

Eliminate panic-capable fixed-byte string truncation from production service diagnostics while preserving the existing 256/512-byte bounds, credential/control-character handling, and service lifecycle semantics.

This is a deliberately narrow post-M006 corrective.

## 2. Why this corrective is ready

Rust `String::truncate` panics when the requested byte index is not a UTF-8 character boundary.

Production `eggup-service` currently performs fixed-byte truncation in:

- `ServiceError::invalid`;
- `ServiceError::bounded`;
- the shared 256-byte command-output `truncate` helper.

These helpers can receive manager/subprocess/caller-derived Unicode text. A diagnostic bound must not convert unusual Unicode into a process panic.

`disposition.rs::bound_detail` already uses the correct pattern by backing the limit down to `is_char_boundary`.

No architecture change is required.

## 3. Current implementation evidence

M006 daemon disposition/revalidation semantics are closed and should remain untouched.

The defect is isolated to bounded diagnostic construction and command-output display helpers in `eggup-service`.

Existing service execution already bounds captured output size independently; this corrective only makes the final string representation panic-free.

## 4. Invariants that must not regress

- all service diagnostics remain finitely bounded;
- no control-character/credential exposure is introduced;
- manager raw output remains subject to existing capture bounds;
- ownership classification does not change;
- no destructive action on Foreign/Unknown;
- transition deadlines do not change;
- service manager executable/environment policy does not change;
- M005/M006 rollback/disposition semantics do not change;
- public error categories remain source/behavior compatible unless a change is unavoidable.

## 5. Scope

### In scope

- add one UTF-8-safe byte-bounding helper inside `eggup-service`;
- use it for every production fixed-byte diagnostic truncation;
- preserve existing 256/512 byte ceilings;
- add Unicode boundary tests;
- audit nearby string slicing/truncation in production service code;
- update service docs/roadmap/registry/closure.

### Explicitly out of scope

- lifecycle orchestration changes;
- service-manager behavior changes;
- new event/progress API;
- acquisition changes;
- Gregg migration;
- public error taxonomy redesign;
- package publication.

## 6. Required production changes

Implement a helper equivalent in behavior to:

```text
fn truncate_utf8_bytes(s: String, max: usize) -> String
```

It must:

- leave shorter strings unchanged;
- cap by bytes, preserving the existing bounded-output guarantee;
- back down to a valid character boundary;
- never panic for valid Rust `String`;
- avoid allocating more than necessary.

Replace direct production `String::truncate(256/512)` calls on arbitrary service diagnostics.

Do not use lossy conversion to solve the boundary problem where the input is already valid UTF-8.

## 7. Ordered work packages

1. Add failing multibyte boundary regression tests around 256/512 bytes.
2. Add the shared safe-bound helper.
3. Replace `ServiceError` truncation.
4. Replace command-output diagnostic truncation.
5. Audit production service module for other fixed byte slicing/truncation.
6. Run full M005/M006 orchestration and manager adapter regression suites.
7. Update roadmap/registry and write closure.

## 8. Failure, cancellation, restart, and contention semantics

No lifecycle failure/cancellation/restart/contention behavior changes.

The only behavioral change is that overlong Unicode diagnostics are safely shortened instead of potentially panicking.

If any implementation edit changes manager calls, timing, ownership, restore behavior, or transaction ordering, stop and split it into a separate corrective.

## 9. Compatibility and migration

No public API change is expected.

Display strings may end at a slightly earlier byte position when a multibyte character crosses the existing ceiling. This is the intended correctness fix.

No downstream migration is required.

## 10. Required tests

At minimum:

- ASCII at exactly 256/512 bytes;
- ASCII above each bound;
- two-byte code point crossing each bound;
- three-byte code point crossing each bound;
- four-byte code point crossing each bound;
- mixed manager error text remains bounded;
- permission remediation text remains bounded;
- existing service adapter/orchestration tests remain green.

## 11. Required verification commands

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test -p eggup-service --all-targets --all-features --locked
cargo test --workspace --all-targets --all-features --locked
cargo doc --workspace --no-deps --locked
cargo +1.89.0 check --workspace --all-targets --locked
./scripts/check-local.sh
git diff --check
```

Run hosted Stable/MSRV/macOS/Windows qualification.

## 12. Documentation updates

Only bounded-diagnostic behavior and planning status need updates. Do not expand the service architecture documentation with unrelated changes.

## 13. Acceptance criteria

M007 closes only when:

- all production service fixed-byte diagnostic truncation is UTF-8 safe;
- multibyte boundary regressions are covered;
- existing byte ceilings remain enforced;
- M005/M006 lifecycle/disposition tests are unchanged/green;
- hosted qualification is green;
- no medium-or-higher related finding remains open.

## 14. Stop conditions

Stop and write another plan if the audit uncovers:

- unbounded subprocess capture rather than only final diagnostic truncation;
- credential leakage;
- lifecycle mutation defects;
- a shared cross-crate diagnostics API that would require a new public crate/contract.

## 15. Closure evidence required

Record:

- implementation SHA(s);
- audited truncation sites;
- multibyte regression matrix;
- service regression results;
- hosted platform results;
- unresolved findings by severity.

## 16. Handoff notes

Keep this corrective narrow. `disposition.rs::bound_detail` already demonstrates the desired UTF-8-boundary pattern; consolidate behavior without refactoring service state machines.
