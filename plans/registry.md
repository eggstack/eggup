# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `01554361be11ea4d851610eb8aee8840d3048e01`

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
| Acquisition transport | M001-M004 | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M003 | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap | M001-M002 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo | `plans/closure/consumer-adoption/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.0 crates remain:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

`eggup-dist` exists in the workspace but remains unpublished.

Existing simple consumers:

- eggsact at `eggstack/eggsact@576f4b0`;
- stegoeggo at `eggstack/stegoeggo@10d8448`.

## Post-closure review findings

The core remains qualified with no newly identified medium-or-higher defect.

Corrective work identified after reviewing implementation SHA `889a234c` has
been closed sequentially; closure records capture verification evidence.

### Acquisition

M004 closed the remaining M003 review findings. Public `FetchLimits` values
are validated at every transport boundary, and post-link temp cleanup failure
can no longer report ordinary failure after a complete destination exists.
Details and environment limits are in
`plans/closure/acquisition-transport/004-status.md`.

### Service

Service M003 closed the post-M002 restart, deadline, executable/environment,
and config-identity findings. See
`plans/closure/service-lifecycle/003-status.md` for evidence and platform
limits.

### Distribution

M002 closed the schema-v1 findings: template grammar is strict at parse time,
and expanded release/install filename namespaces reject exact and ASCII-case
collisions. Distribution M003 validators are now dependency-ready for plan
authoring. See `plans/closure/distribution-bootstrap/002-status.md`.

Eggsearch and distribution validators are now dependency-ready for plan
authoring. Gregg remains gated on corrected-path footprint evidence.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | qualified 0.1.0 | broader bundle/platform evidence later |
| Acquisition transport | corrected | M004 closed; optional M005 footprint evidence |
| Service lifecycle | corrected | M003 closed; Windows SCM is next |
| Distribution/bootstrap | corrected | M003 validators dependency-ready for plan authoring |
| Consumer adoption | simple tier closed | eggsearch planning-ready; Gregg awaits footprint evidence |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Service lifecycle | M004 | dependency-ready / plan needed | — | service M003 closed |
| Distribution/bootstrap | M003 | dependency-ready / plan needed | — | distribution M002 closed |
| Consumer adoption | M003 | dependency-ready / plan needed | — | acquisition M004 + service M003 closed |

Distribution M003, service M004, and eggsearch M003 are dependency-ready for plan authoring.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M005 lightweight/curl adapter | deferred/evidence-driven | corrected-path footprint evidence, especially Gregg |
| Service lifecycle | M005 update-lifecycle integration | blocked / plan intentionally unwritten | service M004 + corrected core |
| Distribution/bootstrap | M003 validators | dependency-ready / plan needed | corrected schema M002 closed |
| Distribution/bootstrap | M004 generators/adoptions | blocked / plan intentionally unwritten | distribution M003 |
| Consumer adoption | M003 eggsearch | dependency-ready / plan needed | acquisition M004 + service M003 closed |
| Consumer adoption | M004 Gregg | blocked / plan intentionally unwritten | corrected-path footprint decision |
| Consumer adoption | M005 CodeGG | later / blocked | mature bundle core + distribution M003 evidence |
| Consumer adoption | M006 Egress | later / blocked | archive contract + distribution M003 evidence |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M004 corrective [closed] --+
                                       +--> eggsearch M003 planning (after service M003)
service M003 corrective [closed] --------+
                 |
                 +-------------------------> Gregg corrected-path measurement/planning
                                            (plus footprint decision)

distribution M002 corrective [closed] --> distribution M003 validator planning
                                           |
                                           +--> later CodeGG/Egress evidence
```

No new service-aware/updater-bearing consumer migration should begin until its relevant corrective gates close.

## Current project state

- Rust baseline: 1.89.
- Core: verified multi-artifact transaction/ownership/rollback boundary remains qualified.
- Acquisition: M004 validation/promotion corrective is closed; optional M005 still needs corrected-path footprint evidence.
- Service: M003 Unix adapter corrective is closed; M004 Windows SCM can be planned, and eggsearch planning is unblocked.
- Distribution: TOML v1 uniqueness/template corrective M002 is closed; M003 validators are ready for plan authoring.
- Consumer adoption: eggsact/stegoeggo remain the only completed consumers.
- Release process: manual crates.io publication only.
- Previously planned 0.1.1 publication must wait until acquisition/service corrective disposition is known; never publish known-contract defects merely to preserve the old version plan.

## Next handoff

Ready now:

- Service M004 Windows SCM implementation-plan authoring.
- Consumer M003 eggsearch adoption-plan authoring.
- Distribution M003 validator plan authoring.

Acquisition M004, distribution M002, and service M003 are closed with closure
records. After each future corrective:

1. create the matching closure record with the actual implementation SHA;
2. update the source subsystem roadmap;
3. update this registry;
4. keep downstream work blocked if any medium-or-higher issue remains;
5. only then author the newly dependency-ready downstream plan.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
