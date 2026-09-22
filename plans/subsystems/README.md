# Subsystem Roadmaps

Subsystem roadmaps translate Eggup's canonical architecture into coherent dependency-aware workstreams. They are not direct coding-agent checklists.

## Naming

```text
<subsystem>-roadmap.md
```

## Required structure

Each roadmap should contain:

1. purpose and ownership boundary;
2. work classification: invariants, capabilities, infrastructure, polish;
3. non-goals;
4. current-state evidence;
5. target architecture;
6. dependency graph with hard/interface/soft/operational classifications;
7. ordered milestones;
8. cross-cutting storage/protocol/security/recovery/performance/docs requirements;
9. verification strategy;
10. risks and decision points;
11. completion definition;
12. milestone status table.

Roadmaps MUST preserve completed history and MUST link active milestones to implementation and closure records.

## Initial Eggup subsystems

- `verified-update-core-roadmap.md`
- `acquisition-transport-roadmap.md`
- `service-lifecycle-roadmap.md`
- `distribution-bootstrap-roadmap.md`
- `consumer-adoption-roadmap.md`

Do not create new subsystems merely to split files. Create one when ownership, dependencies, or closure criteria are meaningfully distinct.
