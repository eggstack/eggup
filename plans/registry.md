# Eggup Planning Registry

Status: active

Last implementation baseline reviewed: `19935ec3610a5238af33a9d4f05a14925ceac25c` (Eggpack manifest adapter M001a qualification corrective; no production-code delta)

Plan-authoring baseline for Eggpack Manifest Interoperability M003: `77fe72a9f6e34e48e1f667d2869b05fd010c1462`

Latest closure/planning evidence baseline reviewed before M003 authoring: `77fe72a9f6e34e48e1f667d2869b05fd010c1462` (adapter M001a closure/status reconciliation)

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

`eggup-core` must remain usable without Eggpack. The optional manifest adapter translates stable Eggpack release evidence into Eggup deployment inputs without importing producer build/CI machinery. Eggpack interoperability M001a is closed at `eggstack/eggpack@678bbf04f5a02827003a1d9ab83ba4f0e6360e41`; Eggup adapter M001 is historically closed and M001a qualification/closure hardening is closed at `plans/closure/eggpack-manifest-interoperability/001a-status.md`. Real-consumer M003 plan authoring is now dependency-ready.

## Recently closed foundation

| Workstream | Closed work | Evidence |
|---|---|---|
| Verified update core | M001-M006 | `plans/closure/verified-update-core/` |
| Acquisition transport | M001-M004 | `plans/closure/acquisition-transport/` |
| Service lifecycle | M001-M004 | `plans/closure/service-lifecycle/` |
| Distribution/bootstrap (archived/transferred) | M001-M004 | `plans/closure/distribution-bootstrap/` |
| Consumer adoption | M001 eggsact + M002 stegoeggo + M003 eggsearch | `plans/closure/consumer-adoption/` |

Implementation wave `889a234c` added/closed acquisition M003, service M002, and distribution M001. Hosted CI at that exact SHA passed stable fmt/clippy/test/doc, Rust 1.89 check, macOS tests, and Windows workspace check.

Published 0.1.0 crates remain:

- `eggup-acquisition`
- `eggup-core`
- `eggup-eggfetch`
- `eggup-service`

`eggup-dist` was unpublished and existed only as migration predecessor evidence. It was removed after Eggpack Contract M002 qualified the complete M003 behavior; see `plans/closure/distribution-bootstrap/004-status.md`.

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

M001-M003 are closed predecessor work. M003 is the terminal Eggup producer-side implementation and provides the conformance behavior Eggpack Contract M002 ported and independently qualified. The implementation is `9941c58d7039410c728860f9e4e382881d4ccf54`; closure is `4169c8021b447fe73c8ee3ea71a80a535c940f54`.

Distribution M004 removed `eggup-dist` after Eggpack Contract M002 closed at `82f799f3d971b2999ac14c2d8fc1b965370e0f58`; see `plans/closure/distribution-bootstrap/004-status.md`. The subsystem is archived/transferred, with no active Eggup producer-distribution milestone.

Eggsearch M003, Service M004, Verified Update Core M007, and Service M005 are closed. M005 provides reusable lifecycle/transaction composition, but no service-aware consumer migration is currently scheduled. Gregg remains gated on its corrected-path footprint evidence; Service M005 does not affect that gate.

## Active subsystem roadmaps

| Subsystem | Status | Next milestone |
|---|---|---|
| Verified update core | M001-M007 closed/qualified | — |
| Acquisition transport | corrected | M004 closed; optional M005 footprint evidence |
| Service lifecycle | M001-M005 closed | no service-aware consumer migration scheduled |
| Distribution/bootstrap | archived/transferred; M001-M004 closed | no further Eggup producer work |
| Eggpack manifest interoperability | active; M003 plan registered | M003 Eggsact adoption is producer-evidence gated |
| Consumer adoption | simple, eggsearch, and CodeGG M005 closed | Gregg remains footprint-gated |

## Dependency-ready implementation work

| Subsystem | Milestone | Status | Plan | Dependencies |
|---|---|---|---|---|
| Planning/closure hygiene corrective | C001 post-batch status and evidence reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/001-post-batch-status-and-evidence-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/001-status.md` | CodeGG M005 + core M007 closed |
| Planning/closure hygiene corrective | C002 C001 commit + registry baseline reconciliation | closed | `plans/implementation/planning-closure-hygiene-corrective/002-c001-commit-and-registry-baseline-reconciliation.md`; `plans/closure/planning-closure-hygiene-corrective/002-status.md` | — |
| Verified update core | M007 post-commit policy / deferred finalization | closed | `plans/implementation/verified-update-core/007-post-commit-policy-and-deferred-finalization.md`; `plans/closure/verified-update-core/007-status.md` | — |
| Consumer adoption | M005 CodeGG managed-runfile bundle | closed | `plans/implementation/consumer-adoption/005-codegg-managed-runfile-bundle-adoption.md` | `plans/closure/consumer-adoption/005-status.md` |
| Service lifecycle | M005 prepared-transaction lifecycle integration | closed | `plans/implementation/service-lifecycle/005-prepared-transaction-lifecycle-integration.md`; `plans/closure/service-lifecycle/005-status.md` | — |
| Distribution/bootstrap | M004 retirement | closed | `plans/implementation/distribution-bootstrap/004-retire-eggup-dist-authority.md`; `plans/closure/distribution-bootstrap/004-status.md` | — |
| Eggpack manifest interoperability | M001 ReleaseManifest v1 direct/bundle adapter | closed (historical) | `plans/implementation/eggpack-manifest-interoperability/001-release-manifest-v1-adapter.md` | `plans/closure/eggpack-manifest-interoperability/001-status.md`; post-closure findings tracked by M001a |
| Eggpack manifest interoperability | M001a adapter qualification + closure hardening | closed | `plans/implementation/eggpack-manifest-interoperability/001a-adapter-qualification-and-closure-hardening-corrective.md` | `plans/closure/eggpack-manifest-interoperability/001a-status.md`; full regression matrix qualified |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | planned / producer-evidence gated | `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md` | preflight current Eggsact release naming + producer-owned manifest artifact convention; adapter/API fixture qualification may proceed without inventing producer policy |

CodeGG M005, Verified Update Core M007, and Service Lifecycle M005 are closed. Planning/closure hygiene C001 and C002 are closed. M005 completes the reusable service-aware transaction mechanism; no downstream consumer plan is newly unblocked because Eggsearch M003 is already closed and Gregg M004 remains footprint-gated. Eggpack Interop M001a is closed with the full adapter regression matrix. M003 now has a registered Eggsact real-consumer adoption plan; execution must honor its producer-evidence preflight before changing the normal hosted-release path. There is no dependency-ready producer-distribution implementation work in Eggup; that subsystem is archived/transferred.

## Planned / blocked work

| Subsystem | Milestone | State | Blocker |
|---|---|---|---|
| Acquisition transport | M005 lightweight/curl adapter | deferred/evidence-driven | corrected-path footprint evidence, especially Gregg |
| Planning/closure hygiene corrective | C001 post-batch status/evidence reconciliation | closed | `plans/closure/planning-closure-hygiene-corrective/001-status.md` |
| Planning/closure hygiene corrective | C002 final commit/baseline bookkeeping | closed | `plans/closure/planning-closure-hygiene-corrective/002-status.md` |
| Service lifecycle | M005 prepared-transaction lifecycle integration | closed | `plans/closure/service-lifecycle/005-status.md` |
| Distribution/bootstrap | M004 retire `eggup-dist` | closed | `plans/closure/distribution-bootstrap/004-status.md` |
| Consumer adoption | M004 Gregg | blocked / plan intentionally unwritten | corrected-path footprint decision |
| Consumer adoption | M005 CodeGG | closed | `plans/closure/consumer-adoption/005-status.md`; producer release mapping remains application/Eggpack-owned |
| Consumer adoption | M006 Egress | blocked | archive update/extraction transaction contract |
| Consumer adoption | M007 EggPool selective | deferred | broader core maturity |
| Eggpack manifest interoperability | M003 Eggsact real-consumer manifest adoption | planned / producer-evidence gated | live Eggsact asset naming and a producer-owned ReleaseManifest artifact convention must be reconciled first; do not invent either in Eggup |
| Authenticity/signatures | future | ADR required | trust standard not selected |

## Immediate execution graph

```text
acquisition M004 [closed] --+
                            +--> eggsearch M003 [closed]
service M003 [closed] ------+          |
                                       +--> service-aware adoption evidence recorded

service M003 [closed] ----------> service M004 Windows SCM [closed]
                                      |
                                      +--------------------------+
                                                                 |
core M006 [closed] --> core M007 post-commit policy [CLOSED] ---+
                                                                 |
CodeGG M005 [closed] ---------------------------------------------+--> planning/closure hygiene C001 [CLOSED]
                                                                           |
`--> service M005 implementation [CLOSED]

distribution M003 [closed/frozen] ---> Eggpack Contract M002 [closed]
                                              |
                                              `--> distribution M004 retirement [closed; subsystem archived]

Eggpack Interop M001a [CLOSED; corrected fixtures]
            |
            `--> eggup-eggpack adapter M001 [CLOSED HISTORICALLY]
                    |
                    v
              adapter M001a corrective [CLOSED]
                    |
                    `--> M003 Eggsact real-consumer adoption [PLAN REGISTERED; PRODUCER-EVIDENCE GATE]

core M006 + acquisition M004 [closed] --> CodeGG M005 managed bundle [CLOSED]
                                            |
                                            `--> producer release mapping stays in application/Eggpack; strict archive extraction stays CodeGG-owned

Egress consumer adoption remains blocked on local archive extraction/update transaction semantics.

consumer acquisition M004 + service M003 --> eggsearch M003 [closed]
                                                  |
                                                  +--> service-aware consumer evidence recorded

Gregg M004 remains separate: corrected Eggfetch footprint measurement -> adopt directly
                                                            \-> acquisition M005 only if justified
```

The runtime corrective gates are closed. Eggsearch M003, distribution M003/M004, service M004/M005, CodeGG M005, and Verified Update Core M007 have reviewed closure evidence. Planning/closure hygiene C001/C002 are closed. No service-aware consumer migration is currently scheduled; Gregg remains footprint-gated. Eggpack manifest consumption is structurally implemented, and adapter M001a has closed with the full regression matrix and truthful service packageability reconciliation. Historical M001 remains closed evidence; lower Eggup crates remain Eggpack-independent. M003 is registered for Eggsact and is producer-evidence gated as described by its implementation plan. Archive work still requires the Phase 10 extraction contract. No Eggup installer-generator replacement is authorized.

## Current project state

- Rust baseline: 1.89.
- Core: M001-M007 are qualified; M007 adds the accepted post-commit `KeepInstalled | RollBack` boundary without service coupling. Closure: `plans/closure/verified-update-core/007-status.md`.
- Acquisition: M004 validation/promotion corrective is closed; optional M005 still needs corrected-path footprint evidence.
- Service: M001-M005 are closed; M005 closure is at `plans/closure/service-lifecycle/005-status.md`. No service-aware consumer migration is scheduled.
- Distribution: M001-M003 remain historical predecessor evidence; M004 removed the producer crate after Eggpack Contract M002 closure. The subsystem is archived/transferred to Eggpack.
- Consumer adoption: eggsact/stegoeggo and eggsearch M003 are closed; Gregg still needs its own footprint decision.
- CodeGG M005 closed using CodeGG-owned strict extraction of the existing verified archive and Eggup's multi-artifact transaction; it did not recreate producer release authority in Eggup. Evidence: `plans/closure/consumer-adoption/005-status.md`. Egress remains separately blocked on a generic archive transaction/extraction contract.
- Eggpack interoperability: M001 adapter is historically closed; M001a corrective closed at `plans/closure/eggpack-manifest-interoperability/001a-status.md` with the expanded qualification matrix and reconciled scope without broadening adapter authority. M003 is now registered as an Eggsact real-consumer adoption handoff; its producer-evidence preflight must reconcile current live asset naming and the manifest artifact convention before hosted-release activation.
- Release process: manual crates.io publication only.
- The previously selected lockstep 0.1.1 patch is now eligible for separate qualification/publication when directed because the corrective gates are closed. Eggsearch M003 may use an immutable path/git source for local qualification until that release exists; no publication is implicit in these plans.

## Next handoff

The next interoperability handoff is the registered M003 Eggsact real-consumer manifest adoption plan at `plans/implementation/eggpack-manifest-interoperability/003-eggsact-real-consumer-manifest-adoption.md`. Eggsact is selected because it is already a qualified direct Eggup consumer. Begin with the plan's producer-evidence preflight: reconcile current live Eggsact asset naming with Eggpack producer authority and identify the producer-owned ReleaseManifest artifact convention. If that convention is not yet available, perform only the bounded Eggup adapter/API qualification work and return the producer blocker rather than inventing release policy in Eggup. M001a is closed; Service M005 and planning hygiene remain closed; Gregg remains independently footprint-gated.

Do not author or implement an Eggup installer generator. Keep producer behavior in Eggpack and archive extraction outside the adapter. Execute M003 only through its registered Eggsact plan and producer-evidence gate.

After each implementation pass:

1. create the matching closure record with the actual implementation SHA(s);
2. update the source subsystem roadmap;
3. update this registry;
4. write a new corrective if any medium-or-higher issue remains;
5. only then author newly dependency-ready downstream work.

## Registry update rule

Keep this file limited to active/ready work, recent closure context, blockers, and next dependency transitions. Detailed requirements belong in the implementation plans and subsystem roadmaps.
