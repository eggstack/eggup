---
name: planning-workflow
description: Eggup plan-to-closure workflow — normative docs, 16-section implementation plans, closure evidence, registry updates
---

# Eggup planning workflow

Use this when authoring or reviewing any `plans/` milestone work. Summary of
`plans/003-planning-process.md` (normative), `plans/implementation/README.md`,
and `plans/closure/README.md`. Deep dive is
[`architecture/tooling-governance.md`](../../../architecture/tooling-governance.md) §6.

## Normative docs (do not edit in ordinary work)

- `plans/000-long-term-specification.md`, `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`, `plans/003-planning-process.md`
- Accepted ADRs in `plans/adrs/` are superseded, never rewritten
  (`proposed → accepted → deprecated | superseded`).
- Classify cross-repo release/update milestones against ADR-0004 before authoring
  Eggup work. `distribution-bootstrap` is archived/transferred — do not author
  producer-side work or an installer generator here; that belongs to Eggpack.

## Classification (required on every roadmap and plan)

`Invariant` · `Capability` · `Infrastructure` · `Polish`.
**Infrastructure alone is not a completed capability until a real consumer proves
the path.**

A milestone is dependency-ready when its **hard** dependencies are closed and its
**interface** dependencies are sufficiently specified. Soft and operational
dependencies do not block.

## Severity gate

**"medium-or-higher" is the blocking threshold.** Any unresolved finding at that
level keeps the milestone open and forces a new corrective plan; happy-path-only
evidence cannot close one. The vocabulary is used consistently across the
closure record set but is not formally defined in any single document — when you
record a finding, state the severity and the reason explicitly rather than
assuming a shared scale.

## Corrective passes

A corrective plan names the original plan and its closure, enumerates the
requirements left unclosed, explains **how the gap went undetected**, and adds
regression evidence. Repeated correctives on one milestone should trigger a
roadmap decomposition review rather than another patch.

## Work flow (in order)

1. Read the subsystem roadmap in `plans/subsystems/`.
2. Write one bounded plan at `plans/implementation/<subsystem>/NNN-short-title.md`.
   Preamble fields: `Status:`, `Repository baseline: <exact SHA>`, `Source roadmap:`,
   `Long-term requirements:`, `Applicable ADRs:`, `Primary class:`.
   Then the 16 sections: 1 Objective · 2 Why this milestone is ready ·
   3 Current implementation evidence · 4 Invariants that must not regress ·
   5 Scope (In scope / Explicitly out of scope) · 6 Required production changes ·
   7 Ordered work packages · 8 Failure, cancellation, restart, and contention
   semantics · 9 Compatibility and migration · 10 Required tests ·
   11 Required verification commands · 12 Documentation updates ·
   13 Acceptance criteria · 14 Stop conditions · 15 Closure evidence required ·
   16 Handoff notes.
   §5 non-goals, §8 failure semantics, and §11 exact command strings carry the
   governance weight; §3/§14 must hold exact SHAs, not vague `main` references.
3. Implement and verify.
4. Write closure at `plans/closure/<subsystem>/NNN-status.md` (same number) per
   `plans/closure/README.md`. Required: status (closed / conditionally closed /
   corrective pass required / blocked), source plan and roadmap, reviewed baseline,
   implementation commits, executive finding, requirement→evidence matrix, production
   evidence, exact commands with results, invariant review, failure/rollback/recovery
   review, compatibility and migration review, security review, docs/operations
   evidence, unresolved findings with severity, roadmap disposition, registry updates.
   Happy-path-only is insufficient. Separate Linux/macOS/Windows results; a lane you
   did not run is **not run**, never inferred.
5. Update `plans/registry.md` (the active control surface — transitions, blockers,
   and next dependency work only; do not duplicate plan detail) and the subsystem
   roadmap status table.
6. Only then author newly dependency-ready downstream work.

## Naming

ADR `adrs/ADR-NNNN-short-title.md` · roadmap `subsystems/<subsystem>-roadmap.md`
· plan `implementation/<subsystem>/NNN-short-title.md`
· closure `closure/<subsystem>/NNN-status.md`.

Corrective plans take a lowercase-letter suffix after the number and reuse it for
their closure — `001a-status.md` for `001a-*.md` — so a corrective stays paired with
the milestone it corrects. `planning-closure-hygiene-corrective` is the docs-only
workstream for reconciling the registry, roadmaps, and closure records against
reality; it has no subsystem roadmap.

## Gotchas

- A milestone is not complete because code landed. It is complete when the closure
  record exists and the registry and roadmap agree with it.
- Superseded predecessors keep their closure records; mark them historical or
  superseded in the registry rather than deleting them.
- `plans/000`–`003` and accepted ADRs are not the place to record new work; a change
  to them is itself a milestone.
