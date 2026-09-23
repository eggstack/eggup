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
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch | `plans/closure/consumer-adoption/` |

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
collisions. M003 then closed the release, archive-member, and mapping
conformance validators; see
`plans/closure/distribution-bootstrap/003-status.md` for the report, fixtures,
package, MSRV, and hosted CI evidence.

Eggsearch M003 and distribution M003 are closed. Service M004 Windows SCM is
the next plan in the authorized sequential batch. Gregg remains gated on its
own corrected-path footprint evidence.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | qualified 0.1.0 | broader bundle/platform evidence later |
| Acquisition transport | corrected | M004 closed; optional M005 footprint evidence |
| Service lifecycle | corrected | M004 Windows SCM ready for handoff |
| Distribution/bootstrap | M003 validators closed | M004 generator/adoption plan authoring |
| Consumer adoption | simple and eggsearch service-aware tiers closed | Gregg awaits footprint evidence |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Service lifecycle | M004 | **ready for handoff** | `plans/implementation/service-lifecycle/004-windows-scm-adapter.md` | service M003 closed |

The three-plan execution batch is being completed sequentially. Eggsearch M003 deliberately excludes Windows SCM migration; its closure leaves service M004 independent. Distribution M003 is closed; service M004 is next.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M005 lightweight/curl adapter | deferred/evidence-driven | corrected-path footprint evidence, especially Gregg |
| Service lifecycle | M005 update-lifecycle integration | blocked / plan intentionally unwritten | service M004 + corrected core |
| Distribution/bootstrap | M004 generators/adoptions | ready for plan authoring | distribution M003 closed |
| Consumer adoption | M004 Gregg | blocked / plan intentionally unwritten | corrected-path footprint decision |
| Consumer adoption | M005 CodeGG | ready for plan authoring | mature bundle core + distribution M003 evidence closed |
| Consumer adoption | M006 Egress | blocked | archive update/extraction transaction contract |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M004 [closed] --+
                            +--> eggsearch M003 [closed]
service M003 [closed] ------+          |
                                       +--> service-aware adoption evidence recorded

service M003 [closed] ----------> service M004 Windows SCM [ready]
                                      |
                                      +--> service M005 orchestration planning

distribution M002 [closed] -----> distribution M003 validators [closed]
                                      |
                                      +--> distribution M004 plan authoring
                                      +--> CodeGG M005 plan authoring
                                      `--> Egress evidence improved; archive contract remains blocked

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

Gregg M004 remains separate: corrected Eggfetch footprint measurement -> adopt directly
                                                            \-> acquisition M005 only if justified
```

The corrective gates are closed. Eggsearch M003 and distribution M003 have reviewed closure evidence. Service M004 is next in the active batch; distribution M004 and CodeGG M005 can proceed to plan authoring.

## Current project state

- Rust baseline: 1.89.
- Core: verified multi-artifact transaction/ownership/rollback boundary remains qualified.
- Acquisition: M004 validation/promotion corrective is closed; optional M005 still needs corrected-path footprint evidence.
- Service: M003 Unix adapter corrective is closed; M004 Windows SCM is ready for handoff.
- Distribution: M003 conformance validators are closed; M004 plan authoring is ready.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg still needs its own footprint decision.
- CodeGG M005 plan authoring is unblocked by core qualification plus distribution M003 observation evidence. Egress remains blocked on archive transaction/extraction semantics.
- Release process: manual crates.io publication only.
- The previously selected lockstep 0.1.1 patch is now eligible for separate qualification/publication when directed because the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until that release exists; no publication is implicit in these plans.

## Next handoff

Remaining plan in the active implementation batch:

- `plans/implementation/service-lifecycle/004-windows-scm-adapter.md`

Planning baseline for this batch: `4495df6241b3fac9e396553727cf8d3d497ff3cd`.
Eggsearch refreshed evidence baseline: `eggstack/eggsearch@68ae2fa5457c3fb8fa335851d60d0dce3e98aa08`; implementation closure: `plans/closure/consumer-adoption/003-status.md`.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
