# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `d894ae63a8963914e545a6d93dc3db92b998138c`

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
| `plans/adrs/ADR-0001-layered-mechanism-and-policy-ownership.md` | core/acquisition/service remain layered; original producer-distribution ownership is superseded by ADR-0004 |
| `plans/adrs/ADR-0002-multi-artifact-transaction-and-rollback.md` | ArtifactSet is the mutation unit; rollback and post-commit policy are explicit |
| `plans/adrs/ADR-0003-verification-layers-and-transport-neutrality.md` | integrity/authenticity/candidate identity are distinct; core has no mandatory transport |
| `plans/adrs/ADR-0004-eggpack-producer-eggup-consumer-boundary.md` | Eggpack owns producer release contracts/construction/evidence; Eggup owns local deployment; applications own release/install policy |

## Producer/consumer ownership guard

Eggup owns consumer-side acquisition, verification, candidate validation, local extraction required for installation, ownership/staging/locking/commit/rollback/recovery, install receipts, and service lifecycle. Producer release contracts, release conformance, package/archive construction, final release manifests, bootstrap-installer generation, generated release CI, staging/publication, and producer provenance belong to `eggstack/eggpack`.

`eggup-core` must remain usable without Eggpack. A future optional manifest adapter may translate stable Eggpack release evidence into Eggup deployment inputs without importing producer build/CI machinery.

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M006 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M004 | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M004 | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap predecessor | M001-M003 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch | `plans/closure/consumer-adoption/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.0 crates remain:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

`eggup-dist` remains unpublished and exists only as migration predecessor evidence. It is frozen after closed M003; no new producer functionality is authorized.

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

M001-M003 are closed predecessor work. M003 is the terminal Eggup producer-side implementation and provides the conformance behavior that Eggpack Contract M002 must port and independently qualify. The implementation is `9941c58d7039410c728860f9e4e382881d4ccf54`; closure is `4169c8021b447fe73c8ee3ea71a80a535c940f54`.

Distribution M004 is now a blocked retirement milestone, not a generator milestone: `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`. It may execute only after Eggpack Contract M002 closes.

Eggsearch M003 and service M004 Windows SCM are closed. Service M005 prepared-transaction integration is ready for plan authoring. Gregg remains gated on its corrected-path footprint evidence.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | qualified 0.1.0 | broader bundle/platform evidence later |
| Acquisition transport | corrected | M004 closed; optional M005 footprint evidence |
| Service lifecycle | M004 Windows SCM closed | M005 prepared-transaction integration ready for plan authoring |
| Distribution/bootstrap | migration-only; M003 closed/frozen | M004 retirement blocked on Eggpack Contract M002 |
| Consumer adoption | simple and eggsearch service-aware tiers closed | Gregg awaits footprint evidence |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Service lifecycle | M004 | closed | `plans/implementation/service-lifecycle/004-windows-scm-adapter.md`; `plans/closure/service-lifecycle/004-status.md` | — |
| Service lifecycle | M005 | ready for plan authoring | — | service M004 + verified-update-core M005 closed |
| Distribution/bootstrap | M004 retirement | blocked | `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md` | Eggpack Contract M002 closure |

There is no dependency-ready producer-distribution implementation work in Eggup. Distribution M004 is registered only so the duplicate authority can be removed after Eggpack closes its port. CodeGG runtime deployment planning may proceed only if it does not recreate producer target/asset/release-contract authority.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M005 lightweight/curl adapter | deferred/evidence-driven | corrected-path footprint evidence, especially Gregg |
| Service lifecycle | M005 update-lifecycle integration | ready for plan authoring | service M004 + verified-update-core M005 closed |
| Distribution/bootstrap | M004 retire `eggup-dist` | blocked | Eggpack Contract M002 closure |
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

service M003 [closed] ----------> service M004 Windows SCM [closed]
                                      |
                                      +--> service M005 orchestration plan authoring [ready]

distribution M003 [closed/frozen] ---> Eggpack Contract M002 [external required]
                                              |
                                              `--> distribution M004 retirement [blocked]

verified multi-artifact core [qualified] --> CodeGG M005 runtime plan authoring
                                              |
                                              `--> producer release mapping stays in Eggpack

Egress consumer adoption remains blocked on local archive extraction/update transaction semantics.

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

Gregg M004 remains separate: corrected Eggfetch footprint measurement -> adopt directly
                                                            \-> acquisition M005 only if justified
```

The corrective gates are closed. Eggsearch M003, distribution M003, and service M004 have reviewed closure evidence. Service M005 and CodeGG runtime deployment can proceed to plan authoring. Distribution M004 cannot execute until Eggpack Contract M002 closes, and no Eggup installer-generator replacement is authorized.

## Current project state

- Rust baseline: 1.89.
- Core: verified multi-artifact transaction/ownership/rollback boundary remains qualified.
- Acquisition: M004 validation/promotion corrective is closed; optional M005 still needs corrected-path footprint evidence.
- Service: M004 Windows SCM is closed; M005 update-lifecycle integration is ready for plan authoring.
- Distribution: M003 is closed/frozen predecessor evidence; producer authority is assigned to Eggpack; M004 retirement is blocked on Eggpack Contract M002.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg still needs its own footprint decision.
- CodeGG M005 plan authoring is unblocked by core qualification plus distribution M003 observation evidence. Egress remains blocked on archive transaction/extraction semantics.
- Release process: manual crates.io publication only.
- The previously selected lockstep 0.1.1 patch is now eligible for separate qualification/publication when directed because the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until that release exists; no publication is implicit in these plans.

## Next handoff

Next Eggup plan-authoring candidates are:

- service-lifecycle M005 prepared-transaction/update-lifecycle integration;
- CodeGG M005 runtime deployment integration, with ADR-0004 explicitly excluding producer distribution-contract work.

Distribution M004 is already registered but blocked on Eggpack Contract M002 closure. Do not author or implement an Eggup installer generator.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
