# Consumer Adoption M006 — Execution Status

Disposition: **blocked; consumer milestone not closed**

Source plan: `plans/implementation/consumer-adoption/006-egress-archive-pair-adoption.md`

Source roadmap: `plans/subsystems/consumer-adoption-roadmap.md`

Cross-repository owner: `eggstack/eggress: plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md` (per the Eggup plan; see Blocker evidence below).

This record closes the current bounded Eggup-side execution pass and preserves its evidence. It does not claim M006 acceptance: no Egress consumer code was changed, no Eggup dependency was cut over, and no publication was performed. Both omissions are required by the plan itself, not gaps in this pass.

## Eggup-side gate progress (the only in-repo M006 work)

The plan's sole Eggup-owned prerequisite was the versioned core/archive package boundary (Package M008). That gate is now qualified:

- Core M008 closed at `plans/closure/verified-update-core/008-status.md`: workspace `0.1.2`, `cargo package` + `cargo publish --dry-run` green for `eggup-core` and `eggup-archive`, clean external fixture proving the M001d bound-source flow, explicit manual publish order (core before archive).
- Eggpack Interop M002 closed at `plans/closure/eggpack-manifest-interoperability/002-status.md`: archive-projection → `ArchivePlan` → bound-source staging helpers qualified; direct Egress-side archive flow consumes `eggup-archive` per its own plan.

No Eggup production change belongs to M006, and none was made: the plan's handoff notes assign all consumer code changes to `eggstack/eggress`.

## Blocker evidence

Two independent stops apply, either sufficient alone:

1. **Publication gate (plan Section 14).** "Stop before dependency merge if Package M008 is not published in a crates.io-usable form." M008 is qualified but unpublished: publication is a separate maintainer action and this pass performs no publication. A merged/publishable Egress dependency therefore still cannot cut over. Pre-merge qualification against an immutable Eggup revision remains permitted by work package A, but see stop 2.

2. **Consumer-premise absence in the available checkout.** The plan's Section 3 premise ("Egress currently owns … curl-based archive/checksum download … bespoke same-filesystem backup/replace/rollback pair transaction") does not hold in the inspected Egress checkout (`/Users/davidbowman/projects/eggress`, head `6b0edc2`):
   - no updater/self-update module exists (`update.rs`, `self_update`, `updater`, and the `eggress update` command all absent from `crates/`, `tests/`, and `docs/`);
   - the mirrored delivery plan `plans/implementation/delivery/003-eggup-archive-pair-self-update-adoption.md` is absent (no `plans/implementation/` tree at all);
   - `docs/release/RELEASE_PROCESS.md` covers release *publishing* (checksums/SBOM/GitHub Release creation), not release *consumption*.
   
   With no Egress updater to migrate and no registered Egress delivery plan, there is no consumer surface for work packages B–F to attach to, and inventing one inside Eggup would violate the plan's ownership rule ("Actual consumer code changes belong in `eggstack/eggress`").

## Requirement disposition

| Requirement | Result / evidence |
|---|---|
| Versioned Eggup core/archive package availability (§2 gate) | Qualified, not published. M008 closure above; `cargo publish` remains a separate maintainer action. Pre-merge qualification against an immutable revision is permitted but not started (see consumer-premise stop). |
| Egress adapter / pair-transaction replacement (§6–§7, packages B–E) | Not started; no updater exists in the available checkout to adapt or replace. No Egress production source or dependency was changed. |
| Local helper deletion after parity (§5, package E) | Not applicable; nothing to delete. |
| Cross-platform/package/size qualification (§7F, §10–§11) | Not run; Egress verification commands belong to the consumer repo and require the consumer implementation to exist. |
| CLI exit-code / release-policy preservation (§4) | Not applicable; no consumer behavior changed. |
| Closure evidence: Eggup version, parity matrix, fault injection, package evidence (§15) | Partially satisfiable from the Eggup side (version `0.1.2`, M008 package evidence) but the consumer-owned rows cannot be produced here. |

## Invariant review

All §4 invariants hold vacuously: nothing in Egress was touched, so no release/checksum/version/CLI policy could regress. The Eggup-side M008/M002 changes that M006 depends on were reviewed in their own closures with no medium-or-higher finding.

## What unblocks M006

1. Maintainer publishes `eggup-core 0.1.2` then `eggup-archive 0.1.2` to crates.io (M008 manual order), satisfying the §14 merge gate; **and**
2. Egress registers its delivery plan and presents the updater surface the Eggup plan contracts against (or revises the contract if Egress has no self-updater, in which case this Eggup plan needs a corrective, not an implementation pass).

Until both hold, M006 remains planned/blocked. No corrective is opened: the stops are external gates, not Eggup defects.
