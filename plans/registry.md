# Eggup Planning Registry

Status: active

Last planning baseline reviewed: `d4f3459`

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
| Verified update core | active | M005 package qualification (plan not written) |
| Acquisition transport | proposed / ready to plan | M001 after core M002 interface |
| Service lifecycle | proposed / ready to plan | M001 after core M003 |
| Distribution/bootstrap | proposed / later phase | M001 after first consumer adoption evidence |
| Consumer adoption | proposed / blocked | eggsact + stegoeggo after core M003/M004 and transport M002 |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Verified update core | M005 | **ready to plan; implementation plan not written** | — | M001-M004 closed |

## Planned but blocked implementation work

| Subsystem | Milestone | Status | Plan | Blocker |
|---|---|---|---|---|
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
- Production Rust workspace: verified local core M001-M004 complete; transport, services, and consumer integrations remain unimplemented.
- Published crates: none.
- Release process: none.
- Consumer integrations: none.
- Closure records: M001-M004 closed.
- Corrective plans: none.

## Next handoff

Hand only this plan to the next implementation agent:

No next core implementation plan is written yet; M005 is ready for planning.

M001-M004 are closed. Acquisition transport and service lifecycle planning are
unblocked at their interface gates; consumer adoption remains blocked on
transport work.

## Registry update rule

After each implementation pass:

1. create the matching closure record;
2. update the source subsystem status table;
3. update this registry;
4. write a corrective plan instead of marking partial work closed;
5. only then mark dependent milestones ready.
