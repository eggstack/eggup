# Consumer Adoption Milestone 002 — Stegoeggo Second Adoption

Status: ready for handoff (unblocked by adoption M001 closure; refreshed against stegoeggo HEAD at execution time)

Repository baseline for Eggup planning: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

Source roadmap:

- `plans/subsystems/consumer-adoption-roadmap.md`

Primary class: capability / compatibility

## 1. Objective

Adopt Eggup in stegoeggo as the second independent single-binary consumer and use the migration to detect/remove any eggsact-specific assumptions from the shared API.

## 2. Dependencies

Hard:

- eggsact adoption M001 closure;
- the same or newer qualified Eggup package versions.

Before handoff, refresh this plan against current stegoeggo HEAD and the closure findings from eggsact.

## 3. Invariants

- stegoeggo retains release/target/CLI policy;
- Eggup owns staging, integrity, candidate validation, ownership revalidation, commit/rollback;
- no second HTTP/TLS stack;
- existing supported-target and updater security behavior does not regress;
- consumer-specific compatibility quirks do not get added to Eggup unless they are demonstrably generic.

## 4. Scope

- replace stegoeggo local updater mechanics;
- use `eggup-eggfetch`;
- translate existing release selection to exact acquisition requests;
- provide destination ownership proof;
- map transaction outcomes;
- delete superseded code;
- update tests/docs;
- measure dependency/binary-size delta.

## 5. Required implementation review

Reinspect current:

- `stegoeggo-cli/src/update.rs`;
- release workflow/asset naming;
- target mapping;
- candidate output;
- current Eggfetch features;
- updater/install tests.

Compare with eggsact adoption closure before deciding whether an API issue is generic or consumer-local.

## 6. Required tests

Same core failure classes as eggsact plus stegoeggo-specific target/asset fixtures.

Add a cross-consumer compatibility check in Eggup documentation or examples if both consumers use the same shared path with different release-policy adapters.

## 7. Acceptance criteria

- two independent consumers use the same Eggup core/acquisition contracts;
- neither consumer requires access to private Eggup modules;
- no eggsact-specific policy has leaked into Eggup;
- duplicated updater code is removed from stegoeggo;
- dependency/size impact is acceptable and recorded.

## 8. Stop conditions

If stegoeggo exposes a generic deficiency in Eggup, stop the migration and write an Eggup corrective plan before adding a local workaround that duplicates security-sensitive machinery.

## 9. Closure evidence required

- before/after code ownership;
- comparison with eggsact adapter;
- verification matrix;
- dependency/size measurement;
- genericity finding;
- recommendation on whether eggsearch/Gregg migrations may begin.
