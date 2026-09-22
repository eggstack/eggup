# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `9f527beb20da585bc3cf56f45f1fd96fd418c124`

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

## Historical core closure state

| Milestone | Historical status | Closure |
|---|---|---|
| Core M001 foundation | closed | `plans/closure/verified-update-core/001-status.md` |
| Core M002 domain/preparation | closed | `plans/closure/verified-update-core/002-status.md` |
| Core M003 transaction/rollback | closed; post-closure findings feed M005 | `plans/closure/verified-update-core/003-status.md` |
| Core M004 integrity/candidate validation | closed; post-closure findings feed M005 | `plans/closure/verified-update-core/004-status.md` |

The historical records remain valid evidence of what was implemented and tested at those points. They do not override the later pre-adoption safety findings recorded in M005.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | active corrective | M005 safety/API corrective |
| Acquisition transport | plans written / blocked | M001 after core M005 |
| Service lifecycle | first plan written / blocked | M001 after core M005 |
| Distribution/bootstrap | proposed / later phase | wait for first consumer evidence |
| Consumer adoption | first two plans written / blocked | eggsact after core M006 + transport M002 |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Verified update core | M005 | **ready for handoff** | `plans/implementation/verified-update-core/005-prequalification-safety-and-api-corrective.md` | M001-M004 historical closure complete |

## Planned but blocked implementation work

| Subsystem | Milestone | Status | Plan | Blocker |
|---|---|---|---|---|
| Verified update core | M006 | blocked | `plans/implementation/verified-update-core/006-core-package-qualification.md` | core M005 |
| Acquisition transport | M001 | blocked | `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md` | core M005 |
| Acquisition transport | M002 | blocked | `plans/implementation/acquisition-transport/002-eggfetch-adapter.md` | transport M001 |
| Service lifecycle | M001 | blocked | `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md` | core M005 |
| Consumer adoption | M001 eggsact | blocked | `plans/implementation/consumer-adoption/001-eggsact-first-adoption.md` | core M006 + transport M002 |
| Consumer adoption | M002 stegoeggo | blocked | `plans/implementation/consumer-adoption/002-stegoeggo-second-adoption.md` | adoption M001 |

## Core M005 corrective scope

M005 is the release/adoption gate for the post-M004 review findings:

- explicit `Absent | Owned | Foreign | Unknown` destination ownership;
- no unchecked destination-parent creation;
- final staged digest/metadata revalidation under lock;
- removal of unenforced authenticity-required state;
- correction of cleanup-vs-post-commit policy naming;
- structured transaction failure phase/cause;
- real recovery-path reporting;
- private transaction-owned permissions;
- truthful stale-lock support/documentation;
- current capability documentation.

Do not begin package publication or consumer migration until M005 closes.

## Immediate execution order

```text
core M005 safety/API corrective
        |
        +-------------------------+
        |                         |
        v                         v
core M006 package          acquisition M001 seam
qualification                     |
                                  v
                           acquisition M002 Eggfetch
        |                         |
        +------------+------------+
                     |
                     v
              eggsact adoption M001
                     |
                     v
             stegoeggo adoption M002
                     |
                     v
       broader consumer API qualification
                     |
          +----------+-----------+
          |                      |
          v                      v
  eggsearch/Gregg          CodeGG/Egress later
```

Service-lifecycle M001 may begin after core M005 in parallel with core M006/acquisition, but real manager adapters should wait until its manager-neutral ownership model closes.

## Deferred implementation-plan authoring

The following roadmap work intentionally has no detailed implementation plan yet:

- service-lifecycle M002-M004;
- optional lightweight/curl acquisition M003;
- eggsearch adoption;
- Gregg adoption;
- CodeGG adoption;
- Egress adoption;
- EggPool selective adoption;
- distribution/bootstrap M001-M003;
- authenticity/signature support.

These depend on evidence from the corrected core and first two consumers. Writing detailed handoffs now would violate the planning rule against speculative implementation plans.

## Current project state

- Planning system: established.
- Production Rust workspace: core M001-M004 implemented.
- Hosted CI: current reviewed implementation HEAD passed stable checks and Rust 1.89 MSRV.
- Published crates: none.
- Release process: none.
- Consumer integrations: none.
- Active corrective: core M005.
- Next package qualification: core M006, blocked.
- Transport implementation: planned, blocked.
- Service implementation: first contract plan written, blocked.
- First consumer migrations: planned, blocked.

## Next handoff

Hand only:

`plans/implementation/verified-update-core/005-prequalification-safety-and-api-corrective.md`

After M005 implementation:

1. create `plans/closure/verified-update-core/005-status.md`;
2. reconcile the verified-update-core roadmap and this registry;
3. if closed, unblock M006, acquisition M001, and service-lifecycle M001;
4. if any medium-or-higher safety/API issue remains, write a new corrective rather than beginning adoption.

## Registry update rule

After each implementation pass:

1. create the matching closure record;
2. update the source subsystem status table;
3. update this registry;
4. write a corrective plan instead of marking partial work closed;
5. only then mark dependent milestones ready.
