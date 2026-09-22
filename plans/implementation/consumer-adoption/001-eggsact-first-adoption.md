# Consumer Adoption Milestone 001 — Eggsact First Adoption

Status: blocked on core M006 and acquisition M002 closure

Repository baseline for Eggup planning: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/consumer-adoption-roadmap.md`

Primary class: capability / compatibility

## 1. Objective

Use eggsact as Eggup's first real consumer by replacing its duplicated single-binary staging/integrity/candidate/replacement machinery with `eggup-core` + `eggup-eggfetch`, while preserving eggsact-owned release selection and Cargo fallback policy.

## 2. Why eggsact is first

Eggsact already has:

- Eggfetch-native bounded transport;
- single-binary release assets;
- SHA-256 verification;
- target mapping;
- candidate identity/version checks;
- explicit Cargo fallback conditions.

It is therefore a narrow consumer that can expose accidental assumptions without requiring service lifecycle or bundle behavior.

## 3. Dependencies

Hard:

- verified-update-core M006 package qualification;
- acquisition M002 Eggfetch adapter.

The implementation plan must be refreshed against the then-current eggsact HEAD before handoff.

## 4. Invariants

- eggsact remains owner of release/version authority;
- fallback to Cargo occurs only under the same explicitly approved conditions;
- TLS/checksum/candidate failures never become fallback;
- no second HTTP/TLS stack;
- no fetched shell execution;
- existing CLI/output contract remains unless a deliberate change is documented;
- updater failure preserves the old executable or returns actionable RecoveryRequired state.

## 5. Scope

- add Eggup dependencies to eggsact;
- translate eggsact release result into Eggup acquisition + InstallPlan;
- implement existing-destination ownership verifier using eggsact's known executable identity/path evidence;
- map Eggup transaction outcomes into eggsact CLI;
- delete superseded local generic updater code;
- retain target/release/Cargo fallback policy in eggsact;
- add compatibility fixtures/tests;
- measure dependency/binary-size change.

## 6. Explicitly out of scope

- changing eggsact release hosting;
- service management;
- installer rewrite;
- Cargo fallback removal;
- target expansion;
- Eggup API compatibility hacks used only by eggsact.

## 7. Required implementation review before editing

Reinspect eggsact's current:

- `src/update.rs`;
- target table;
- release metadata path;
- exact fallback classifier;
- candidate version output;
- updater tests;
- Cargo feature set.

Do not rely solely on the historical census in Eggup planning.

## 8. Required tests

- already-current/no-op policy remains consumer-owned;
- exact release asset success;
- checksum mismatch;
- candidate wrong identity/version;
- 404 fallback only if existing eggsact policy permits it;
- 5xx/TLS/timeout does not fallback;
- ownership failure;
- rollback outcome mapping;
- recovery-required mapping;
- unsupported target;
- dependency graph contains one intended HTTP/TLS stack.

## 9. Acceptance criteria

- eggsact's generic updater implementation is materially deleted, not wrapped;
- Eggup handles local verified transaction mechanics;
- eggsact owns only release/target/fallback/CLI policy;
- behavior parity is demonstrated;
- no high/medium migration defect remains.

## 10. Stop conditions

Stop if adoption requires adding eggsact-specific release semantics to Eggup, or if Eggup cannot represent a required existing safety behavior without weakening it.

## 11. Closure evidence required

- before/after updater module inventory;
- deleted duplicated code summary;
- behavior matrix;
- dependency and binary-size comparison;
- full eggsact verification;
- Eggup version/API used;
- any generic defect discovered and whether it requires an Eggup corrective.
