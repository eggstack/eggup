# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `8f6ce48cda5bdeb593939077bcca452cdd5f2800`

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
| `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md` | core owns local deployment mechanism; transport, service, distribution, and consumer release policy remain separate |
| `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet is the mutation unit; rollback and post-commit policy are explicit |
| `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md` | integrity/authenticity/candidate identity are distinct; core has no mandatory transport |

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M006 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M002 | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001 manager-neutral contract | `plans/closure/service-lifecycle/001-status.md` |
| Consumer adoption | M001 eggsact + M002 stegoeggo | `plans/closure/consumer-adoption/` |

Published 0.1.0 crates:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

Two independent consumers are live on the shared simple-update path without an Eggup API fork:

- eggsact at `eggstack/eggsact@576f4b0`;
- stegoeggo at `eggstack/stegoeggo@10d8448`.

## Current post-adoption findings

The 0.1.0 core safety corrective remains closed with no known medium-or-higher core defect.

Acquisition post-adoption review identified a new corrective gate:

- per-request `FetchLimits` timeout values are not currently authoritative in the Eggfetch adapter;
- artifact temp files are generated but not exclusively created;
- artifact promotion relies on platform rename behavior when the destination already exists/races in;
- upstream transport/proxy error detail requires an explicit credential-redaction audit.

These are tracked by acquisition M003 and gate additional updater-bearing consumer migrations.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | qualified 0.1.0 | broader bundle/platform evidence later |
| Acquisition transport | active corrective | M003 contract/temp-file hardening |
| Service lifecycle | active | M002 Unix manager adapters |
| Distribution/bootstrap | active planning | M001 versioned DistributionContract schema |
| Consumer adoption | simple tier closed | eggsearch/Gregg after shared-layer gates |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Acquisition transport | M003 | **ready for handoff — primary gate** | `plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md` | M001-M002 closed |
| Service lifecycle | M002 | **ready for handoff** | `plans/implementation/service-lifecycle/002-unix-manager-adapters.md` | service M001 closed |
| Distribution/bootstrap | M001 | **ready for handoff** | `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md` | first-consumer evidence satisfied |

Service M002 and distribution M001 may run in parallel with acquisition M003. Additional updater-bearing consumer migration must wait for acquisition M003 closure.

## Planned but blocked / intentionally unwritten work

| Subsystem | Milestone | State | Blocker / planning rule |
|---|---|---|---|
| Acquisition transport | M004 lightweight/curl adapter | deferred/evidence-driven | remeasure corrected Eggfetch path, especially Gregg |
| Service lifecycle | M003 Windows SCM | plan intentionally unwritten | use M002 adapter evidence before freezing platform surface |
| Service lifecycle | M004 update-lifecycle integration | plan intentionally unwritten | service M002/M003 + core |
| Distribution/bootstrap | M002 validators | plan intentionally unwritten | distribution M001 schema closure |
| Distribution/bootstrap | M003 generators/adoptions | plan intentionally unwritten | distribution M002 |
| Consumer adoption | M003 eggsearch | blocked / unwritten | acquisition M003 + service M002 |
| Consumer adoption | M004 Gregg | blocked / unwritten | acquisition M003 + corrected-path footprint decision |
| Consumer adoption | M005 CodeGG | later | mature bundle + distribution evidence |
| Consumer adoption | M006 Egress | later | archive/distribution contract evidence |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
                         +--> service M002 Unix adapters --------+
                         |                                       |
acquisition M003 --------+--> broader consumer migration gate    +--> eggsearch M003 planning
(primary corrective)     |                                       |
                         +--> Gregg corrected-path measurement ---+--> Gregg M004 planning
                         
distribution M001 schema --> distribution M002 planning
                         --> later CodeGG/Egress contract evidence
```

The three current plans are independent enough to execute concurrently, but acquisition M003 remains the safety gate for any new updater-bearing consumer adoption.

## Current project state

- Rust baseline: 1.89.
- Workspace crates: four published 0.1.0 crates listed above.
- Hosted CI at implementation baseline `8f6ce48`: stable checks, MSRV 1.89, macOS tests, and Windows check all passed.
- Core: verified multi-artifact transaction, ownership proof, staged digest revalidation, rollback/recovery receipts.
- Acquisition: published and proven by two consumers; M003 corrective pending.
- Service: manager-neutral contract published; real Unix adapters pending.
- Distribution: no crate yet; M001 schema is now evidence-ready.
- Consumer adoption: eggsact/stegoeggo closed; no second-tier migration should begin before the relevant gates close.
- Release process: manual crates.io publication only.

## Next handoff

Primary:

`plans/implementation/acquisition-transport/003-contract-and-tempfile-hardening-corrective.md`

Safe parallel handoffs:

- `plans/implementation/service-lifecycle/002-unix-manager-adapters.md`
- `plans/implementation/distribution-bootstrap/001-distribution-contract-schema.md`

After each implementation pass:

1. create the matching closure record;
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective instead of closing work with a medium-or-higher unresolved finding;
5. only then author the newly dependency-ready downstream implementation plan.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and the next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
