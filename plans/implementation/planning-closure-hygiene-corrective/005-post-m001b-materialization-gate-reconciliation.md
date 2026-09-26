# Planning / Closure Hygiene Corrective C005 — Post-M001b Materialization Gate Reconciliation

Status: implemented

Repository baseline: `d46d35e891bbbc72fced3213ea28fdd6b1ec5ed9`

Primary class: polish/corrective

Related runtime work:

- `plans/implementation/archive-extraction/001c-handle-relative-member-materialization-corrective.md`

Original records corrected by this pass:

- `plans/registry.md`
- `plans/subsystems/archive-extraction-roadmap.md`
- `plans/subsystems/consumer-adoption-roadmap.md`
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`
- `plans/closure/archive-extraction/001b-status.md`
- `plans/closure/acquisition-transport/008-status.md`

## 1. Objective

Reconcile the active planning/closure surfaces after the post-M001b review found a separate archive member-materialization authority gap.

The runtime finding is not a failure of M001b's handle-bound cleanup implementation. It is a distinct post-closure issue: member creation still uses the recorded root pathname, while cleanup uses the retained root handle. The planning system must therefore:

- preserve M001b's cleanup closure honestly;
- register Archive M001c as the active corrective;
- re-block Egress M006 and Eggpack Interoperability M002 until M001c closes;
- remove stale top-level registry metadata that still says the M001b closure was pending;
- reconcile the M008 closure narrative so its executive text no longer contradicts the already-recorded hosted qualification supplement.

No production code changes belong in C005.

## 2. Readiness and dependencies

C005 is dependency-ready.

Evidence is already established:

- M001b implementation `057399640b03bb0fcee1ea86fdc5707f812e3579` is green on hosted run `36222536670`;
- documentation/closure head `d46d35e891bbbc72fced3213ea28fdd6b1ec5ed9` is green on CI run `36222870057`;
- M001c is registered as the runtime corrective;
- current registry metadata still names `0573996` as the latest planning registration head with “closure pending hosted matrix,” despite closure already landing at `d46d35e`;
- current M008 closure begins with a qualification supplement saying M008 is closed, but the later Executive finding still says hosted qualification is pending;
- consumer/Eggpack roadmaps currently mark their archive handoffs ready based on M001b, which is no longer sufficient after the M001c finding.

This is documentation/evidence reconciliation only.

## 3. Current evidence

Current truthful runtime state:

- Verified core remains closed/qualified.
- Service remains closed/qualified.
- Acquisition M008 is closed and hosted-qualified.
- Archive M001b cleanup authority is closed and hosted-qualified.
- Archive M001c is the active materialization-authority corrective.
- Egress M006 must wait for M001c.
- Eggpack archive M002 must wait for M001c.
- Gregg remains separately writable/unwritten and untouched.
- Eggsact manifest adoption M003 remains blocked on producer-owned publication convention.

Current planning defects:

1. registry top metadata lags the actual documentation/closure head;
2. registry/roadmaps say Egress/Eggpack are ready despite the newly identified M001c hard dependency;
3. M001b closure lacks a concise post-closure addendum distinguishing its valid cleanup closure from the new materialization finding;
4. M008 closure contains both “fully closure-qualified” and “qualification pending” statements.

## 4. Invariants that must not regress

- historical closure evidence remains historical; do not rewrite M001b as if its cleanup implementation failed;
- M001c is represented as a new post-closure corrective, not silently folded into M001b;
- downstream archive consumers remain blocked until M001c closes;
- M008 remains closed; C005 only removes contradictory stale prose;
- C005 MUST NOT change runtime source, Cargo manifests, Cargo.lock, workflow logic, consumer repos, release artifacts, or package versions;
- Gregg migration remains unauthorized;
- Eggpack/Eggup ownership split remains unchanged;
- registry remains compact and points to detailed plans instead of duplicating them.

## 5. Scope and non-scope

### In scope

- registry baseline/head/status reconciliation;
- archive roadmap M001c registration/status;
- consumer adoption M006 re-gating;
- Eggpack interoperability M002 re-gating;
- M001b closure post-closure finding addendum;
- M008 closure executive/narrative consistency;
- C005 closure record.

### Explicitly out of scope

- implementing M001c;
- changing archive code;
- changing M001b runtime behavior;
- authoring Egress M006;
- authoring Eggpack M002;
- touching Gregg;
- release/publication changes;
- changing canonical ADRs/specification unless a new contradiction is separately identified.

## 6. Required documentation changes

### 6.1 Registry

Update top metadata to record:

- latest reviewed runtime/documentation head `d46d35e891bbbc72fced3213ea28fdd6b1ec5ed9` before M001c registration;
- successful current-head CI run `36222870057`;
- M001c as the active dependency-ready archive corrective;
- M001b cleanup closure retained;
- M008 closed;
- Egress M006 / Eggpack M002 blocked on M001c.

After registration commits land, record the exact M001c/C005 planning registration head without pretending it is a runtime implementation head.

### 6.2 Archive roadmap

Represent:

~~~text
M001 -> M001a historical -> M001b cleanup [CLOSED]
                                  |
                                  v
                         M001c materialization [READY]
                                  |
                      +-----------+-----------+
                      v                       v
               Egress M006              Eggpack M002
                 [BLOCKED]                 [BLOCKED]
~~~

The roadmap must distinguish “cleanup authority closed” from “full archive authority not yet ready for consumers.”

### 6.3 Consumer roadmap

Change Egress M006 from ready-to-author to blocked on M001c closure.

Do not author the Egress implementation plan as part of C005.

### 6.4 Eggpack interoperability roadmap

Change archive handoff M002 from ready-to-author to blocked on M001c closure.

Do not modify M003's independent producer-convention blocker.

### 6.5 M001b closure addendum

Add a post-closure finding that states:

- M001b's recursive cleanup fix remains valid and closed;
- later review found member materialization still pathname-authorized;
- M001c tracks that separate invariant;
- downstream readiness is therefore re-gated.

Do not alter the original M001b requirement/evidence matrix to claim tests failed when they did not.

### 6.6 M008 closure narrative

Replace the stale Executive-finding/pending-qualification wording with a single truthful statement consistent with the qualification supplement and hosted run `36222536670`.

Preserve the historical failed run `36220815378` as context if useful, but it must not read as the current state.

## 7. Ordered work packages

1. Update archive roadmap with M001c and downstream blocks.
2. Update consumer adoption M006 gate.
3. Update Eggpack M002 gate.
4. Reconcile registry top metadata, active subsystem table, dependency-ready table, execution graph, current project state, and next handoff.
5. Add M001b closure post-closure finding/addendum.
6. Reconcile M008 closure executive text with its already-present hosted qualification supplement.
7. Search all planning/closure surfaces for stale “M001b unblocks/ready” statements and correct active contradictions.
8. Run documentation-only diff validation.
9. Write C005 closure record with exact changed files and explicit zero-runtime-delta statement.

## 8. Failure, restart, and contention semantics

C005 has no runtime failure/restart semantics.

If a status cannot be reconciled without deciding a runtime/API question, stop and leave it blocked rather than inventing closure evidence.

If concurrent commits move the planning baseline, refetch and reconcile rather than overwriting newer status text.

## 9. Compatibility and migration

No API, dependency, binary, package, or consumer behavior changes.

The only operational effect is planning: Egress M006 and Eggpack M002 become blocked again until M001c closes.

## 10. Required tests / checks

At minimum:

- no active registry row says Egress M006 is ready while M001c is open;
- no active registry row says Eggpack M002 is ready while M001c is open;
- archive roadmap contains M001c and correct graph/table status;
- consumer roadmap M006 blocker is M001c;
- Eggpack roadmap M002 blocker is M001c;
- M001b closure says cleanup remains closed but M001c is open;
- M008 closure has no current-state contradiction about hosted qualification;
- no source/Cargo/workflow file changed in C005.

## 11. Required verification commands

Documentation-only verification:

~~~text
git diff --check
git diff --name-only <baseline>..HEAD
rg -n "M001c|M001b|Egress M006|Eggpack.*M002|qualification pending|ready to author" plans/
~~~

If the implementation agent has a local checkout, it MAY run `cargo check --workspace --all-targets --locked` as a sanity check, but C005 does not require a new runtime matrix because it changes no runtime code.

## 12. Documentation updates

C005 itself updates only planning/closure documents:

- `plans/registry.md`;
- archive roadmap;
- consumer adoption roadmap;
- Eggpack interoperability roadmap;
- M001b closure record;
- M008 closure record;
- C005 closure record.

No README/architecture/changelog update is needed unless a current statement directly contradicts the M001c gate.

## 13. Acceptance criteria

C005 closes when:

- M001c is visible as the sole active archive runtime corrective;
- M001b remains accurately closed for cleanup authority;
- M008 is consistently closed everywhere;
- Egress M006 and Eggpack M002 are blocked on M001c;
- registry top metadata reflects the actual latest reviewed/registration heads;
- no active planning surface claims archive consumer readiness while M001c is open;
- the C005 diff contains no runtime source, Cargo, workflow, consumer, or release changes.

## 14. Stop conditions

Stop and write a different planning/ADR change if:

- M001c implementation lands before C005 and changes the dependency graph;
- a newly discovered runtime issue changes M008 or M001b closure status materially;
- reconciliation requires changing canonical architecture ownership;
- concurrent planning changes make the current baseline stale.

## 15. Closure evidence required

Record:

- C005 implementation SHA;
- exact files changed;
- before/after status summary;
- registry registration head;
- proof that Egress/Eggpack are blocked;
- proof that M008 closure narrative is internally consistent;
- explicit statement that runtime source/Cargo/workflow/package state did not change.

## 16. Handoff notes

C005 is intentionally documentation-only.

Do not use it to hide or downgrade M001c. The purpose is to make the control surfaces accurately represent the runtime finding while preserving valid historical closure evidence.

M001c and C005 may proceed independently after registration, but Egress M006 and Eggpack M002 remain blocked until M001c itself closes with hosted qualification.
