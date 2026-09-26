# Planning and Closure Hygiene Corrective C004 — Post-M007/M001 Status and Baseline Reconciliation

Status: closed

Repository baseline: `ea51fe12a7c9120028b727eb5e40411e9b10f8e2`

Source governance:

- `plans/003-planning-process.md#11-corrective-passes`
- `plans/003-planning-process.md#12-closure-records`
- `plans/003-planning-process.md#13-registry`
- `plans/003-planning-process.md#14-baseline-handling`

Related evidence:

- `plans/closure/acquisition-transport/007-status.md`
- `plans/closure/archive-extraction/001-status.md`
- `plans/closure/service-lifecycle/007-status.md`
- acquisition M008 plan;
- archive M001a plan.

Primary class: polish / planning-integrity corrective

## 1. Objective

Reconcile the active planning control surface after the Acquisition M007, Archive M001, and Service M007 implementation wave, while registering the newly discovered Acquisition M008 and Archive M001a safety correctives.

The runtime implementation state is ahead of several roadmap/registry labels. This docs-only pass must make the active planning surfaces agree on:

- latest implementation/closure baseline;
- Acquisition M007 conditional closure and M008 readiness;
- Service M007 closure;
- Archive M001 closure followed by M001a cleanup-authority corrective;
- downstream Egress M006 and Eggpack interoperability M002 returning to blocked-on-corrective status until M001a closes.

## 2. Why this milestone is ready

All facts required for reconciliation are already present in the repository.

Concrete stale/contradictory state includes:

- registry top implementation baseline still points to Acquisition M007 implementation even though Archive M001 and Service M007 landed later;
- acquisition roadmap top/status table says M007 conditionally closed, but its dependency graph and milestone body still say `[READY]` / `Status: ready`;
- service roadmap M007 body contains both `Status: ready` and `Status: closed`, and its graph still labels M007 `[READY]`;
- archive/consumer/Eggpack roadmaps currently say Egress M006 and Eggpack M002 are ready to author, but M001a now represents an unclosed extraction cleanup invariant that those consumers would inherit.

No runtime source edit is required for C004.

## 3. Current implementation evidence

Current main head at plan authoring: `ea51fe12a7c9120028b727eb5e40411e9b10f8e2`.

The implementation wave after planning head `b619fbd...` includes:

- Acquisition M007 production/CI/closure work through `e7a8570...` / closure commit `94d855b...`;
- Archive M001 implementation `230f768...` and closure `ef47bb6...`;
- Service M007 implementation `adb7c09...`, CI follow-ups `f87b040...` / `5138f12...`, closure `a152575...`, and retry-evidence docs `ea51fe1...`.

The registry correctly captures many detailed states, but its top baseline and several roadmap status labels lag those commits.

## 4. Invariants that must not regress

- historical closure records remain historical evidence and are not rewritten to hide conditional/platform limits;
- Acquisition M007 remains conditionally closed, not fully closed;
- Service M007 remains closed;
- Archive M001 remains historically closed but M001a is the active corrective on its cleanup invariant;
- Egress M006 and Eggpack M002 do not implement against an open medium safety corrective;
- Gregg remains untouched and M004 remains intentionally unwritten;
- Eggsact manifest M003 remains blocked on producer convention;
- no runtime source, Cargo manifest, lockfile, workflow, consumer repo, release, or package changes occur in C004.

## 5. Scope

### In scope

- reconcile registry top baseline metadata;
- register Acquisition M008;
- register Archive M001a;
- register C004 itself;
- correct acquisition roadmap graph/body status labels;
- correct service roadmap graph/body status labels;
- update archive roadmap to show M001a ready and M002/M003 blocked on M001a;
- update consumer adoption M006 blocker from "ready to author" to M001a closure;
- update Eggpack interoperability M002 blocker similarly;
- update immediate execution graph and next-handoff text;
- create C004 closure record after execution.

### Explicitly out of scope

- implementing M008;
- implementing M001a;
- authoring Egress M006;
- authoring Eggpack M002;
- changing runtime CI;
- changing the Windows curl loopback disposition;
- changing service timing tests;
- changing producer conventions;
- Gregg migration.

## 6. Required production changes

None. This is documentation/planning reconciliation only.

Required planning edits:

1. Registry baseline metadata must identify `ea51fe12a7c9120028b727eb5e40411e9b10f8e2` or the exact newer pre-C004 head as the latest reviewed implementation/closure state.
2. Acquisition roadmap must show M007 conditionally closed and M008 ready in every active status representation.
3. Service roadmap must show M007 closed exactly once and remove stale `READY` labels.
4. Archive roadmap must add M001a as ready and gate downstream archive integrations on its closure.
5. Consumer adoption M006 and Eggpack interoperability M002 must return to blocked status until M001a closure.
6. Registry must distinguish:
   - M008 ready;
   - M001a ready;
   - C004 ready/active;
   - Egress/Eggpack archive integrations blocked on M001a.
7. Historical statements in prior plans/closures need not be rewritten unless they appear in the active control surface as current state.

## 7. Ordered work packages

1. Refresh main and record exact pre-C004 baseline.
2. Re-read registry plus acquisition/service/archive/consumer/Eggpack roadmaps.
3. Reconcile acquisition M007/M008 status.
4. Reconcile service M007 status.
5. Register archive M001a and re-gate downstream archive integrations.
6. Update registry baseline, dependency-ready table, planned/blocked table, graph, project-state, and next-handoff sections.
7. Create C004 closure record.
8. Mark C004 closed after all active surfaces agree.
9. Run docs/planning consistency searches and `git diff --check`.

## 8. Failure, cancellation, restart, and contention semantics

Not applicable to runtime behavior.

The planning equivalent is fail-closed: if status evidence conflicts, preserve the more conservative blocked/conditional state until the contradiction is resolved. Do not promote a milestone based on inference.

## 9. Compatibility and migration

No code/API migration.

This pass changes planning status only:

- Egress M006: ready-to-author -> blocked on Archive M001a;
- Eggpack M002: ready-to-author -> blocked on Archive M001a;
- Acquisition M008: newly ready;
- Archive M001a: newly ready.

These transitions are required to avoid downstream implementation against known-open invariants.

## 10. Required tests

Planning consistency checks must confirm no active contradictory forms remain, including:

~~~text
acquisition M007 ... [READY]
M007 Status: ready
service M007 ... [READY]
service M007 Status: ready
Egress M006 ... ready to author
Eggpack M002 ... ready to author
~~~

except when clearly quoted as historical defect evidence inside C004 itself.

Confirm active surfaces contain:

- Acquisition M007 conditionally closed;
- Acquisition M008 ready;
- Service M007 closed;
- Archive M001 closed historically;
- Archive M001a ready;
- Egress M006 blocked on M001a;
- Eggpack M002 blocked on M001a;
- current implementation/closure baseline.

## 11. Required verification commands

~~~text
git diff --check
cargo fmt --all -- --check
~~~

No runtime requalification is required solely for Markdown edits. If hosted CI runs on the documentation commit, record it truthfully but do not make it a prerequisite for closing a docs-only reconciliation unless repository policy requires it.

## 12. Documentation updates

C004 itself updates only planning and closure documentation:

- `plans/registry.md`;
- acquisition roadmap;
- service roadmap;
- archive roadmap;
- consumer adoption roadmap;
- Eggpack interoperability roadmap;
- C004 closure record.

Do not edit architecture/runtime docs in this pass.

## 13. Acceptance criteria

C004 closes only when:

- registry baseline metadata reflects the latest implementation/closure wave;
- Acquisition M007 has no active `READY` contradiction;
- Acquisition M008 is registered ready;
- Service M007 has no duplicate/stale ready state;
- Archive M001a is registered ready;
- Egress M006 and Eggpack M002 are blocked on M001a;
- immediate execution graph and next handoff agree with the tables;
- no runtime files changed;
- no medium-or-higher planning/status inconsistency remains.

## 14. Stop conditions

Stop and split the work if reconciliation discovers:

- a runtime implementation after `ea51fe12a7c9120028b727eb5e40411e9b10f8e2` that changes the technical findings;
- M007/M001/M007 closure evidence that materially contradicts the current statuses;
- an already-landed Egress or Eggpack integration;
- a new medium runtime defect outside M008/M001a scope.

## 15. Closure evidence required

Record:

- starting and closing SHAs;
- exact planning files changed;
- stale status phrases removed;
- baseline metadata before/after;
- dependency transitions before/after;
- confirmation of zero runtime-source changes;
- `git diff --check`;
- `cargo fmt --all -- --check`;
- unresolved findings by severity.

## 16. Handoff notes

C004 is the bookkeeping companion to the two runtime correctives, not a substitute for them.

After C004 closes, the implementation handoff order is:

1. Archive M001a and Acquisition M008 may proceed independently.
2. Egress M006 and Eggpack interoperability M002 remain blocked until Archive M001a closes.
3. Once M001a closes, re-author/re-enable those archive integrations against the corrected cleanup authority contract.
