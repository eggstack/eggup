# Planning and Closure Hygiene Corrective C003 — M003 Registry and Closure Reconciliation

Status: planned / dependency-ready

Eggup repository baseline reviewed: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b`

Related implementation/status evidence:

- Eggpack Manifest Interoperability M003 plan: `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`
- bounded Eggup adapter/API implementation: `cbfa8fa3870dae22d16775408a3135ea682c9365`
- bounded M003 execution/status record: `plans/closure/eggpack-manifest-interoperability/003-status.md`
- M003 planning/status commit: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b`
- hosted Eggup CI for current HEAD: run `36037573793`, all configured lanes passed
- prior planning-hygiene closures:
  - `plans/closure/planning-closure-hygiene-corrective/001-status.md`
  - `plans/closure/planning-closure-hygiene-corrective/002-status.md`

Source governance:

- `plans/003-planning-process.md#11-corrective-passes`
- `plans/003-planning-process.md#12-closure-records`
- `plans/003-planning-process.md#13-registry`
- `plans/003-planning-process.md#14-baseline-handling`

Primary class: polish / planning-integrity corrective

## 1. Objective

Reconcile the compact planning registry and M003 execution-status evidence after the bounded Eggpack Manifest Interoperability M003 pass.

M003 behaved correctly: Eggup implemented the bounded JSON parse/project helper, qualified it, inspected the intended Eggsact producer path, and then stopped at the plan's producer-evidence gate rather than inventing release naming or publication policy. The detailed M003 roadmap and execution record describe that state accurately.

The compact registry still contains several pre-M003 or pre-later-milestone summaries. This corrective makes the control surface internally consistent without changing runtime behavior or changing M003's blocked disposition.

The pass closes these concrete findings:

1. `plans/registry.md` still lists `19935ec3610a5238af33a9d4f05a14925ceac25c` as the last reviewed implementation baseline even though production adapter work landed at `cbfa8fa3870dae22d16775408a3135ea682c9365`.
2. The registry still records pre-execution M003 metadata such as "Real-consumer M003 plan authoring is now dependency-ready" even though the plan has been partially executed and is now blocked on producer-owned Eggsact artifact/manifest conventions.
3. The registry's "Recently closed foundation" table still says Verified Update Core M001-M006 and Service Lifecycle M001-M004 even though Core M007 and Service M005 are closed.
4. The current M003 execution-status record contains strong local verification but does not yet record the hosted current-HEAD CI run `36037573793` that passed Stable checks, Rust 1.89, macOS tests, and Windows check.
5. Registry top metadata distinguishes neither the latest production implementation baseline nor the latest planning/status baseline after the bounded M003 pass clearly enough for the next handoff.

This is documentation/evidence reconciliation only. It must not convert M003 to closed, remove the producer blocker, authorize M004, or modify any Rust production source.

## 2. Readiness and dependencies

This corrective is dependency-ready.

Required facts are already established:

- Eggup adapter M001a is closed;
- M003 bounded adapter/API implementation exists at `cbfa8fa3870dae22d16775408a3135ea682c9365`;
- the bounded execution record exists and explicitly marks real-consumer M003 blocked;
- current HEAD `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` has hosted CI run `36037573793` with all four configured jobs passing;
- Core M007 is closed;
- Service M005 is closed;
- no M003 consumer implementation was made in Eggsact, so no consumer closure evidence must be synthesized.

No Eggpack or Eggsact production dependency is required to execute this documentation pass.

## 3. Findings to reconcile

### 3.1 Implementation-baseline drift

Current registry top metadata still names:

~~~text
Last implementation baseline reviewed:
19935ec3610a5238af33a9d4f05a14925ceac25c
~~~

That SHA is the M001a qualification corrective and had no production-code delta.

The newer Eggup production implementation relevant to the active interoperability work is:

~~~text
cbfa8fa3870dae22d16775408a3135ea682c9365
feat(eggpack): add bounded manifest JSON projection
~~~

The current planning/status baseline is:

~~~text
da1b4a8048bf863e6a653c25f1ba56bc42f4531b
planning: record M003 producer gate blocker
~~~

The registry must distinguish those two facts rather than leave the older M001a baseline as the apparent current implementation state.

### 3.2 Stale M003 readiness wording

The registry producer/consumer ownership summary still says:

~~~text
Real-consumer M003 plan authoring is now dependency-ready.
~~~

That was true before plan execution. It is no longer the current state.

The truthful current state is:

- M003 plan exists;
- the bounded Eggup adapter/API portion was implemented and qualified;
- the real Eggsact updater adoption did not start because the producer gate failed;
- M003 remains blocked until Eggpack/Eggsact establish:
  - a producer-owned Eggsact contract matching live artifact names;
  - a ReleaseManifest publication/addressing convention.

The registry must use that state consistently.

### 3.3 Foundation summary lag

The "Recently closed foundation" table currently understates completed work:

- Verified Update Core is shown as M001-M006, but M007 is closed/qualified.
- Service Lifecycle is shown as M001-M004, but M005 is closed.

These are summary defects only; detailed subsystem sections already record the later closures.

### 3.4 M003 hosted-CI evidence omission

The M003 status record truthfully reports local verification and says Linux/Windows native or hosted Eggsact evidence was not run. That statement concerns the consumer lane and must remain unchanged.

Separately, the Eggup repository itself now has hosted CI at current HEAD:

~~~text
run 36037573793
head da1b4a8048bf863e6a653c25f1ba56bc42f4531b
Stable checks  success
MSRV check     success
macOS tests    success
Windows check  success
~~~

The status record should add this as Eggup repository qualification evidence without implying any hosted Eggsact consumer qualification occurred.

### 3.5 Handoff state

The next interoperability dependency is producer-side, not additional Eggup implementation.

After this corrective closes, the registry should make the handoff explicit:

~~~text
Eggpack / Eggsact producer evidence
        |
        +--> live artifact mapping authority
        +--> manifest publication/addressing convention
        |
        v
resume blocked M003 Eggsact updater adoption
        |
        v
M004 package/API promotion only after M003 closure + publishable manifest crate
~~~

Archive M002 remains independently blocked on the Phase 10 extraction contract.

## 4. Invariants

- M003 remains blocked and is not reclassified as closed.
- The M003 execution-status record remains an execution-status/partial-closure record, not a completed milestone closure.
- Historical SHAs and historical statements remain truthful.
- Do not rewrite the M003 record to imply consumer tests, dependency alignment, release binary-size measurement, or hosted Eggsact checks occurred.
- Hosted Eggup CI and hosted Eggsact qualification are distinct evidence categories.
- No Rust source, Cargo manifest, lockfile, workflow, release, tag, package, or consumer repository change is authorized.
- No Eggpack producer convention may be invented during this pass.
- ADR-0004 remains authoritative: Eggpack owns producer release evidence, Eggup owns local deployment mechanics, applications own release/install policy.
- M004 remains blocked on real M003 closure plus a publishable `eggpack-manifest`.
- M002 archive extraction remains independently blocked.
- Core M007 and Service M005 closure records remain historical authority; this pass only corrects summary references to them.

## 5. Scope

### In scope

- update `plans/registry.md` top baseline metadata;
- replace stale M003 plan-authoring readiness language with the current blocked-after-bounded-qualification state;
- correct the "Recently closed foundation" table to Core M001-M007 and Service M001-M005;
- sweep the active registry for other wording that incorrectly represents M003 as merely ready/unstarted;
- add hosted Eggup CI run `36037573793` to `plans/closure/eggpack-manifest-interoperability/003-status.md`;
- explicitly distinguish that hosted Eggup CI from unrun hosted Eggsact consumer qualification;
- create this corrective's closure record;
- mark C003 closed in the registry after evidence is recorded;
- preserve the producer-side next handoff.

### Explicitly out of scope

- any Rust implementation;
- changes to `eggup-eggpack` public API;
- Eggsact updater changes;
- Eggpack contract/manifest production changes;
- choosing a manifest filename;
- changing Eggsact release asset naming;
- archive extraction work;
- Gregg footprint work;
- authenticity/signature design;
- publishing `eggup-eggpack`, `eggpack-manifest`, or Eggup 0.1.1;
- M004 plan authoring;
- rewriting prior closure evidence to appear newer than it was.

## 6. Required changes

### A. Registry top metadata

Update `plans/registry.md` to record at minimum:

~~~text
Last implementation baseline reviewed:
cbfa8fa3870dae22d16775408a3135ea682c9365
(Eggpack manifest adapter bounded JSON projection)

Latest planning/status baseline reviewed:
da1b4a8048bf863e6a653c25f1ba56bc42f4531b
(M003 producer-gate execution/status record)
~~~

The exact wording may differ, but a production implementation SHA must not be replaced by a planning-only SHA.

The historical plan-authoring baseline `77fe72a...` may remain only if clearly labeled as historical plan-authoring context and useful; otherwise remove it from the compact registry to reduce stale control metadata.

### B. Registry M003 state

Replace active wording equivalent to:

~~~text
Real-consumer M003 plan authoring is now dependency-ready
~~~

with a concise current-state statement:

- bounded Eggup adapter/API qualification completed;
- real Eggsact adoption blocked on producer-owned live artifact mapping and manifest publication/addressing convention;
- resume M003 only after that evidence exists.

Ensure the active subsystem table, dependency-ready/planned work table, immediate execution graph, project-state summary, and next-handoff section all agree.

Historical references inside old implementation/closure records do not need rewriting.

### C. Foundation summary

Update:

~~~text
Verified update core | M001-M006
Service lifecycle    | M001-M004
~~~

to:

~~~text
Verified update core | M001-M007
Service lifecycle    | M001-M005
~~~

and retain their existing closure directories as evidence pointers.

Do not inflate consumer-adoption closure ranges: Gregg, Egress, and EggPool remain unresolved/deferred as already documented.

### D. M003 execution-status hosted CI supplement

Add a small section or paragraph to:

`plans/closure/eggpack-manifest-interoperability/003-status.md`

recording:

- hosted Eggup CI run: `36037573793`;
- head SHA: `da1b4a8048bf863e6a653c25f1ba56bc42f4531b`;
- Stable checks: passed;
- Rust 1.89 MSRV check: passed;
- macOS tests: passed;
- Windows check: passed.

State explicitly:

- this qualifies current Eggup HEAD and the bounded adapter/API change;
- it does not constitute hosted Eggsact consumer verification;
- it does not remove the M003 producer blocker.

Do not modify the existing statement that hosted Eggsact checks were not run.

### E. Corrective closure record

Create:

`plans/closure/planning-closure-hygiene-corrective/003-status.md`

The record must contain:

- starting baseline `da1b4a8048bf863e6a653c25f1ba56bc42f4531b`;
- exact documentation commits used for the corrective;
- requirement-to-evidence matrix for all five findings;
- exact registry fields/summary statements corrected;
- exact M003 status supplement added;
- hosted CI run `36037573793` evidence;
- confirmation that no production source changed;
- confirmation M003 remains blocked;
- confirmation M004 and archive M002 remain blocked;
- unresolved findings by severity;
- final next-handoff statement.

After closure, set this plan's status to closed and register the closure path.

## 7. Ordered work packages

1. Refresh Eggup HEAD and confirm `da1b4a...` remains the current pre-corrective baseline.
2. Re-read the registry, M003 plan, M003 execution-status record, Core M007 closure, and Service M005 closure.
3. Confirm hosted CI run `36037573793` still reports all four configured jobs as successful.
4. Update registry baseline metadata.
5. Reconcile every active M003 state representation in the registry.
6. Correct the closed-foundation table to Core M007 and Service M005.
7. Add the narrow hosted-Eggup-CI supplement to the M003 execution-status record.
8. Create the C003 closure record.
9. Mark C003 closed in the registry and this implementation plan.
10. Run planning/documentation consistency checks and `git diff --check`.

## 8. Verification strategy

At minimum verify:

### Registry consistency

Search active registry text for stale forms such as:

~~~text
19935ec3610a5238af33a9d4f05a14925ceac25c
Real-consumer M003 plan authoring is now dependency-ready
Verified update core | M001-M006
Service lifecycle | M001-M004
M003 ... ready for plan authoring
~~~

Any remaining occurrence must either be intentionally historical and clearly labeled, or be corrected.

Confirm the registry contains:

- `cbfa8fa3870dae22d16775408a3135ea682c9365` as current reviewed production implementation baseline;
- `da1b4a8048bf863e6a653c25f1ba56bc42f4531b` as the reviewed pre-corrective planning/status baseline;
- M003 blocked-after-bounded-qualification wording;
- producer-side next handoff;
- Core M001-M007;
- Service M001-M005.

### Closure/status consistency

Confirm `plans/closure/eggpack-manifest-interoperability/003-status.md` continues to say:

- M003 is blocked/not closed;
- no Eggsact production source/dependency change was made;
- hosted Eggsact consumer checks were not run;
- producer convention remains the blocker.

Also confirm it newly records hosted Eggup CI `36037573793` without conflating the two evidence categories.

### Repository checks

Because this is docs-only:

~~~bash
git diff --check
cargo fmt --all -- --check
~~~

Run the repository's normal local check if convenient/required by current workflow, but do not claim runtime requalification is necessary for Markdown-only edits.

Hosted CI triggered by the documentation commits may be recorded in C003 closure if it completes before closure. If it has not run, do not delay or invent it; the already-passed `36037573793` run is sufficient evidence for the production M003 implementation state.

## 9. Failure/stop conditions

Stop and open a different corrective if:

- current M003 status evidence contradicts the claim that only producer convention is blocking;
- the hosted run `36037573793` is not actually successful on all configured jobs;
- Core M007 or Service M005 lacks real closure evidence;
- reconciliation discovers an unrecorded production change after `cbfa8fa...`;
- registry cleanup would require deciding Eggpack/Eggsact artifact naming or manifest publication policy;
- M003 can no longer truthfully remain blocked for the reasons in its execution record.

Do not solve a producer-side problem by editing Eggup planning labels.

## 10. Acceptance criteria

C003 closes only when:

- registry implementation baseline points to `cbfa8fa...`, not the older M001a no-production-delta baseline;
- registry planning/status metadata reflects `da1b4a...` or the exact newer corrective baseline appropriately;
- no active registry wording says M003 is merely ready for plan authoring;
- Core M007 and Service M005 appear in the compact closed-foundation summary;
- the M003 execution-status record includes hosted Eggup CI run `36037573793`;
- the M003 execution-status record still clearly says hosted Eggsact consumer checks were not run;
- M003 remains blocked on producer-owned Eggsact artifact/manifest conventions;
- M004 and archive M002 remain blocked;
- a C003 closure record exists;
- no production source changed;
- no medium-or-higher planning/evidence inconsistency remains.

## 11. Closure evidence required

The C003 closure must record:

- exact starting and closing Eggup SHAs;
- files changed;
- stale registry values/phrases removed or reclassified;
- corrected Core/Service foundation ranges;
- hosted CI run `36037573793` and its four successful jobs;
- explicit distinction between Eggup hosted CI and absent Eggsact hosted consumer verification;
- `git diff --check` result;
- `cargo fmt --all -- --check` result;
- confirmation of zero production-source changes;
- unresolved findings with severity;
- final dependency graph and next handoff.

## 12. Handoff after closure

After C003 closes, no further Eggup-side interoperability implementation is dependency-ready solely from this pass.

The next required evidence is producer-side:

1. Eggpack/Eggsact establish the authoritative live Eggsact artifact mapping.
2. Eggpack/Eggsact establish how a ReleaseManifest is produced, named, and addressed for that release.
3. Resume the blocked consumer portion of Eggup M003 against those facts.
4. Only after M003 real-consumer closure and a publishable `eggpack-manifest` should M004 package/API promotion be authored.

Separately:

- M002 archive extraction remains blocked on the Phase 10 safe extraction contract;
- Gregg remains footprint-gated;
- authenticity remains a future ADR-governed line.

This corrective must leave that dependency structure explicit and unchanged.
