---
name: planning-workflow
description: Eggup plan-to-closure workflow — normative docs, 16-section implementation plans, closure evidence, registry updates
---

# Eggup planning workflow

Use this when authoring or reviewing any `plans/` milestone work.

## Normative docs (do not edit in ordinary work)

- `plans/000-long-term-specification.md`, `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`, `plans/003-planning-process.md`
- Accepted ADRs in `plans/adrs/` are superseded, never rewritten.
- Classify cross-repo release/update milestones against ADR-0004 before authoring Eggup work.

## Work flow (in order)

1. Read the subsystem roadmap in `plans/subsystems/`.
2. Write one bounded plan at `plans/implementation/<subsystem>/NNN-short-title.md`
   following the 16-section template in `plans/implementation/README.md`.
   Required: SHA baseline, explicit non-goals, failure/rollback semantics,
   compat/migration note, tests, verification commands, closure evidence.
3. Implement and verify.
4. Write closure at `plans/closure/<subsystem>/NNN-status.md` (same number)
   per `plans/closure/README.md`. Required: status, baselines + commits,
   requirement→evidence matrix, exact commands with results,
   invariant/failure/compat/security/docs reviews, severitized residuals,
   per-platform results where applicable. Happy-path-only is insufficient.
5. Update `plans/registry.md` (active/ready work, blockers, transitions only)
   and the roadmap status table.
6. Never author an Eggup installer generator or producer-side work — that
   belongs to Eggpack. `distribution-bootstrap` is archived/transferred.

## Naming

ADR `adrs/ADR-NNNN-short-title.md` · roadmap `subsystems/<subsystem>-roadmap.md`
· plan `implementation/<subsystem>/NNN-short-title.md`
· closure `closure/<subsystem>/NNN-status.md`.
Every roadmap/plan classifies work as Invariant / Capability / Infrastructure /
Polish; infrastructure alone is not a completed capability until a real
consumer proves the path.
