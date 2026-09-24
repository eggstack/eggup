# Planning and Closure Hygiene Corrective C002 — C001 Commit and Registry Baseline Reconciliation

Status: closed

Repository baseline reviewed: `a88e482d84d3d8bafa429a2973baf82c8ed90597`

Source governance:

- `plans/003-planning-process.md#11-corrective-passes`
- `plans/003-planning-process.md#12-closure-records`
- `plans/003-planning-process.md#13-registry`
- `plans/003-planning-process.md#14-baseline-handling`

Predecessor:

- C001 plan: `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md`
- C001 closure: `plans/closure/planning-closure-hygiene-corrective/001-status.md`
- C001 closing commit: `47bd68255534d3be5798968a4c290c6f202b3cb7`

Primary class: polish / planning-integrity corrective

## 1. Objective

Close the final bookkeeping defect left by C001 without changing any production code or reopening its substantive evidence.

Two facts must be made explicit and internally consistent:

1. the C001 closure record must name the exact commit that landed C001, `47bd68255534d3be5798968a4c290c6f202b3cb7`, rather than describing that commit indirectly;
2. the registry header must stop presenting `2cab1f97ef30fa347c2030da321462459672c521` as the latest reviewed closure/planning baseline after C001 and later planning work have landed.

This corrective also clarifies the registry metadata so future plan-registration commits do not recreate the same self-reference ambiguity.

## 2. Readiness and dependencies

C001 is closed and its substantive evidence is accepted.

No production dependency blocks C002.

Service Lifecycle M005 is independently runtime/API-ready after Core M007. C002 is only a small planning-order cleanup and MUST NOT be represented as a runtime dependency of M005.

## 3. Current evidence and detection gap

At the reviewed baseline:

- `plans/closure/planning-closure-hygiene-corrective/001-status.md` says the Eggup corrective commit is "the single commit landing this record" but does not spell out `47bd68255534d3be5798968a4c290c6f202b3cb7`;
- `plans/registry.md` still says:
  - `Last implementation baseline reviewed: 8d5fc12f...` — correct;
  - `Planning baseline for this implementation-plan batch: 66acd739...` — historical/stale for the current plan-authoring batch;
  - `Latest closure/planning baseline reviewed: 2cab1f97...` — stale after C001 and the later Eggpack-adapter gate planning commit `a88e482d...`.

Detection gap: C001 attempted to identify the commit containing its own closure from inside that same commit. A Git commit cannot contain its own SHA without a recursive rewrite. The correct pattern is to distinguish the **implementation/corrective commit being evidenced** from the later closure/status bookkeeping commit, rather than require a closure document to self-reference the SHA of the commit that contains it.

## 4. Invariants

- No Rust or other production source changes.
- Do not alter C001's measured CodeGG size/LOC evidence.
- Do not change C001's closed disposition.
- Do not rewrite historical implementation SHAs.
- `Last implementation baseline reviewed` remains a production-code SHA and must not be replaced by a docs-only commit.
- Registry planning metadata must have stable semantics that do not require a file to know the SHA of its own commit.
- Do not introduce another requirement that a closure record contain the SHA of the same commit that creates/edits that record.
- Service Lifecycle M005 remains runtime/API-ready regardless of C002.

## 5. Scope and non-scope

### In scope

- explicit C001 closing/landing SHA in the C001 closure record;
- registry header metadata reconciliation;
- wording that distinguishes production implementation baseline, plan-authoring baseline, and latest closed-work/planning evidence baseline;
- a C002 closure record;
- registry status transition for C002.

### Out of scope

- changing C001 evidence;
- Core M007;
- CodeGG;
- service lifecycle implementation;
- Eggpack interoperability;
- package publication;
- canonical specification or ADR changes.

## 6. Required documentation changes

### A. C001 closure

Replace the indirect commit description with an explicit historical fact:

```text
C001 landing commit: 47bd68255534d3be5798968a4c290c6f202b3cb7
```

Keep the starting SHA and CodeGG evidence unchanged.

Do not claim that C001 knew this SHA before the commit existed.

### B. Registry baseline semantics

Reconcile the header so each field has one stable meaning.

Required meanings:

- **Last implementation baseline reviewed** — latest reviewed production-code implementation relevant to Eggup runtime behavior. At this point this remains Core M007 `8d5fc12f7224145285f22d0975a7bb91e1e363ea`.
- **Plan-authoring baseline for the current batch** — exact repository SHA reviewed before the current C002 + Service M005 plan-authoring batch. At plan authoring this is `a88e482d84d3d8bafa429a2973baf82c8ed90597`.
- **Latest closed-work/planning evidence baseline reviewed** — an already-existing commit that can be named without self-reference. It must be refreshed to the exact latest reviewed predecessor at C002 execution time and MUST be no older than C001's `47bd6825...`. If later planning commits such as `a88e482d...` are included in the review, record that later SHA instead.

Preferred wording is to replace the ambiguous "Latest closure/planning baseline reviewed" label with a label whose semantics make clear it identifies the latest **already-existing reviewed predecessor**, not the SHA of the commit currently being authored.

### C. C002 closure

Create:

- `plans/closure/planning-closure-hygiene-corrective/002-status.md`.

The closure must record the exact corrective implementation/docs commit that changed the C001 closure + registry metadata. It does **not** need to contain the SHA of the same closure-status commit that contains the record.

If execution uses two commits:

1. commit A performs the bookkeeping correction;
2. commit B writes/updates the C002 closure and registry status;

then the closure records commit A as the corrective implementation commit and Git history is the authority for commit B. This avoids recursive self-reference.

## 7. Ordered work packages

1. Refresh Eggup HEAD and confirm C001 still lands at `47bd6825...`.
2. Confirm no later production implementation supersedes Core M007.
3. Update the C001 closure with the exact landing SHA.
4. Reconcile registry baseline labels/values using the stable semantics above.
5. Search for stale statements that still present `2cab1f97...` as the latest planning/closure state.
6. Run docs-only verification.
7. Write C002 closure and mark C002 closed.

## 8. Failure/contention semantics

No runtime state changes.

If a later production implementation has landed before C002 executes, update the implementation-baseline field to that exact reviewed production SHA and explain the change in closure evidence.

If a later planning commit has landed, use it as the plan/evidence predecessor only after reviewing it. Do not blindly stamp current HEAD.

If the only way to satisfy a proposed field is to predict the current commit's future SHA, change the field semantics instead of attempting self-reference.

## 9. Compatibility and migration

None. Documentation/control-surface only.

## 10. Verification

At minimum:

```text
git grep -n "2cab1f97ef30fa347c2030da321462459672c521" -- plans/registry.md
git grep -n "single commit landing this record" -- plans/closure/planning-closure-hygiene-corrective/001-status.md
git grep -n "47bd68255534d3be5798968a4c290c6f202b3cb7" -- plans/closure/planning-closure-hygiene-corrective/001-status.md
cargo fmt --all -- --check
git diff --check
```

Full runtime tests are not required for Markdown-only changes; if not run, say so explicitly.

## 11. Documentation

Expected files:

- `plans/closure/planning-closure-hygiene-corrective/001-status.md`;
- `plans/registry.md`;
- this plan's status;
- new `plans/closure/planning-closure-hygiene-corrective/002-status.md`.

No canonical document or ADR change.

## 12. Acceptance criteria

C002 closes only when:

- C001 closure explicitly records `47bd68255534d3be5798968a4c290c6f202b3cb7`;
- registry production implementation baseline remains truthful;
- current plan-authoring baseline reflects the current batch rather than the old `66acd739...` batch;
- the stale `2cab1f97...` "latest" claim is removed or relabeled with stable reviewed-predecessor semantics;
- no self-referential SHA requirement remains;
- no production source changes;
- Service Lifecycle M005 remains ready.

## 13. Stop conditions

Stop and widen the corrective only if:

- a newer production implementation changes the runtime baseline;
- C001 substantive evidence is found incorrect;
- the registry has another contradictory active-state entry that cannot be corrected without reopening a milestone.

## 14. Closure evidence required

Record:

- starting repository SHA;
- exact C001 landing SHA;
- exact C002 corrective implementation/docs SHA;
- before/after registry header text;
- files changed;
- verification commands/results;
- explicit statement that no production code changed;
- unresolved findings and severity;
- Service M005 readiness disposition.

## 15. Handoff notes

C002 is intentionally tiny. It may be executed immediately before Service Lifecycle M005 implementation.

The Service M005 implementation plan may be authored and registered now because C002 cannot change its runtime/API dependencies.
