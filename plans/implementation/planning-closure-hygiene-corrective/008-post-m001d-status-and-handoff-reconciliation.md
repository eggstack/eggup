# Planning / Closure Hygiene Corrective C008 — Post-M001d Status and Handoff Reconciliation

Status: implemented; closed by `plans/closure/planning-closure-hygiene-corrective/008-status.md`

Repository baseline: `863f2446a282b78f34fa86d5d7422a1f9c230862`

Primary class: polish/corrective

Source evidence:

- `plans/closure/archive-extraction/001c-status.md`
- `plans/closure/archive-extraction/001d-status.md`
- M001d implementation `09064a34896eaeb131b8b9c5ff787e9613d0e563`
- M001d Windows qualification corrective `18d83decd0c894279dd5e5d89407e1423f839d99`
- hosted qualification run `36335233644`
- current-head CI run `36335617284`

## 1. Objective

Reconcile active Eggup planning after Archive M001d closed the M001c Section 14 handoff stop.

Historical M001c closure remains truthful: M001c implemented handle-relative write authority and stopped because path-only handoff could not prove source-object binding. M001d subsequently supplied the object-bound handoff and explicitly closed that stop. Active planning must therefore stop describing M001c itself as currently blocked.

C008 is documentation/planning only.

## 2. Readiness and dependencies

C008 is dependency-ready because M001d is closed and hosted-qualified on all supported CI lanes.

No runtime work is required to resolve the inconsistency.

## 3. Current evidence and defects

Active surfaces are internally inconsistent:

- the archive roadmap status prose says the M001c stop is closed by M001d;
- the same roadmap dependency graph/table still labels M001c `BLOCKED`;
- the registry planned/blocked table still labels M001c blocked;
- current next-handoff state correctly says Egress M006 and Eggpack M002 are writable.

The correct active status is historical/continued-by-M001d with the Section 14 stop resolved.

## 4. Invariants that must not regress

- do not rewrite M001c historical closure as if it independently solved handoff authority;
- M001d remains the closure that resolves the M001c stop;
- M001b/M001c/M001d qualification SHAs and run IDs remain unchanged;
- Egress M006 and Eggpack M002 remain writable;
- no runtime, Cargo, workflow, release, or consumer change.

## 5. Scope and non-scope

### In scope

- archive roadmap M001c graph/table/status wording;
- registry M001c active/planned wording;
- registry latest planning-head bookkeeping;
- C008 closure evidence.

### Out of scope

- editing historical M001c closure findings;
- M001d runtime changes;
- authoring downstream implementation in this corrective;
- Eggpack producer changes;
- Egress migration.

## 6. Required planning changes

Use an active-state phrase equivalent to:

`historical stop resolved by M001d; write authority implemented at 09c953f`

The graph should show M001c as a completed predecessor whose unresolved handoff was continued and closed by M001d, not as an active blocker.

## 7. Ordered work packages

1. Reconcile archive roadmap status/graph/table.
2. Reconcile registry planned/blocked and execution graph wording.
3. Verify downstream writability remains unchanged.
4. Create C008 closure record and update registration head.

## 8. Failure/restart/contention semantics

None; docs-only.

Concurrent planning changes must be refetched and preserved rather than overwritten.

## 9. Compatibility and migration

No runtime/API compatibility effect.

## 10. Required checks

- no active `M001c ... blocked` claim remains except quoted historical context;
- M001c closure remains historical and unchanged;
- M001d remains closed;
- Egress M006 and Eggpack M002 remain writable;
- diff contains planning files only.

## 11. Verification commands

~~~text
git diff --check
rg -n "M001c|M001d|Egress M006|Eggpack.*M002" plans/
git diff --name-only <baseline>..HEAD
~~~

## 12. Documentation updates

- `plans/subsystems/archive-extraction-roadmap.md`
- `plans/registry.md`
- `plans/closure/planning-closure-hygiene-corrective/008-status.md`

## 13. Acceptance criteria

C008 closes when active planning consistently represents M001c as a historical predecessor whose handoff stop was resolved by M001d, with no runtime delta and no downstream gate regression.

## 14. Stop conditions

Stop if any source evidence indicates M001d did not actually close the M001c handoff invariant or current CI is no longer green.

## 15. Closure evidence required

Record exact docs commits, file list, before/after status wording, M001d closure/run evidence, and zero-runtime-delta proof.

## 16. Handoff notes

After C008 closes, newly unblocked archive consumers may be planned against M001d. Do not rewrite M001c history; reconcile only active state.
