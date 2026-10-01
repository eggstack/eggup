# Planning / Closure Hygiene Corrective C010 — Post-Egress and M003 Readiness Reconciliation

Status: ready for handoff

Repository baseline: `3fd4c433126d144d73bf7dcbb5c6e3de0be99717`

Affected records:

- `plans/registry.md`
- `plans/subsystems/archive-extraction-roadmap.md`
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`
- `plans/subsystems/consumer-adoption-roadmap.md` (verification-only unless drift is found)
- `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`
- `plans/closure/consumer-adoption/006-status.md` (historical evidence; amend only if an active-status addendum is required)
- `plans/closure/eggpack-manifest-interoperability/003-status.md` (historical execution record; preserve stop history)

Primary class: polish/corrective

## 1. Objective

Reconcile active Eggup planning after two downstream gates changed state:

1. Verified Update Core M009 published `eggup-core 0.1.2` and `eggup-archive 0.1.2` on 2026-09-28, and Egress Delivery M003 subsequently landed with hosted Linux/macOS/Windows updater evidence on 2026-09-29, closing Consumer Adoption M006.
2. Eggpack Ecosystem M001 / Eggsact Distribution M005 established the real `v1.2.7` producer contract and published `release-manifest.json`, satisfying the producer-owned gate that previously stopped Eggpack Interoperability M003.

The corrective is docs-only. It must make every active status surface agree that Egress M006 is closed and Eggpack Interop M003 is ready to resume, while preserving historical blocked/stop evidence as historical evidence.

## 2. Why this corrective is ready

The repository already contains authoritative current-state evidence:

- `plans/registry.md` records the 0.1.2 publication and Egress M006 closure;
- `plans/subsystems/consumer-adoption-roadmap.md` records M006 closed 2026-09-29;
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md` records the M003 producer gate as satisfied and the real consumer path as ready to resume;
- `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md` has been re-armed against the producer evidence;
- commit `3fd4c433126d144d73bf7dcbb5c6e3de0be99717` reconciled the registry's primary M003 readiness surfaces.

Remaining drift is duplicated status text, not unresolved runtime behavior.

## 3. Current planning defects

### 3.1 Archive roadmap still reports Egress as blocked

`plans/subsystems/archive-extraction-roadmap.md` still states or implies all of the following:

- top-level status: M002 Egress adoption is blocked;
- dependency graph: Consumer Adoption M006 is blocked;
- M001d text: downstream Egress remains blocked;
- M002 milestone text: 0.1.2 publication still requires maintainer action;
- milestone table: 0.1.2 is unpublished and M002 is blocked;
- completion definition treats future Egress migration as outstanding.

Those claims were superseded by M009 publication and Egress Delivery M003.

### 3.2 Registry contains an old producer-gate sentence

The registry's canonical current-state tables and next-handoff section correctly mark Interop M003 ready to resume, but the producer/consumer ownership guard still says real Eggsact adoption is blocked on producer-owned live artifact mapping and ReleaseManifest publication/addressing.

That sentence must be reconciled with the established `v1.2.7` producer evidence.

### 3.3 Immediate execution graph retains historical M003 blocker wording

The registry's large execution graph still contains the old:

`M003 real-consumer adoption [BLOCKED: producer artifact + manifest convention]`

edge even though later current-state text says the producer gate is satisfied.

Historical stop evidence belongs in M003's closure/execution record; the active execution graph must show the current dependency state.

### 3.4 The active M003 plan retains an obsolete archive-handoff blocker

`plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md` currently ends its closure-evidence section by saying Eggpack Interop M002/archive extraction remains blocked on the Phase 10 extraction contract. That dependency has since closed through Archive M001d, Interop M002/M002a, Core M009 publication, and Egress M006. Reconcile that sentence so the M003 handoff does not reintroduce an already-closed blocker.

## 4. Invariants that must not regress

- Do not rewrite history: the earlier M003 stop was correct at the time and remains valid evidence.
- Do not erase C009's earlier finding that Egress had an updater surface before publication.
- Do not claim crates other than the documented 0.1.2 publication set were published.
- Do not claim `eggup-eggpack` is published; it remains a leaf adapter under the existing promotion gate.
- Do not authorize automatic crates.io publication.
- Do not move producer release policy, artifact naming, bootstrap generation, release CI, or publication into Eggup.
- Keep `eggup-core` independent of Eggpack.
- Do not authorize Gregg M004 or EggPool M007.
- Do not convert integrity evidence into an authenticity/signature claim.
- No runtime source, dependency, Cargo metadata, or API change is permitted in C010.

## 5. Scope

### In scope

- reconcile active roadmap/status prose for Egress M006;
- reconcile active registry prose/graph for Eggpack Interop M003;
- ensure subsystem status tables, dependency graphs, completion definitions, and next-handoff text agree;
- distinguish historical blocked execution records from current active state;
- refresh planning-registration baseline after the docs-only pass;
- create a C010 closure record with exact changed files and zero-runtime-delta evidence.

### Explicitly out of scope

- implementing Interop M003;
- modifying Eggsact or Eggpack;
- publishing `eggup-eggpack` or `eggpack-manifest`;
- authoring Interop M004;
- modifying Egress;
- reopening archive extraction runtime work;
- broad prose cleanup unrelated to the two status transitions.

## 6. Required documentation changes

### 6.1 Archive extraction roadmap

Update the active status surface so:

- M002 Egress adoption is closed;
- the dependency graph points from M001d to a closed Egress adoption edge;
- M001d no longer names Egress as blocked;
- M002 records the 0.1.2 publication and Egress Delivery M003 landing;
- the milestone table points to the M006 closure/addendum and has no publication blocker;
- the completion definition records the Egress adoption criterion as satisfied.

Preserve historical C009 wording only where explicitly described as historical.

### 6.2 Registry

Update:

- producer/consumer ownership guard: M003 producer evidence exists; consumer integration remains unimplemented;
- immediate execution graph: M003 real-consumer adoption is ready after baseline refresh rather than producer-blocked;
- any remaining duplicated active prose that calls Egress M006 merely executable/blocked instead of closed;
- latest planning/closure baseline after C010 closure.

Do not duplicate detailed M003 implementation requirements into the registry.

### 6.3 Eggpack interoperability roadmap

Verify, rather than mechanically rewrite, that:

- M003 producer gate is recorded satisfied;
- M003 remains open/ready, not falsely closed;
- M004 remains blocked on M003 real-consumer closure and a publishable upstream package/API state.

Change only concrete stale text found during execution.

### 6.4 Consumer adoption roadmap

Verify that M006 remains closed and Gregg/EggPool states remain unchanged. Change only concrete contradictory text.

### 6.5 Active M003 implementation plan

Correct only stale dependency/status prose discovered during reconciliation. In particular, remove the obsolete statement that Interop M002/archive extraction remains blocked. Preserve the M003 execution scope, acceptance matrix, stop conditions, producer/consumer boundary, and historical producer-gate record.

## 7. Ordered work packages

1. Re-read the current Eggup head and capture exact active contradictory statements.
2. Reconfirm M009 publication and M006 closure evidence already recorded in-repo.
3. Reconfirm M003 producer-gate resolution from the current interoperability plan/roadmap/status record.
4. Correct the archive extraction roadmap.
5. Correct the registry ownership guard and immediate execution graph.
6. Correct stale dependency prose in the active M003 implementation plan without changing its implementation contract.
7. Search all active planning surfaces for stale Egress publication-block and M003 producer-block language.
8. Amend only active status surfaces; retain historical execution/closure evidence.
9. Run planning consistency checks.
10. Write `plans/closure/planning-closure-hygiene-corrective/010-status.md`.
11. Update registry registration/closure head to the actual docs commit.

## 8. Failure, cancellation, restart, and contention semantics

No runtime semantics.

If current external producer/consumer evidence has changed since `3fd4c433`, stop and refresh the factual state rather than reconciling docs to obsolete evidence.

If an apparent contradiction exists only inside a clearly dated historical execution record, preserve it and add a current-state addendum only when needed to prevent ambiguity.

## 9. Compatibility and migration

No API, package, dependency, release, consumer, or migration behavior changes.

The expected diff is Markdown-only.

## 10. Required verification

At minimum verify:

- no active archive-roadmap status says Egress M006/M002 is blocked on 0.1.2 publication;
- no active registry status says M003 is blocked on absent producer manifest/artifact conventions;
- active M003 surfaces say ready to resume, not closed;
- M004 remains blocked on real M003 closure plus publishable upstream state;
- Egress M006 remains closed with the September 29 delivery evidence;
- `eggup-eggpack` remains unpublished;
- historical stop/blocked records remain truthful.

## 11. Verification commands

~~~text
git diff --check
rg -n "M002 Egress|M006 Egress|0\.1\.2|unpublished|publication|M003|producer.*gate|manifest convention|blocked|ready to resume" plans/
git diff --name-only <baseline>..HEAD
git diff -- crates/ Cargo.toml Cargo.lock
~~~

The final command must be empty for C010.

## 12. Documentation updates

Required:

- `plans/subsystems/archive-extraction-roadmap.md`;
- `plans/registry.md`;
- `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`;
- `plans/closure/planning-closure-hygiene-corrective/010-status.md`.

Conditional only if concrete drift is found:

- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`;
- `plans/subsystems/consumer-adoption-roadmap.md`;
- current-state addenda in historical closure records.

## 13. Acceptance criteria

C010 closes when a reader can determine from every active planning surface, without resolving contradictory duplicated prose, that:

1. archive extraction M001d and Eggpack archive handoff are closed;
2. Egress archive/pair adoption is closed after the published 0.1.2 pair and Delivery M003 landing;
3. Eggpack Interop M003's producer gate is satisfied;
4. M003 real Eggsact runtime adoption is the current interoperability handoff;
5. M004 package/API promotion remains downstream of M003;
6. there is no runtime delta.

## 14. Stop conditions

Stop and re-review if:

- current Egress evidence no longer supports the recorded M006 closure;
- the Eggsact producer contract no longer publishes the established manifest convention;
- M003 has already landed by the time this corrective executes;
- cleanup would require changing runtime code or producer policy rather than status documentation.

## 15. Closure evidence required

Record:

- pre-cleanup Eggup SHA;
- docs implementation SHA;
- final registration SHA;
- exact changed-file list;
- stale claims removed/reclassified;
- current M006 and M003 evidence references;
- `git diff --check` result;
- proof that no runtime/Cargo files changed.

## 16. Handoff notes

C010 is deliberately a planning cleanup preceding M003 implementation. It should not grow into M003 work.

After C010 closes, the next implementation handoff remains `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`. M003 execution must refresh current Eggup/Eggsact/Eggpack baselines before editing consumer code and must preserve the producer/consumer ownership boundary.
