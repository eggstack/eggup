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

## Recently closed work (this wave)

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M006 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M002 plus M003 corrective | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001 contract plus M002 Unix adapters | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap | M001 schema | `plans/closure/distribution-bootstrap/001-status.md` |
| Consumer adoption | M001 eggsact + M002 stegoeggo | `plans/closure/consumer-adoption/` |

Published 0.1.0 crates:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

New (unpublished) workspace crate:

- `eggup-dist` (release-time tooling; never a runtime dependency)

Two independent consumers are live on the shared simple-update path without an Eggup API fork:

- eggsact at `eggstack/eggsact@576f4b0`;
- stegoeggo at `eggstack/stegoeggo@10d8448`.

## Current post-adoption findings

The 0.1.0 core safety corrective remains closed with no known medium-or-higher core defect.

Acquisition M003 corrective is closed with no medium-or-higher issue remaining:

- effective `min(request, adapter ceiling)` timeouts via Eggfetch request overrides;
- exclusive owner-private temp files with no-clobber promotion;
- category-only diagnostics (no upstream/proxy secret echo).

Eggsact (25 updater tests) and stegoeggo (33 updater tests) stay green against the corrective. The broader updater-bearing migration gate is lifted (service/distribution gates still apply per plan).

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | qualified 0.1.0 | broader bundle/platform evidence later |
| Acquisition transport | M001-M003 closed | optional M004 lightweight (evidence-driven) |
| Service lifecycle | M001-M002 closed | M003 Windows SCM (planning-ready) |
| Distribution/bootstrap | M001 closed | M002 validators (planning-ready) |
| Consumer adoption | simple tier closed | eggsearch M003 planning-ready; Gregg measurement-ready |

## Dependency-ready implementation work

No ready-for-handoff implementation plan remains from this wave; all three are closed (see above). Newly planning-ready work (plans intentionally unwritten until this evidence) is tracked below and may now be authored — but no downstream plan is authored in this pass.

## Planned / newly unblocked work

| Subsystem | Milestone | State | Blocker / planning rule |
|---|---|---|---|
| Acquisition transport | M004 lightweight/curl adapter | deferred/evidence-driven | remeasure corrected Eggfetch path, especially Gregg (gate lifted; measurement may proceed) |
| Service lifecycle | M003 Windows SCM | planning-ready (unblocked by M002) | detailed plan may now be authored; do not freeze surface without M002 evidence review |
| Service lifecycle | M004 update-lifecycle integration | plan intentionally unwritten | service M003 + core (M002 closed) |
| Distribution/bootstrap | M002 validators | planning-ready (unblocked by M001) | detailed plan may now be authored against stable v1 |
| Distribution/bootstrap | M003 generators/adoptions | plan intentionally unwritten | distribution M002 |
| Consumer adoption | M003 eggsearch | planning-ready (gates lifted: acquisition M003 + service M002 closed) | detailed plan may now be authored; no migration begins without its own plan/closure |
| Consumer adoption | M004 Gregg | measurement-ready (acquisition gate lifted) | corrected-path footprint decision still required before planning |
| Consumer adoption | M005 CodeGG | later | mature bundle + distribution M002 evidence |
| Consumer adoption | M006 Egress | later | archive/distribution M002 evidence |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M003 (closed) --+
                            +--> eggsearch M003 planning (now allowed)
service M002 (closed) ------+
                            +--> Gregg corrected-path measurement (now allowed)

distribution M001 (closed) --> distribution M002 planning (now allowed)
                           --> later CodeGG/Egress contract evidence
```

## Current project state

- Rust baseline: 1.89.
- Workspace crates: four published 0.1.0 crates listed above plus new unpublished `eggup-dist`.
- Hosted CI at implementation baseline `8f6ce48`: stable checks, MSRV 1.89, macOS tests, and Windows check all passed.
- Core: verified multi-artifact transaction, ownership proof, staged digest revalidation, rollback/recovery receipts.
- Acquisition: M003 corrective closed; eggsact/stegoeggo green; no medium-or-higher issue.
- Service: M002 Unix adapters closed; M001 contract intact; no consumer migrated.
- Distribution: M001 schema closed (TOML v1); no runtime depends on `eggup-dist`.
- Consumer adoption: eggsact/stegoeggo closed; eggsearch/Gregg planning gates lifted (no migration started).
- Release process: manual crates.io publication only. Version decision: next release is workspace lockstep 0.1.1 patch when directed (seam-then-adapter order). No publication performed.

## Next handoff

Downstream planning now allowed (no implementation plan authored in this pass):

- `Distribution M002 validators` (against stable v1 schema);
- `Service M003 Windows SCM` (against M002 adapter evidence);
- `Consumer eggsearch M003` planning (gates lifted);
- Gregg corrected-path footprint remeasurement (evidence for M004 decision).

After each implementation pass:

1. create the matching closure record;
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective instead of closing work with a medium-or-higher unresolved finding;
5. only then author the newly dependency-ready downstream implementation plan.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and the next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
