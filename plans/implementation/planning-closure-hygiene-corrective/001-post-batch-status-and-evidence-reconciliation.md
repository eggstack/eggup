# Planning and Closure Hygiene Corrective C001 — Post-Batch Status and Evidence Reconciliation

Status: ready for handoff

Eggup repository baseline reviewed: `2cab1f97ef30fa347c2030da321462459672c521`

Related completed work:

- Verified Update Core M007 plan: `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`
- Verified Update Core M007 closure: `plans/closure/verified-update-core/007-status.md`
- Consumer Adoption M005 plan: `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md`
- Consumer Adoption M005 closure: `plans/closure/consumer-adoption/005-status.md`
- CodeGG implementation: `dbowm91/codegg@23d84422c71bc1c251ba916a1a6c81e35c341fc9`
- CodeGG follow-up closure/planning commit: `dbowm91/codegg@e508de52083a30a1b234f5f9b619910cad6792d4`

Source governance:

- `plans/003-planning-process.md#11-corrective-passes`
- `plans/003-planning-process.md#12-closure-records`
- `plans/003-planning-process.md#13-registry`
- `plans/003-planning-process.md#14-baseline-handling`

Primary class: polish / planning-integrity corrective

## 1. Objective

Reconcile the small planning and closure-evidence drift left after CodeGG Consumer Adoption M005 and Verified Update Core M007 closed.

This is an evidence/documentation-only corrective. It must not change Eggup or CodeGG production behavior, dependency policy, release policy, transaction semantics, archive semantics, or service lifecycle behavior.

The pass closes four concrete findings:

1. `plans/registry.md` still records `bc25885bd41b86bfdf2f32d1e42856e00829cd7a` as the last reviewed Eggup implementation baseline even though production Core M007 landed at `8d5fc12f7224145285f22d0975a7bb91e1e363ea`.
2. `plans/subsystems/consumer-adoption-roadmap.md` still describes CodeGG's current pattern as check-only and its dependency graph still labels M005 `[ready]`, while M005 is closed.
3. Consumer Adoption M005 required deleted/superseded helper/LOC evidence, but the closure does not explicitly quantify the result or state that no generic updater code remained to delete.
4. Consumer Adoption M005 required a dependency and release-binary-size delta. Dependency-tree evidence is present, but no before/after release-profile size measurement was made; both Eggup and CodeGG closure records explicitly state that no size delta is claimed.

The corrective must make the planning surface truthful and either obtain the missing quantitative evidence or record a precise, reproducible reason it cannot now be obtained. It must not manufacture a number or weaken the original verification-honesty rules.

## 2. Readiness and dependencies

Hard dependencies are closed:

- Consumer Adoption M005 is closed at `plans/closure/consumer-adoption/005-status.md`;
- Verified Update Core M007 is closed and qualified at `plans/closure/verified-update-core/007-status.md`;
- CodeGG's corresponding Eggup adoption follow-up is closed at `dbowm91/codegg@e508de52083a30a1b234f5f9b619910cad6792d4`.

This corrective is dependency-ready.

Service Lifecycle M005 is technically dependency-ready after Core M007, but its detailed plan should be authored **after this corrective closes** so the registry/control surface is clean before the next implementation wave. This is a planning-order gate, not a runtime/API dependency.

## 3. Current evidence and detection gap

### 3.1 Registry baseline drift

Current Eggup registry:

- last implementation baseline reviewed: `bc25885bd41b86bfdf2f32d1e42856e00829cd7a`;
- current latest production Eggup implementation: `8d5fc12f7224145285f22d0975a7bb91e1e363ea`;
- latest M007 closure/planning commit: `2cab1f97ef30fa347c2030da321462459672c521`.

Detection gap: M007 closure updated milestone states but did not refresh the compact registry's implementation-baseline metadata.

### 3.2 Consumer-roadmap stale state

The consumer-adoption roadmap correctly marks M005 closed in its milestone section/table, but two earlier representations remain stale:

- current-state matrix: CodeGG still says `check-only update; verified bundle installer`;
- dependency graph: `M005 CodeGG [ready]`.

Detection gap: closure updated the authoritative milestone/table but did not sweep every duplicated summary representation in the roadmap.

### 3.3 Missing helper/LOC disposition

The M005 implementation plan required closure to record:

- deleted duplicated helpers/LOC.

The closure records implementation behavior and ownership but does not explicitly quantify what was removed versus added.

Important context: the pre-adoption CodeGG baseline had already retired its unsafe in-place shell/curl execution path and was intentionally check-only. Therefore the correct result may be that **zero generic updater helpers remained to delete**, with the new code consisting primarily of CodeGG policy/adaptation plus Eggup API use. The corrective must establish this from the exact baseline-to-implementation diff rather than infer it.

Detection gap: closure focused on security/correctness qualification and did not reconcile this explicit measurement requirement.

### 3.4 Missing release-binary-size delta

The M005 plan required:

- dependency and release-binary-size delta.

Dependency evidence exists. Both Eggup and CodeGG closure records truthfully state that no release-profile before/after size measurement was made.

Detection gap: the implementation was functionally and security-qualified without running the explicitly requested comparative size measurement.

## 4. Invariants

- Historical implementation/closure SHAs remain immutable facts.
- Do not rewrite prior closure text to make it appear that evidence existed at original closure time.
- Supplemental evidence must name the exact revisions measured.
- If size evidence cannot be reproduced, record `not reproducible` / `not available` with the exact reason; never estimate.
- Any size comparison must use the same target, build profile, feature set, toolchain class, and measurement method for both revisions.
- Do not compare a debug binary against a release binary or binaries built with materially different features.
- Do not use current CodeGG HEAD as the "after" revision; the implementation point is `23d84422c71bc1c251ba916a1a6c81e35c341fc9`.
- The pre-adoption comparison point is `220d3638fe043b24e65a7817241543612f9e82af`, unless repository evidence proves a more exact immediately-preimplementation baseline; any replacement baseline must be justified in the corrective closure.
- Helper/LOC evidence must distinguish deleted superseded generic updater mechanics from newly added CodeGG-specific policy/adapter/extraction code.
- No production source changes are authorized in Eggup or CodeGG.
- No package publication, tag, release, or dependency update is authorized.
- ADR-0004 ownership remains unchanged: Eggpack producer, Eggup deployment mechanism, application release/install policy.

## 5. Scope and non-scope

### In scope

- refresh Eggup registry implementation/closure baseline metadata;
- reconcile all CodeGG M005 status representations in the consumer-adoption roadmap;
- gather exact baseline-to-implementation CodeGG diff statistics relevant to updater ownership;
- explicitly state whether superseded generic updater helpers/LOC were deleted, retained, or absent at baseline;
- measure before/after CodeGG release binary size using equivalent builds if reproducible;
- record dependency delta already supported by closure/tree evidence without rerunning unrelated dependency work unless needed for the corrective record;
- create a dedicated corrective closure record;
- add a supplemental pointer from Consumer Adoption M005 closure to the corrective record without rewriting original historical claims;
- update the registry to close C001 and restore Service M005 as the next plan-authoring handoff after the corrective executes.

### Out of scope

- Rust production changes;
- changing M007 APIs or tests;
- changing CodeGG updater behavior;
- changing archive extraction or candidate validation;
- switching CodeGG to `eggup-eggfetch`;
- Windows in-place replacement;
- authenticity/signatures;
- Eggpack ReleaseManifest work;
- Service Lifecycle M005 implementation-plan authoring inside this corrective;
- crates.io publication or lockstep 0.1.1 release;
- retroactively modifying historical commit SHAs or test results.

## 6. Required changes

### A. Registry baseline reconciliation

Update `plans/registry.md` so:

- the last reviewed Eggup production implementation baseline is `8d5fc12f7224145285f22d0975a7bb91e1e363ea`;
- the latest reviewed closure/planning baseline identifies the M007 closure state at `2cab1f97ef30fa347c2030da321462459672c521` or the exact later corrective baseline as appropriate;
- C001 is recorded as active/ready while executing and closed after its closure record lands;
- Service Lifecycle M005 remains the next substantive plan-authoring transition after C001.

Do not use a planning-only SHA as the production implementation baseline.

### B. Consumer-adoption roadmap reconciliation

Update the CodeGG row in the current-state matrix to reflect current reality, for example:

- verified in-place managed-runfile update on supported Linux/macOS;
- manual/non-mutating path on unsupported targets including Windows;
- Eggup value: first multi-artifact bundle adopter / generalized updater blocker resolved.

Update the dependency graph:

- `M005 CodeGG [ready]` -> `[closed]`.

Sweep the roadmap for other wording that incorrectly says CodeGG is still check-only or M005 is still ready. Do not rewrite historical plan/closure descriptions that intentionally describe the pre-adoption state.

### C. Exact helper/LOC evidence

Using exact CodeGG revisions:

- before: `220d3638fe043b24e65a7817241543612f9e82af`;
- after: `23d84422c71bc1c251ba916a1a6c81e35c341fc9`.

Record:

- changed files in the updater/adoption path;
- added/deleted line counts for those files;
- named generic helpers removed, if any;
- named legacy helpers intentionally retained, if any;
- new CodeGG-specific adapter/policy/extraction modules or helpers;
- whether the required "deleted duplicated helpers/LOC" result is nonzero or zero because the unsafe/generic path was already retired by the historical M005 hardening.

Prefer mechanically derived `git diff --stat`, `git diff --numstat`, and targeted source inspection over manual estimates.

If the result is zero removed generic helpers, state that explicitly and explain why this still satisfies the ownership goal: CodeGG had no active generic replacement engine at the baseline; the adoption consumes Eggup rather than introducing another one.

### D. Release-binary-size evidence

Attempt a controlled comparison at the exact before/after revisions.

Use:

- the same supported host target;
- the same release profile;
- the same feature selection;
- the same toolchain when possible;
- the same binary path and file-size command.

Preferred local comparison:

```text
git worktree add <before-dir> 220d3638fe043b24e65a7817241543612f9e82af
git worktree add <after-dir> 23d84422c71bc1c251ba916a1a6c81e35c341fc9

# in each worktree, same host/toolchain/profile:
cargo build --release --locked --bin codegg
# or the repository's canonical release build command if it differs

wc -c target/release/codegg
# and/or stat using the same platform command for both
```

If CodeGG's canonical release build requires a specific feature set or release script, use that same method on both revisions and record it.

Record:

- before bytes;
- after bytes;
- absolute delta;
- percentage delta;
- target/toolchain/profile/features.

Do not attribute the entire binary-size delta causally to Eggup if the implementation revision contains other compatibility edits. Label it a revision-to-revision adoption delta.

If one revision cannot be rebuilt reproducibly due to an unavailable dependency/toolchain/external artifact, record:

- exact command;
- exact failure category;
- whether a narrower equivalent build was attempted;
- why no truthful comparison can be produced.

A demonstrated reproducibility blocker is acceptable closure for this low-severity evidence corrective, but the final record must continue to say no size delta is known.

### E. Supplemental closure evidence

Create:

- `plans/closure/planning-closure-hygiene-corrective/001-status.md`.

The record must include:

- exact Eggup corrective commit(s);
- exact CodeGG before/after SHAs;
- requirement-to-evidence matrix for all four findings;
- roadmap/registry corrections;
- helper/LOC diff evidence;
- size comparison or explicit reproducibility-blocked disposition;
- verification commands actually run;
- statement that no production code changed;
- unresolved findings with severity;
- disposition of Service Lifecycle M005 plan-authoring readiness.

Add a short supplemental pointer to `plans/closure/consumer-adoption/005-status.md` stating that post-closure measurement/status reconciliation lives in the C001 record. Preserve the original statement that no size measurement was available at original M005 closure time.

## 7. Ordered work packages

1. Refresh Eggup and CodeGG HEADs and confirm the named historical implementation SHAs still exist.
2. Reproduce the four findings against the exact current planning files.
3. Gather CodeGG before/after updater diff and helper/LOC evidence.
4. Attempt the controlled release-binary-size comparison.
5. Reconcile the consumer-adoption roadmap matrix/graph and sweep for stale M005 status wording.
6. Refresh registry implementation/closure baseline metadata.
7. Add the C001 closure record and M005 supplemental pointer.
8. Run documentation/planning consistency checks and repository verification appropriate to a docs-only change.
9. Mark C001 closed in the registry and leave Service Lifecycle M005 as the next substantive plan-authoring handoff.

## 8. Failure/contention semantics

This pass changes no runtime state machine.

Planning failure rules:

- if the exact historical revisions cannot be resolved, stop and record the repository-history inconsistency rather than substituting approximate SHAs;
- if equivalent binary builds cannot be produced, record the measurement as unavailable with exact evidence;
- if the diff reveals a still-active duplicated generic updater engine in CodeGG, stop and open a consumer corrective instead of closing this evidence-only pass;
- if a stale status implies an actually unclosed dependency rather than simple wording drift, stop and reconcile dependency state before changing labels;
- do not modify production files merely to make metrics easier to obtain.

## 9. Compatibility and migration

No API, CLI, filesystem, service, release, dependency, or package compatibility change is authorized.

Historical closure records remain valid historical evidence. C001 supplements missing measurement and control-surface accuracy; it does not retroactively claim those measurements existed during M005 closure.

No consumer migration is performed.

## 10. Required tests/checks

At minimum:

Planning/docs:

- search Eggup planning for stale `M005 CodeGG [ready]`;
- search active planning for stale CodeGG `check-only` descriptions and classify historical occurrences;
- search registry for stale `bc25885...` implementation-baseline control metadata;
- verify all linked plan/closure paths exist;
- `git diff --check`.

CodeGG evidence:

- `git diff --stat 220d3638... 23d84422...`;
- `git diff --numstat 220d3638... 23d84422... -- <updater/adoption-relevant paths>`;
- targeted inspection of updater, Cargo manifest/lock, and any newly added managed-upgrade module;
- controlled release build/size commands for both revisions, or recorded exact build blocker.

Repository verification for docs-only Eggup changes:

```bash
git diff --check
cargo fmt --all -- --check
```

A full Eggup test suite is not required solely for Markdown changes unless the repository's current verification script couples documentation/planning checks to tests. If run, report it truthfully.

## 11. Verification commands

Recommended evidence commands, adapted to the executing environment:

```bash
# Eggup
git rev-parse HEAD
git grep -n "M005 CodeGG \[ready\]" -- plans || true
git grep -n "check-only" -- plans/subsystems/consumer-adoption-roadmap.md plans/registry.md || true
git grep -n "bc25885bd41b86bfdf2f32d1e42856e00829cd7a" -- plans/registry.md || true
cargo fmt --all -- --check
git diff --check

# CodeGG historical evidence
git diff --stat 220d3638fe043b24e65a7817241543612f9e82af 23d84422c71bc1c251ba916a1a6c81e35c341fc9
git diff --numstat 220d3638fe043b24e65a7817241543612f9e82af 23d84422c71bc1c251ba916a1a6c81e35c341fc9 -- src/upgrade tests/upgrade.rs Cargo.toml Cargo.lock

# Build each exact revision with one identical canonical release command,
# then record exact codegg binary bytes.
```

Do not treat illustrative path filters above as exhaustive if the implementation placed managed-upgrade code elsewhere; first inspect the actual diff.

## 12. Documentation updates

Expected Eggup files:

- `plans/registry.md`;
- `plans/subsystems/consumer-adoption-roadmap.md`;
- `plans/closure/consumer-adoption/005-status.md` (supplemental pointer only);
- new `plans/closure/planning-closure-hygiene-corrective/001-status.md`;
- this plan status on closure.

No canonical specification or ADR change is expected.

No CodeGG documentation change is required unless the evidence pass discovers that CodeGG's own active control surface is stale. If so, keep any external change narrowly documentation-only and record its exact SHA in C001 closure.

## 13. Acceptance criteria

C001 closes only when:

- Eggup registry production-baseline metadata reflects Core M007 rather than the retired distribution implementation;
- active consumer-adoption planning describes CodeGG M005 as closed and no longer characterizes current supported CodeGG self-update as check-only;
- the CodeGG baseline-to-implementation updater diff is quantified and the duplicated-helper/LOC requirement has an explicit factual disposition;
- the release binary-size delta is either measured under equivalent conditions or explicitly demonstrated unavailable with exact reproducibility evidence;
- the M005 historical closure points to the corrective supplement without falsifying its original "not measured" statement;
- C001 has its own closure record satisfying `plans/003-planning-process.md`;
- no production code changed;
- Service Lifecycle M005 remains the next substantive plan-authoring transition;
- no medium-or-higher planning/evidence defect remains open.

## 14. Stop conditions

Stop and open a different corrective if:

- source inspection finds an active duplicated generic updater implementation in CodeGG;
- the current CodeGG behavior does not match the M005 closed claims;
- M007 production code or its closure evidence is inconsistent enough to require a core corrective;
- a size comparison would require modifying historical source or dependency policy;
- planning cleanup exposes a producer/consumer ownership regression against ADR-0004.

## 15. Closure evidence required

The C001 closure must record:

- exact Eggup starting and closing SHAs;
- exact CodeGG before/after SHAs;
- four-finding requirement/evidence/result matrix;
- exact planning files changed;
- exact stale strings removed/corrected;
- updater/adoption diff stat and relevant numstat;
- named removed/retained/new helper ownership;
- before/after release binary byte counts and delta, or exact reproducibility-blocked evidence;
- commands actually run and results;
- confirmation that no production source changed;
- unresolved findings with severity;
- explicit handoff: whether Service Lifecycle M005 may now be authored.

## 16. Handoff notes

This corrective is the immediate planning handoff.

After C001 closes cleanly, author Service Lifecycle M005 against the already-qualified Core M007 `ValidatedTransaction::commit_with_post_commit` API.

Do not combine Service M005 implementation planning into this cleanup pass; keeping the evidence reconciliation separate makes the next service plan start from a trustworthy registry and roadmap state.
