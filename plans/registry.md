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
| Core M003 transaction/rollback | closed; post-closure findings fed M005 | `plans/closure/verified-update-core/003-status.md` |
| Core M004 integrity/candidate validation | closed; post-closure findings fed M005 | `plans/closure/verified-update-core/004-status.md` |
| Core M005 safety/API corrective | closed | `plans/closure/verified-update-core/005-status.md` |
| Core M006 package qualification | closed | `plans/closure/verified-update-core/006-status.md` |

The historical records remain valid evidence of what was implemented and tested at those points. M005 corrects the pre-adoption contract defects found after M002-M004.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | M005-M006 closed; qualified | broader consumer API qualification after first adoptions |
| Acquisition transport | M001-M002 closed | lightweight M003 only if footprint evidence requires |
| Service lifecycle | M001 closed | M002 Unix adapters (no detailed plan yet) |
| Distribution/bootstrap | proposed / later phase | wait for first consumer evidence (two adopters now live) |
| Consumer adoption | M001-M002 closed | eggsearch/Gregg per roadmap (no detailed plans yet) |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| *(none — all seven handed-off milestones closed)* | — | — | — | — |

## Planned but blocked implementation work

| Subsystem | Milestone | Status | Plan | Blocker |
|---|---|---|---|---|
| *(none active)* | — | — | — | — |

## Core M005 corrective scope (closed)

M005 was the release/adoption gate for the post-M004 review findings and is now closed (`plans/closure/verified-update-core/005-status.md`):

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

Do not begin package publication or consumer migration until M006 qualifies the corrected API.

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
- Production Rust workspace: core M001-M006 + acquisition M001-M002 + service M001 implemented, verified (72 tests: 37 core + 12 acquisition + 13 eggfetch + 10 service, stable + 1.89), packages qualified and published.
- Hosted CI: lanes defined for stable, MSRV 1.89, macOS, and Windows-check; local verification green.
- Published crates: `eggup-acquisition 0.1.0`, `eggup-core 0.1.0`, `eggup-service 0.1.0`, `eggup-eggfetch 0.1.0` (manual `cargo publish`, seam → core/service → adapter order).
- Release process: manual publication only; no automation.
- Consumer integrations: `eggsact` (`eggstack/eggsact@576f4b0`, +1.7%) and `stegoeggo` (`eggstack/stegoeggo@10d8448`, +2.9%) adopted on Eggup 0.1.0; single stack each; full suites green; no eggsact-specific leakage (only argv/expectation/bound config differs).
- Active work: none — all seven handed-off milestones closed. Next: eggsearch/Gregg/bundle tiers per roadmaps (no detailed plans yet, by discipline).

## Next handoff

Hand (any order; service M001 may run in parallel):

- `plans/implementation/verified-update-core/006-core-package-qualification.md`
- `plans/implementation/acquisition-transport/001-acquisition-seam-and-fixture-transport.md`
- `plans/implementation/service-lifecycle/001-manager-neutral-state-and-ownership.md`

After each implementation:

1. create the matching closure record;
2. reconcile the source subsystem roadmap and this registry;
3. only then mark dependent milestones ready (M002 after transport M001; adoption M001 after core M006 + transport M002; adoption M002 after adoption M001);
4. if any medium-or-higher safety/API issue remains, write a new corrective rather than beginning adoption.

## Registry update rule

After each implementation pass:

1. create the matching closure record;
2. update the source subsystem status table;
3. update this registry;
4. write a corrective plan instead of marking partial work closed;
5. only then mark dependent milestones ready.
