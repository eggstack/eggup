# Eggup Planning Registry

Status: active

Last planning baseline reviewed: `5b6cf13ed7e6e2bd40951216f29ea68d7333aadf`

This file is the compact control surface for active Eggup planning. Detailed requirements live in the linked plans and roadmaps.

## Canonical documents

| Document | Status |
|---|---|
| `plans/000-long-term-specification.md` | normative |
| `plans/001-terminology-and-domain-model.md` | normative |
| `plans/002-long-term-roadmap.md` | active |
| `plans/003-planning-process.md` | normative |

## Accepted architecture decisions

| ADR | Decision |
|---|---|
| `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md` | eggup-core owns local deployment mechanism; transport, service, distribution, and consumer release policy remain separate |
| `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet is the mutation unit; rollback and post-commit policy are explicit |
| `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md` | integrity/authenticity/candidate identity are distinct; core has no mandatory transport |

## Active subsystem roadmaps

| Subsystem | Status | Next dependency-ready milestone |
|---|---|---|
| Verified update core | active | M001 repository/workspace foundation |
| Acquisition transport | proposed / blocked | M001 after core M002 interface |
| Service lifecycle | proposed / blocked | M001 after core M003 |
| Distribution/bootstrap | proposed / later phase | M001 after first consumer adoption evidence |
| Consumer adoption | proposed / blocked | eggsact + stegoeggo after core M003/M004 and transport M002 |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Verified update core | M001 | **ready for handoff** | `plans/implementation/verified-update-core/001-repository-workspace-foundation.md` | none |

## Planned but blocked implementation work

| Subsystem | Milestone | Status | Plan | Blocker |
|---|---|---|---|---|
| Verified update core | M002 | blocked | `plans/implementation/verified-update-core/002-domain-and-prepared-transaction.md` | M001 closure |
| Verified update core | M003 | blocked | `plans/implementation/verified-update-core/003-transaction-commit-rollback.md` | M002 closure |
| Verified update core | M004 | blocked | `plans/implementation/verified-update-core/004-integrity-and-candidate-validation.md` | M002 closure |
| Acquisition transport | M001 | blocked | not yet written | stable core M002 stage/acquisition interface |
| Service lifecycle | M001 | blocked | not yet written | stable core transaction semantics |
| Consumer adoption | eggsact / stegoeggo | blocked | not yet written | core transaction + verification + Eggfetch adapter |

## External interface motivation

CodeGG currently records a blocked generic-updater milestone because no independently consumable generalized updater package exists outside CodeGG. Eggup is intended to satisfy that interface only after its core has been proven against simpler consumers.

Do not make CodeGG the first consumer merely to clear that blocker. The first adoption gate is eggsact/stegoeggo so accidental CodeGG bundle assumptions do not define the library prematurely.

## Immediate execution order

```text
core M001 foundation
      |
      v
core M002 domain/preparation
      |
      +------------------+
      |                  |
      v                  v
core M003 transaction  core M004 verification
      |                  |
      +---------+--------+
                |
                v
acquisition M001/M002
                |
        +-------+-------+
        |               |
        v               v
    eggsact          stegoeggo
        \               /
         \             /
          v           v
        API qualification
                |
      later service/bundle consumers
```

## Current project state

- Planning system: established.
- Production Rust workspace: absent by design; owned by core M001.
- Published crates: none.
- Release process: none.
- Consumer integrations: none.
- Closure records: none.
- Corrective plans: none.

## Next handoff

Hand only this plan to the first implementation agent:

`plans/implementation/verified-update-core/001-repository-workspace-foundation.md`

Do not hand M002-M004 concurrently before M001 closure because their repository baselines and concrete module layout must be reconciled against what M001 actually establishes.

## Registry update rule

After each implementation pass:

1. create the matching closure record;
2. update the source subsystem status table;
3. update this registry;
4. write a corrective plan instead of marking partial work closed;
5. only then mark dependent milestones ready.
