# Planning / Closure Hygiene Corrective C012 — Post-M004 Roadmap and Registry Reconciliation

Status: ready for handoff

Repository baseline: `3b82d5397e728649a666690868a1e2d0fe42460d`

Primary class: polish / corrective

Affected active planning surfaces:

- `plans/002-long-term-roadmap.md`
- `plans/registry.md`
- `plans/subsystems/archive-extraction-roadmap.md`

Historical evidence consulted but not rewritten:

- `plans/closure/eggpack-manifest-interoperability/004-status.md`
- `plans/closure/consumer-adoption/006-status.md`
- `plans/closure/archive-extraction/001d-status.md`
- `plans/closure/eggpack-manifest-interoperability/002-status.md`

## 1. Objective

Reconcile Eggup's active planning/docs after Eggpack Interoperability M004 closed and after the Phase 10 Egress/archive convergence evidence was already complete.

This pass is documentation-only. It must:

1. remove stale pre-M004 publication language from the active registry;
2. refresh active planning-head bookkeeping to the post-M004 reconciliation baseline;
3. mark long-term Roadmap Phase 9 complete on the now-closed Eggpack authority/interoperability chain;
4. mark long-term Roadmap Phase 10 complete on the landed Egress archive/pair migration;
5. replace stale future-tense archive-roadmap prose that still says the Eggpack archive handoff is pending;
6. preserve historical closure/addendum text as dated evidence rather than rewriting history.

## 2. Readiness and evidence

C012 is ready because all substantive evidence already exists:

- Eggpack Interoperability M004 is closed; `eggup-acquisition 0.1.2`, `eggup-eggfetch 0.1.2`, and `eggup-eggpack 0.1.2` are published from `02a1d32`, with hosted run `37090397398` green and registry-only adapter/Eggsact-shaped proofs passing.
- Eggpack published `eggpack-manifest 0.1.0`; the adapter now consumes the registry version.
- Archive M001d is closed with handle-backed source handoff.
- Eggpack archive handoff M002/M002a is closed with hosted qualification.
- Consumer Adoption M006 is closed via Egress Delivery M003 at `19e6dc7`; its landed addendum records deletion of bespoke `replace_pair` backup/rollback machinery, exact staged-version agreement, Eggup commit/rollback disposition mapping, and Linux/macOS/Windows updater qualification.

No runtime decision is required.

## 3. Detected planning defects

### 3.1 Registry next-handoff contradiction

The active `## Next handoff` paragraph still says:

`No other 0.1.2 workspace crate was published.`

That was correct before M004 but now contradicts the same paragraph's later statement that M004 published `eggup-acquisition`, `eggup-eggfetch`, and `eggup-eggpack` 0.1.2.

### 3.2 Registry header trails the actual post-M004 docs head

The registry header identifies `ea1f1c5` as the latest planning/closure head while `main` is `3b82d53`, a docs-only M004 status reconciliation commit.

The header should distinguish the M004 publication/closure identity from the latest planning/docs reconciliation head.

### 3.3 Long-term Phase 9 remains written as future work

Phase 9 still says an adapter may be added after ReleaseManifest v1 stabilizes. In reality:

- producer-side distribution authority is transferred to Eggpack;
- `eggup-dist` is retired;
- the optional adapter exists, is consumer-qualified, and is published;
- its producer schema dependency is registry-resolvable;
- lower Eggup layers remain Eggpack-independent.

The phase exit criteria are satisfied and should be marked complete without changing the architecture.

### 3.4 Long-term Phase 10 lacks completion status

The Phase 10 exit criteria are satisfied:

- Egress Delivery M003 removed bespoke pair backup/rollback helpers;
- the archive path uses bounded extraction and Eggup transaction machinery;
- exact staged-version agreement/sibling-pair policy prevents a successful mismatched pair;
- hosted updater/archive lanes passed on Linux, macOS, and Windows.

The canonical roadmap should record Phase 10 complete.

### 3.5 Archive roadmap retains pre-handoff future tense

The active archive roadmap still says it will "later consume" Eggpack archive projection evidence and says `eggup-eggpack` returns `ArchiveExtractionRequired` pending interoperability M002.

M002/M002a have already closed. The active current-evidence section should describe the qualified handoff rather than the historical pre-M002 state.

## 4. Invariants

- No runtime source, Cargo manifest, Cargo.lock, workflow, release, tag, or package changes.
- Do not rewrite historical closure records or erase old blocked states that were correct when recorded.
- Do not imply Phase 11 EggPool adoption is required or ready; it remains selective/deferred.
- Do not start Phase 12 authenticity work; its ADR gate remains intact.
- Do not claim Phase 13 / 1.0 API stabilization is complete.
- Do not authorize Gregg M004.
- Keep producer publication/build authority in Eggpack.
- Keep `eggup-core` independent of Eggpack.
- Do not reopen or republish immutable 0.1.2 versions.

## 5. Scope

### In scope

- docs-only edits to the three active surfaces listed above;
- explicit Phase 9 and Phase 10 completion annotations with closure evidence;
- current-state wording for the published 0.1.2 set;
- current-state archive/Eggpack handoff prose;
- registry registration/closure bookkeeping;
- C012 closure record.

### Out of scope

- any Rust/Cargo/workflow change;
- Eggsact Git-to-registry migration;
- Gregg or EggPool migration planning;
- authenticity ADR authoring;
- new Phase 13 implementation work;
- editing Eggpack/Egress repositories.

## 6. Required documentation changes

### 6.1 Long-term roadmap

For Phase 9:

- add an explicit complete status;
- record the retired producer-distribution path and closed/published adapter seam;
- change the future-tense optional-adapter deliverable into completed evidence;
- state that all exit criteria are satisfied.

For Phase 10:

- add an explicit complete status;
- record Archive M001d + Interop M002/M002a + Egress M006/Delivery M003 as closure evidence;
- state that bespoke pair rollback was removed and exact pair-version checks plus Eggup transaction semantics satisfy the mismatch exit criterion.

Do not alter Phase 11-13 direction.

### 6.2 Registry

- replace the stale "No other 0.1.2 workspace crate was published" sentence with the truthful current registry set;
- refresh latest planning/docs head bookkeeping;
- register and then close C012;
- keep the next handoff focused on Eggsact-owned registry migration and intentional future/deferred work.

### 6.3 Archive extraction roadmap

- change "later consume Eggpack ManifestProjection::Archive" to current capability;
- replace the pre-M002 `ArchiveExtractionRequired` current-evidence paragraph with the qualified M002/M002a handoff;
- record Egress M006 as the concrete completed Phase 10 proof;
- leave historical milestone descriptions/closure evidence intact.

## 7. Ordered work packages

1. Capture current HEAD and active docs.
2. Register C012.
3. Update the canonical long-term roadmap Phase 9/10 status.
4. Reconcile the archive roadmap's active capability/current-evidence prose.
5. Reconcile registry header and next-handoff publication wording.
6. Search changed active surfaces for remaining contradictory pre-M004/pre-M002 text.
7. Verify no non-`plans/` file changed.
8. Write C012 closure record.
9. Mark the source plan closed and registry row closed.
10. Refresh the registry closure-registration head.

## 8. Failure / restart semantics

No runtime semantics.

If runtime/Cargo changes land concurrently, rebase the docs baseline and do not make qualification claims about the new runtime delta.

If a later consumer migration lands concurrently, reflect it only if authoritative closure evidence is present; otherwise preserve the current handoff.

## 9. Compatibility / migration

None. This pass changes planning truthfulness only.

Eggsact's Git-to-registry migration remains a separate consumer-owned action.

## 10. Tests and verification

Required:

```text
git diff --check
git status --short
git diff <baseline>..HEAD -- crates/ Cargo.toml Cargo.lock .github/
rg -n "No other 0.1.2 workspace crate was published|later consume Eggpack|returns .*ArchiveExtractionRequired|M004 remains blocked|eggup-eggpack.*unpublished" plans/registry.md plans/002-long-term-roadmap.md plans/subsystems/archive-extraction-roadmap.md
```

No Cargo test is required because C012 is docs-only. If any runtime/Cargo path appears in the diff, stop and re-scope.

## 11. Documentation

This corrective is itself documentation maintenance. Closure evidence must enumerate exact active statements corrected and note historical text intentionally preserved.

## 12. Acceptance criteria

C012 closes when:

- registry next-handoff text reflects the complete published 0.1.2 set;
- registry planning/docs head bookkeeping no longer stops at the pre-final M004 reconciliation head;
- Phase 9 is explicitly complete;
- Phase 10 is explicitly complete;
- archive roadmap current capability/evidence no longer describes M002 as future work;
- Phase 11-13 direction is unchanged;
- no runtime/Cargo/workflow file changed;
- no medium-or-higher planning contradiction remains in the touched active surfaces.

## 13. Stop conditions

Stop and open a separate corrective if:

- Phase 9/10 closure evidence is found insufficient for their stated exit criteria;
- an active runtime/package defect is discovered;
- canonical roadmap completion would require changing an accepted architecture decision;
- current consumer evidence conflicts with the archived closure records.

## 14. Closure evidence

Create `plans/closure/planning-closure-hygiene-corrective/012-status.md` containing:

- baseline and implementation/docs commit;
- changed-file inventory;
- requirement-to-evidence matrix;
- exact verification output;
- zero-runtime-delta proof;
- invariant review;
- historical-text preservation note;
- unresolved findings;
- disposition.

## 15. Handoff notes

After C012, no additional Eggup implementation is unlocked by Phase 9/10 closure. The immediate interoperability follow-on remains Eggsact-owned registry migration. EggPool remains selective/deferred; Phase 12 still requires an ADR; Phase 13 remains future stabilization work.
