# Milestone Implementation Plans

Implementation plans are bounded coding-agent handoff artifacts.

## Layout

```text
implementation/<subsystem>/NNN-short-title.md
```

## Required template

```markdown
# <Subsystem> Milestone NNN — <Title>

Status: ready for handoff | active | blocked | implemented | superseded

Repository baseline: <commit SHA>

Source roadmap:
- plans/subsystems/<subsystem>-roadmap.md

Long-term requirements:
- plans/000-long-term-specification.md
- plans/001-terminology-and-domain-model.md

Applicable ADRs:
- plans/adrs/ADR-NNNN-...

Primary class: invariant | capability | infrastructure | polish

## 1. Objective
## 2. Why this milestone is ready
## 3. Current implementation evidence
## 4. Invariants that must not regress
## 5. Scope
### In scope
### Explicitly out of scope
## 6. Required production changes
## 7. Ordered work packages
## 8. Failure, cancellation, restart, and contention semantics
## 9. Compatibility and migration
## 10. Required tests
## 11. Required verification commands
## 12. Documentation updates
## 13. Acceptance criteria
## 14. Stop conditions
## 15. Closure evidence required
## 16. Handoff notes
```

Plans must state exact safety/failure semantics for any artifact mutation or external process behavior. A consumer-facing migration plan must identify the old implementation that can be deleted after successful adoption.
