# Eggup Planning System

This directory separates durable architectural direction from temporary implementation planning for Eggup.

Eggup is intended to become Eggstack's reusable verified installation, update, rollback, and optional service-lifecycle substrate. Planning must preserve a strict distinction between shared mechanism and application-specific release policy.

## Canonical long-term documents

The following files define the intended product and architecture and MUST NOT be edited as part of ordinary implementation work:

- `000-long-term-specification.md` — normative end-state specification and invariants.
- `001-terminology-and-domain-model.md` — normative language and core type model.
- `002-long-term-roadmap.md` — dependency-ordered capability roadmap.
- `003-planning-process.md` — rules for deriving and managing interim plans.

The first three are stable architectural references. Changes require an explicit long-term architecture decision, not implementation convenience.

## Planning hierarchy

```text
Long-term specification and terminology
        |
        v
Architecture decision records
        |
        v
Master long-term roadmap
        |
        v
Subsystem roadmaps
        |
        v
Milestone implementation plans
        |
        v
Implementation and verification
        |
        v
Closure records and archive
```

## Directory roles

- `adrs/` — durable architecture decisions. Accepted decisions are superseded, not rewritten.
- `subsystems/` — subsystem specifications and dependency-ordered roadmaps.
- `implementation/` — bounded plans handed to implementation agents.
- `closure/` — verification, evidence, residual-risk, and completion records.
- `archive/` — completed or superseded interim planning retained for traceability.
- `registry.md` — compact index of active work, dependencies, blockers, and closure state.

## Core rule

Long-term documents state what Eggup owns and what must remain true. Interim documents state what an implementation agent should do next against a specific repository baseline.

Eggup MUST centralize reusable deployment mechanics without absorbing consumer-specific release authority, version semantics, CLI presentation, database/config migration policy, or application-specific health semantics.

## Required classification

Every roadmap and implementation plan distinguishes:

- **Invariant** — a property that must always remain true.
- **Capability** — consumer- or operator-visible behavior.
- **Infrastructure** — internal machinery required by capabilities.
- **Polish** — diagnostics, ergonomics, performance, cleanup, or documentation.

Infrastructure is not a completed capability until a real consumer proves the path.

## Naming conventions

- ADR: `adrs/ADR-NNNN-short-title.md`
- Subsystem roadmap: `subsystems/<subsystem>-roadmap.md`
- Milestone implementation plan: `implementation/<subsystem>/NNN-short-title.md`
- Closure record: `closure/<subsystem>/NNN-status.md`
- Archived document: preserve original relative structure beneath `archive/`

## Starting a workstream

1. Read the canonical long-term documents.
2. Resolve durable architecture questions through ADRs.
3. Create or update a subsystem roadmap.
4. Select one dependency-ready milestone.
5. Write a bounded implementation plan.
6. Implement and verify.
7. Write a closure record.
8. Update `registry.md` and the subsystem roadmap.
9. Archive superseded interim material when appropriate.

No milestone is complete merely because code lands.
