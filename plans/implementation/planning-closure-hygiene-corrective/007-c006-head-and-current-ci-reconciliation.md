# Planning / Closure Hygiene Corrective C007 — C006 Head and Current-CI Reconciliation

Status: implemented (see `plans/closure/planning-closure-hygiene-corrective/007-status.md`)

Repository baseline: `510111fa67bafd0e188dad744eed4d224a015b76`

Primary class: polish/corrective

Related runtime corrective:

- `plans/implementation/service-lifecycle/008-operation-deadline-test-determinism-corrective.md`

Original records corrected by this pass:

- `plans/registry.md`
- `plans/closure/planning-closure-hygiene-corrective/006-status.md`
- `plans/subsystems/archive-extraction-roadmap.md`
- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`

Current evidence:

- C006 closure commit: `510111fa67bafd0e188dad744eed4d224a015b76`
- current-head CI run: `36261671380`
- run result: failed
- failed job: macOS tests
- failed test: `unix_tests::transition_deadline_rejects_zero_and_only_shrinks`
- Stable Linux: passed
- Rust 1.89 MSRV: passed
- Windows: passed
- open issues/PRs at review: none

## 1. Objective

Make the planning control surface accurately represent the repository after C006 closure and the immediately following current-head CI failure.

The current planning state contains three bookkeeping/evidence defects:

1. the registry's latest planning registration head still names pre-closure `f463aa7` instead of C006 closure head `510111f`;
2. the registry still presents prior green run `36257992801` as the relevant current-head evidence and does not disclose failed current-head run `36261671380`;
3. the C006 closure record says its implementation SHA is merely “visible in git log at commit time” instead of recording exact closure SHA `510111f`.

A fourth scheduling correction is required: M001d's contract is ready, but security-sensitive runtime implementation should not begin from a known-red repository baseline. Service M008 owns the unrelated current-head test determinism failure. M001d must therefore be marked **contract-ready / execution-blocked on Service M008 closure + green hosted qualification**, then rebased to that new green head before implementation.

C007 is documentation/planning only.

## 2. Readiness and dependencies

C007 is dependency-ready now.

It does not depend on M008 closure to record current truth. Its job is to:

- record the current failure honestly;
- register M008 as the active repo-baseline corrective;
- gate M001d execution until M008 restores green qualification;
- fix C006 closure/head bookkeeping.

When M008 later closes, its normal closure/registry update will supersede the temporary red-baseline gate and rebase M001d.

## 3. Current evidence

Current main before this planning batch:

~~~text
510111fa67bafd0e188dad744eed4d224a015b76
docs(plans): close C006, correct M001d bound-source contract
~~~

CI run `36261671380`:

| Lane | Result |
|---|---|
| Stable Linux | passed |
| Rust 1.89 MSRV | passed |
| Windows archive/acquisition/curl/service + workspace check | passed |
| macOS full workspace tests | failed |

macOS failure:

~~~text
unix_tests::transition_deadline_rejects_zero_and_only_shrinks
called Result::unwrap() on an Err value:
Manager("transition deadline exhausted before manager command")
~~~

C006 itself is docs-only; no Rust/Cargo/workflow change in C006 introduced the failure. Service M008 is the corrective for the timing-sensitive test.

## 4. Invariants that must not regress

- planning surfaces distinguish current failure from previously green evidence;
- C006 remains closed;
- M001d's corrected contract remains valid;
- M001d is not falsely described as executable from a known-red baseline;
- M008 remains scoped to test determinism unless evidence shows a runtime defect;
- Egress M006 and Eggpack M002 remain blocked on M001d closure;
- Gregg remains untouched;
- C007 changes no runtime source, Cargo file, workflow, release, package, or consumer repository.

## 5. Scope and non-scope

### In scope

- registry head/evidence correction;
- C006 closure exact-SHA correction;
- register Service M008;
- mark Service Lifecycle M008 ready;
- mark M001d contract-ready but execution-blocked on M008 green-baseline restoration;
- update archive roadmap/status language accordingly;
- update M001d header/readiness language so implementation cannot begin until M008 closes and the plan is rebased;
- write C007 closure record after those docs changes land.

### Explicitly out of scope

- implementing Service M008;
- modifying `OperationDeadline`;
- rerunning CI as the only fix;
- implementing M001d;
- Egress/Eggpack plan authoring;
- Gregg migration;
- changing historical hosted results.

## 6. Required documentation changes

### 6.1 Registry top metadata

Record:

- C006 exact closure head `510111f`;
- current-head CI run `36261671380` failed on macOS;
- Stable/MSRV/Windows passed;
- prior green runs remain historical evidence, not current-head success;
- Service M008 ready;
- M001d contract-ready but execution-blocked on M008 closure + new green matrix.

### 6.2 C006 closure exact commit

Replace vague text equivalent to:

~~~text
This closure batch (docs-only; the commit SHA is visible in git log at commit time)
~~~

with exact closure SHA:

~~~text
510111fa67bafd0e188dad744eed4d224a015b76
~~~

Do not otherwise rewrite C006 historical evidence.

### 6.3 Service roadmap

Reopen the service roadmap only for narrow corrective M008:

~~~text
M001-M007 closed
       |
       v
M008 OperationDeadline test determinism [READY]
~~~

M007 remains closed; M008 is a post-closure test/qualification corrective, not a reopening of lifecycle semantics.

### 6.4 Archive/M001d gate

Represent M001d as:

~~~text
contract corrected by C006 [READY]
execution gate: Service M008 closure + green hosted baseline + rebase
~~~

This is an operational baseline gate, not an archive dependency.

Do not change Egress/Eggpack dependency ownership: they remain blocked on M001d closure.

### 6.5 M001d baseline discipline

Until M008 closes, M001d's current baseline remains historical planning evidence only.

Before M001d runtime implementation begins:

- update its exact repository baseline to the new post-M008 green head;
- record the fresh green workflow run;
- verify no intervening runtime changes invalidate its source analysis.

M008 closure is responsible for triggering this rebase/unblock.

## 7. Ordered work packages

1. Fix registry top heads/current-CI evidence.
2. Register Service M008 in service roadmap and registry.
3. Correct C006 closure exact implementation SHA.
4. Mark M001d contract-ready/execution-blocked on M008 in archive roadmap, M001d plan status/readiness text, and registry.
5. Search for active claims that current main is green or M001d is immediately executable and correct them.
6. Write C007 closure record with zero-runtime-delta evidence.

## 8. Failure, restart, cancellation, and contention semantics

No runtime semantics.

If M008 implementation lands concurrently, refetch and reconcile to the actual new head rather than preserving the temporary red-baseline gate.

## 9. Compatibility and migration

No API/consumer behavior changes.

No consumer migration.

## 10. Required checks

- registry latest closure/planning evidence includes `510111f`;
- registry explicitly records failed run `36261671380`;
- prior `36257992801` is not labeled current-head evidence;
- C006 closure names exact SHA `510111f`;
- service roadmap has M008 ready;
- M001d is contract-ready but execution-blocked on M008;
- Egress/Eggpack remain blocked on M001d;
- no runtime/Cargo/workflow changes in C007.

## 11. Required verification commands

~~~text
git diff --check
git diff --name-only <c007-baseline>..HEAD
rg -n "510111f|36261671380|36257992801|M008|M001d|current-head|green" plans/
~~~

No Cargo run is required for C007.

## 12. Documentation updates

- `plans/registry.md`;
- `plans/subsystems/service-lifecycle-roadmap.md`;
- `plans/subsystems/archive-extraction-roadmap.md`;
- `plans/implementation/archive-extraction/001d-handle-backed-source-handoff.md`;
- `plans/closure/planning-closure-hygiene-corrective/006-status.md`;
- `plans/closure/planning-closure-hygiene-corrective/007-status.md`.

## 13. Acceptance criteria

C007 closes when:

- current planning truth names `510111f` and failed run `36261671380`;
- C006 closure exact SHA is recorded;
- Service M008 is the only new runtime corrective for the red baseline;
- M001d's contract remains ready but execution is gated on M008 closure + green qualification + rebase;
- downstream archive consumers remain blocked on M001d;
- C007 has zero runtime delta.

## 14. Stop conditions

Stop and revise the graph if:

- M008 investigation finds a real production deadline defect rather than test nondeterminism;
- another current-head CI failure appears outside the known macOS test;
- M001d implementation lands before this reconciliation;
- concurrent commits supersede `510111f`.

## 15. Closure evidence required

Record:

- C007 implementation SHA;
- exact files changed;
- exact C006 SHA correction;
- before/after registry head evidence;
- current failed CI evidence;
- M008 registration;
- M001d temporary execution gate;
- zero-runtime-delta proof.

## 16. Handoff notes

C007 exists so the planning system does not call a red branch green.

It does not downgrade the corrected M001d contract. It only prevents starting that security-sensitive runtime pass from a baseline whose current hosted macOS matrix is known to fail.

Service M008 should close first; its closure should then rebase M001d onto the fresh green head and restore M001d to executable-ready.
