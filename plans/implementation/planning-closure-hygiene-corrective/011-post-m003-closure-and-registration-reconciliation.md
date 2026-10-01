# Planning / Closure Hygiene Corrective C011 — Post-M003 Closure and Registration Reconciliation

Status: ready for handoff

Repository baseline: `538e3e5605cf3c315c10e5be200c8896de7379b1`

Affected records:

- `plans/registry.md`
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`
- `plans/closure/eggpack-manifest-interoperability/003a-status.md` (verification-only)
- `plans/closure/eggpack-manifest-interoperability/003-status.md` (verification-only)
- `plans/implementation/eggpack-manifest-interoperability/004a-package-api-promotion-readiness-preflight.md`

Primary class: polish / corrective

## 1. Objective

Reconcile Eggup's active planning control surface after M003a and M003 both closed on 2026-10-01.

The substantive interoperability state is already correct in the roadmap/body of the registry:

- M003a caller-bound destination policy corrective closed at implementation `39ff62602e39b14b31f5a8f154905d3343034db7` with hosted run `36890986000` green;
- M003 Eggsact real-consumer adoption closed at consumer `eggstack/eggsact@65c916ba3b0f02916203ec1aad09d7e5c023c278` on Eggup pin `e336b3203183aa84d174e7d7bfa087ce4b61b077`, with Eggsact CI `36902758482` and release-drift run `36902758396` green;
- M004 package/API promotion is now the next interoperability decision point.

However, the registry header still identifies an older planning-registration head and an older fully-qualified runtime head, describing M003a as ready and M003 as blocked. C011 makes the active bookkeeping internally consistent and registers the newly discovered M004 publication-chain preflight without rewriting historical evidence.

C011 is docs-only.

## 2. Why this corrective is ready

All underlying runtime/consumer evidence already exists in-repo:

- `plans/closure/eggpack-manifest-interoperability/003a-status.md`;
- `plans/closure/eggpack-manifest-interoperability/003-status.md`;
- `plans/subsystems/eggpack-manifest-interoperability-roadmap.md`;
- current registry body rows already mark M003a/M003 closed;
- current Eggup HEAD `538e3e56...` is the M003 closure documentation head.

No unresolved runtime decision is required to correct the stale header/control-surface text.

## 3. Current planning defects

### 3.1 Registry header lags the body

The registry header still reports:

- latest planning registration head `7abbfc05...`, when the repository has since closed M003a and M003;
- latest reviewed runtime head `e8e07eb...`, which predates the qualified M003a adapter implementation;
- wording that says M003a is ready and M003 is blocked.

Later registry sections correctly record both milestones as closed.

### 3.2 Next-step description understates the M004 publication chain

The registry and interoperability roadmap currently describe M004 primarily as waiting for a publishable `eggpack-manifest`.

That is necessary but not sufficient for registry-only promotion. At the current tree:

- `eggup-eggpack` is `publish = false`;
- it depends on `eggpack-manifest = 0.1.0` by immutable Git revision;
- it depends on `eggup-core = 0.1.2`, `eggup-archive = 0.1.2`, and `eggup-acquisition = 0.1.2`;
- `eggup-core 0.1.2` and `eggup-archive 0.1.2` are published;
- Eggup's registry states no other 0.1.2 workspace crate was published;
- the real Eggsact M003 proof intentionally used one immutable Eggup Git source identity.

M004 therefore needs a package-graph/readiness preflight before a publication plan can be dependency-ready.

### 3.3 Eggpack cross-repo status is stale but is not Eggup-owned

Eggpack's current interoperability roadmap still describes the Eggsact M003 runtime adoption as ready to resume, while Eggup has closed it.

C011 must record this as cross-repo planning drift and point to the authoritative Eggup closure. It MUST NOT rewrite Eggpack documents from Eggup or move producer-side planning authority into this repository.

## 4. Invariants

- Preserve M003a and M003 closure records as historical evidence.
- Do not alter runtime code, Cargo manifests, dependency versions, package publication flags, or lockfiles.
- Do not claim a crate is published without registry evidence.
- Do not authorize automatic crates.io publication.
- Do not move `eggpack-manifest` publication ownership into Eggup.
- Do not assume that every Eggup workspace crate must publish at 0.1.2; M004a must determine the minimum coherent set mechanically.
- Keep Eggpack producer/build/publication policy outside Eggup.
- Keep `eggup-core` usable without Eggpack.
- Preserve the M003 consumer qualification fact that Git-only dependencies were intentionally not released publicly.

## 5. Scope

### In scope

- refresh registry header bookkeeping to the actual post-M003 closure state;
- identify `39ff626` / M003a as the newest qualified Eggup runtime delta and `538e3e56` as the current closure/planning head;
- ensure active registry/roadmap prose names M004a readiness preflight before M004 publication;
- register M004a as dependency-ready;
- mark M004 proper as blocked on M004a findings plus any producer-side publication prerequisite it identifies;
- record Eggpack roadmap drift as a cross-repo follow-up, not an Eggup runtime blocker;
- create C011 closure evidence when executed.

### Out of scope

- runtime/API implementation;
- changing `publish = false`;
- publishing any crate;
- editing Eggpack or Eggsact repositories;
- authoring the final M004 publication plan before M004a closes;
- authenticity/signature work;
- Gregg/EggPool adoption decisions.

## 6. Required changes

### 6.1 Registry header

Update the compact header so it truthfully distinguishes:

- current planning/closure HEAD;
- newest runtime implementation with full hosted qualification;
- current real-consumer closure evidence.

Recommended semantics:

- latest planning/closure head: the post-C011 closure/registration head;
- latest qualified Eggup runtime delta: M003a implementation `39ff626`, qualified by run `36890986000`;
- M003 consumer qualification: Eggsact `65c916b`, CI `36902758482`, drift `36902758396`.

Do not call the M003 docs closure commit a new runtime implementation.

### 6.2 Interoperability roadmap and registry M004 wording

Replace the single-condition wording "needs publishable eggpack-manifest" with a precise dependency transition:

1. M003 is closed.
2. M004a registry/package readiness preflight is ready.
3. M004a determines the exact registry publication set and producer-side prerequisite.
4. M004 proper remains blocked until M004a closes and every external registry dependency is actually resolvable.

### 6.3 Cross-repo drift note

Record that the Eggpack interoperability roadmap is stale with respect to M003 closure and should be reconciled in Eggpack's own planning process.

Do not treat that stale prose as evidence that M003 is open.

## 7. Ordered work packages

1. Re-read HEAD, registry, interoperability roadmap, M003a closure, and M003 closure.
2. Verify no runtime commit landed after `39ff626` in Eggup.
3. Correct registry header fields and duplicated active status prose.
4. Register M004a in registry and roadmap.
5. Mark M004 proper blocked on M004a + external registry availability.
6. Search active Eggup planning files for stale "M003 ready/blocked" statements.
7. Change only active control surfaces; preserve historical bounded-pass text.
8. Record Eggpack cross-repo drift without editing Eggpack.
9. Run docs/planning consistency checks.
10. Write `plans/closure/planning-closure-hygiene-corrective/011-status.md`.
11. Refresh registry registration/closure head to the actual docs commit.

## 8. Failure / restart semantics

No runtime semantics.

If new Eggup runtime work lands during C011, refresh the baseline and do not overwrite newer qualification language.

If M004a is implemented/closed concurrently, reconcile to the newer state rather than preserving an intermediate "ready" status.

## 9. Verification

Run and record:

```text
git diff --check
git status --short
rg -n "M003.*ready|M003.*blocked|M003a.*ready|7abbfc05|e8e07eb" plans/registry.md plans/subsystems plans/implementation
rg -n "M004|eggpack-manifest|eggup-acquisition|publish" plans/registry.md plans/subsystems/eggpack-manifest-interoperability-roadmap.md
```

No Cargo command is required for a docs-only pass unless execution discovers an accidental runtime/Cargo delta.

## 10. Documentation / closure evidence

Create `plans/closure/planning-closure-hygiene-corrective/011-status.md` containing:

- exact baseline and closure commit;
- files changed;
- confirmation of zero runtime/Cargo delta;
- stale statements corrected;
- M004a registration evidence;
- cross-repo Eggpack drift note;
- exact verification commands/results;
- unresolved findings and next dependency transition.

## 11. Acceptance criteria

C011 closes when:

- registry header and body agree that M003a/M003 are closed;
- newest qualified runtime and current planning/closure heads are distinguished correctly;
- M004a is the next dependency-ready interoperability handoff;
- M004 proper is not represented as publication-ready;
- the broader registry dependency chain is named accurately;
- historical closure evidence is preserved;
- no runtime/Cargo file changed.

## 12. Stop conditions

Stop and write a runtime/package corrective instead if reconciliation discovers:

- an unqualified Eggup runtime delta after M003a;
- a package manifest already changed toward publication without a plan;
- a registry dependency whose version/API cannot be reconciled without code changes;
- evidence that M003 consumer adoption did not actually use the closed M003a seam.

## 13. Handoff notes

C011 is bookkeeping only. It should be executed independently of M004a if desired, but both may be completed in one documentation-focused maintenance pass if their evidence remains unchanged.

The substantive next technical planning task is M004a, not M004 publication itself.
