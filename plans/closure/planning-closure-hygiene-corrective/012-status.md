# Planning / Closure Hygiene Corrective C012 — Closure and Verification Record

Status: closed

Source plan: `plans/implementation/planning-closure-hygiene-corrective/012-post-m004-roadmap-and-registry-reconciliation.md`

Repository baseline: `3b82d5397e728649a666690868a1e2d0fe42460d`

Plan authoring commit: `f8d9c3badc627f08b801e697133d0549eae691b0`

Plan registration commit: `f21f50666d2c523873813d317421743c2051c5f1`

Docs implementation commits:

- `387b576d1cb1d3eca6d4030f124fd1e870d0a791` — mark long-term Roadmap Phase 9 and Phase 10 complete;
- `ebbf67a44ccac2e76e03364fe56ee020a04e50aa` — reconcile archive roadmap with the closed Eggpack/Egress handoffs;
- `f3cfb8634ec408df10ac6cec3c83558d363e2e9f` — reconcile the post-M004 registry handoff/publication wording;
- `831fed7585be75f2edb6cbc800bcce47be9bf5b8` — mark the C012 implementation applied.

## Executive finding

C012 closes the remaining active planning drift after M004 without changing runtime, package, workflow, or release state.

The canonical long-term roadmap now records Phase 9 (Eggpack authority cutover / manifest interoperability) and Phase 10 (Egress archive/bundle convergence) as complete against their actual closure evidence. The active archive roadmap no longer describes the Eggpack archive handoff as future work. The registry no longer claims that no additional 0.1.2 crates were published after the core/archive pair and instead records the complete published set.

No Phase 11-13 direction changed. EggPool remains selective/deferred, authenticity still requires an ADR, and public API stabilization remains future work.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Remove stale pre-M004 publication language | `plans/registry.md` now records the published 0.1.2 set as core/archive/acquisition/eggfetch/eggpack and explicitly leaves curl/service/transport-footprint unpublished | passed |
| Refresh active planning-head bookkeeping | registry header distinguishes the post-M004 docs head from the M004 closure/publication head; C012 registration is recorded | passed |
| Mark Phase 9 complete | `plans/002-long-term-roadmap.md` Phase 9 now has `Status: complete`, completed adapter/publication evidence, and all exit criteria marked satisfied | passed |
| Mark Phase 10 complete | long-term roadmap Phase 10 now has `Status: complete`, Archive M001d + Interop M002/M002a + Egress M006 evidence, and both exit criteria marked satisfied | passed |
| Reconcile archive current evidence | capability now consumes `ManifestProjection::Archive` through the qualified handoff; current evidence records closed M002/M002a and landed Egress Delivery M003 rather than `ArchiveExtractionRequired`-only future state | passed |
| Preserve historical closure evidence | no historical closure record was edited | passed |
| Preserve Phase 11-13 direction | no Phase 11, 12, or 13 content changed | passed |
| Zero runtime/package/workflow delta | repository compare from `3b82d53` to `831fed7` changes only `plans/002-long-term-roadmap.md`, C012 plan, `plans/registry.md`, and `plans/subsystems/archive-extraction-roadmap.md` | passed |

## Phase 9 closure basis

Phase 9's exit criteria are all satisfied:

- Eggup has no active producer distribution crate; `eggup-dist` was retired under Distribution M004.
- Eggpack owns producer release contracts/conformance/manifests/build/CI/publication.
- `eggup-core` has no Eggpack dependency.
- `eggup-eggpack` is a narrow consumer adapter and is published as 0.1.2.
- M003 proved real Eggsact consumption while preserving application-owned release/origin/fallback/destination policy.
- M004 proved registry-only adapter and Eggsact-shaped graphs.

Primary evidence: `plans/closure/eggpack-manifest-interoperability/004-status.md`.

## Phase 10 closure basis

Phase 10's exit criteria are all satisfied:

- Archive M001d closed object/handle-backed source handoff.
- Interop M002/M002a closed the Eggpack archive-evidence path.
- Egress Delivery M003 / Consumer M006 removed bespoke `replace_pair` backup/rollback and extraction helpers.
- Egress retains exact staged-version agreement and sibling-pair policy before Eggup transaction commit.
- Hosted updater/archive qualification passed Linux/macOS/Windows at run `36639694985`.

Primary evidence: `plans/closure/consumer-adoption/006-status.md` M003-landed addendum plus archive/interoperability closure records.

## Verification actually performed

Repository-side compare of baseline `3b82d5397e728649a666690868a1e2d0fe42460d` to implementation head `831fed7585be75f2edb6cbc800bcce47be9bf5b8`:

- status: ahead;
- commits: 6;
- changed files: exactly four, all under `plans/`;
- no `crates/`, root `Cargo.toml`, `Cargo.lock`, or `.github/` path changed.

Content checks on the active touched surfaces confirmed no remaining occurrence of:

- `No other 0.1.2 workspace crate was published`;
- `later consume Eggpack ManifestProjection::Archive`;
- current-evidence prose saying the adapter merely returns `ArchiveExtractionRequired`;
- `M004 remains blocked`;
- current prose saying `eggup-eggpack` remains unpublished.

The long-term roadmap contains explicit `Status: **complete**.` markers under both Phase 9 and Phase 10.

No Cargo test was run because the change is documentation-only and repository comparison proves zero runtime/Cargo/workflow delta. Local shell-only checks listed in the source plan were not represented as having run; equivalent repository compare/content verification above is the evidence actually collected.

## Invariant review

- Producer build/release/publication ownership remains in Eggpack.
- Lower Eggup layers remain Eggpack-independent.
- Historical blocked states and closure records remain untouched.
- No immutable 0.1.2 package/tag/release was changed or republished.
- Gregg M004 remains intentionally unwritten.
- EggPool M007 remains deferred/selective.
- Authenticity Phase 12 still requires an ADR.
- Phase 13 remains future API stabilization.

## Unresolved findings

No medium-or-higher planning contradiction remains in the touched active surfaces.

Low bookkeeping detail: as with prior docs-only closures, the final registry header can point to the closure/registration commit but cannot self-reference the final head without creating another commit. The normal one-commit registration-head convention is retained.

## Disposition

Closed. No follow-up Eggup implementation plan is unlocked by C012.

Immediate interoperability continuation remains consumer-owned Eggsact Git-to-registry migration. Eggup's next substantive roadmap work should come only from an explicitly selected Phase 11/12/13 line or a newly discovered corrective.
